// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::borrow::Borrow;
use std::fmt::{self, Debug, Display};

use graphql_client::{GraphQLQuery, QueryBody, Response};
use itertools::Itertools;
use percent_encoding::{utf8_percent_encode, AsciiSet, PercentEncode, CONTROLS};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{self, HeaderValue};
use reqwest::Url;
use serde::de::Error as SerdeError;
use serde::de::{DeserializeOwned, Unexpected};
use serde::ser::Serialize;
use serde::{Deserialize, Deserializer, Serializer};
use thiserror::Error;

use crate::types::*;

const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'/');

#[derive(Debug, Error)]
// TODO #[non_exhaustive]
pub enum TokenError {
    #[error("header value error: {}", source)]
    HeaderValue {
        #[from]
        source: header::InvalidHeaderValue,
    },
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
}

type TokenResult<T> = Result<T, TokenError>;

/// A Gitlab API token
///
/// Gitlab supports two kinds of tokens
#[derive(Debug, Clone)]
enum Token {
    /// A personal access token, obtained through Gitlab user settings
    Private(String),
    /// An OAuth2 token, obtained through the OAuth2 flow
    OAuth2(String),
}

impl Token {
    /// Sets the appropirate header on the request.
    ///
    /// Depending on the token type, this will be either the Private-Token header
    /// or the Authorization header.
    /// Returns an error if the token string cannot be parsed as a header value.
    pub fn set_header(&self, req: RequestBuilder) -> TokenResult<RequestBuilder> {
        Ok(match self {
            Token::Private(token) => {
                let mut token_header_value = HeaderValue::from_str(&token)?;
                token_header_value.set_sensitive(true);
                req.header("PRIVATE-TOKEN", token_header_value)
            },
            Token::OAuth2(token) => {
                let value = format!("Bearer {}", token);
                let mut token_header_value = HeaderValue::from_str(&value)?;
                token_header_value.set_sensitive(true);
                req.header("Authorization", token_header_value)
            },
        })
    }
}

#[derive(Debug, Error)]
// TODO #[non_exhaustive]
pub enum GitlabError {
    #[error("failed to parse url: {}", source)]
    UrlParse {
        #[from]
        source: url::ParseError,
    },
    #[error("no such user: {}", user)]
    NoSuchUser { user: String },
    #[error("error setting token header: {}", source)]
    TokenError {
        #[from]
        source: TokenError,
    },
    #[error("communication with gitlab: {}", source)]
    Communication {
        #[from]
        source: reqwest::Error,
    },
    #[error("gitlab HTTP error: {}", status)]
    Http { status: reqwest::StatusCode },
    #[error("could not parse JSON response: {}", source)]
    Json {
        #[source]
        source: serde_json::Error,
    },
    #[error("milestone without an ID found")]
    InvalidMilestone,
    #[error("gitlab server error: {}", msg)]
    Gitlab { msg: String },
    #[error("graphql error: [\"{}\"]", message.iter().format("\", \""))]
    GraphQL { message: Vec<graphql_client::Error> },
    #[error("no response from gitlab")]
    NoResponse {},
    #[error("could not parse {} data from JSON: {}", typename, source)]
    DataType {
        #[source]
        source: serde_json::Error,
        typename: &'static str,
    },
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
}

impl GitlabError {
    fn no_such_user(user: &str) -> Self {
        GitlabError::NoSuchUser {
            user: user.into(),
        }
    }

    fn http(status: reqwest::StatusCode) -> Self {
        GitlabError::Http {
            status,
        }
    }

    fn json(source: serde_json::Error) -> Self {
        GitlabError::Json {
            source,
        }
    }

    fn from_gitlab(value: serde_json::Value) -> Self {
        let msg = value
            .pointer("/message")
            .or_else(|| value.pointer("/error"))
            .and_then(|s| s.as_str())
            .unwrap_or_else(|| "<unknown error>");

        GitlabError::Gitlab {
            msg: msg.into(),
        }
    }

    fn graphql(message: Vec<graphql_client::Error>) -> Self {
        GitlabError::GraphQL {
            message,
        }
    }

    fn no_response() -> Self {
        GitlabError::NoResponse {}
    }

    fn data_type<T>(source: serde_json::Error) -> Self {
        GitlabError::DataType {
            source,
            typename: any::type_name::<T>(),
        }
    }
}

type GitlabResult<T> = Result<T, GitlabError>;

/// A representation of the Gitlab API for a single user.
///
/// Separate users should use separate instances of this.
#[derive(Clone)]
pub struct Gitlab {
    /// The client to use for API calls.
    client: Client,
    /// The base URL to use for API calls.
    base_url: Url,
    /// The URL to use for GraphQL API calls.
    graphql_url: Url,
    /// The secret token to use when communicating with Gitlab.
    token: Token,
}

impl Debug for Gitlab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Gitlab")
            .field("base_url", &self.base_url)
            .field("graphql_url", &self.graphql_url)
            .finish()
    }
}

/// Optional information for commit statuses.
#[derive(Debug)]
pub struct CommitStatusInfo<'a> {
    /// The refname of the commit being tested.
    pub refname: Option<&'a str>,
    /// The name of the status (defaults to `"default"` on the Gitlab side).
    pub name: Option<&'a str>,
    /// A URL to associate with the status.
    pub target_url: Option<&'a str>,
    /// A description of the status check.
    pub description: Option<&'a str>,
}

/// Optional information for merge requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeRequestStateFilter {
    /// Get the opened/reopened merge requests.
    Opened,
    /// Get the closes merge requests.
    Closed,
    /// Get the merged merge requests.
    Merged,
}

enum_serialize!(MergeRequestStateFilter -> "state",
    Opened => "opened",
    Closed => "closed",
    Merged => "merged",
);

/// Should a certificate be validated in tls connections.
/// The Insecure option is used for self-signed certificates.
#[derive(Debug, Clone)]
enum CertPolicy {
    Default,
    Insecure,
}

