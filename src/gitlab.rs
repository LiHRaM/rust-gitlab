// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crates::itertools::Itertools;
use crates::reqwest::{Client, Method, RequestBuilder, Url};
use crates::serde::{Deserialize, Deserializer, Serializer};
use crates::serde::de::Error as SerdeError;
use crates::serde::de::{DeserializeOwned, Unexpected};
use crates::serde::ser::Serialize;
use crates::serde_json;
use crates::url::percent_encoding::{PATH_SEGMENT_ENCODE_SET, percent_encode};

use error::*;
use types::*;

use std::borrow::Borrow;
use std::fmt::{self, Display, Debug};

/// A representation of the Gitlab API for a single user.
///
/// Separate users should use separate instances of this.
pub struct Gitlab {
    /// The base URL to use for API calls.
    base_url: Url,
    /// The secret token to use when communicating with Gitlab.
    token: String,
}

impl Debug for Gitlab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Gitlab")
            .field("base_url", &self.base_url)
            .finish()
    }
}

// The header Gitlab uses to authenticate the user.
header!{ (GitlabPrivateToken, "PRIVATE-TOKEN") => [String] }

#[derive(Debug)]
/// Optional information for commit statuses.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Optional information for merge requests.
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

impl Gitlab {
    /// Create a new Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new<T: ToString>(host: &str, token: T) -> Result<Self> {
        Self::_new("https", host, token.to_string())
    }

    /// Create a new non-SSL Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new_insecure<T: ToString>(host: &str, token: T) -> Result<Self> {
        Self::_new("http", host, token.to_string())
    }

    /// Internal method to create a new Gitlab client.
    fn _new(protocol: &str, host: &str, token: String) -> Result<Self> {
        let base_url = Url::parse(&format!("{}://{}/api/v4/", protocol, host))
            .chain_err(|| ErrorKind::UrlParse)?;

        let api = Gitlab {
            base_url: base_url,
            token: token,
        };

        // Ensure the API is working.
        let _: UserPublic = api._get("user")?;

        Ok(api)
    }

    /// The user the API is acting as.
    pub fn current_user(&self) -> Result<UserPublic> {
        self._get("user")
    }

    /// Get all user accounts
    pub fn users<T: UserResult>(&self) -> Result<Vec<T>> {
        self._get_paged("users")
    }

    /// Find a user by id.
    pub fn user<T: UserResult>(&self, user: UserId) -> Result<T> {
        self._get(&format!("users/{}", user))
    }

    /// Find a user by username.
    pub fn user_by_name<T: UserResult>(&self, name: &str) -> Result<T> {
        let mut users = self._get_paged_with_param("users", &[("username", name)])?;
        users.pop()
            .ok_or_else(|| Error::from_kind(ErrorKind::Gitlab("no such user".to_string())))
    }

    /// Get all accessible projects.
    pub fn projects(&self) -> Result<Vec<Project>> {
        self._get_paged("projects")
    }

    /// Get all owned projects.
    pub fn owned_projects(&self) -> Result<Vec<Project>> {
        self._get_paged("projects/owned")
    }

    /// Get all projects.
    ///
    /// Requires administrator privileges.
    pub fn all_projects(&self) -> Result<Vec<Project>> {
        self._get_paged("projects/all")
    }

    /// Find a project by id.
    pub fn project(&self, project: ProjectId) -> Result<Project> {
        self._get(&format!("projects/{}", project))
    }

    /// Find a project by name.
    pub fn project_by_name(&self, name: &str) -> Result<Project> {
        self._get(&format!("projects/{}",
                           percent_encode(name.as_bytes(), PATH_SEGMENT_ENCODE_SET)))
    }

    /// Get a project's hooks.
    pub fn hooks(&self, project: ProjectId) -> Result<Vec<ProjectHook>> {
        self._get_paged(&format!("projects/{}/hooks", project))
    }

    /// Get a project hook.
    pub fn hook(&self, project: ProjectId, hook: HookId) -> Result<ProjectHook> {
        self._get(&format!("projects/{}/hooks/{}", project, hook))
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
        vec![("job_events", Self::bool_param_value(events.job())),
             ("issues_events", Self::bool_param_value(events.issues())),
             ("merge_requests_events", Self::bool_param_value(events.merge_requests())),
             ("note_events", Self::bool_param_value(events.note())),
             ("pipeline_events", Self::bool_param_value(events.pipeline())),
             ("push_events", Self::bool_param_value(events.push())),
             ("wiki_page_events", Self::bool_param_value(events.wiki_page()))]
    }

    /// Add a project hook.
    pub fn add_hook(&self, project: ProjectId, url: &str, events: WebhookEvents) -> Result<ProjectHook> {
        let mut flags = Self::event_flags(events);
        flags.push(("url", url));

        self._post_with_param(&format!("projects/{}/hooks", project), &flags)
    }

    /// Get the team members of a group.
    pub fn group_members(&self, group: GroupId) -> Result<Vec<Member>> {
        self._get_paged(&format!("groups/{}/members", group))
    }

    /// Get a team member of a group.
    pub fn group_member(&self, group: GroupId, user: UserId) -> Result<Member> {
        self._get(&format!("groups/{}/members/{}", group, user))
    }

    /// Get the team members of a project.
    pub fn project_members(&self, project: ProjectId) -> Result<Vec<Member>> {
        self._get_paged(&format!("projects/{}/members", project))
    }

    /// Get a team member of a project.
    pub fn project_member(&self, project: ProjectId, user: UserId) -> Result<Member> {
        self._get(&format!("projects/{}/members/{}", project, user))
    }

    /// Add a user to a project.
    pub fn add_user_to_project(&self, project: ProjectId, user: UserId, access: AccessLevel)
                               -> Result<Member> {
        let user_str = format!("{}", user);
        let access_str = format!("{}", access);

        self._post_with_param(&format!("projects/{}/members", project),
                              &[("user", &user_str), ("access", &access_str)])
    }

    /// Get branches for a project.
    pub fn branches(&self, project: ProjectId) -> Result<Vec<RepoBranch>> {
        self._get_paged(&format!("projects/{}/branches", project))
    }

    /// Get a branch.
    pub fn branch(&self, project: ProjectId, branch: &str) -> Result<RepoBranch> {
        self._get(&format!("projects/{}/repository/branches/{}",
                           project,
                           percent_encode(branch.as_bytes(), PATH_SEGMENT_ENCODE_SET)))
    }

    /// Get a commit.
    pub fn commit(&self, project: ProjectId, commit: &str) -> Result<RepoCommitDetail> {
        self._get(&format!("projects/{}/repository/commits/{}", project, commit))
    }

    /// Get comments on a commit.
    pub fn commit_comments(&self, project: ProjectId, commit: &str) -> Result<Vec<CommitNote>> {
        self._get_paged(&format!("projects/{}/repository/commits/{}/comments",
                                 project,
                                 commit))
    }

    /// Get comments on a commit.
    pub fn create_commit_comment(&self, project: ProjectId, commit: &str, body: &str)
                                 -> Result<CommitNote> {
        self._post_with_param(&format!("projects/{}/repository/commits/{}/comment",
                                       project,
                                       commit),
                              &[("note", body)])
    }

    /// Get comments on a commit.
    pub fn create_commit_line_comment(&self, project: ProjectId, commit: &str, body: &str,
                                      path: &str, line: u64)
                                      -> Result<CommitNote> {
        let line_str = format!("{}", line);
        let line_type = LineType::New;

        self._post_with_param(&format!("projects/{}/repository/commits/{}/comment",
                                       project,
                                       commit),
                              &[("note", body),
                                ("path", path),
                                ("line", &line_str),
                                ("line_type", line_type.as_str())])
    }

    /// Get the latest statuses of a commit.
    pub fn commit_latest_statuses(&self, project: ProjectId, commit: &str)
                                  -> Result<Vec<CommitStatus>> {
        self._get_paged(&format!("projects/{}/repository/commits/{}/statuses",
                                 project,
                                 commit))
    }

    /// Get the all statuses of a commit.
    pub fn commit_all_statuses(&self, project: ProjectId, commit: &str)
                               -> Result<Vec<CommitStatus>> {
        self._get_paged_with_param(&format!("projects/{}/repository/commits/{}/statuses",
                                            project,
                                            commit),
                                   &[("all", "true")])
    }

    /// Get the latest builds of a commit.
    pub fn commit_latest_builds(&self, project: ProjectId, commit: &str) -> Result<Vec<Job>> {
        self._get_paged(&format!("projects/{}/repository/commits/{}/builds", project, commit))
    }

    /// Get the all builds of a commit.
    pub fn commit_all_builds(&self, project: ProjectId, commit: &str) -> Result<Vec<Job>> {
        self._get_paged_with_param(&format!("projects/{}/repository/commits/{}/builds",
                                            project,
                                            commit),
                                   &[("all", "true")])
    }

    /// Create a status message for a commit.
    pub fn create_commit_status(&self, project: ProjectId, sha: &str, state: StatusState,
                                info: &CommitStatusInfo)
                                -> Result<CommitStatus> {
        let path = &format!("projects/{}/statuses/{}", project, sha);

        let mut params = vec![("state", state.as_str())];

        info.refname.map(|v| params.push(("ref", v)));
        info.name.map(|v| params.push(("name", v)));
        info.target_url.map(|v| params.push(("target_url", v)));
        info.description.map(|v| params.push(("description", v)));

        self._post_with_param(path, &params)
    }

    /// Get the issues for a project.
    pub fn issues(&self, project: ProjectId) -> Result<Vec<Issue>> {
        self._get_paged(&format!("projects/{}/issues", project))
    }

    /// Get issues.
    pub fn issue(&self, project: ProjectId, issue: IssueInternalId) -> Result<Issue> {
        self._get(&format!("projects/{}/issues/{}", project, issue))
    }

    /// Get the notes from a issue.
    pub fn issue_notes(&self, project: ProjectId, issue: IssueInternalId) -> Result<Vec<Note>> {
        self._get_paged(&format!("projects/{}/issues/{}/notes", project, issue))
    }

    /// Create a note on a issue.
    pub fn create_issue_note(&self, project: ProjectId, issue: IssueInternalId, content: &str)
                             -> Result<Note> {
        let path = &format!("projects/{}/issues/{}/notes", project, issue);

        self._post_with_param(path, &[("body", content)])
    }

    /// Get the merge requests for a project.
    pub fn merge_requests(&self, project: ProjectId) -> Result<Vec<MergeRequest>> {
        self._get_paged(&format!("projects/{}/merge_requests", project))
    }

    /// Get the merge requests with a given state.
    pub fn merge_requests_with_state(&self, project: ProjectId, state: MergeRequestStateFilter)
                                     -> Result<Vec<MergeRequest>> {
        self._get_paged_with_param(&format!("projects/{}/merge_requests", project),
                                   &[("state", state.as_str())])
    }

    /// Get merge requests.
    pub fn merge_request(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                         -> Result<MergeRequest> {
        self._get(&format!("projects/{}/merge_requests/{}", project, merge_request))
    }

    /// Get the issues that will be closed when a merge request is merged.
    pub fn merge_request_closes_issues(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                                       -> Result<Vec<IssueReference>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/closes_issues",
                                 project,
                                 merge_request))
    }

    /// Get the notes from a merge request.
    pub fn merge_request_notes(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                               -> Result<Vec<Note>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/notes",
                                 project,
                                 merge_request))
    }

    /// Award a merge request note with an award.
    pub fn award_merge_request_note(&self, project: ProjectId, merge_request: MergeRequestInternalId,
                                    note: NoteId, award: &str)
                                    -> Result<AwardEmoji> {
        let path = &format!("projects/{}/merge_requests/{}/notes/{}/award_emoji",
                            project,
                            merge_request,
                            note);
        self._post_with_param(path, &[("name", award)])
    }

    /// Get the awards for a merge request.
    pub fn merge_request_awards(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                                -> Result<Vec<AwardEmoji>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/award_emoji",
                                 project,
                                 merge_request))
    }

    /// Get the awards for a merge request note.
    pub fn merge_request_note_awards(&self, project: ProjectId, merge_request: MergeRequestInternalId,
                                     note: NoteId)
                                     -> Result<Vec<AwardEmoji>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/notes/{}/award_emoji",
                                 project,
                                 merge_request,
                                 note))
    }

    /// Create a note on a merge request.
    pub fn create_merge_request_note(&self, project: ProjectId, merge_request: MergeRequestInternalId,
                                     content: &str)
                                     -> Result<Note> {
        let path = &format!("projects/{}/merge_requests/{}/notes",
                            project,
                            merge_request);
        self._post_with_param(path, &[("body", content)])
    }

    /// Get issues closed by a merge request.
    pub fn get_issues_closed_by_merge_request(&self, project: ProjectId,
                                              merge_request: MergeRequestInternalId)
                                              -> Result<Vec<Issue>> {
        let path = &format!("projects/{}/merge_requests/{}/closes_issues",
                            project,
                            merge_request);
        self._get_paged(path)
    }

    /// Set the labels on an issue.
    pub fn set_issue_labels<I, L>(&self, project: ProjectId, issue: IssueInternalId, labels: I)
                                  -> Result<Issue>
        where I: IntoIterator<Item = L>,
              L: Display,
    {
        let path = &format!("projects/{}/issues/{}",
                            project,
                            issue);
        self._put_with_param(path, &[("labels", labels.into_iter().join(","))])
    }

    /// Set the labels on a merge request.
    pub fn set_merge_request_labels<I, L>(&self, project: ProjectId, merge_request: MergeRequestInternalId, labels: I)
                                  -> Result<MergeRequest>
        where I: IntoIterator<Item = L>,
              L: Display,
    {
        let path = &format!("projects/{}/merge_requests/{}",
                            project,
                            merge_request);
        self._put_with_param(path, &[("labels", labels.into_iter().join(","))])
    }

    /// Create a URL to an API endpoint.
    fn _mk_url(&self, url: &str) -> Result<Url> {
        debug!(target: "gitlab", "api call {}", url);
        Ok(self.base_url.join(url).chain_err(|| ErrorKind::UrlParse)?)
    }

    /// Create a URL to an API endpoint with query parameters.
    fn _mk_url_with_param<I, K, V>(&self, url: &str, param: I) -> Result<Url>
        where I: IntoIterator,
              I::Item: Borrow<(K, V)>,
              K: AsRef<str>,
              V: AsRef<str>,
    {
        let mut full_url = self._mk_url(url)?;
        full_url.query_pairs_mut().extend_pairs(param);
        Ok(full_url)
    }

    /// Refactored code which talks to Gitlab and transforms error messages properly.
    fn _comm<T>(&self, req: RequestBuilder) -> Result<T>
        where T: DeserializeOwned,
    {
        let req = req.header(GitlabPrivateToken(self.token.to_string()));
        let rsp = req.send().chain_err(|| ErrorKind::Communication)?;
        if !rsp.status().is_success() {
            let v = serde_json::from_reader(rsp).chain_err(|| ErrorKind::Deserialize)?;
            return Err(Error::from_gitlab(v));
        }

        let v = serde_json::from_reader(rsp).chain_err(|| ErrorKind::Deserialize)?;
        debug!(target: "gitlab",
               "received data: {:?}",
               v);
        Ok(serde_json::from_value::<T>(v).chain_err(|| ErrorKind::Deserialize)?)
    }

    /// Create a `GET` request to an API endpoint.
    fn _get<T>(&self, url: &str) -> Result<T>
        where T: DeserializeOwned,
    {
        let param: &[(&str, &str)] = &[];
        self._get_with_param(url, param)
    }

    /// Create a `GET` request to an API endpoint with query parameters.
    fn _get_with_param<T, I, K, V>(&self, url: &str, param: I) -> Result<T>
        where T: DeserializeOwned,
              I: IntoIterator,
              I::Item: Borrow<(K, V)>,
              K: AsRef<str>,
              V: AsRef<str>,
    {
        let full_url = self._mk_url_with_param(url, param)?;
        let req = Client::new().chain_err(|| ErrorKind::Communication)?.get(full_url);
        self._comm(req)
    }

    /// Create a `POST` request to an API endpoint.
    fn _post<T>(&self, url: &str) -> Result<T>
        where T: DeserializeOwned,
    {
        let param: &[(&str, &str)] = &[];
        self._post_with_param(url, param)
    }

    /// Create a `POST` request to an API endpoint with query parameters.
    fn _post_with_param<T, U>(&self, url: &str, param: U) -> Result<T>
        where T: DeserializeOwned,
              U: Serialize,
    {
        let full_url = self._mk_url(url)?;
        let req = Client::new().chain_err(|| ErrorKind::Communication)?.post(full_url).form(&param);
        self._comm(req)
    }

    /// Create a `PUT` request to an API endpoint.
    fn _put<T>(&self, url: &str) -> Result<T>
        where T: DeserializeOwned,
    {
        let param: &[(&str, &str)] = &[];
        self._put_with_param(url, param)
    }

    /// Create a `PUT` request to an API endpoint with query parameters.
    fn _put_with_param<T, U>(&self, url: &str, param: U) -> Result<T>
        where T: DeserializeOwned,
              U: Serialize,
    {
        let full_url = self._mk_url(url)?;
        let req = Client::new().chain_err(|| ErrorKind::Communication)?.request(Method::Put, full_url).form(&param);
        self._comm(req)
    }

    /// Handle paginated queries. Returns all results.
    fn _get_paged<T>(&self, url: &str) -> Result<Vec<T>>
        where T: DeserializeOwned,
    {
        let param: &[(&str, &str)] = &[];
        self._get_paged_with_param(url, param)
    }

    /// Handle paginated queries with query parameters. Returns all results.
    fn _get_paged_with_param<T, I, K, V>(&self, url: &str, param: I) -> Result<Vec<T>>
        where T: DeserializeOwned,
              I: IntoIterator,
              I::Item: Borrow<(K, V)>,
              K: AsRef<str>,
              V: AsRef<str>,
    {
        let mut page_num = 1;
        let per_page = 100;
        let per_page_str = &format!("{}", per_page);

        let full_url = self._mk_url_with_param(url, param)?;

        let mut results: Vec<T> = vec![];

        loop {
            let page_str = &format!("{}", page_num);
            let mut page_url = full_url.clone();
            page_url.query_pairs_mut()
                .extend_pairs(&[("page", page_str), ("per_page", per_page_str)]);
            let req = Client::new().chain_err(|| ErrorKind::Communication)?.get(page_url);

            let page: Vec<T> = self._comm(req)?;
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
