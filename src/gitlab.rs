// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::borrow::Borrow;
use std::convert::TryInto;
use std::fmt::{self, Debug, Display};

use bytes::Bytes;
use graphql_client::{GraphQLQuery, QueryBody, Response};
use http::{HeaderMap, Response as HttpResponse};
use itertools::Itertools;
use log::{debug, error, info, warn};
use percent_encoding::{utf8_percent_encode, AsciiSet, PercentEncode, CONTROLS};
use reqwest::blocking::{Client, Response as ReqwestResponse};
use serde::de::DeserializeOwned;
use serde::de::Error as SerdeError;
use serde::ser::Serialize;
use serde::{Deserialize, Deserializer, Serializer};
use thiserror::Error;
use url::Url;

use crate::api::projects::{self, pipelines};
use crate::api::users::{CurrentUser, User, Users};
use crate::api::{self, common, groups, Query};
use crate::auth::{Auth, AuthError};
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
pub enum GitlabError {
    #[error("failed to parse url: {}", source)]
    UrlParse {
        #[from]
        source: url::ParseError,
    },
    #[error("no such user: {}", user)]
    #[deprecated(since = "0.1300.0", note = "unnecessary with the new API pattern")]
    NoSuchUser { user: String },
    #[error("error setting auth header: {}", source)]
    AuthError {
        #[from]
        source: AuthError,
    },
    #[error("communication with gitlab: {}", source)]
    Communication {
        #[from]
        source: reqwest::Error,
    },
    #[error("gitlab HTTP error: {}", status)]
    Http { status: reqwest::StatusCode },
    #[error("could not parse JSON response: {}", source)]
    #[deprecated(since = "0.1300.0", note = "unnecessary with the new API pattern")]
    Json {
        #[source]
        source: serde_json::Error,
    },
    #[error("milestone without an ID found")]
    #[deprecated(since = "0.1300.0", note = "unnecessary with the new API pattern")]
    InvalidMilestone,
    #[error("gitlab server error: {}", msg)]
    #[deprecated(since = "0.1300.0", note = "unnecessary with the new API pattern")]
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
    #[error("api error: {}", source)]
    Api {
        #[from]
        source: api::ApiError<RestError>,
    },
    #[error("invalid status state for new commit status: {:?}", state)]
    #[deprecated(since = "0.1300.0", note = "unnecessary with the new API pattern")]
    InvalidStatusState { state: StatusState },
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
}

impl GitlabError {
    fn no_such_user(user: &str) -> Self {
        #[allow(deprecated)]
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
        #[allow(deprecated)]
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

        #[allow(deprecated)]
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
    rest_url: Url,
    /// The URL to use for GraphQL API calls.
    graphql_url: Url,
    /// The authentication information to use when communicating with Gitlab.
    auth: Auth,
}

impl Debug for Gitlab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Gitlab")
            .field("rest_url", &self.rest_url)
            .field("graphql_url", &self.graphql_url)
            .finish()
    }
}