impl Gitlab {
    /// Create a new Gitlab API representation.
    ///
    /// The `token` should be a valid [personal access token](https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html).
    /// Errors out if `token` is invalid.
    pub fn new<H, T>(host: H, token: T) -> GitlabResult<Self>
    where
        H: AsRef<str>,
        T: ToString,
    {
        Self::new_impl(
            "https",
            host.as_ref(),
            Token::Private(token.to_string()),
            CertPolicy::Default,
        )
    }

    /// Create a new non-SSL Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new_insecure<H, T>(host: H, token: T) -> GitlabResult<Self>
    where
        H: AsRef<str>,
        T: ToString,
    {
        Self::new_impl(
            "http",
            host.as_ref(),
            Token::Private(token.to_string()),
            CertPolicy::Insecure,
        )
    }

    /// Create a new Gitlab API representation.
    ///
    /// The `token` should be a valid [OAuth2 token](https://docs.gitlab.com/ee/api/oauth2.html).
    /// Errors out if `token` is invalid.
    pub fn with_oauth2<H, T>(host: H, token: T) -> GitlabResult<Self>
    where
        H: AsRef<str>,
        T: ToString,
    {
        Self::new_impl(
            "https",
            host.as_ref(),
            Token::OAuth2(token.to_string()),
            CertPolicy::Default,
        )
    }

    /// Create a new non-SSL Gitlab API representation.
    ///
    /// The `token` should be a valid [OAuth2 token](https://docs.gitlab.com/ee/api/oauth2.html).
    /// Errors out if `token` is invalid.
    pub fn with_oauth2_insecure<H, T>(host: H, token: T) -> GitlabResult<Self>
    where
        H: AsRef<str>,
        T: ToString,
    {
        Self::new_impl(
            "http",
            host.as_ref(),
            Token::OAuth2(token.to_string()),
            CertPolicy::Default,
        )
    }

    /// Internal method to create a new Gitlab client.
    fn new_impl(
        protocol: &str,
        host: &str,
        token: Token,
        cert_validation: CertPolicy,
    ) -> GitlabResult<Self> {
        let base_url = Url::parse(&format!("{}://{}/api/v4/", protocol, host))?;
        let graphql_url = Url::parse(&format!("{}://{}/api/graphql", protocol, host))?;

        let client = match cert_validation {
            CertPolicy::Insecure => {
                Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()?
            },
            CertPolicy::Default => Client::new(),
        };

        let api = Gitlab {
            client,
            base_url,
            graphql_url,
            token,
        };

        // Ensure the API is working.
        let _: UserPublic = api.current_user()?;

        Ok(api)
    }

    /// Create a new Gitlab API client builder.
    pub fn builder<H, T>(host: H, token: T) -> GitlabBuilder
    where
        H: ToString,
        T: ToString,
    {
        GitlabBuilder::new(host, token)
    }

    /// Send a GraphQL query.
    pub fn graphql<Q>(&self, query: &QueryBody<Q::Variables>) -> GitlabResult<Q::ResponseData>
    where
        Q: GraphQLQuery,
        Q::Variables: Debug,
        for<'d> Q::ResponseData: Deserialize<'d>,
    {
        info!(
            target: "gitlab",
            "sending GraphQL query '{}' {:?}",
            query.operation_name,
            query.variables,
        );
        let rsp: Response<Q::ResponseData> =
            self.send(self.client.post(self.graphql_url.clone()).json(query))?;

        if let Some(errs) = rsp.errors {
            return Err(GitlabError::graphql(errs));
        }
        rsp.data.ok_or_else(GitlabError::no_response)
    }

    /// The user the API is acting as.
    pub fn current_user(&self) -> GitlabResult<UserPublic> {
        self.get_with_param("user", query_param_slice![])
    }

