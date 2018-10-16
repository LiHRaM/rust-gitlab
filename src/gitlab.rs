// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crates::itertools::Itertools;
use crates::percent_encoding::{PATH_SEGMENT_ENCODE_SET, PercentEncode, utf8_percent_encode};
use crates::reqwest::{Client, RequestBuilder, Url};
use crates::reqwest::header::HeaderValue;
use crates::serde::{Deserialize, Deserializer, Serializer};
use crates::serde::de::Error as SerdeError;
use crates::serde::de::{DeserializeOwned, Unexpected};
use crates::serde::ser::Serialize;
use crates::serde_json;

use error::*;
use types::*;

use std::borrow::Borrow;
use std::fmt::{self, Display, Debug};

/// A representation of the Gitlab API for a single user.
///
/// Separate users should use separate instances of this.
pub struct Gitlab {
    /// The client to use for API calls.
    client: Client,
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
    pub fn new<H, T>(host: H, token: T) -> Result<Self>
        where H: AsRef<str>,
              T: ToString,
    {
        Self::new_impl("https", host.as_ref(), token.to_string())
    }

    /// Create a new non-SSL Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new_insecure<H, T>(host: H, token: T) -> Result<Self>
        where H: AsRef<str>,
              T: ToString,
    {
        Self::new_impl("http", host.as_ref(), token.to_string())
    }

    /// Internal method to create a new Gitlab client.
    fn new_impl(protocol: &str, host: &str, token: String) -> Result<Self> {
        let base_url = Url::parse(&format!("{}://{}/api/v4/", protocol, host))
            .chain_err(|| ErrorKind::UrlParse)?;

        let api = Gitlab {
            client: Client::new(),
            base_url: base_url,
            token: token,
        };

        // Ensure the API is working.
        let _: UserPublic = api.current_user()?;

        Ok(api)
    }

    /// Create a new Gitlab API client builder.
    pub fn builder<H, T>(host: H, token: T) -> GitlabBuilder
        where H: ToString,
              T: ToString,
    {
        GitlabBuilder::new(host, token)
    }

    /// The user the API is acting as.
    pub fn current_user(&self) -> Result<UserPublic> {
        self.get("user")
    }

    /// Get all user accounts
    pub fn users<T>(&self) -> Result<Vec<T>>
        where T: UserResult,
    {
        self.get_paged("users")
    }

    /// Find a user by id.
    pub fn user<T>(&self, user: UserId) -> Result<T>
        where T: UserResult,
    {
        self.get(&format!("users/{}", user))
    }

    /// Find a user by username.
    pub fn user_by_name<T, N>(&self, name: N) -> Result<T>
        where T: UserResult,
              N: AsRef<str>,
    {
        let mut users = self.get_paged_with_param("users", &[("username", name.as_ref())])?;
        users.pop()
            .ok_or_else(|| Error::from_kind(ErrorKind::Gitlab("no such user".to_string())))
    }

    /// Get all accessible projects.
    pub fn projects(&self) -> Result<Vec<Project>> {
        self.get_paged("projects")
    }

    /// Get all owned projects.
    pub fn owned_projects(&self) -> Result<Vec<Project>> {
        self.get_paged_with_param("projects", &[("owned", "true")])
    }

    /// Find a project by id.
    pub fn project(&self, project: ProjectId) -> Result<Project> {
        self.get(&format!("projects/{}", project))
    }

    /// A URL-safe name for projects.
    fn url_name(name: &str) -> PercentEncode<PATH_SEGMENT_ENCODE_SET> {
        utf8_percent_encode(name, PATH_SEGMENT_ENCODE_SET)
    }

    /// Find a project by name.
    pub fn project_by_name<N>(&self, name: N) -> Result<Project>
        where N: AsRef<str>,
    {
        self.get(&format!("projects/{}",
                          Self::url_name(name.as_ref())))
    }

    /// Get a project's hooks.
    pub fn hooks(&self, project: ProjectId) -> Result<Vec<ProjectHook>> {
        self.get_paged(&format!("projects/{}/hooks", project))
    }