/// Optional information for commit statuses.
#[derive(Debug)]
#[deprecated(since = "0.1300.0", note = "unnecessary with the new API pattern")]
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
#[deprecated(since = "0.1300.0", note = "unnecessary with the new API pattern")]
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
        T: Into<String>,
    {
        Self::new_impl(
            "https",
            host.as_ref(),
            Auth::Token(token.into()),
            CertPolicy::Default,
        )
    }

    /// Create a new non-SSL Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new_insecure<H, T>(host: H, token: T) -> GitlabResult<Self>
    where
        H: AsRef<str>,
        T: Into<String>,
    {
        Self::new_impl(
            "http",
            host.as_ref(),
            Auth::Token(token.into()),
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
        T: Into<String>,
    {
        Self::new_impl(
            "https",
            host.as_ref(),
            Auth::OAuth2(token.into()),
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
        T: Into<String>,
    {
        Self::new_impl(
            "http",
            host.as_ref(),
            Auth::OAuth2(token.into()),
            CertPolicy::Default,
        )
    }

    /// Internal method to create a new Gitlab client.
    fn new_impl(
        protocol: &str,
        host: &str,
        auth: Auth,
        cert_validation: CertPolicy,
    ) -> GitlabResult<Self> {
        let rest_url = Url::parse(&format!("{}://{}/api/v4/", protocol, host))?;
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
            rest_url,
            graphql_url,
            auth,
        };

        // Ensure the API is working.
        let _: UserPublic = CurrentUser::builder().build().unwrap().query(&api)?;

        Ok(api)
    }

    /// Create a new Gitlab API client builder.
    pub fn builder<H, T>(host: H, token: T) -> GitlabBuilder
    where
        H: Into<String>,
        T: Into<String>,
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
    #[deprecated(
        since = "0.1209.2",
        note = "use `gitlab::api::users::CurrentUser.query()` instead"
    )]
    pub fn current_user(&self) -> GitlabResult<UserPublic> {
        Ok(CurrentUser::builder().build().unwrap().query(self)?)
    }

    /// Get all user accounts
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::users::Users.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::users::User.query()` instead"
    )]
    pub fn user<T, I, K, V>(&self, user: UserId, _: I) -> GitlabResult<T>
    where
        T: UserResult,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(User::builder()
            .user(user.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Find a user by username.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::users::Users.query()` instead"
    )]
    pub fn user_by_name<T, N>(&self, name: N) -> GitlabResult<T>
    where
        T: UserResult,
        N: AsRef<str>,
    {
        api::paged(
            Users::builder().username(name.as_ref()).build().unwrap(),
            api::Pagination::All,
        )
        .query(self)?
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
    #[allow(clippy::cognitive_complexity)]
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::CreateProject.query()` instead"
    )]
    #[allow(deprecated)]
    pub fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> GitlabResult<Project> {
        let mut builder = projects::CreateProject::builder();

        builder.name(name.as_ref());

        let convert = |level| {
            match level {
                FeatureVisibilityLevel::Disabled => projects::FeatureAccessLevel::Disabled,
                FeatureVisibilityLevel::Private => projects::FeatureAccessLevel::Private,
                FeatureVisibilityLevel::Enabled | FeatureVisibilityLevel::Public => {
                    projects::FeatureAccessLevel::Enabled
                },
            }
        };
        let convert_pub = |level| {
            match level {
                FeatureVisibilityLevel::Disabled => projects::FeatureAccessLevelPublic::Disabled,
                FeatureVisibilityLevel::Private => projects::FeatureAccessLevelPublic::Private,
                FeatureVisibilityLevel::Enabled => projects::FeatureAccessLevelPublic::Enabled,
                FeatureVisibilityLevel::Public => projects::FeatureAccessLevelPublic::Public,
            }
        };
        let convert_vis = |level| {
            match level {
                VisibilityLevel::Public => api::common::VisibilityLevel::Public,
                VisibilityLevel::Internal => api::common::VisibilityLevel::Internal,
                VisibilityLevel::Private => api::common::VisibilityLevel::Private,
            }
        };
        let convert_mm = |method| {
            match method {
                MergeMethod::Merge => api::projects::MergeMethod::Merge,
                MergeMethod::RebaseMerge => api::projects::MergeMethod::RebaseMerge,
                MergeMethod::FastForward => api::projects::MergeMethod::FastForward,
            }
        };
        let convert_git = |strategy| {
            match strategy {
                BuildGitStrategy::Fetch => api::projects::BuildGitStrategy::Fetch,
                BuildGitStrategy::Clone => api::projects::BuildGitStrategy::Clone,
            }
        };
        let convert_auto_deploy = |strategy| {
            match strategy {
                "continuous" => Some(api::projects::AutoDevOpsDeployStrategy::Continuous),
                "manual" => Some(api::projects::AutoDevOpsDeployStrategy::Manual),
                "timed_incremental" => {
                    Some(api::projects::AutoDevOpsDeployStrategy::TimedIncremental)
                },
                unknown => {
                    warn!(
                        target: "gitlab",
                        "unknown auto_devops_deploy_strategy '{}'; ignoring",
                        unknown,
                    );
                    None
                },
            }
        };

        let path_save;
        if let Some(path) = path {
            path_save = path;
            builder.path(path_save.as_ref());
        }
        if let Some(params) = params {
            if let Some(namespace_id) = params.namespace_id {
                builder.namespace_id(namespace_id);
            }
            if let Some(default_branch) = params.default_branch {
                builder.default_branch(default_branch);
            }
            if let Some(description) = params.description {
                builder.description(description);
            }
            if let Some(issues_access_level) = params.issues_access_level {
                builder.issues_access_level(convert(issues_access_level));
            }
            if let Some(repository_access_level) = params.repository_access_level {
                builder.repository_access_level(convert(repository_access_level));
            }
            if let Some(merge_requests_access_level) = params.merge_requests_access_level {
                builder.merge_requests_access_level(convert(merge_requests_access_level));
            }
            if let Some(builds_access_level) = params.builds_access_level {
                builder.builds_access_level(convert(builds_access_level));
            }
            if let Some(wiki_access_level) = params.wiki_access_level {
                builder.wiki_access_level(convert(wiki_access_level));
            }
            if let Some(snippets_access_level) = params.snippets_access_level {
                builder.snippets_access_level(convert(snippets_access_level));
            }
            if let Some(pages_access_level) = params.pages_access_level {
                builder.pages_access_level(convert_pub(pages_access_level));
            }
            if let Some(resolve_outdated_diff_discussions) =
                params.resolve_outdated_diff_discussions
            {
                builder.resolve_outdated_diff_discussions(resolve_outdated_diff_discussions);
            }
            if let Some(container_registry_enabled) = params.container_registry_enabled {
                builder.container_registry_enabled(container_registry_enabled);
            }
            if let Some(shared_runners_enabled) = params.shared_runners_enabled {
                builder.shared_runners_enabled(shared_runners_enabled);
            }
            if let Some(visibility) = params.visibility {
                builder.visibility(convert_vis(visibility));
            }
            if let Some(import_url) = params.import_url {
                builder.import_url(import_url);
            }
            if let Some(public_builds) = params.public_builds {
                builder.public_builds(public_builds);
            }
            if let Some(only_allow_merge_if_pipeline_succeeds) =
                params.only_allow_merge_if_pipeline_succeeds
            {
                builder
                    .only_allow_merge_if_pipeline_succeeds(only_allow_merge_if_pipeline_succeeds);
            }
            if let Some(only_allow_merge_if_all_discussions_are_resolved) =
                params.only_allow_merge_if_all_discussions_are_resolved
            {
                builder.only_allow_merge_if_all_discussions_are_resolved(
                    only_allow_merge_if_all_discussions_are_resolved,
                );
            }
            if let Some(merge_method) = params.merge_method {
                builder.merge_method(convert_mm(merge_method));
            }
            if let Some(autoclose_referenced_issues) = params.autoclose_referenced_issues {
                builder.autoclose_referenced_issues(autoclose_referenced_issues);
            }
            if let Some(lfs_enabled) = params.lfs_enabled {
                builder.lfs_enabled(lfs_enabled);
            }
            if let Some(request_access_enabled) = params.request_access_enabled {
                builder.request_access_enabled(request_access_enabled);
            }
            if let Some(tag_list) = params.tag_list {
                builder.tags(tag_list.into_iter());
            }
            if let Some(printing_merge_request_link_enabled) =
                params.printing_merge_request_link_enabled
            {
                builder.printing_merge_request_link_enabled(printing_merge_request_link_enabled);
            }
            if let Some(build_git_strategy) = params.build_git_strategy {
                builder.build_git_strategy(convert_git(build_git_strategy));
            }
            if let Some(build_timeout) = params.build_timeout {
                builder.build_timeout(build_timeout);
            }
            if let Some(auto_cancel_pending_pipelines) = params.auto_cancel_pending_pipelines {
                builder.auto_cancel_pending_pipelines(auto_cancel_pending_pipelines);
            }
            if let Some(build_coverage_regex) = params.build_coverage_regex {
                builder.build_coverage_regex(build_coverage_regex);
            }
            if let Some(ci_config_path) = params.ci_config_path {
                builder.ci_config_path(ci_config_path);
            }
            if let Some(auto_devops_enabled) = params.auto_devops_enabled {
                builder.auto_devops_enabled(auto_devops_enabled);
            }
            if let Some(auto_devops_deploy_strategy) = params.auto_devops_deploy_strategy {
                if let Some(strategy) = convert_auto_deploy(&auto_devops_deploy_strategy) {
                    builder.auto_devops_deploy_strategy(strategy);
                }
            }
            if let Some(repository_storage) = params.repository_storage {
                builder.repository_storage(repository_storage);
            }
            if let Some(approvals_before_merge) = params.approvals_before_merge {
                builder.approvals_before_merge(approvals_before_merge);
            }
            if let Some(external_authorization_classification_label) =
                params.external_authorization_classification_label
            {
                builder.external_authorization_classification_label(
                    external_authorization_classification_label,
                );
            }
            if let Some(mirror) = params.mirror {
                builder.mirror(mirror);
            }
            if let Some(mirror_trigger_builds) = params.mirror_trigger_builds {
                builder.mirror_trigger_builds(mirror_trigger_builds);
            }
            if let Some(template_name) = params.template_name {
                builder.template_name(template_name);
            }
            if let Some(template_project_id) = params.template_project_id {
                builder.template_project_id(template_project_id);
            }
            if let Some(use_custom_template) = params.use_custom_template {
                builder.use_custom_template(use_custom_template);
            }
            if let Some(group_with_project_templates_id) = params.group_with_project_templates_id {
                builder.group_with_project_templates_id(group_with_project_templates_id);
            }
            if let Some(packages_enabled) = params.packages_enabled {
                builder.packages_enabled(packages_enabled);
            }
        }

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Create a new file in repository
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::files::CreateFile.query()` instead"
    )]
    #[allow(deprecated)]
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
        Ok(projects::repository::files::CreateFile::builder()
            .project(project.value())
            .file_path(file_path.as_ref())
            .branch(branch.as_ref())
            .content(content.as_ref().as_bytes())
            .commit_message(commit_message.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Set project description
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::EditProject.query()` instead"
    )]
    pub fn set_project_description<T: AsRef<str>>(
        &self,
        project: ProjectId,
        description: T,
    ) -> GitlabResult<Project> {
        Ok(projects::EditProject::builder()
            .project(project.value())
            .description(description.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Set project default branch
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::EditProject.query()` instead"
    )]
    pub fn set_project_default_branch<T: AsRef<str>>(
        &self,
        project: ProjectId,
        branch: T,
    ) -> GitlabResult<Project> {
        Ok(projects::EditProject::builder()
            .project(project.value())
            .default_branch(branch.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Set project features access level
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::EditProject.query()` instead"
    )]
    #[allow(deprecated)]
    pub fn set_project_feature_access_level(
        &self,
        project: ProjectId,
        feature: ProjectFeatures,
    ) -> GitlabResult<Project> {
        let mut builder = projects::EditProject::builder();
        builder.project(project.value());

        let convert = |level| {
            match level {
                FeatureVisibilityLevel::Disabled => projects::FeatureAccessLevel::Disabled,
                FeatureVisibilityLevel::Private => projects::FeatureAccessLevel::Private,
                FeatureVisibilityLevel::Enabled | FeatureVisibilityLevel::Public => {
                    projects::FeatureAccessLevel::Enabled
                },
            }
        };

        match feature {
            ProjectFeatures::Issues(level) => builder.issues_access_level(convert(level)),
            ProjectFeatures::Repository(level) => builder.repository_access_level(convert(level)),
            ProjectFeatures::MergeRequests(level) => {
                builder.merge_requests_access_level(convert(level))
            },
            ProjectFeatures::Builds(level) => builder.builds_access_level(convert(level)),
            ProjectFeatures::Wiki(level) => builder.wiki_access_level(convert(level)),
            ProjectFeatures::Snippets(level) => builder.snippets_access_level(convert(level)),
        };

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Get all accessible projects.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::Projects.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::Projects.query()` instead"
    )]
    pub fn owned_projects(&self) -> GitlabResult<Vec<Project>> {
        Ok(api::paged(
            projects::Projects::builder().owned(true).build().unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Find a project by id.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::Project.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::Project.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::environments::Environments.query()` instead"
    )]
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

    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::environments::Environment.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::groups::CreateGroup.query()` instead"
    )]
    pub fn create_group<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: P,
        params: Option<CreateGroupParams>,
    ) -> GitlabResult<Group> {
        let mut builder = groups::CreateGroup::builder();

        builder.name(name.as_ref()).path(path.as_ref());

        if let Some(params) = params {
            let convert_vis = |level| {
                match level {
                    VisibilityLevel::Public => api::common::VisibilityLevel::Public,
                    VisibilityLevel::Internal => api::common::VisibilityLevel::Internal,
                    VisibilityLevel::Private => api::common::VisibilityLevel::Private,
                }
            };
            let convert_project_level = |level| {
                match level {
                    AccessLevel::Admin | AccessLevel::Owner => {
                        warn!(
                            target: "gitlab",
                            "project creation may not be limited to {:?}; setting to NoOne",
                            level,
                        );
                        Some(api::groups::GroupProjectCreationAccessLevel::NoOne)
                    },
                    AccessLevel::Maintainer => {
                        Some(api::groups::GroupProjectCreationAccessLevel::Maintainer)
                    },
                    AccessLevel::Developer => {
                        Some(api::groups::GroupProjectCreationAccessLevel::Developer)
                    },
                    AccessLevel::Reporter | AccessLevel::Guest | AccessLevel::Anonymous => {
                        warn!(
                            target: "gitlab",
                            "project creation may not be limited to {:?}; ignoring",
                            level,
                        );
                        None
                    },
                }
            };
            let convert_subgroup_level = |level| {
                match level {
                    AccessLevel::Admin => {
                        warn!(
                            target: "gitlab",
                            "subgroup creation may not be limited to administrators; downgrading to Owner",
                        );
                        Some(api::groups::SubgroupCreationAccessLevel::Owner)
                    },
                    AccessLevel::Owner => Some(api::groups::SubgroupCreationAccessLevel::Owner),
                    AccessLevel::Maintainer => {
                        Some(api::groups::SubgroupCreationAccessLevel::Maintainer)
                    },
                    AccessLevel::Developer
                    | AccessLevel::Reporter
                    | AccessLevel::Guest
                    | AccessLevel::Anonymous => {
                        warn!(
                            target: "gitlab",
                            "subgroup creation may not be limited to {:?}; ignoring",
                            level,
                        );
                        None
                    },
                }
            };

            if let Some(description) = params.description {
                builder.description(description);
            }
            if let Some(visibility) = params.visibility {
                builder.visibility(convert_vis(visibility));
            }
            if let Some(share_with_group_lock) = params.share_with_group_lock {
                builder.share_with_group_lock(share_with_group_lock);
            }
            if let Some(require_two_factor_authentication) =
                params.require_two_factor_authentication
            {
                builder.require_two_factor_authentication(require_two_factor_authentication);
            }
            if let Some(project_creation_level) = params.project_creation_level {
                if let Some(level) = convert_project_level(project_creation_level) {
                    builder.project_creation_level(level);
                }
            }
            if let Some(auto_devops_enabled) = params.auto_devops_enabled {
                builder.auto_devops_enabled(auto_devops_enabled);
            }
            if let Some(subgroup_creation_level) = params.subgroup_creation_level {
                if let Some(level) = convert_subgroup_level(subgroup_creation_level) {
                    builder.subgroup_creation_level(level);
                }
            }
            if let Some(emails_disabled) = params.emails_disabled {
                builder.emails_disabled(emails_disabled);
            }
            if let Some(mentions_disabled) = params.mentions_disabled {
                builder.mentions_disabled(mentions_disabled);
            }
            if let Some(lfs_enabled) = params.lfs_enabled {
                builder.lfs_enabled(lfs_enabled);
            }
            if let Some(request_access_enabled) = params.request_access_enabled {
                builder.request_access_enabled(request_access_enabled);
            }
            if let Some(parent_id) = params.parent_id {
                builder.parent_id(parent_id.value());
            }
            if let Some(shared_runners_minutes_limit) = params.shared_runners_minutes_limit {
                builder.shared_runners_minutes_limit(shared_runners_minutes_limit);
            }
            if let Some(extra_shared_runners_minutes_limit) =
                params.extra_shared_runners_minutes_limit
            {
                builder.extra_shared_runners_minutes_limit(extra_shared_runners_minutes_limit);
            }
        }

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Get all accessible groups.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::groups::Groups.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::groups::Group.query()` instead"
    )]
    pub fn group_by_name<N>(&self, name: N) -> GitlabResult<Group>
    where
        N: AsRef<str>,
    {
        Ok(groups::Group::builder()
            .group(name.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get a project's hooks.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::hooks::Hooks.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::hooks::Hook.query()` instead"
    )]
    pub fn hook<I, K, V>(&self, project: ProjectId, hook: HookId, _: I) -> GitlabResult<ProjectHook>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(projects::hooks::Hook::builder()
            .project(project.value())
            .hook(hook.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Add a project hook.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::hooks::CreateHook.query()` instead"
    )]
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
        let mut builder = projects::hooks::CreateHook::builder();

        builder
            .project(project.value())
            .url(url.as_ref())
            .job_events(events.job())
            .issues_events(events.issues())
            .confidential_issues_events(events.confidential_issues())
            .merge_requests_events(events.merge_requests())
            .note_events(events.note())
            .pipeline_events(events.pipeline())
            .push_events(events.push())
            .wiki_page_events(events.wiki_page());

        if let Some(enable_ssl_verification) = enable_ssl_verification {
            builder.enable_ssl_verification(enable_ssl_verification);
        }
        if let Some(token) = token.as_ref() {
            builder.token(token.as_ref());
        }

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Get the team members of a group.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::groups::members::GroupMembers.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::groups::members::GroupMember.query()` instead"
    )]
    pub fn group_member<I, K, V>(&self, group: GroupId, user: UserId, _: I) -> GitlabResult<Member>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(groups::members::GroupMember::builder()
            .group(group.value())
            .user(user.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the team members of a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::members::ProjectMembers.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::members::ProjectMember.query()` instead"
    )]
    pub fn project_member<I, K, V>(
        &self,
        project: ProjectId,
        user: UserId,
        _: I,
    ) -> GitlabResult<Member>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(projects::members::ProjectMember::builder()
            .project(project.value())
            .user(user.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Add a user to a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::members::AddProjectMember.query()` instead"
    )]
    pub fn add_user_to_project(
        &self,
        project: ProjectId,
        user: UserId,
        access: AccessLevel,
    ) -> GitlabResult<Member> {
        Ok(projects::members::AddProjectMember::builder()
            .project(project.value())
            .user(user.value())
            .access_level(match access {
                AccessLevel::Anonymous => common::AccessLevel::Anonymous,
                AccessLevel::Guest => common::AccessLevel::Guest,
                AccessLevel::Reporter => common::AccessLevel::Reporter,
                AccessLevel::Developer => common::AccessLevel::Developer,
                AccessLevel::Maintainer => common::AccessLevel::Maintainer,
                AccessLevel::Owner => common::AccessLevel::Owner,
                AccessLevel::Admin => common::AccessLevel::Admin,
            })
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Add a user to a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::members::AddProjectMember.query()` instead"
    )]
    pub fn add_user_to_project_by_name<P>(
        &self,
        project: P,
        user: UserId,
        access: AccessLevel,
    ) -> GitlabResult<Member>
    where
        P: AsRef<str>,
    {
        Ok(projects::members::AddProjectMember::builder()
            .project(project.as_ref())
            .user(user.value())
            .access_level(match access {
                AccessLevel::Anonymous => common::AccessLevel::Anonymous,
                AccessLevel::Guest => common::AccessLevel::Guest,
                AccessLevel::Reporter => common::AccessLevel::Reporter,
                AccessLevel::Developer => common::AccessLevel::Developer,
                AccessLevel::Maintainer => common::AccessLevel::Maintainer,
                AccessLevel::Owner => common::AccessLevel::Owner,
                AccessLevel::Admin => common::AccessLevel::Admin,
            })
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Create a branch for a project
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::branches::CreateBranch.query()` instead"
    )]
    pub fn create_branch<V: AsRef<str>>(
        &self,
        project: ProjectId,
        name: V,
        reference: V,
    ) -> GitlabResult<RepoBranch> {
        Ok(projects::repository::branches::CreateBranch::builder()
            .project(project.value())
            .branch(name.as_ref())
            .ref_(reference.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get branches for a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::branches::Branches.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::branches::Branch.query()` instead"
    )]
    pub fn branch<B, I, K, V>(
        &self,
        project: ProjectId,
        branch: B,
        _: I,
    ) -> GitlabResult<RepoBranch>
    where
        B: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(projects::repository::branches::Branch::builder()
            .project(project.value())
            .branch(branch.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Protect a branch
    ///
    /// # Arguments
    /// * project: The project id
    /// * branch: The name of the branch or wildcard
    /// * push_access_level: Access level allowed to push (defaults: maintainers)
    /// * merge_access_level: Access level allowed to merge (defaults:  maintainers)
    /// * unprotect_access_level: Access level allowed to unproctect (defaults: maintainers)
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::protected_branches::ProtectBranch.query()` instead"
    )]
    pub fn protect_branch<B: AsRef<str>>(
        &self,
        project: ProjectId,
        branch: B,
        push_access_level: Option<AccessLevel>,
        merge_access_level: Option<AccessLevel>,
        unprotect_access_level: Option<AccessLevel>,
    ) -> GitlabResult<ProtectedRepoBranch> {
        let mut builder = projects::protected_branches::ProtectBranch::builder();

        let convert = |level| {
            match level {
                AccessLevel::Anonymous | AccessLevel::Guest | AccessLevel::Reporter => {
                    projects::protected_branches::ProtectedAccessLevel::NoAccess
                },
                AccessLevel::Developer => {
                    projects::protected_branches::ProtectedAccessLevel::Developer
                },
                AccessLevel::Maintainer | AccessLevel::Owner => {
                    projects::protected_branches::ProtectedAccessLevel::Maintainer
                },
                AccessLevel::Admin => projects::protected_branches::ProtectedAccessLevel::Admin,
            }
        };

        builder.project(project.value()).name(branch.as_ref());

        if let Some(push) = push_access_level {
            builder.push_access_level(convert(push));
        }
        if let Some(merge) = merge_access_level {
            builder.merge_access_level(convert(merge));
        }
        if let Some(unprotect) = unprotect_access_level {
            builder.unprotect_access_level(convert(unprotect));
        }

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Unprotect a branch
    ///
    /// # Arguments
    /// * project: The project id
    /// * branch: The name of the branch or wildcard
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::protected_branches::UnprotectBranch.query()` instead"
    )]
    pub fn unprotect_branch<B: AsRef<str>>(
        &self,
        project: ProjectId,
        branch: B,
    ) -> GitlabResult<()> {
        Ok(projects::protected_branches::UnprotectBranch::builder()
            .project(project.value())
            .name(branch.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get a commit.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::Commit.query()` instead"
    )]
    pub fn commit<C>(&self, project: ProjectId, commit: C) -> GitlabResult<RepoCommitDetail>
    where
        C: AsRef<str>,
    {
        Ok(projects::repository::commits::Commit::builder()
            .project(project.value())
            .commit(commit.as_ref())
            .stats(true)
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get comments on a commit.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CommitComments.query()` instead"
    )]
    pub fn commit_comments<C, I, K, V>(
        &self,
        project: ProjectId,
        commit: C,
        _: I,
    ) -> GitlabResult<Vec<CommitNote>>
    where
        C: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(api::paged(
            projects::repository::commits::CommitComments::builder()
                .project(project.value())
                .commit(commit.as_ref())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Get comments on a commit.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CommentOnCommit.query()` instead"
    )]
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
        Ok(projects::repository::commits::CommentOnCommit::builder()
            .project(project.value())
            .commit(commit.as_ref())
            .note(body.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get comments on a commit.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CommentOnCommit.query()` instead"
    )]
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
        Ok(projects::repository::commits::CommentOnCommit::builder()
            .project(project.as_ref())
            .commit(commit.as_ref())
            .note(body.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get comments on a commit.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CommentOnCommit.query()` instead"
    )]
    pub fn create_commit_line_comment(
        &self,
        project: ProjectId,
        commit: &str,
        body: &str,
        path: &str,
        line: u64,
    ) -> GitlabResult<CommitNote> {
        Ok(projects::repository::commits::CommentOnCommit::builder()
            .project(project.value())
            .commit(commit)
            .note(body)
            .path(path)
            .line(line)
            .line_type(projects::repository::commits::LineType::New)
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the latest statuses of a commit.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CommitStatuses.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CommitStatuses.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CommitStatuses.query()` instead"
    )]
    pub fn commit_all_statuses<C>(
        &self,
        project: ProjectId,
        commit: C,
    ) -> GitlabResult<Vec<CommitStatus>>
    where
        C: AsRef<str>,
    {
        Ok(projects::repository::commits::CommitStatuses::builder()
            .project(project.value())
            .commit(commit.as_ref())
            .all(true)
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the latest builds of a commit.
    #[deprecated(since = "0.1210.1", note = "deprecated by GitLab")]
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
    #[deprecated(since = "0.1210.1", note = "deprecated by GitLab")]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CreateCommitStatus.query()` instead"
    )]
    #[allow(deprecated)]
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
        let mut builder = projects::repository::commits::CreateCommitStatus::builder();
        builder
            .project(project.value())
            .commit(sha.as_ref())
            .state(match state {
                StatusState::Pending => projects::repository::commits::CommitStatusState::Pending,
                StatusState::Running => projects::repository::commits::CommitStatusState::Running,
                StatusState::Success => projects::repository::commits::CommitStatusState::Success,
                StatusState::Failed => projects::repository::commits::CommitStatusState::Failed,
                StatusState::Canceled => projects::repository::commits::CommitStatusState::Canceled,
                StatusState::Created | StatusState::Skipped | StatusState::Manual => {
                    return Err(GitlabError::InvalidStatusState {
                        state,
                    });
                },
            });

        info.refname.map(|refname| builder.ref_(refname));
        info.name.map(|name| builder.name(name));
        info.target_url
            .map(|target_url| builder.target_url(target_url));
        info.description
            .map(|description| builder.description(description));

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Create a status message for a commit.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::repository::commits::CreateCommitStatus.query()` instead"
    )]
    #[allow(deprecated)]
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
        let mut builder = projects::repository::commits::CreateCommitStatus::builder();
        builder
            .project(project.as_ref())
            .commit(sha.as_ref())
            .state(match state {
                StatusState::Pending => projects::repository::commits::CommitStatusState::Pending,
                StatusState::Running => projects::repository::commits::CommitStatusState::Running,
                StatusState::Success => projects::repository::commits::CommitStatusState::Success,
                StatusState::Failed => projects::repository::commits::CommitStatusState::Failed,
                StatusState::Canceled => projects::repository::commits::CommitStatusState::Canceled,
                StatusState::Created | StatusState::Skipped | StatusState::Manual => {
                    return Err(GitlabError::InvalidStatusState {
                        state,
                    });
                },
            });

        info.refname.map(|refname| builder.ref_(refname));
        info.name.map(|name| builder.name(name));
        info.target_url
            .map(|target_url| builder.target_url(target_url));
        info.description
            .map(|description| builder.description(description));

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Get the labels for a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::labels::Labels.query()` instead"
    )]
    pub fn labels(&self, project: ProjectId) -> GitlabResult<Vec<Label>> {
        Ok(projects::labels::Labels::builder()
            .project(project.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the labels with open/closed/merge requests count
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::labels::Labels.query()` instead"
    )]
    pub fn labels_with_counts(&self, project: ProjectId) -> GitlabResult<Vec<Label>> {
        Ok(projects::labels::Labels::builder()
            .project(project.value())
            .with_counts(true)
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get label by ID.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::labels::Label.query()` instead"
    )]
    pub fn label(&self, project: ProjectId, label: LabelId) -> GitlabResult<Label> {
        Ok(projects::labels::Label::builder()
            .project(project.value())
            .label(label.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the issues for a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::issues::Issues.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::issues::Issue.query()` instead"
    )]
    pub fn issue<I, K, V>(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
        _: I,
    ) -> GitlabResult<Issue>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(projects::issues::Issue::builder()
            .project(project.value())
            .issue(issue.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the notes from a issue.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::issues::notes::IssueNotes.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::issues::notes::IssueNotes.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::labels::CreateLabel.query()` instead"
    )]
    pub fn create_label(&self, project: ProjectId, label: Label) -> GitlabResult<Label> {
        let mut builder = projects::labels::CreateLabel::builder();

        builder
            .project(project.value())
            .name(label.name)
            .color(label.color.value());

        if let Some(description) = label.description {
            builder.description(description);
        }
        if let Some(priority) = label.priority {
            builder.priority(priority);
        }

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Create a new milestone
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::{groups,projects}::milestones::Create{Group,Project}Milestone.query()` instead"
    )]
    #[allow(deprecated)]
    pub fn create_milestone(&self, milestone: Milestone) -> GitlabResult<Milestone> {
        if let Some(project) = milestone.project_id {
            let mut builder = projects::milestones::CreateProjectMilestone::builder();

            builder.project(project.value()).title(milestone.title);

            if let Some(d) = milestone.description {
                builder.description(d);
            }
            if let Some(d) = milestone.due_date {
                builder.due_date(d);
            }
            if let Some(s) = milestone.start_date {
                builder.start_date(s);
            }

            Ok(builder.build().unwrap().query(self)?)
        } else if let Some(group) = milestone.group_id {
            let mut builder = groups::milestones::CreateGroupMilestone::builder();

            builder.group(group.value()).title(milestone.title);

            if let Some(d) = milestone.description {
                builder.description(d);
            }
            if let Some(d) = milestone.due_date {
                builder.due_date(d);
            }
            if let Some(s) = milestone.start_date {
                builder.start_date(s);
            }

            Ok(builder.build().unwrap().query(self)?)
        } else {
            Err(GitlabError::InvalidMilestone)
        }
    }

    /// Create a new issue
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::issues::CreateIssue.query()` instead"
    )]
    pub fn create_issue(&self, project: ProjectId, issue: Issue) -> GitlabResult<Issue> {
        let mut builder = projects::issues::CreateIssue::builder();

        builder
            .project(project.value())
            .title(issue.title)
            .confidential(issue.confidential)
            .created_at(issue.created_at);

        if issue.iid.value() != 0 {
            builder.iid(issue.iid.value());
        }
        if let Some(description) = issue.description {
            builder.description(description);
        }
        if let Some(assignees) = issue.assignees {
            builder.assignee_ids(assignees.into_iter().map(|u| u.id.value()));
        }
        if let Some(milestone) = issue.milestone {
            builder.milestone_id(milestone.id.value());
        }
        if !issue.labels.is_empty() {
            builder.labels(issue.labels);
        }
        if let Some(due_date) = issue.due_date {
            builder.due_date(due_date);
        }

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Get the resource label events from an issue.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::issues::IssueResourceLabelEvents.query()` instead"
    )]
    pub fn issue_label_events(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
    ) -> GitlabResult<Vec<ResourceLabelEvent>> {
        Ok(api::paged(
            projects::issues::IssueResourceLabelEvents::builder()
                .project(project.value())
                .issue(issue.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Create a note on a issue.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::issues::notes::CreateIssueNote.query()` instead"
    )]
    pub fn create_issue_note<C>(
        &self,
        project: ProjectId,
        issue: IssueInternalId,
        content: C,
    ) -> GitlabResult<Note>
    where
        C: AsRef<str>,
    {
        Ok(projects::issues::notes::CreateIssueNote::builder()
            .project(project.value())
            .issue(issue.value())
            .body(content.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Create a note on a issue.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::issues::notes::CreateIssueNote.query()` instead"
    )]
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
        Ok(projects::issues::notes::CreateIssueNote::builder()
            .project(project.as_ref())
            .issue(issue.value())
            .body(content.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Edit a note on an issue.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::issues::notes::EditIssueNote.query()` instead"
    )]
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
        Ok(projects::issues::notes::EditIssueNote::builder()
            .project(project.value())
            .issue(issue.value())
            .note(note.value())
            .body(content.as_ref())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the merge requests for a project.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::MergeRequests.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::MergeRequests.query()` instead"
    )]
    #[allow(deprecated)]
    pub fn merge_requests_with_state(
        &self,
        project: ProjectId,
        state: MergeRequestStateFilter,
    ) -> GitlabResult<Vec<MergeRequest>> {
        let convert = |state| {
            match state {
                MergeRequestStateFilter::Opened => {
                    projects::merge_requests::MergeRequestState::Opened
                },
                MergeRequestStateFilter::Closed => {
                    projects::merge_requests::MergeRequestState::Closed
                },
                MergeRequestStateFilter::Merged => {
                    projects::merge_requests::MergeRequestState::Merged
                },
            }
        };

        Ok(api::paged(
            projects::merge_requests::MergeRequests::builder()
                .project(project.value())
                .state(convert(state))
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Create a new merge request
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::CreateMergeRequest.query()` instead"
    )]
    pub fn create_merge_request(
        &self,
        project: ProjectId,
        params: CreateMergeRequestParams,
    ) -> GitlabResult<MergeRequest> {
        let mut builder = projects::merge_requests::CreateMergeRequest::builder();

        builder
            .project(project.value())
            .source_branch(params.source_branch)
            .target_branch(params.target_branch)
            .title(params.title);

        if let Some(assignee_id) = params.assignee_id {
            builder.assignee(assignee_id.value());
        }
        if let Some(assignee_ids) = params.assignee_ids {
            if assignee_ids.is_empty() {
                builder.unassigned();
            } else {
                builder.assignees(assignee_ids.into_iter().map(|id| id.value()));
            }
        }
        if let Some(description) = params.description {
            builder.description(description);
        }
        if let Some(target_project_id) = params.target_project_id {
            builder.target_project_id(target_project_id.value());
        }
        let labels_save;
        if let Some(labels) = params.labels {
            labels_save = labels;
            builder.labels(labels_save.split(','));
        }
        if let Some(milestone_id) = params.milestone_id {
            builder.milestone_id(milestone_id.value());
        }
        if let Some(remove_source_branch) = params.remove_source_branch {
            builder.remove_source_branch(remove_source_branch);
        }
        if let Some(allow_collaboration) = params.allow_collaboration {
            builder.allow_collaboration(allow_collaboration);
        }
        if let Some(squash) = params.squash {
            builder.squash(squash);
        }

        Ok(builder.build().unwrap().query(self)?)
    }

    /// Get all pipelines for a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipelines::Pipelines.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipeline::Pipeline.query()` instead"
    )]
    pub fn pipeline(&self, project: ProjectId, id: PipelineId) -> GitlabResult<Pipeline> {
        Ok(pipelines::Pipeline::builder()
            .project(project.value())
            .pipeline(id.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get variables of a pipeline.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipelines::PipelineVariables.query()` instead"
    )]
    pub fn pipeline_variables(
        &self,
        project: ProjectId,
        id: PipelineId,
    ) -> GitlabResult<Vec<PipelineVariable>> {
        Ok(api::paged(
            projects::pipelines::PipelineVariables::builder()
                .project(project.value())
                .pipeline(id.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Create a new pipeline.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipelines::CreatePipeline.query()` instead"
    )]
    pub fn create_pipeline(
        &self,
        project: ProjectId,
        ref_: ObjectId,
        variables: &[PipelineVariable],
    ) -> GitlabResult<Pipeline> {
        Ok(pipelines::CreatePipeline::builder()
            .project(project.value())
            .ref_(ref_.value().as_str())
            .variables(variables.iter().map(|variable| {
                pipelines::PipelineVariable::builder()
                    .key(variable.key.as_str())
                    .value(variable.value.as_str())
                    .variable_type(match variable.variable_type {
                        PipelineVariableType::EnvVar => pipelines::PipelineVariableType::EnvVar,
                        PipelineVariableType::File => pipelines::PipelineVariableType::File,
                    })
                    .build()
                    .unwrap()
            }))
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Retry jobs in a pipeline.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipelines::RetryPipeline.query()` instead"
    )]
    pub fn retry_pipeline(&self, project: ProjectId, id: PipelineId) -> GitlabResult<Pipeline> {
        Ok(pipelines::RetryPipeline::builder()
            .project(project.value())
            .pipeline(id.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Cancel a pipeline.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipelines::CancelPipeline.query()` instead"
    )]
    pub fn cancel_pipeline(&self, project: ProjectId, id: PipelineId) -> GitlabResult<Pipeline> {
        Ok(pipelines::CancelPipeline::builder()
            .project(project.value())
            .pipeline(id.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get a list of jobs for a pipeline.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipelines::jobs::Jobs.query()` instead"
    )]
    pub fn pipeline_jobs<I, K, V>(
        &self,
        project: ProjectId,
        pipeline_id: PipelineId,
        params: I,
    ) -> GitlabResult<Vec<Job>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.get_paged_with_param(
            format!("projects/{}/pipelines/{}/jobs", project, pipeline_id),
            params,
        )
    }

    /// Get a log for a specific job of a project.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::pipelines::jobs::JobTrace.query()` instead"
    )]
    pub fn job_log(&self, project: ProjectId, job_id: JobId) -> GitlabResult<Vec<u8>> {
        Ok(api::raw(
            projects::jobs::JobTrace::builder()
                .project(project.value())
                .job(job_id.value())
                .build()
                .unwrap(),
        )
        .query(self)?)
    }

    /// Get a single merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::MergeRequest.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::IssuesClosedBy.query()` instead"
    )]
    pub fn merge_request_closes_issues<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        _: I,
    ) -> GitlabResult<Vec<IssueReference>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(projects::merge_requests::IssuesClosedBy::builder()
            .project(project.value())
            .merge_request(merge_request.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get the discussions from a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::MergeRequestDiscussions.query()` instead"
    )]
    pub fn merge_request_discussions<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        _: I,
    ) -> GitlabResult<Vec<Discussion>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(api::paged(
            projects::merge_requests::discussions::MergeRequestDiscussions::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Get the notes from a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::MergeRequestNotes.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::MergeRequestNotes.query()` instead"
    )]
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
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::awards::CreateMergeRequestNoteAward.query()` instead"
    )]
    pub fn award_merge_request_note(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        award: &str,
    ) -> GitlabResult<AwardEmoji> {
        Ok(
            projects::merge_requests::notes::awards::CreateMergeRequestNoteAward::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .note(note.value())
                .name(award)
                .build()
                .unwrap()
                .query(self)?,
        )
    }

    /// Award a merge request note with an award.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::awards::CreateMergeRequestNoteAward.query()` instead"
    )]
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
        Ok(
            projects::merge_requests::notes::awards::CreateMergeRequestNoteAward::builder()
                .project(project.as_ref())
                .merge_request(merge_request.value())
                .note(note.value())
                .name(award)
                .build()
                .unwrap()
                .query(self)?,
        )
    }

    /// Get the awards for a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::awards::MergeRequestAwards.query()` instead"
    )]
    pub fn merge_request_awards<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        _: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(api::paged(
            projects::merge_requests::awards::MergeRequestAwards::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Get the awards for a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::awards::MergeRequestAwards.query()` instead"
    )]
    pub fn merge_request_awards_by_name<P, I, K, V>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        _: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(api::paged(
            projects::merge_requests::awards::MergeRequestAwards::builder()
                .project(project.as_ref())
                .merge_request(merge_request.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Get the awards for a merge request note.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::awards::MergeRequestNoteAwards.query()` instead"
    )]
    pub fn merge_request_note_awards<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        _: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(api::paged(
            projects::merge_requests::notes::awards::MergeRequestNoteAwards::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .note(note.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Get the awards for a merge request note.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::awards::MergeRequestNoteAwards.query()` instead"
    )]
    pub fn merge_request_note_awards_by_name<P, I, K, V>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        note: NoteId,
        _: I,
    ) -> GitlabResult<Vec<AwardEmoji>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(api::paged(
            projects::merge_requests::notes::awards::MergeRequestNoteAwards::builder()
                .project(project.as_ref())
                .merge_request(merge_request.value())
                .note(note.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    /// Get the resource label events from a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::MergeRequestResourceLabelEvents.query()` instead"
    )]
    pub fn merge_request_label_events(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
    ) -> GitlabResult<Vec<ResourceLabelEvent>> {
        Ok(api::paged(
            projects::merge_requests::MergeRequestResourceLabelEvents::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .build()
                .unwrap(),
            api::Pagination::All,
        )
        .query(self)?)
    }

    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::discussions::CreateMergeRequestDiscussion.query()` instead"
    )]
    pub fn create_merge_request_discussion(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        content: &str,
    ) -> GitlabResult<Discussion> {
        Ok(
            projects::merge_requests::discussions::CreateMergeRequestDiscussion::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .body(content)
                .build()
                .unwrap()
                .query(self)?,
        )
    }

    /// Create a note on a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::CreateMergeRequestNote.query()` instead"
    )]
    pub fn create_merge_request_note(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        content: &str,
    ) -> GitlabResult<Note> {
        Ok(
            projects::merge_requests::notes::CreateMergeRequestNote::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .body(content)
                .build()
                .unwrap()
                .query(self)?,
        )
    }

    /// Create a note on a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::CreateMergeRequestNote.query()` instead"
    )]
    pub fn create_merge_request_note_by_name<P>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        content: &str,
    ) -> GitlabResult<Note>
    where
        P: AsRef<str>,
    {
        Ok(
            projects::merge_requests::notes::CreateMergeRequestNote::builder()
                .project(project.as_ref())
                .merge_request(merge_request.value())
                .body(content)
                .build()
                .unwrap()
                .query(self)?,
        )
    }

    /// Edit a note on a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::EditMergeRequestNote.query()` instead"
    )]
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
        Ok(
            projects::merge_requests::notes::EditMergeRequestNote::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .note(note.value())
                .body(content.as_ref())
                .build()
                .unwrap()
                .query(self)?,
        )
    }

    /// Edit a note on a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::notes::EditMergeRequestNote.query()` instead"
    )]
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
        Ok(
            projects::merge_requests::notes::EditMergeRequestNote::builder()
                .project(project.value())
                .merge_request(merge_request.value())
                .note(note.value())
                .body(content.as_ref())
                .build()
                .unwrap()
                .query(self)?,
        )
    }

    /// Get issues closed by a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::IssuesClosedBy.query()` instead"
    )]
    pub fn get_issues_closed_by_merge_request<I, K, V>(
        &self,
        project: ProjectId,
        merge_request: MergeRequestInternalId,
        _: I,
    ) -> GitlabResult<Vec<Issue>>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(projects::merge_requests::IssuesClosedBy::builder()
            .project(project.value())
            .merge_request(merge_request.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Get issues closed by a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::IssuesClosedBy.query()` instead"
    )]
    pub fn get_issues_closed_by_merge_request_by_name<P, I, K, V>(
        &self,
        project: P,
        merge_request: MergeRequestInternalId,
        _: I,
    ) -> GitlabResult<Vec<Issue>>
    where
        P: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(projects::merge_requests::IssuesClosedBy::builder()
            .project(project.as_ref())
            .merge_request(merge_request.value())
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Closes an issue
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::issues::EditIssue.query()` instead"
    )]
    pub fn close_issue(&self, project: ProjectId, issue: IssueInternalId) -> GitlabResult<Issue> {
        Ok(projects::issues::EditIssue::builder()
            .project(project.value())
            .issue(issue.value())
            .state_event(projects::issues::IssueStateEvent::Close)
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Set the labels on an issue.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::issues::EditIssue.query()` instead"
    )]
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
        Ok(projects::issues::EditIssue::builder()
            .project(project.value())
            .issue(issue.value())
            .labels(labels.into_iter().map(|label| format!("{}", label)))
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Set the labels on an issue.
    #[deprecated(
        since = "0.1210.1",
        note = "use `gitlab::api::projects::issues::EditIssue.query()` instead"
    )]
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
        Ok(projects::issues::EditIssue::builder()
            .project(project.as_ref())
            .issue(issue.value())
            .labels(labels.into_iter().map(|label| format!("{}", label)))
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Set the labels on a merge request.
    #[deprecated(
        since = "0.1300.0",
        note = "use `gitlab::api::projects::merge_requests::EditMergeRequest.query()` instead"
    )]
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
        Ok(projects::merge_requests::EditMergeRequest::builder()
            .project(project.value())
            .merge_request(merge_request.value())
            .labels(labels.into_iter().map(|l| format!("{}", l)))
            .build()
            .unwrap()
            .query(self)?)
    }

    /// Create a URL to an API endpoint.
    fn create_url<U>(&self, url: U) -> GitlabResult<Url>
    where
        U: AsRef<str>,
    {
        debug!(target: "gitlab", "api call {}", url.as_ref());
        Ok(self.rest_url.join(url.as_ref())?)
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
    fn send_impl(&self, req: reqwest::blocking::RequestBuilder) -> GitlabResult<ReqwestResponse> {
        let auth_headers = {
            let mut headers = HeaderMap::default();
            self.auth.set_header(&mut headers)?;
            headers
        };
        let rsp = req.headers(auth_headers).send()?;
        let status = rsp.status();
        if status.is_server_error() {
            return Err(GitlabError::http(status));
        }

        Ok(rsp)
    }

    /// Refactored code which talks to Gitlab and transforms error messages properly.
    fn send<T>(&self, req: reqwest::blocking::RequestBuilder) -> GitlabResult<T>
    where
        T: DeserializeOwned,
    {
        let rsp = self.send_impl(req)?;
        let status = rsp.status();
        let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
        if !status.is_success() {
            return Err(GitlabError::from_gitlab(v));
        }

        debug!(target: "gitlab", "received data: {:?}", v);
        serde_json::from_value::<T>(v).map_err(GitlabError::data_type::<T>)
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

#[derive(Debug, Error)]
// TODO #[non_exhaustive]
pub enum RestError {
    #[error("error setting auth header: {}", source)]
    AuthError {
        #[from]
        source: AuthError,
    },
    #[error("communication with gitlab: {}", source)]
    Communication {
        #[from]
        source: reqwest::Error,
    },
    #[error("`http` error: {}", source)]
    Http {
        #[from]
        source: http::Error,
    },
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
}

impl api::Client for Gitlab {
    type Error = RestError;

    fn rest_endpoint(&self, endpoint: &str) -> Result<Url, api::ApiError<Self::Error>> {
        debug!(target: "gitlab", "REST api call {}", endpoint);
        Ok(self.rest_url.join(endpoint)?)
    }

    fn rest(
        &self,
        mut request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<HttpResponse<Bytes>, api::ApiError<Self::Error>> {
        let call = || -> Result<_, RestError> {
            self.auth.set_header(request.headers_mut().unwrap())?;
            let http_request = request.body(body)?;
            let request = http_request.try_into()?;
            let rsp = self.client.execute(request)?;

            let mut http_rsp = HttpResponse::builder()
                .status(rsp.status())
                .version(rsp.version());
            let headers = http_rsp.headers_mut().unwrap();
            for (key, value) in rsp.headers() {
                headers.insert(key, value.clone());
            }
            Ok(http_rsp.body(rsp.bytes()?)?)
        };
        call().map_err(api::ApiError::client)
    }
}

pub struct GitlabBuilder {
    protocol: &'static str,
    host: String,
    token: Auth,
    cert_validation: CertPolicy,
}

impl GitlabBuilder {
    /// Create a new Gitlab API client builder.
    pub fn new<H, T>(host: H, token: T) -> Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        Self {
            protocol: "https",
            host: host.into(),
            token: Auth::Token(token.into()),
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
        if let Auth::Token(token) = self.token.clone() {
            self.token = Auth::OAuth2(token);
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
