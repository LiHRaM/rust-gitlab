// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate ease;
use self::ease::Error as EaseError;
use self::ease::{Request, Response, Url};

extern crate serde;
use self::serde::{Deserialize, Deserializer, Serialize, Serializer};
use self::serde::de::Error as SerdeError;

extern crate serde_json;

extern crate url;
use self::url::percent_encoding::{PATH_SEGMENT_ENCODE_SET, percent_encode};

use super::error::Error;
use super::types::*;

use std::fmt::{self, Debug};

#[derive(Clone)]
/// A representation of the Gitlab API for a single user.
///
/// Separate users should use separate instances of this.
pub struct Gitlab {
    base_url: Url,
    token: String,
}

impl Debug for Gitlab {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Gitlab {{ {} }}", self.base_url)
    }
}

// The header Gitlab uses to authenticate the user.
header!{ (GitlabPrivateToken, "PRIVATE-TOKEN") => [String] }

/// A JSON value return from Gitlab.
pub type GitlabResult<T> = Result<T, Error>;

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
    pub fn new<T: ToString>(host: &str, token: T) -> GitlabResult<Self> {
        Self::_new("https", host, token.to_string())
    }

    /// Create a new non-SSL Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new_insecure<T: ToString>(host: &str, token: T) -> GitlabResult<Self> {
        Self::_new("http", host, token.to_string())
    }

    fn _new(protocol: &str, host: &str, token: String) -> GitlabResult<Self> {
        let base_url = try!(Url::parse(&format!("{}://{}/api/v3/", protocol, host)));

        let api = Gitlab {
            base_url: base_url,
            token: token,
        };

        // Ensure the API is working.
        let _: UserFull = try!(api._get("user"));

        Ok(api)
    }

    /// The user the API is acting as.
    pub fn current_user(&self) -> GitlabResult<UserFull> {
        self._get("user")
    }

    /// Get all user accounts
    pub fn users<T: UserResult>(&self) -> GitlabResult<Vec<T>> {
        self._get_paged("users")
    }

    /// Find a user by id.
    pub fn user<T: UserResult>(&self, user: UserId) -> GitlabResult<T> {
        self._get(&format!("users/{}", user))
    }

    /// Find a user by username.
    pub fn user_by_name<T: UserResult>(&self, name: &str) -> GitlabResult<T> {
        let mut req = try!(self._mkrequest("users"));

        req.param("username", name);

        let mut users = try!(Self::_get_paged_req(req));

        users.pop()
            .ok_or_else(|| Error::Gitlab("no such user".to_string()))
    }

    /// Get all accessible projects.
    pub fn projects(&self) -> GitlabResult<Vec<Project>> {
        self._get_paged("projects")
    }

    /// Get all owned projects.
    pub fn owned_projects(&self) -> GitlabResult<Vec<Project>> {
        self._get_paged("projects/owned")
    }

    /// Get all projects.
    ///
    /// Requires administrator privileges.
    pub fn all_projects(&self) -> GitlabResult<Vec<Project>> {
        self._get_paged("projects/all")
    }

    /// Find a project by id.
    pub fn project(&self, project: ProjectId) -> GitlabResult<Project> {
        self._get(&format!("projects/{}", project))
    }

    /// Find a project by name.
    pub fn project_by_name(&self, name: &str) -> GitlabResult<Project> {
        self._get(&format!("projects/{}",
                           percent_encode(name.as_bytes(), PATH_SEGMENT_ENCODE_SET)))
    }

    /// Get a project's hooks.
    pub fn hooks(&self, project: ProjectId) -> GitlabResult<Vec<Hook>> {
        self._get_paged(&format!("projects/{}/hooks", project))
    }

    /// Get a project hook.
    pub fn hook(&self, project: ProjectId, hook: HookId) -> GitlabResult<Hook> {
        self._get(&format!("projects/{}/hooks/{}", project, hook))
    }

    fn bool_param_value(value: bool) -> &'static str {
        if value {
            "true"
        } else {
            "false"
        }
    }

    fn set_event_flags(request: &mut Request, events: WebhookEvents) {
        request
            .param("build_events", Self::bool_param_value(events.build()))
            .param("issues_events", Self::bool_param_value(events.issues()))
            .param("merge_requests_events",
                   Self::bool_param_value(events.merge_requests()))
            .param("note_events", Self::bool_param_value(events.note()))
            .param("pipeline_events", Self::bool_param_value(events.pipeline()))
            .param("push_events", Self::bool_param_value(events.push()))
            .param("wiki_page_events",
                   Self::bool_param_value(events.wiki_page()));
    }

    /// Add a project hook.
    pub fn add_hook(&self, project: ProjectId, url: &str, events: WebhookEvents)
                    -> GitlabResult<Hook> {
        let mut req = try!(self._mkrequest(&format!("projects/{}/hooks", project)));
        Self::set_event_flags(&mut req, events);

        req.param("url", url);

        Self::_post_req(req)
    }

    /// Get the team members of a group.
    pub fn group_members(&self, group: GroupId) -> GitlabResult<Vec<Member>> {
        self._get_paged(&format!("groups/{}/members", group))
    }

    /// Get a team member of a group.
    pub fn group_member(&self, group: GroupId, user: UserId) -> GitlabResult<Member> {
        self._get(&format!("groups/{}/members/{}", group, user))
    }

    /// Get the team members of a project.
    pub fn project_members(&self, project: ProjectId) -> GitlabResult<Vec<Member>> {
        self._get_paged(&format!("projects/{}/members", project))
    }

    /// Get a team member of a project.
    pub fn project_member(&self, project: ProjectId, user: UserId) -> GitlabResult<Member> {
        self._get(&format!("projects/{}/members/{}", project, user))
    }

    /// Add a user to a project.
    pub fn add_user_to_project(&self, project: ProjectId, user: UserId, access: AccessLevel)
                               -> GitlabResult<Member> {
        let user_str = format!("{}", user);
        let access_str = format!("{}", access);

        let mut req = try!(self._mkrequest(&format!("projects/{}/members", project)));

        req.param("user", &user_str)
            .param("access", &access_str);

        Self::_post_req(req)
    }

    /// Get branches for a project.
    pub fn branches(&self, project: ProjectId) -> GitlabResult<Vec<RepoBranch>> {
        self._get_paged(&format!("projects/{}/branches", project))
    }

    /// Get a branch.
    pub fn branch(&self, project: ProjectId, branch: &str) -> GitlabResult<RepoBranch> {
        self._get(&format!("projects/{}/repository/branches/{}",
                           project,
                           percent_encode(branch.as_bytes(), PATH_SEGMENT_ENCODE_SET)))
    }

    /// Get a commit.
    pub fn commit(&self, project: ProjectId, commit: &str) -> GitlabResult<RepoCommitDetail> {
        self._get(&format!("projects/{}/repository/commits/{}", project, commit))
    }

    /// Get comments on a commit.
    pub fn commit_comments(&self, project: ProjectId, commit: &str)
                           -> GitlabResult<Vec<CommitNote>> {
        self._get_paged(&format!("projects/{}/repository/commits/{}/comments",
                                 project,
                                 commit))
    }

    /// Get comments on a commit.
    pub fn create_commit_comment(&self, project: ProjectId, commit: &str, body: &str)
                                 -> GitlabResult<CommitNote> {
        let mut req = try!(self._mkrequest(&format!("projects/{}/repository/commits/{}/comment",
                                                    project,
                                                    commit)));

        req.param("note", body);

        Self::_post_req(req)
    }

    /// Get comments on a commit.
    pub fn create_commit_line_comment(&self, project: ProjectId, commit: &str, body: &str,
                                      path: &str, line: u64)
                                      -> GitlabResult<CommitNote> {
        let line_str = format!("{}", line);
        let line_type = LineType::New;

        let mut req = try!(self._mkrequest(&format!("projects/{}/repository/commits/{}/comment",
                                                    project,
                                                    commit)));

        req.param("note", body)
            .param("path", path)
            .param("line", &line_str)
            .param("line_type", line_type.as_str());

        Self::_post_req(req)
    }

    /// Get the latest statuses of a commit.
    pub fn commit_latest_statuses(&self, project: ProjectId, commit: &str)
                                  -> GitlabResult<Vec<CommitStatus>> {
        self._get_paged(&format!("projects/{}/repository/commits/{}/statuses",
                                 project,
                                 commit))
    }

    /// Get the all statuses of a commit.
    pub fn commit_all_statuses(&self, project: ProjectId, commit: &str)
                               -> GitlabResult<Vec<CommitStatus>> {
        let mut req = try!(self._mkrequest(&format!("projects/{}/repository/commits/{}/statuses",
                                                    project,
                                                    commit)));

        req.param("all", "true");

        Self::_get_paged_req(req)
    }

    /// Get the latest builds of a commit.
    pub fn commit_latest_builds(&self, project: ProjectId, commit: &str)
                                -> GitlabResult<Vec<Build>> {
        self._get_paged(&format!("projects/{}/repository/commits/{}/builds", project, commit))
    }

    /// Get the all builds of a commit.
    pub fn commit_all_builds(&self, project: ProjectId, commit: &str) -> GitlabResult<Vec<Build>> {
        let mut req = try!(self._mkrequest(&format!("projects/{}/repository/commits/{}/builds",
                                                    project,
                                                    commit)));

        req.param("all", "true");

        Self::_get_paged_req(req)
    }

    /// Create a status message for a commit.
    pub fn create_commit_status(&self, project: ProjectId, sha: &str, state: StatusState,
                                info: &CommitStatusInfo)
                                -> GitlabResult<CommitStatus> {
        let path = &format!("projects/{}/statuses/{}", project, sha);
        let mut req = try!(self._mkrequest(path));

        req.param("state", state.as_str());

        info.refname.map(|v| req.param("ref", v));
        info.name.map(|v| req.param("name", v));
        info.target_url.map(|v| req.param("target_url", v));
        info.description.map(|v| req.param("description", v));

        Self::_post_req(req)
    }

    /// Get the issues for a project.
    pub fn issues(&self, project: ProjectId) -> GitlabResult<Vec<Issue>> {
        self._get_paged(&format!("projects/{}/issues", project))
    }

    /// Get issues.
    pub fn issue(&self, project: ProjectId, issue: IssueId) -> GitlabResult<Issue> {
        self._get(&format!("projects/{}/issues/{}", project, issue))
    }

    /// Get the notes from a issue.
    pub fn issue_notes(&self, project: ProjectId, issue: IssueId) -> GitlabResult<Vec<Note>> {
        self._get_paged(&format!("projects/{}/issues/{}/notes", project, issue))
    }

    /// Create a note on a issue.
    pub fn create_issue_note(&self, project: ProjectId, issue: IssueId, content: &str)
                             -> GitlabResult<Note> {
        let path = &format!("projects/{}/issues/{}/notes", project, issue);

        let mut req = try!(self._mkrequest(path));

        req.param("body", content);

        Self::_post_req(req)
    }

    /// Get the merge requests for a project.
    pub fn merge_requests(&self, project: ProjectId) -> GitlabResult<Vec<MergeRequest>> {
        self._get_paged(&format!("projects/{}/merge_requests", project))
    }

    /// Get the merge requests with a given state.
    pub fn merge_requests_with_state(&self, project: ProjectId, state: MergeRequestStateFilter)
                                     -> GitlabResult<Vec<MergeRequest>> {
        let mut req = try!(self._mkrequest(&format!("projects/{}/merge_requests", project)));

        req.param("state", state.as_str());

        Self::_get_paged_req(req)
    }

    /// Get merge requests.
    pub fn merge_request(&self, project: ProjectId, merge_request: MergeRequestId)
                         -> GitlabResult<MergeRequest> {
        self._get(&format!("projects/{}/merge_requests/{}", project, merge_request))
    }

    /// Get the issues that will be closed when a merge request is merged.
    pub fn merge_request_closes_issues(&self, project: ProjectId, merge_request: MergeRequestId)
                                       -> GitlabResult<Vec<IssueReference>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/closes_issues",
                                 project,
                                 merge_request))
    }

    /// Get the notes from a merge request.
    pub fn merge_request_notes(&self, project: ProjectId, merge_request: MergeRequestId)
                               -> GitlabResult<Vec<Note>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/notes",
                                 project,
                                 merge_request))
    }

    /// Award a merge request note with an award.
    pub fn award_merge_request_note(&self, project: ProjectId, merge_request: MergeRequestId,
                                    note: NoteId, award: &str)
                                    -> GitlabResult<AwardEmoji> {
        let path = &format!("projects/{}/merge_requests/{}/notes/{}/award_emoji",
                            project,
                            merge_request,
                            note);
        let mut req = try!(self._mkrequest(path));

        req.param("name", award);

        Self::_post_req(req)
    }

    /// Get the awards for a merge request.
    pub fn merge_request_awards(&self, project: ProjectId, merge_request: MergeRequestId)
                                -> GitlabResult<Vec<AwardEmoji>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/award_emoji",
                                 project,
                                 merge_request))
    }

    /// Get the awards for a merge request note.
    pub fn merge_request_note_awards(&self, project: ProjectId, merge_request: MergeRequestId,
                                     note: NoteId)
                                     -> GitlabResult<Vec<AwardEmoji>> {
        self._get_paged(&format!("projects/{}/merge_requests/{}/notes/{}/award_emoji",
                                 project,
                                 merge_request,
                                 note))
    }

    /// Create a note on a merge request.
    pub fn create_merge_request_note(&self, project: ProjectId, merge_request: MergeRequestId,
                                     content: &str)
                                     -> GitlabResult<Note> {
        let path = &format!("projects/{}/merge_requests/{}/notes",
                            project,
                            merge_request);

        let mut req = try!(self._mkrequest(path));

        req.param("body", content);

        Self::_post_req(req)
    }

    // Create a request with the proper common metadata for authentication.
    //
    // This method exists because we want to store the current user in the structure, but we don't
    // have a `self` before we create the structure. Making it `Option<>` is a little silly and
    // refactoring this out is worth the cleaner API.
    fn _mkrequest1(base_url: &Url, token: &str, url: &str) -> GitlabResult<Request> {
        let full_url = try!(base_url.join(url));
        let mut req = Request::new(full_url);

        debug!(target: "gitlab", "api call {}", url);

        req.header(GitlabPrivateToken(token.to_string()));

        Ok(req)
    }

    // Create a request with the proper common metadata for authentication.
    fn _mkrequest(&self, url: &str) -> GitlabResult<Request> {
        Self::_mkrequest1(&self.base_url, &self.token, url)
    }

    // Refactored code which talks to Gitlab and transforms error messages properly.
    fn _comm<F, T>(req: Request, f: F) -> GitlabResult<T>
        where F: FnOnce(Request) -> Result<Response, EaseError>,
              T: Deserialize,
    {
        match f(req) {
            Ok(rsp) => {
                let v = try!(rsp.from_json().map_err(Error::Ease));

                debug!(target: "gitlab",
                       "received data: {}",
                       v);

                Ok(try!(serde_json::from_value::<T>(v)))
            },
            Err(err) => {
                if let EaseError::UnsuccessfulResponse(rsp) = err {
                    Err(Error::from_gitlab(try!(rsp.from_json())))
                } else {
                    Err(Error::Ease(err))
                }
            },
        }
    }

    fn _get_req<T: Deserialize>(req: Request) -> GitlabResult<T> {
        Self::_comm(req, |mut req| req.get())
    }

    fn _get<T: Deserialize>(&self, url: &str) -> GitlabResult<T> {
        Self::_get_req(try!(self._mkrequest(url)))
    }

    fn _post_req<T: Deserialize>(req: Request) -> GitlabResult<T> {
        Self::_comm(req, |mut req| req.post())
    }

    fn _post<T: Deserialize>(&self, url: &str) -> GitlabResult<T> {
        Self::_post_req(try!(self._mkrequest(url)))
    }

    fn _put_req<T: Deserialize>(req: Request) -> GitlabResult<T> {
        Self::_comm(req, |mut req| req.put())
    }

    fn _put<T: Deserialize>(&self, url: &str) -> GitlabResult<T> {
        Self::_put_req(try!(self._mkrequest(url)))
    }

    fn _get_paged_req<T: Deserialize>(req: Request) -> GitlabResult<Vec<T>> {
        let mut page_num = 1;
        let per_page = 100;
        let per_page_str = &format!("{}", per_page);

        let mut results: Vec<T> = vec![];

        loop {
            let page_str = &format!("{}", page_num);
            let mut page_req = req.clone();
            page_req.param("page", page_str)
                .param("per_page", per_page_str);
            let page = try!(Self::_get_req::<Vec<T>>(page_req));
            let page_len = page.len();

            results.extend(page.into_iter());

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

    // Handle paginated queries. Returns all results.
    fn _get_paged<T: Deserialize>(&self, url: &str) -> GitlabResult<Vec<T>> {
        Self::_get_paged_req(try!(self._mkrequest(url)))
    }
}