    /// Get a project hook.
    pub fn hook(&self, project: ProjectId, hook: HookId) -> Result<ProjectHook> {
        self.get(&format!("projects/{}/hooks/{}", project, hook))
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
             ("confidential_issues_events", Self::bool_param_value(events.confidential_issues())),
             ("merge_requests_events", Self::bool_param_value(events.merge_requests())),
             ("note_events", Self::bool_param_value(events.note())),
             ("pipeline_events", Self::bool_param_value(events.pipeline())),
             ("push_events", Self::bool_param_value(events.push())),
             ("wiki_page_events", Self::bool_param_value(events.wiki_page()))]
    }

    /// Add a project hook.
    pub fn add_hook<U>(&self, project: ProjectId, url: U, events: WebhookEvents) -> Result<ProjectHook>
        where U: AsRef<str>,
    {
        let mut flags = Self::event_flags(events);
        flags.push(("url", url.as_ref()));

        self.post_with_param(&format!("projects/{}/hooks", project), &flags)
    }

    /// Get the team members of a group.
    pub fn group_members(&self, group: GroupId) -> Result<Vec<Member>> {
        self.get_paged(&format!("groups/{}/members", group))
    }

    /// Get a team member of a group.
    pub fn group_member(&self, group: GroupId, user: UserId) -> Result<Member> {
        self.get(&format!("groups/{}/members/{}", group, user))
    }

    /// Get the team members of a project.
    pub fn project_members(&self, project: ProjectId) -> Result<Vec<Member>> {
        self.get_paged(&format!("projects/{}/members", project))
    }

    /// Get a team member of a project.
    pub fn project_member(&self, project: ProjectId, user: UserId) -> Result<Member> {
        self.get(&format!("projects/{}/members/{}", project, user))
    }

    /// Add a user to a project.
    pub fn add_user_to_project(&self, project: ProjectId, user: UserId, access: AccessLevel)
                               -> Result<Member> {
        let user_str = format!("{}", user);
        let access_str = format!("{}", access);

        self.post_with_param(&format!("projects/{}/members", project),
                             &[("user", &user_str), ("access", &access_str)])
    }

    /// Add a user to a project.
    pub fn add_user_to_project_by_name<P>(&self, project: P, user: UserId, access: AccessLevel)
                                          -> Result<Member>
        where P: AsRef<str>,
    {
        let user_str = format!("{}", user);
        let access_str = format!("{}", access);

        self.post_with_param(&format!("projects/{}/members", Self::url_name(project.as_ref())),
                             &[("user", &user_str), ("access", &access_str)])
    }

    /// Get branches for a project.
    pub fn branches(&self, project: ProjectId) -> Result<Vec<RepoBranch>> {
        self.get_paged(&format!("projects/{}/branches", project))
    }

    /// Get a branch.
    pub fn branch<B>(&self, project: ProjectId, branch: B) -> Result<RepoBranch>
        where B: AsRef<str>,
    {
        self.get(&format!("projects/{}/repository/branches/{}",
                          project,
                          Self::url_name(branch.as_ref())))
    }

    /// Get a commit.
    pub fn commit<C>(&self, project: ProjectId, commit: C) -> Result<RepoCommitDetail>
        where C: AsRef<str>,
    {
        self.get_with_param(
            &format!("projects/{}/repository/commits/{}", project, commit.as_ref()),
            &[("stats", "true")])
    }

    /// Get comments on a commit.
    pub fn commit_comments<C>(&self, project: ProjectId, commit: C) -> Result<Vec<CommitNote>>
        where C: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/repository/commits/{}/comments",
                                project,
                                commit.as_ref()))
    }

    /// Get comments on a commit.
    pub fn create_commit_comment<C, B>(&self, project: ProjectId, commit: C, body: B)
                                       -> Result<CommitNote>
        where C: AsRef<str>,
              B: AsRef<str>,
    {
        self.post_with_param(&format!("projects/{}/repository/commits/{}/comment",
                                      project,
                                      commit.as_ref()),
                             &[("note", body.as_ref())])
    }

    /// Get comments on a commit.
    pub fn create_commit_comment_by_name<P, C, B>(&self, project: P, commit: C, body: B)
                                                  -> Result<CommitNote>
        where P: AsRef<str>,
              C: AsRef<str>,
              B: AsRef<str>,
    {
        self.post_with_param(&format!("projects/{}/repository/commits/{}/comment",
                                      Self::url_name(project.as_ref()),
                                      commit.as_ref()),
                             &[("note", body.as_ref())])
    }

    /// Get comments on a commit.
    pub fn create_commit_line_comment(&self, project: ProjectId, commit: &str, body: &str,
                                      path: &str, line: u64)
                                      -> Result<CommitNote> {
        let line_str = format!("{}", line);
        let line_type = LineType::New;

        self.post_with_param(&format!("projects/{}/repository/commits/{}/comment",
                                      project,
                                      commit),
                             &[("note", body),
                               ("path", path),
                               ("line", &line_str),
                               ("line_type", line_type.as_str())])
    }

    /// Get the latest statuses of a commit.
    pub fn commit_latest_statuses<C>(&self, project: ProjectId, commit: C)
                                     -> Result<Vec<CommitStatus>>
        where C: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/repository/commits/{}/statuses",
                                project,
                                commit.as_ref()))
    }

    /// Get the latest statuses of a commit.
    pub fn commit_latest_statuses_by_name<P, C>(&self, project: P, commit: C)
                                                -> Result<Vec<CommitStatus>>
        where P: AsRef<str>,
              C: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/repository/commits/{}/statuses",
                                Self::url_name(project.as_ref()),
                                commit.as_ref()))
    }

    /// Get the all statuses of a commit.
    pub fn commit_all_statuses<C>(&self, project: ProjectId, commit: C)
                                  -> Result<Vec<CommitStatus>>
        where C: AsRef<str>,
    {
        self.get_paged_with_param(&format!("projects/{}/repository/commits/{}/statuses",
                                           project,
                                           commit.as_ref()),
                                  &[("all", "true")])
    }

    /// Get the latest builds of a commit.
    pub fn commit_latest_builds<C>(&self, project: ProjectId, commit: C) -> Result<Vec<Job>>
        where C: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/repository/commits/{}/builds", project, commit.as_ref()))
    }

    /// Get the all builds of a commit.
    pub fn commit_all_builds<C>(&self, project: ProjectId, commit: C) -> Result<Vec<Job>>
        where C: AsRef<str>,
    {
        self.get_paged_with_param(&format!("projects/{}/repository/commits/{}/builds",
                                           project,
                                           commit.as_ref()),
                                  &[("all", "true")])
    }

    /// Create a status message for a commit.
    pub fn create_commit_status<S>(&self, project: ProjectId, sha: S, state: StatusState,
                                   info: &CommitStatusInfo)
                                   -> Result<CommitStatus>
        where S: AsRef<str>,
    {
        let path = &format!("projects/{}/statuses/{}", project, sha.as_ref());

        let mut params = vec![("state", state.as_str())];

        info.refname.map(|v| params.push(("ref", v)));
        info.name.map(|v| params.push(("name", v)));
        info.target_url.map(|v| params.push(("target_url", v)));
        info.description.map(|v| params.push(("description", v)));

        self.post_with_param(path, &params)
    }

    /// Create a status message for a commit.
    pub fn create_commit_status_by_name<P, S>(&self, project: P, sha: S, state: StatusState,
                                              info: &CommitStatusInfo)
                                              -> Result<CommitStatus>
        where P: AsRef<str>,
              S: AsRef<str>,
    {
        let path = &format!("projects/{}/statuses/{}", Self::url_name(project.as_ref()), sha.as_ref());

        let mut params = vec![("state", state.as_str())];

        info.refname.map(|v| params.push(("ref", v)));
        info.name.map(|v| params.push(("name", v)));
        info.target_url.map(|v| params.push(("target_url", v)));
        info.description.map(|v| params.push(("description", v)));

        self.post_with_param(path, &params)
    }

    /// Get the issues for a project.
    pub fn issues(&self, project: ProjectId) -> Result<Vec<Issue>> {
        self.get_paged(&format!("projects/{}/issues", project))
    }

    /// Get issues.
    pub fn issue(&self, project: ProjectId, issue: IssueInternalId) -> Result<Issue> {
        self.get(&format!("projects/{}/issues/{}", project, issue))
    }

    /// Get the notes from a issue.
    pub fn issue_notes(&self, project: ProjectId, issue: IssueInternalId) -> Result<Vec<Note>> {
        self.get_paged(&format!("projects/{}/issues/{}/notes", project, issue))
    }

    /// Get the notes from a issue.
    pub fn issue_notes_by_name<P>(&self, project: P, issue: IssueInternalId) -> Result<Vec<Note>>
        where P: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/issues/{}/notes", Self::url_name(project.as_ref()), issue))
    }

    /// Create a note on a issue.
    pub fn create_issue_note<C>(&self, project: ProjectId, issue: IssueInternalId, content: C)
                                -> Result<Note>
        where C: AsRef<str>,
    {
        let path = &format!("projects/{}/issues/{}/notes", project, issue);

        self.post_with_param(path, &[("body", content.as_ref())])
    }

    /// Create a note on a issue.
    pub fn create_issue_note_by_name<P, C>(&self, project: P, issue: IssueInternalId, content: C)
                                           -> Result<Note>
        where P: AsRef<str>,
              C: AsRef<str>,
    {
        let path = &format!("projects/{}/issues/{}/notes", Self::url_name(project.as_ref()), issue);

        self.post_with_param(path, &[("body", content.as_ref())])
    }

    /// Get the merge requests for a project.
    pub fn merge_requests(&self, project: ProjectId) -> Result<Vec<MergeRequest>> {
        self.get_paged(&format!("projects/{}/merge_requests", project))
    }

    /// Get the merge requests with a given state.
    pub fn merge_requests_with_state(&self, project: ProjectId, state: MergeRequestStateFilter)
                                     -> Result<Vec<MergeRequest>> {
        self.get_paged_with_param(&format!("projects/{}/merge_requests", project),
                                  &[("state", state.as_str())])
    }

    /// Get merge requests.
    pub fn merge_request(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                         -> Result<MergeRequest> {
        self.get(&format!("projects/{}/merge_requests/{}", project, merge_request))
    }

    /// Get the issues that will be closed when a merge request is merged.
    pub fn merge_request_closes_issues(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                                       -> Result<Vec<IssueReference>> {
        self.get_paged(&format!("projects/{}/merge_requests/{}/closes_issues",
                                project,
                                merge_request))
    }

    /// Get the notes from a merge request.
    pub fn merge_request_notes(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                               -> Result<Vec<Note>> {
        self.get_paged(&format!("projects/{}/merge_requests/{}/notes",
                                project,
                                merge_request))
    }

    /// Get the notes from a merge request.
    pub fn merge_request_notes_by_name<P>(&self, project: P, merge_request: MergeRequestInternalId)
                                          -> Result<Vec<Note>>
        where P: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/merge_requests/{}/notes",
                                Self::url_name(project.as_ref()),
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
        self.post_with_param(path, &[("name", award)])
    }

    /// Award a merge request note with an award.
    pub fn award_merge_request_note_by_name<P>(&self, project: P, merge_request: MergeRequestInternalId,
                                               note: NoteId, award: &str)
                                               -> Result<AwardEmoji>
        where P: AsRef<str>,
    {
        let path = &format!("projects/{}/merge_requests/{}/notes/{}/award_emoji",
                            Self::url_name(project.as_ref()),
                            merge_request,
                            note);
        self.post_with_param(path, &[("name", award)])
    }

    /// Get the awards for a merge request.
    pub fn merge_request_awards(&self, project: ProjectId, merge_request: MergeRequestInternalId)
                                -> Result<Vec<AwardEmoji>> {
        self.get_paged(&format!("projects/{}/merge_requests/{}/award_emoji",
                                project,
                                merge_request))
    }

    /// Get the awards for a merge request.
    pub fn merge_request_awards_by_name<P>(&self, project: P, merge_request: MergeRequestInternalId)
                                           -> Result<Vec<AwardEmoji>>
        where P: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/merge_requests/{}/award_emoji",
                                Self::url_name(project.as_ref()),
                                merge_request))
    }

    /// Get the awards for a merge request note.
    pub fn merge_request_note_awards(&self, project: ProjectId, merge_request: MergeRequestInternalId,
                                     note: NoteId)
                                     -> Result<Vec<AwardEmoji>> {
        self.get_paged(&format!("projects/{}/merge_requests/{}/notes/{}/award_emoji",
                                project,
                                merge_request,
                                note))
    }

    /// Get the awards for a merge request note.
    pub fn merge_request_note_awards_by_name<P>(&self, project: P, merge_request: MergeRequestInternalId,
                                                note: NoteId)
                                                -> Result<Vec<AwardEmoji>>
        where P: AsRef<str>,
    {
        self.get_paged(&format!("projects/{}/merge_requests/{}/notes/{}/award_emoji",
                                Self::url_name(project.as_ref()),
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
        self.post_with_param(path, &[("body", content)])
    }

    /// Create a note on a merge request.
    pub fn create_merge_request_note_by_name<P>(&self, project: P, merge_request: MergeRequestInternalId,
                                                content: &str)
                                                -> Result<Note>
        where P: AsRef<str>,
    {
        let path = &format!("projects/{}/merge_requests/{}/notes",
                            Self::url_name(project.as_ref()),
                            merge_request);
        self.post_with_param(path, &[("body", content)])
    }

    /// Get issues closed by a merge request.
    pub fn get_issues_closed_by_merge_request(&self, project: ProjectId,
                                              merge_request: MergeRequestInternalId)
                                              -> Result<Vec<Issue>> {
        let path = &format!("projects/{}/merge_requests/{}/closes_issues",
                            project,
                            merge_request);
        self.get_paged(path)
    }

    /// Get issues closed by a merge request.
    pub fn get_issues_closed_by_merge_request_by_name<P>(&self, project: P,
                                                         merge_request: MergeRequestInternalId)
                                                         -> Result<Vec<Issue>>
        where P: AsRef<str>,
    {
        let path = &format!("projects/{}/merge_requests/{}/closes_issues",
                            Self::url_name(project.as_ref()),
                            merge_request);
        self.get_paged(path)
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
        self.put_with_param(path, &[("labels", labels.into_iter().join(","))])
    }

    /// Set the labels on an issue.
    pub fn set_issue_labels_by_name<P, I, L>(&self, project: P, issue: IssueInternalId, labels: I)
                                             -> Result<Issue>
                                             where P: AsRef<str>,
              I: IntoIterator<Item = L>,
              L: Display,
    {
        let path = &format!("projects/{}/issues/{}",
                            Self::url_name(project.as_ref()),
                            issue);
        self.put_with_param(path, &[("labels", labels.into_iter().join(","))])
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
        self.put_with_param(path, &[("labels", labels.into_iter().join(","))])
    }

    /// Create a URL to an API endpoint.
    fn create_url(&self, url: &str) -> Result<Url> {
        debug!(target: "gitlab", "api call {}", url);
        self.base_url.join(url).chain_err(|| ErrorKind::UrlParse)
    }

    /// Create a URL to an API endpoint with query parameters.
    fn create_url_with_param<I, K, V>(&self, url: &str, param: I) -> Result<Url>
        where I: IntoIterator,
              I::Item: Borrow<(K, V)>,
              K: AsRef<str>,
              V: AsRef<str>,
    {
        let mut full_url = self.create_url(url)?;
        full_url.query_pairs_mut().extend_pairs(param);
        Ok(full_url)
    }

    /// Refactored code which talks to Gitlab and transforms error messages properly.
    fn send<T>(&self, req: RequestBuilder) -> Result<T>
        where T: DeserializeOwned,
    {
        let mut token_header_value = HeaderValue::from_str(&self.token)
            .map_err(|_| ErrorKind::HeaderValueParse)?;
        token_header_value.set_sensitive(true);
        let rsp = req.header("PRIVATE-TOKEN", token_header_value)
            .send()
            .chain_err(|| ErrorKind::Communication)?;
        let status = rsp.status();
        if status.is_server_error() {
            return Err(ErrorKind::Gitlab(
                       format!("server error: {} {:?}", status.as_u16(),
                               status.canonical_reason())).into());
        }
        let success = status.is_success();
        let v = serde_json::from_reader(rsp).chain_err(|| ErrorKind::Deserialize)?;
        if !success {
            return Err(Error::from_gitlab(v));
        }

        debug!(target: "gitlab",
               "received data: {:?}",
               v);
        serde_json::from_value::<T>(v).chain_err(|| ErrorKind::Deserialize)
    }

    /// Create a `GET` request to an API endpoint.
    fn get<T>(&self, url: &str) -> Result<T>
        where T: DeserializeOwned,
    {
        let param: &[(&str, &str)] = &[];
        self.get_with_param(url, param)
    }

    /// Create a `GET` request to an API endpoint with query parameters.
    fn get_with_param<T, I, K, V>(&self, url: &str, param: I) -> Result<T>
        where T: DeserializeOwned,
              I: IntoIterator,
              I::Item: Borrow<(K, V)>,
              K: AsRef<str>,
              V: AsRef<str>,
    {
        let full_url = self.create_url_with_param(url, param)?;
        let req = self.client.get(full_url);
        self.send(req)
    }

    /// Create a `POST` request to an API endpoint with query parameters.
    fn post_with_param<T, U>(&self, url: &str, param: U) -> Result<T>
        where T: DeserializeOwned,
              U: Serialize,
    {
        let full_url = self.create_url(url)?;
        self.send(self.client
            .post(full_url)
            .form(&param))
    }

    /// Create a `PUT` request to an API endpoint with query parameters.
    fn put_with_param<T, U>(&self, url: &str, param: U) -> Result<T>
        where T: DeserializeOwned,
              U: Serialize,
    {
        let full_url = self.create_url(url)?;
        self.send(self.client.put(full_url)
            .form(&param))
    }

    /// Handle paginated queries. Returns all results.
    fn get_paged<T>(&self, url: &str) -> Result<Vec<T>>
        where T: DeserializeOwned,
    {
        let param: &[(&str, &str)] = &[];
        self.get_paged_with_param(url, param)
    }

    /// Handle paginated queries with query parameters. Returns all results.
    fn get_paged_with_param<T, I, K, V>(&self, url: &str, param: I) -> Result<Vec<T>>
        where T: DeserializeOwned,
              I: IntoIterator,
              I::Item: Borrow<(K, V)>,
              K: AsRef<str>,
              V: AsRef<str>,
    {
        let mut page_num = 1;
        let per_page = 100;
        let per_page_str = &format!("{}", per_page);

        let full_url = self.create_url_with_param(url, param)?;

        let mut results: Vec<T> = vec![];

        loop {
            let page_str = &format!("{}", page_num);
            let mut page_url = full_url.clone();
            page_url.query_pairs_mut()
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
    token: String,
}

impl GitlabBuilder {
    /// Create a new Gitlab API client builder.
    pub fn new<H, T>(host: H, token: T) -> Self
        where H: ToString,
              T: ToString,
    {
        Self {
            protocol: "https",
            host: host.to_string(),
            token: token.to_string(),
        }
    }

    /// Switch to an insecure protocol (http instead of https).
    pub fn insecure(&mut self) -> &mut Self
    {
        self.protocol = "http";
        self
    }

    pub fn build(&self) -> Result<Gitlab> {
        Gitlab::new_impl(self.protocol, &self.host, self.token.clone())
    }
}