    /// Get all user accounts
    pub fn users<T, I, K, V>(&self, params: I) -> GitlabResult<Vec<T>>
    where
        T: UserResult,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param("users", params)
    }

    /// Find a user by id.
    pub fn user<T, I, K, V>(&self, user: UserId, params: I) -> GitlabResult<T>
    where
        T: UserResult,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(format!("users/{}", user), params)
    }

    /// Find a user by username.
    pub fn user_by_name<T, N>(&self, name: N) -> GitlabResult<T>
    where
        T: UserResult,
        N: AsRef<str>,
    {
        let mut users = self.get_paged_with_param("users", &[("username", name.as_ref())])?;
        users
            .pop()
            .ok_or_else(|| GitlabError::no_such_user(name.as_ref()))
    }

    /// Create a project
    ///
    /// # Arguments:
    /// * name: the name of the project
    /// * path: the path of the project. Optional: name is used if None
    /// * params: optional arguments for project creation
    ///
    /// # Example
    /// ```rust, no_run
    /// use gitlab::{Gitlab, CreateProjectParams, GitlabBuilder};
    ///
    /// let gitlab = GitlabBuilder::new("host", "token").build().unwrap();
    /// let params = CreateProjectParams::builder()
    ///                     .description("Splendid project")
    ///                     .build()
    ///                     .unwrap();
    /// gitlab.create_project("My Project", Some("project"), Some(params));
    /// ```
    pub fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> GitlabResult<Project> {
        let url = "projects";

        let mut merged_params = params.unwrap_or_default();

        let path = match path.as_ref() {
            None => name.as_ref(),
            Some(s) => s.as_ref(),
        };
        merged_params.name = Some(name.as_ref().to_string());
        merged_params.path = Some(path.to_string());

        self.post_with_param(url, &merged_params)
    }

    /// Create a new file in repository
    pub fn create_file<F, B, C, M>(
        &self,
        project: ProjectId,
        file_path: F,
        branch: B,
        content: C,
        commit_message: M,
    ) -> GitlabResult<RepoFile>
    where
        F: AsRef<str>,
        B: AsRef<str>,
        C: AsRef<str>,
        M: AsRef<str>,
    {
        let url = format!(
            "projects/{}/repository/files/{}",
            project,
            file_path.as_ref()
        );
        self.post_with_param(
            url,
            &[
                ("branch", branch.as_ref()),
                ("content", content.as_ref()),
                ("commit_message", commit_message.as_ref()),
            ],
        )
    }

    /// Set project description
    pub fn set_project_description<T: AsRef<str>>(
        &self,
        project: ProjectId,
        description: T,
    ) -> GitlabResult<Project> {
        let url = format!("projects/{}", project);
        self.put_with_param(url, &[("description", description.as_ref())])
    }

    /// Set project default branch
    pub fn set_project_default_branch<T: AsRef<str>>(
        &self,
        project: ProjectId,
        branch: T,
    ) -> GitlabResult<Project> {
        let url = format!("projects/{}", project);
        self.put_with_param(url, &[("default_branch", branch.as_ref())])
    }

    /// Set project features access level
    pub fn set_project_feature_access_level(
        &self,
        project: ProjectId,
        feature: ProjectFeatures,
    ) -> GitlabResult<Project> {
        let url = format!("projects/{}", project);
        self.put_with_param(url, &[(feature.name(), feature.access_level())])
    }

    /// Get all accessible projects.
    pub fn projects<I, K, V>(&self, params: I) -> GitlabResult<Vec<Project>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param("projects", params)
    }

    /// Get all owned projects.
    pub fn owned_projects(&self) -> GitlabResult<Vec<Project>> {
        self.get_paged_with_param("projects", &[("owned", "true")])
    }

    /// Find a project by id.
    pub fn project<I, K, V>(&self, project: ProjectId, params: I) -> GitlabResult<Project>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(format!("projects/{}", project), params)
    }

    /// A URL-safe name for projects.
    fn url_name(name: &str) -> PercentEncode {
        utf8_percent_encode(name, PATH_SEGMENT_ENCODE_SET)
    }

    /// Find a project by name.
    pub fn project_by_name<N, I, K, V>(&self, name: N, params: I) -> GitlabResult<Project>
    where
        N: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(
            format!("projects/{}", Self::url_name(name.as_ref())),
            params,
        )
    }

    /// Get all accessible environments.
    pub fn environments<I, K, V>(
        &self,
        project: ProjectId,
        params: I,
    ) -> GitlabResult<Vec<Environment>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("projects/{}/environments", project), params)
    }

    pub fn environment<I, K, V>(
        &self,
        project: ProjectId,
        environment: EnvironmentId,
        params: I,
    ) -> GitlabResult<Environment>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(
            format!("projects/{}/environments/{}", project, environment),
            params,
        )
    }

    /// Create a group
    ///
    /// # Arguments:
    /// * name: the name of the group
    /// * path: the path of the group
    /// * params: optional arguments for group creation
    ///
    /// # Example
    /// ```rust, no_run
    /// use gitlab::{Gitlab, CreateGroupParams, GitlabBuilder};
    ///
    /// let gitlab = GitlabBuilder::new("host", "token").build().unwrap();
    /// let params = CreateGroupParams::builder()
    ///                     .description("A description")
    ///                     .auto_devops_enabled(false)
    ///                     .build()
    ///                     .unwrap();
    /// gitlab.create_group("A group", "A path", Some(params));
    /// ```
    pub fn create_group<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: P,
        params: Option<CreateGroupParams>,
    ) -> GitlabResult<Group> {
        let url = "groups";

        let mut merged_params = params.unwrap_or_default();
        merged_params.name = Some(name.as_ref().to_string());
        merged_params.path = Some(path.as_ref().to_string());

        self.post_with_param(url, &merged_params)
    }

    /// Get all accessible groups.
    pub fn groups<I, K, V>(&self, params: I) -> GitlabResult<Vec<Group>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param("groups", params)
    }

    /// Find a group by its name.
    pub fn group_by_name<N>(&self, name: N) -> GitlabResult<Group>
    where
        N: AsRef<str>,
    {
        let param: &[(&str, &str)] = &[];
        self.get_with_param(format!("groups/{}", Self::url_name(name.as_ref())), param)
    }

    /// Get a project's hooks.
    pub fn hooks<I, K, V>(&self, project: ProjectId, params: I) -> GitlabResult<Vec<ProjectHook>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("projects/{}/hooks", project), params)
    }

    /// Get a project hook.
    pub fn hook<I, K, V>(
        &self,
        project: ProjectId,
        hook: HookId,
        params: I,
    ) -> GitlabResult<ProjectHook>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(format!("projects/{}/hooks/{}", project, hook), params)
    }

    /// Convert a boolean parameter into an HTTP request value.
    fn bool_param_value(value: bool) -> &'static str {
        if value {
            "true"
        } else {
            "false"
        }
    }

    /// HTTP parameters required to register to a project.
    fn event_flags(events: WebhookEvents) -> Vec<(&'static str, &'static str)> {
        vec![
            ("job_events", Self::bool_param_value(events.job())),
            ("issues_events", Self::bool_param_value(events.issues())),
            (
                "confidential_issues_events",
                Self::bool_param_value(events.confidential_issues()),
            ),
            (
                "merge_requests_events",
                Self::bool_param_value(events.merge_requests()),
            ),
            ("note_events", Self::bool_param_value(events.note())),
            ("pipeline_events", Self::bool_param_value(events.pipeline())),
            ("push_events", Self::bool_param_value(events.push())),
            (
                "wiki_page_events",
                Self::bool_param_value(events.wiki_page()),
            ),
        ]
    }

    /// Add a project hook.
    pub fn add_hook<U, T>(
        &self,
        project: ProjectId,
        url: U,
        enable_ssl_verification: Option<bool>,
        token: Option<T>,
        events: WebhookEvents,
    ) -> GitlabResult<ProjectHook>
    where
        U: AsRef<str>,
        T: AsRef<str>,
    {
        let mut flags = Self::event_flags(events);
        flags.push(("url", url.as_ref()));
        let s: String;
        if let Some(ssl) = enable_ssl_verification {
            s = ssl.to_string();
            flags.push(("enable_ssl_verification", s.as_str()));
        }
        if let Some(tk) = token.as_ref() {
            flags.push(("token", tk.as_ref()));
        }

        self.post_with_param(format!("projects/{}/hooks", project), &flags)
    }

    /// Get the team members of a group.
    pub fn group_members<I, K, V>(&self, group: GroupId, params: I) -> GitlabResult<Vec<Member>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("groups/{}/members", group), params)
    }

    /// Get a team member of a group.
    pub fn group_member<I, K, V>(
        &self,
        group: GroupId,
        user: UserId,
        params: I,
    ) -> GitlabResult<Member>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(format!("groups/{}/members/{}", group, user), params)
    }

    /// Get the team members of a project.
    pub fn project_members<I, K, V>(
        &self,
        project: ProjectId,
        params: I,
    ) -> GitlabResult<Vec<Member>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("projects/{}/members", project), params)
    }

    /// Get a team member of a project.
    pub fn project_member<I, K, V>(
        &self,
        project: ProjectId,
        user: UserId,
        params: I,
    ) -> GitlabResult<Member>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(format!("projects/{}/members/{}", project, user), params)
    }

    /// Add a user to a project.
    pub fn add_user_to_project(
        &self,
        project: ProjectId,
        user: UserId,
        access: AccessLevel,
    ) -> GitlabResult<Member> {
        let user_str = format!("{}", user);
        let access_str = format!("{}", access);

        self.post_with_param(
            format!("projects/{}/members", project),
            &[("user_id", &user_str), ("access_level", &access_str)],
        )
    }

    /// Add a user to a project.
    pub fn add_user_to_project_by_name<P>(
        &self,
        project: P,
        user: UserId,
        access: AccessLevel,
    ) -> GitlabResult<Member>
    where
        P: AsRef<str>,
    {
        let user_str = format!("{}", user);
        let access_str = format!("{}", access);

        self.post_with_param(
            format!("projects/{}/members", Self::url_name(project.as_ref())),
            &[("user_id", &user_str), ("access_level", &access_str)],
        )
    }

    /// Create a branch for a project
    pub fn create_branch<V: AsRef<str>>(
        &self,
        project: ProjectId,
        name: V,
        reference: V,
    ) -> GitlabResult<RepoBranch> {
        let url = format!("projects/{}/repository/branches", project);
        self.post_with_param(
            url,
            &[("branch", name.as_ref()), ("ref", reference.as_ref())],
        )
    }

    /// Get branches for a project.
    pub fn branches<I, K, V>(&self, project: ProjectId, params: I) -> GitlabResult<Vec<RepoBranch>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("projects/{}/repository/branches", project), params)
    }

    /// Get a branch.
    pub fn branch<B, I, K, V>(
        &self,
        project: ProjectId,
        branch: B,
        params: I,
    ) -> GitlabResult<RepoBranch>
    where
        B: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(
            format!(
                "projects/{}/repository/branches/{}",
                project,
                Self::url_name(branch.as_ref()),
            ),
            params,
        )
    }

    /// Protect a branch
    ///
    /// # Arguments
    /// * project: The project id
    /// * branch: The name of the branch or wildcard
    /// * push_access_level: Access level allowed to push (defaults: maintainers)
    /// * merge_access_level: Access level allowed to merge (defaults:  maintainers)
    /// * unprotect_access_level: Access level allowed to unproctect (defaults: maintainers)
    pub fn protect_branch<B: AsRef<str>>(
        &self,
        project: ProjectId,
        branch: B,
        push_access_level: Option<AccessLevel>,
        merge_access_level: Option<AccessLevel>,
        unprotect_access_level: Option<AccessLevel>,
    ) -> GitlabResult<ProtectedRepoBranch> {
        let url = format!("projects/{}/protected_branches", project);
        self.post_with_param(
            url,
            &[
                ("name", branch.as_ref()),
                (
                    "push_access_level",
                    &u64::from(push_access_level.unwrap_or(AccessLevel::Maintainer)).to_string(),
                ),
                (
                    "merge_access_level",
                    &u64::from(merge_access_level.unwrap_or(AccessLevel::Maintainer)).to_string(),
                ),
                (
                    "unprotect_access_level",
                    &u64::from(unprotect_access_level.unwrap_or(AccessLevel::Maintainer))
                        .to_string(),
                ),
            ],
        )
    }

    /// Get a commit.
    pub fn commit<C>(&self, project: ProjectId, commit: C) -> GitlabResult<RepoCommitDetail>
    where
        C: AsRef<str>,
    {
        self.get_with_param(
            format!(
                "projects/{}/repository/commits/{}",
                project,
                commit.as_ref(),
            ),
            &[("stats", "true")],
        )
    }

    /// Get comments on a commit.
    pub fn commit_comments<C, I, K, V>(
        &self,
        project: ProjectId,
        commit: C,
        params: I,
    ) -> GitlabResult<Vec<CommitNote>>
    where
        C: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/repository/commits/{}/comments",
                project,
                commit.as_ref(),
            ),
            params,
        )
    }

    /// Get comments on a commit.
    pub fn create_commit_comment<C, B>(
        &self,
        project: ProjectId,
        commit: C,
        body: B,
    ) -> GitlabResult<CommitNote>
    where
        C: AsRef<str>,
        B: AsRef<str>,
    {
        self.post_with_param(
            format!(
                "projects/{}/repository/commits/{}/comment",
                project,
                commit.as_ref(),
            ),
            &[("note", body.as_ref())],
        )
    }

    /// Get comments on a commit.
    pub fn create_commit_comment_by_name<P, C, B>(
        &self,
        project: P,
        commit: C,
        body: B,
    ) -> GitlabResult<CommitNote>
    where
        P: AsRef<str>,
        C: AsRef<str>,
        B: AsRef<str>,
    {
        self.post_with_param(
            format!(
                "projects/{}/repository/commits/{}/comment",
                Self::url_name(project.as_ref()),
                commit.as_ref(),
            ),
            &[("note", body.as_ref())],
        )
    }

    /// Get comments on a commit.
    pub fn create_commit_line_comment(
        &self,
        project: ProjectId,
        commit: &str,
        body: &str,
        path: &str,
        line: u64,
    ) -> GitlabResult<CommitNote> {
        let line_str = format!("{}", line);
        let line_type = LineType::New;

        self.post_with_param(
            format!("projects/{}/repository/commits/{}/comment", project, commit),
            &[
                ("note", body),
                ("path", path),
                ("line", &line_str),
                ("line_type", line_type.as_str()),
            ],
        )
    }

    /// Get the latest statuses of a commit.
    pub fn commit_latest_statuses<C, I, K, V>(
        &self,
        project: ProjectId,
        commit: C,
        params: I,
    ) -> GitlabResult<Vec<CommitStatus>>
    where
        C: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/repository/commits/{}/statuses",
                project,
                commit.as_ref(),
            ),
            params,
        )
    }

    /// Get the latest statuses of a commit.
    pub fn commit_latest_statuses_by_name<P, C, I, K, V>(
        &self,
        project: P,
        commit: C,
        params: I,
    ) -> GitlabResult<Vec<CommitStatus>>
    where
        P: AsRef<str>,
        C: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/repository/commits/{}/statuses",
                Self::url_name(project.as_ref()),
                commit.as_ref(),
            ),
            params,
        )
    }

    /// Get the all statuses of a commit.
    pub fn commit_all_statuses<C>(
        &self,
        project: ProjectId,
        commit: C,
    ) -> GitlabResult<Vec<CommitStatus>>
    where
        C: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/repository/commits/{}/statuses",
                project,
                commit.as_ref(),
            ),
            &[("all", "true")],
        )
    }

    /// Get the latest builds of a commit.
    pub fn commit_latest_builds<C, I, K, V>(
        &self,
        project: ProjectId,
        commit: C,
        params: I,
    ) -> GitlabResult<Vec<Job>>
    where
        C: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/repository/commits/{}/builds",
                project,
                commit.as_ref(),
            ),
            params,
        )
    }

    /// Get the all builds of a commit.
    pub fn commit_all_builds<C>(&self, project: ProjectId, commit: C) -> GitlabResult<Vec<Job>>
    where
        C: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/repository/commits/{}/builds",
                project,
                commit.as_ref(),
            ),
            &[("all", "true")],
        )
    }

    /// Create a status message for a commit.
    pub fn create_commit_status<S>(
        &self,
        project: ProjectId,
        sha: S,
        state: StatusState,
        info: &CommitStatusInfo,
    ) -> GitlabResult<CommitStatus>
    where
        S: AsRef<str>,
    {
        let path = format!("projects/{}/statuses/{}", project, sha.as_ref());

        let mut params = vec![("state", state.as_str())];

        if let Some(v) = info.refname {
            params.push(("ref", v))
        }
        if let Some(v) = info.name {
            params.push(("name", v))
        }
        if let Some(v) = info.target_url {
            params.push(("target_url", v))
        }
        if let Some(v) = info.description {
            params.push(("description", v))
        }

        self.post_with_param(path, &params)
    }

    /// Create a status message for a commit.
    pub fn create_commit_status_by_name<P, S>(
        &self,
        project: P,
        sha: S,
        state: StatusState,
        info: &CommitStatusInfo,
    ) -> GitlabResult<CommitStatus>
    where
        P: AsRef<str>,
        S: AsRef<str>,
    {
        let path = format!(
            "projects/{}/statuses/{}",
            Self::url_name(project.as_ref()),
            sha.as_ref(),
        );

        let mut params = vec![("state", state.as_str())];

        if let Some(v) = info.refname {
            params.push(("ref", v))
        }
        if let Some(v) = info.name {
            params.push(("name", v))
        }
        if let Some(v) = info.target_url {
            params.push(("target_url", v))
        }
        if let Some(v) = info.description {
            params.push(("description", v))
        }

        self.post_with_param(path, &params)
    }

    /// Get the labels for a project.
    pub fn labels(&self, project: ProjectId) -> GitlabResult<Vec<Label>> {
        self.get_paged(format!("projects/{}/labels", project))
    }

    /// Get the labels with open/closed/merge requests count
    pub fn labels_with_counts(&self, project: ProjectId) -> GitlabResult<Vec<Label>> {
        self.get_paged_with_param(
            format!("projects/{}/labels", project),
            &[("with_counts", "true")],
        )
    }

    /// Get label by ID.
    pub fn label(&self, project: ProjectId, label: LabelId) -> GitlabResult<Label> {
        self.get(format!("projects/{}/labels/{}", project, label))
    }

    /// Get the issues for a project.
    pub fn issues<I, K, V>(&self, project: ProjectId, params: I) -> GitlabResult<Vec<Issue>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("projects/{}/issues", project), params)
    }

    /// Get issues.
    pub fn issue<I, K, V>(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
        params: I,
    ) -> GitlabResult<Issue>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(format!("projects/{}/issues/{}", project, issue), params)
    }

    /// Get the notes from a issue.
    pub fn issue_notes<I, K, V>(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
        params: I,
    ) -> GitlabResult<Vec<Note>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!("projects/{}/issues/{}/notes", project, issue),
            params,
        )
    }

    /// Get the notes from a issue.
    pub fn issue_notes_by_name<P, I, K, V>(
        &self,
        project: P,
        issue: IssueInternalId,
        params: I,
    ) -> GitlabResult<Vec<Note>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/issues/{}/notes",
                Self::url_name(project.as_ref()),
                issue,
            ),
            params,
        )
    }

    /// Create a new label
    pub fn create_label(&self, project: ProjectId, label: Label) -> GitlabResult<Label> {
        let path = format!("projects/{}/labels", project);

        let mut params: Vec<(&str, String)> = Vec::new();

        params.push(("name", label.name));
        params.push(("color", label.color.value()));

        if let Some(d) = label.description {
            params.push(("description", d));
        }

        if let Some(p) = label.priority {
            params.push(("priority", p.to_string()));
        }

        self.post_with_param(path, &params)
    }

    /// Create a new milestone
    pub fn create_milestone(&self, milestone: Milestone) -> GitlabResult<Milestone> {
        let path = if let Some(project) = milestone.project_id {
            format!("projects/{}/milestones", project)
        } else if let Some(group) = milestone.group_id {
            format!("groups/{}/milestones", group)
        } else {
            return Err(GitlabError::InvalidMilestone);
        };

        let mut params: Vec<(&str, String)> = Vec::new();

        params.push(("title", milestone.title));

        if let Some(d) = milestone.description {
            params.push(("description", d));
        }

        if let Some(d) = milestone.due_date {
            params.push(("due_date", d.to_string()))
        }

        if let Some(s) = milestone.start_date {
            params.push(("start_date", s.to_string()))
        }

        self.post_with_param(path, &params)
    }

    /// Create a new issue
    pub fn create_issue(&self, project: ProjectId, issue: Issue) -> GitlabResult<Issue> {
        let path = format!("projects/{}/issues", project);

        let mut params: Vec<(&str, String)> = Vec::new();

        if issue.iid.value() != 0 {
            params.push(("iid", issue.iid.value().to_string()));
        }

        params.push(("title", issue.title));

        if let Some(d) = issue.description {
            params.push(("description", d));
        }

        params.push(("confidential", issue.confidential.to_string()));

        if let Some(v) = issue.assignees {
            params.extend(
                v.into_iter()
                    .map(|x| ("assignee_ids[]", x.id.value().to_string())),
            );
        }

        if let Some(m) = issue.milestone {
            params.push(("milestone_id", m.id.value().to_string()))
        }

        if !issue.labels.is_empty() {
            params.push(("labels", issue.labels.join(",")));
        }

        params.push(("created_at", issue.created_at.to_string()));

        if let Some(d) = issue.due_date {
            params.push(("due_date", d.to_string()))
        }

        self.post_with_param(path, &params)
    }

    /// Get the resource label events from an issue.
    pub fn issue_label_events(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
    ) -> GitlabResult<Vec<ResourceLabelEvent>> {
        self.get_paged(format!(
            "projects/{}/issues/{}/resource_label_events",
            project, issue,
        ))
    }

    /// Create a note on a issue.
    pub fn create_issue_note<C>(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
        content: C,
    ) -> GitlabResult<Note>
    where
        C: AsRef<str>,
    {
        let path = format!("projects/{}/issues/{}/notes", project, issue);

        self.post_with_param(path, &[("body", content.as_ref())])
    }

    /// Create a note on a issue.
    pub fn create_issue_note_by_name<P, C>(
        &self,
        project: P,
        issue: IssueInternalId,
        content: C,
    ) -> GitlabResult<Note>
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        let path = format!(
            "projects/{}/issues/{}/notes",
            Self::url_name(project.as_ref()),
            issue,
        );

        self.post_with_param(path, &[("body", content.as_ref())])
    }

    /// Edit a note on an issue.
    pub fn set_issue_note<C>(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
        note: NoteId,
        content: C,
    ) -> GitlabResult<Note>
    where
        C: AsRef<str>,
    {
        let path = format!("projects/{}/issues/{}/notes/{}", project, issue, note);

        self.put_with_param(path, &[("body", content.as_ref())])
    }

    /// Get the merge requests for a project.
    pub fn merge_requests<I, K, V>(
        &self,
        project: ProjectId,
        params: I,
    ) -> GitlabResult<Vec<MergeRequest>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("projects/{}/merge_requests", project), params)
    }

    /// Get the merge requests with a given state.
    pub fn merge_requests_with_state(
        &self,
        project: ProjectId,
        state: MergeRequestStateFilter,
    ) -> GitlabResult<Vec<MergeRequest>> {
        self.get_paged_with_param(
            format!("projects/{}/merge_requests", project),
            &[("state", state.as_str())],
        )
    }

    /// Create a new merge request
    pub fn create_merge_request(
        &self,
        project: ProjectId,
        params: CreateMergeRequestParams,
    ) -> GitlabResult<MergeRequest> {
        self.post_with_param(format!("projects/{}/merge_requests", project), &params)
    }

    /// Get all pipelines for a project.
    pub fn pipelines<I, K, V>(
        &self,
        project: ProjectId,
        params: I,
    ) -> GitlabResult<Vec<PipelineBasic>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(format!("projects/{}/pipelines", project), params)
    }

    /// Get a single pipeline.
    pub fn pipeline(&self, project: ProjectId, id: PipelineId) -> GitlabResult<Pipeline> {
        self.get(format!("projects/{}/pipelines/{}", project, id))
    }

    /// Get variables of a pipeline.
    pub fn pipeline_variables(
        &self,
        project: ProjectId,
        id: PipelineId,
    ) -> GitlabResult<Vec<PipelineVariable>> {
        self.get(format!("projects/{}/pipelines/{}/variables", project, id))
    }

    /// Create a new pipeline.
    pub fn create_pipeline(
        &self,
        project: ProjectId,
        ref_: ObjectId,
        variables: &[PipelineVariable],
    ) -> GitlabResult<Pipeline> {
        use serde::Serialize;
        #[derive(Debug, Serialize)]
        struct CreatePipelineParams<'a> {
            ref_: ObjectId,
            variables: &'a [PipelineVariable],
        }

        self.post_with_param(
            format!("projects/{}/pipeline", project),
            CreatePipelineParams {
                ref_,
                variables,
            },
        )
    }

    /// Retry jobs in a pipeline.
    pub fn retry_pipeline(&self, project: ProjectId, id: PipelineId) -> GitlabResult<Pipeline> {
        self.post(format!("projects/{}/pipelines/{}/retry", project, id))
    }

    /// Cancel a pipeline.
    pub fn cancel_pipeline(&self, project: ProjectId, id: PipelineId) -> GitlabResult<Pipeline> {
        self.post(format!("projects/{}/pipelines/{}/cancel", project, id))
    }

    /// Delete a pipeline.
    ///
    /// NOTE Not implemented.
    #[allow(unused)]
    fn delete_pipeline(&self, project: ProjectId, id: PipelineId) -> GitlabResult<Pipeline> {
        unimplemented!();
    }

    /// Get merge requests.
    pub fn merge_request<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<MergeRequest>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_with_param(
            format!("projects/{}/merge_requests/{}", project, merge_request),
            params,
        )
    }

    /// Get the issues that will be closed when a merge request is merged.
    pub fn merge_request_closes_issues<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<IssueReference>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/closes_issues",
                project, merge_request,
            ),
            params,
        )
    }

    /// Get the discussions from a merge request.
    pub fn merge_request_discussions<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<Discussion>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/discussions",
                project, merge_request,
            ),
            params,
        )
    }

    /// Get the notes from a merge request.
    pub fn merge_request_notes<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<Note>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/notes",
                project, merge_request,
            ),
            params,
        )
    }

    /// Get the notes from a merge request.
    pub fn merge_request_notes_by_name<P, I, K, V>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<Note>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/notes",
                Self::url_name(project.as_ref()),
                merge_request,
            ),
            params,
        )
    }

    /// Award a merge request note with an award.
    pub fn award_merge_request_note(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        award: &str,
    ) -> GitlabResult<AwardEmoji> {
        let path = format!(
            "projects/{}/merge_requests/{}/notes/{}/award_emoji",
            project, merge_request, note,
        );
        self.post_with_param(path, &[("name", award)])
    }

    /// Award a merge request note with an award.
    pub fn award_merge_request_note_by_name<P>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        award: &str,
    ) -> GitlabResult<AwardEmoji>
    where
        P: AsRef<str>,
    {
        let path = format!(
            "projects/{}/merge_requests/{}/notes/{}/award_emoji",
            Self::url_name(project.as_ref()),
            merge_request,
            note,
        );
        self.post_with_param(path, &[("name", award)])
    }

    /// Get the awards for a merge request.
    pub fn merge_request_awards<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/award_emoji",
                project, merge_request,
            ),
            params,
        )
    }

    /// Get the awards for a merge request.
    pub fn merge_request_awards_by_name<P, I, K, V>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/award_emoji",
                Self::url_name(project.as_ref()),
                merge_request,
            ),
            params,
        )
    }

    /// Get the awards for a merge request note.
    pub fn merge_request_note_awards<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        params: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/notes/{}/award_emoji",
                project, merge_request, note,
            ),
            params,
        )
    }

    /// Get the awards for a merge request note.
    pub fn merge_request_note_awards_by_name<P, I, K, V>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        params: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/notes/{}/award_emoji",
                Self::url_name(project.as_ref()),
                merge_request,
                note,
            ),
            params,
        )
    }

    /// Get the resource label events from a merge request.
    pub fn merge_request_label_events(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
    ) -> GitlabResult<Vec<ResourceLabelEvent>> {
        self.get_paged(format!(
            "projects/{}/merge_requests/{}/resource_label_events",
            project, merge_request,
        ))
    }

    pub fn create_merge_request_discussion(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        content: &str,
    ) -> GitlabResult<Discussion> {
        let path = format!(
            "projects/{}/merge_requests/{}/discussions",
            project, merge_request
        );
        self.post_with_param(path, &[("body", content)])
    }
    /// Create a note on a merge request.
    pub fn create_merge_request_note(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        content: &str,
    ) -> GitlabResult<Note> {
        let path = format!(
            "projects/{}/merge_requests/{}/notes",
            project, merge_request,
        );
        self.post_with_param(path, &[("body", content)])
    }

    /// Create a note on a merge request.
    pub fn create_merge_request_note_by_name<P>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        content: &str,
    ) -> GitlabResult<Note>
    where
        P: AsRef<str>,
    {
        let path = format!(
            "projects/{}/merge_requests/{}/notes",
            Self::url_name(project.as_ref()),
            merge_request,
        );
        self.post_with_param(path, &[("body", content)])
    }

    /// Edit a note on a merge request.
    pub fn set_merge_request_note<C>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        content: C,
    ) -> GitlabResult<Note>
    where
        C: AsRef<str>,
    {
        let path = format!(
            "projects/{}/merge_requests/{}/notes/{}",
            project, merge_request, note,
        );
        self.put_with_param(path, &[("body", content.as_ref())])
    }

    /// Edit a note on a merge request.
    pub fn modify_merge_request_note<C>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        content: C,
    ) -> GitlabResult<Note>
    where
        C: AsRef<str>,
    {
        let path = &format!(
            "projects/{}/merge_requests/{}/notes/{}",
            project, merge_request, note,
        );
        self.put_with_param(path, &[("body", content.as_ref())])
    }

    /// Get issues closed by a merge request.
    pub fn get_issues_closed_by_merge_request<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<Issue>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/closes_issues",
                project, merge_request,
            ),
            params,
        )
    }

    /// Get issues closed by a merge request.
    pub fn get_issues_closed_by_merge_request_by_name<P, I, K, V>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        params: I,
    ) -> GitlabResult<Vec<Issue>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!(
                "projects/{}/merge_requests/{}/closes_issues",
                Self::url_name(project.as_ref()),
                merge_request,
            ),
            params,
        )
    }

    /// Closes an issue
    pub fn close_issue(&self, project: ProjectId, issue: IssueInternalId) -> GitlabResult<Issue> {
        let path = format!("projects/{}/issues/{}", project, issue);
        self.put_with_param(path, &[("state_event", "close")])
    }

    /// Set the labels on an issue.
    pub fn set_issue_labels<I, L>(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
        labels: I,
    ) -> GitlabResult<Issue>
    where
        I: IntoIterator<Item = L>,
        L: Display,
    {
        let path = format!("projects/{}/issues/{}", project, issue);
        self.put_with_param(path, &[("labels", labels.into_iter().join(","))])
    }

    /// Set the labels on an issue.
    pub fn set_issue_labels_by_name<P, I, L>(
        &self,
        project: P,
        issue: IssueInternalId,
        labels: I,
    ) -> GitlabResult<Issue>
    where
        P: AsRef<str>,
        I: IntoIterator<Item = L>,
        L: Display,
    {
        let path = format!(
            "projects/{}/issues/{}",
            Self::url_name(project.as_ref()),
            issue,
        );
        self.put_with_param(path, &[("labels", labels.into_iter().join(","))])
    }

    /// Set the labels on a merge request.
    pub fn set_merge_request_labels<I, L>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        labels: I,
    ) -> GitlabResult<MergeRequest>
    where
        I: IntoIterator<Item = L>,
        L: Display,
    {
        let path = format!("projects/{}/merge_requests/{}", project, merge_request);
        self.put_with_param(path, &[("labels", labels.into_iter().join(","))])
    }

    /// Create a URL to an API endpoint.
    fn create_url<U>(&self, url: U) -> GitlabResult<Url>
    where
        U: AsRef<str>,
    {
        debug!(target: "gitlab", "api call {}", url.as_ref());
        Ok(self.base_url.join(url.as_ref())?)
    }

    /// Create a URL to an API endpoint with query parameters.
    fn create_url_with_param<U, I, K, V>(&self, url: U, param: I) -> GitlabResult<Url>
    where
        U: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut full_url = self.create_url(url.as_ref())?;
        full_url.query_pairs_mut().extend_pairs(param);
        Ok(full_url)
    }

    /// Refactored code which talks to Gitlab and transforms error messages properly.
    fn send<T>(&self, req: RequestBuilder) -> GitlabResult<T>
    where
        T: DeserializeOwned,
    {
        let rsp = self.token.set_header(req)?.send()?;
        let status = rsp.status();
        if status.is_server_error() {
            return Err(GitlabError::http(status));
        }
        let success = status.is_success();
        let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
        if !success {
            return Err(GitlabError::from_gitlab(v));
        }

        debug!(target: "gitlab", "received data: {:?}", v);
        serde_json::from_value::<T>(v).map_err(GitlabError::data_type::<T>)
    }

    /// Create a `GET` request to an API endpoint.
    fn get<T, U>(&self, url: U) -> GitlabResult<T>
    where
        T: DeserializeOwned,
        U: AsRef<str>,
    {
        self.get_with_param(url, query_param_slice![])
    }

    /// Create a `GET` request to an API endpoint with query parameters.
    fn get_with_param<T, U, I, K, V>(&self, url: U, params: I) -> GitlabResult<T>
    where
        T: DeserializeOwned,
        U: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let full_url = self.create_url_with_param(url, params.into_iter())?;
        let req = self.client.get(full_url);
        self.send(req)
    }

    /// Create a `POST` request to an API endpoint.
    fn post<T, U>(&self, url: U) -> GitlabResult<T>
    where
        T: DeserializeOwned,
        U: AsRef<str>,
    {
        let param: &[(&str, &str)] = &[];
        self.post_with_param(url, param)
    }

    /// Create a `POST` request to an API endpoint with query parameters.
    fn post_with_param<T, U, P>(&self, url: U, param: P) -> GitlabResult<T>
    where
        T: DeserializeOwned,
        U: AsRef<str>,
        P: Serialize,
    {
        let full_url = self.create_url(url)?;
        self.send(self.client.post(full_url).form(&param))
    }

    /// Create a `PUT` request to an API endpoint with query parameters.
    fn put_with_param<T, U, P>(&self, url: U, param: P) -> GitlabResult<T>
    where
        T: DeserializeOwned,
        U: AsRef<str>,
        P: Serialize,
    {
        let full_url = self.create_url(url)?;
        self.send(self.client.put(full_url).form(&param))
    }

    /// Handle paginated queries. Returns all results.
    fn get_paged<T, U>(&self, url: U) -> GitlabResult<Vec<T>>
    where
        T: DeserializeOwned,
        U: AsRef<str>,
    {
        self.get_paged_with_param(url, query_param_slice![])
    }

    /// Handle paginated queries with query parameters. Returns all results.
    fn get_paged_with_param<T, U, I, K, V>(&self, url: U, params: I) -> GitlabResult<Vec<T>>
    where
        T: DeserializeOwned,
        U: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut page_num = 1;
        let per_page = 100;
        let per_page_str = &format!("{}", per_page);

        let full_url = self.create_url_with_param(url, params.into_iter())?;

        let mut results: Vec<T> = vec![];

        loop {
            let page_str = &format!("{}", page_num);
            let mut page_url = full_url.clone();
            page_url
                .query_pairs_mut()
                .extend_pairs(&[("page", page_str), ("per_page", per_page_str)]);
            let req = self.client.get(page_url);

            let page: Vec<T> = self.send(req)?;

            let page_len = page.len();
            results.extend(page);
            // Gitlab used to have issues returning paginated results; these have been fixed since,
            // but if it is needed, the bug manifests as Gitlab returning *all* results instead of
            // just the requested results. This can cause an infinite loop here if the number of
            // total results is exactly equal to `per_page`.
            if page_len != per_page {
                break;
            }
            page_num += 1;
        }

        Ok(results)
    }
}

pub struct GitlabBuilder {
    protocol: &'static str,
    host: String,
    token: Token,
    cert_validation: CertPolicy,
}

impl GitlabBuilder {
    /// Create a new Gitlab API client builder.
    pub fn new<H, T>(host: H, token: T) -> Self
    where
        H: ToString,
        T: ToString,
    {
        Self {
            protocol: "https",
            host: host.to_string(),
            token: Token::Private(token.to_string()),
            cert_validation: CertPolicy::Default,
        }
    }

    /// Switch to an insecure protocol (http instead of https).
    pub fn insecure(&mut self) -> &mut Self {
        self.protocol = "http";
        self
    }

    pub fn cert_insecure(&mut self) -> &mut Self {
        self.cert_validation = CertPolicy::Insecure;
        self
    }

    /// Switch to using an OAuth2 token instead of a personal access token
    pub fn oauth2_token(&mut self) -> &mut Self {
        if let Token::Private(token) = self.token.clone() {
            self.token = Token::OAuth2(token);
        }
        self
    }

    pub fn build(&self) -> GitlabResult<Gitlab> {
        Gitlab::new_impl(
            self.protocol,
            &self.host,
            self.token.clone(),
            self.cert_validation.clone(),
        )
    }
}
