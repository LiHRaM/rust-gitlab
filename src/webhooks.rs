// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Web hook structures
//!
//! These hooks are received from Gitlab when registered as a web hook within a project.
//!
//! Gitlab does not have consistent structures for its hooks, so they often change from
//! version to version.

extern crate chrono;
use self::chrono::{DateTime, NaiveDate, TimeZone, UTC};

extern crate serde;
use self::serde::{Deserialize, Deserializer, Serialize, Serializer};
use self::serde::de::{Error, Unexpected};

extern crate serde_json;
use self::serde_json::Value;

use types::{BuildId, IssueId, IssueState, MergeRequestId, MergeRequestState, MergeStatus,
            MilestoneId, NoteId, NoteType, NoteableId, ObjectId, ProjectId, SnippetId, UserId};

#[derive(Debug, Clone, Copy)]
/// A wrapper struct for dates in web hooks.
///
/// Gitlab does not use a standard date format for dates in web hooks. This structure supports
/// deserializing the formats that have been observed.
pub struct HookDate(DateTime<UTC>);

impl Serialize for HookDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl Deserialize for HookDate {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        let val = String::deserialize(deserializer)?;

        UTC.datetime_from_str(&val, "%Y-%m-%d %H:%M:%S UTC")
            .or_else(|_| {
                DateTime::parse_from_str(&val, "%Y-%m-%d %H:%M:%S %z")
                    .map_err(|err| {
                        D::Error::invalid_value(Unexpected::Other("hook date"),
                                                &format!("{:?}", err).as_str())
                    })
                    .map(|dt| dt.with_timezone(&UTC))
            })
            .map(HookDate)
    }
}

impl AsRef<DateTime<UTC>> for HookDate {
    fn as_ref(&self) -> &DateTime<UTC> {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Project information exposed in hooks.
pub struct ProjectHookAttrs {
    /// The display name of the project.
    pub name: String,
    /// The description of the project.
    pub description: Option<String>,
    /// The URL for the project's homepage.
    pub web_url: String,
    /// The URL to the project avatar.
    pub avatar_url: Option<String>,
    /// The URL to clone the repository over SSH.
    pub git_ssh_url: String,
    /// The URL to clone the repository over HTTPS.
    pub git_http_url: String,
    /// The namespace the project lives in.
    pub namespace: String,
    /// Integral value for the project's visibility.
    pub visibility_level: u64,
    /// The path to the project's repository with its namespace.
    pub path_with_namespace: String,
    /// The default branch for the project.
    pub default_branch: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Wiki project information exposed in hooks.
pub struct ProjectWikiHookAttrs {
    /// The URL for the project's homepage.
    pub web_url: String,
    /// The URL to clone the repository over SSH.
    pub git_ssh_url: String,
    /// The URL to clone the repository over HTTPS.
    pub git_http_url: String,
    /// The path to the project's repository with its namespace.
    pub path_with_namespace: String,
    /// The default branch for the project.
    pub default_branch: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// User information exposed in hooks.
pub struct UserHookAttrs {
    /// The name of the user.
    pub name: String,
    /// The handle of the user.
    pub username: String,
    /// The URL to the avatar of the user.
    pub avatar_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The identity of a user exposed through a hook.
pub struct HookCommitIdentity {
    /// The name of the author or committer.
    pub name: String,
    /// The email address of the author or committer.
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Commit information exposed in hooks.
pub struct CommitHookAttrs {
    /// The commit's ID.
    pub id: ObjectId,
    /// The commit message.
    pub message: String,
    pub timestamp: DateTime<UTC>,
    /// The URL of the commit.
    pub url: String,
    /// The author of the commit.
    pub author: HookCommitIdentity,
    pub added: Option<Vec<String>>,
    pub modified: Option<Vec<String>>,
    pub removed: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A push hook.
pub struct PushHook {
    /// The event which occurred.
    pub object_kind: String,
    /// The old object ID of the ref before the push.
    pub before: ObjectId,
    /// The new object ID of the ref after the push.
    pub after: ObjectId,
    #[serde(rename="ref")]
    /// The name of the reference which has been pushed.
    pub ref_: String,
    /// The new object ID of the ref after the push.
    pub checkout_sha: Option<ObjectId>,
    /// The message for the push (used for annotated tags).
    pub message: Option<String>,
    /// The ID of the user who pushed.
    pub user_id: UserId,
    /// The name of the user who pushed.
    pub user_name: String,
    /// The email address of the user who pushed.
    pub user_email: String,
    /// The URL of the user's avatar.
    pub user_avatar: String,
    /// The ID of the project pushed to.
    pub project_id: ProjectId,
    /// Attributes of the project.
    pub project: ProjectHookAttrs,
    /// The commits pushed to the repository.
    ///
    /// Limited to 20 commits.
    pub commits: Vec<CommitHookAttrs>, // limited to 20 commits
    /// The total number of commits pushed.
    pub total_commits_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Actions which may occur on an issue.
pub enum IssueAction {
    /// The issue was updated.
    Update,
    /// The issue was opened.
    Open,
    /// The issue was closed.
    Close,
    /// The issue was reopened.
    Reopen,
}
enum_serialize!(IssueAction -> "issue action",
    Update => "update",
    Open => "open",
    Close => "close",
    Reopen => "reopen",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Issue information exposed in hooks.
pub struct IssueHookAttrs {
    /// The ID of the issue.
    pub id: IssueId,
    /// The title of the issue.
    pub title: String,
    /// The ID of the assignee of the issue.
    pub assignee_id: Option<UserId>,
    /// The ID of the author of the issue.
    pub author_id: UserId,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// When the issue was created.
    pub created_at: HookDate,
    /// When the issue was last updated.
    pub updated_at: HookDate,
    /// When the issue was deleted.
    pub deleted_at: Option<HookDate>,
    /// When the issue is due.
    pub due_date: Option<NaiveDate>,
    /// The ID of the user which last updated the issue.
    pub updated_by_id: Option<UserId>,
    pub moved_to_id: Option<Value>, // ???
    pub position: u64,
    /// The branch name for the issue.
    pub branch_name: Option<String>,
    /// The description of the issue.
    pub description: String,
    /// The ID of the milestone of the issue.
    pub milestone_id: Option<MilestoneId>,
    /// The state of the issue.
    pub state: IssueState,
    /// The user-visible ID of the issue.
    pub iid: u64,
    /// Whether the issue is confidential or not.
    pub confidential: bool,

    // It seems that notes miss these properties?
    /// The URL of the issue.
    pub url: Option<String>,
    /// The type of action which caused the hook.
    pub action: Option<IssueAction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// An issue hook.
pub struct IssueHook {
    /// The event which occurred.
    pub object_kind: String,
    /// The user which triggered the hook.
    pub user: UserHookAttrs,
    /// The project the hook was created for.
    pub project: ProjectHookAttrs,
    /// Attributes of the issue.
    pub object_attributes: IssueHookAttrs,
    /// The assignee of the issue.
    pub assignee: Option<UserHookAttrs>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Actions which may occur on a merge request.
pub enum MergeRequestAction {
    /// The merge request was updated.
    Update,
    /// The merge request was opened.
    Open,
    /// The merge request was closed.
    Close,
    /// The merge request was reopened.
    Reopen,
    /// The merge request was merged.
    Merge,
}
enum_serialize!(MergeRequestAction -> "merge request action",
    Update => "update",
    Open => "open",
    Close => "close",
    Reopen => "reopen",
    Merge => "merge",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Merge parameters for a merge request.
pub struct MergeRequestParams {
    force_remove_source_branch: Option<Value>, // sigh
}

impl MergeRequestParams {
    /// Whether the author of the merge request indicated that the source branch should be deleted
    /// upon merge or not.
    // https://gitlab.com/gitlab-org/gitlab-ce/issues/20880
    pub fn force_remove_source_branch(&self) -> bool {
        self.force_remove_source_branch
            .as_ref()
            .map_or(false, |val| {
                if let Some(as_str) = val.as_str() {
                    as_str == "1"
                } else if let Some(as_bool) = val.as_bool() {
                    as_bool
                } else {
                    error!(target: "gitlab",
                           "unknown value for force_remove_source_branch: {}", val);
                    false
                }
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Merge request information exposed in hooks.
pub struct MergeRequestHookAttrs {
    /// The source project of the merge request.
    ///
    /// If this is `None`, the source repository has been deleted.
    pub source: Option<ProjectHookAttrs>,
    /// The target project of the merge request.
    pub target: ProjectHookAttrs,
    pub last_commit: Option<CommitHookAttrs>,
    /// Whether the merge request is a work-in-progress or not.
    pub work_in_progress: bool,
    /// The object ID of the merge commit which is currently being handled.
    pub in_progress_merge_commit_sha: Option<ObjectId>,

    /// The ID of the merge request.
    pub id: MergeRequestId,
    /// The target branch of the merge request.
    pub target_branch: String,
    /// The ID of the target project.
    pub target_project_id: ProjectId,
    /// The source branch of the merge request.
    pub source_branch: String,
    /// The ID of the source project.
    pub source_project_id: ProjectId,
    /// The ID of the author of the merge request.
    pub author_id: UserId,
    /// The ID of the assignee of the merge request.
    pub assignee_id: Option<UserId>,
    /// The title of the merge request.
    pub title: String,
    /// When the merge request was created.
    pub created_at: HookDate,
    /// When the merge request was last updated.
    pub updated_at: HookDate,
    /// When the merge request was deleted.
    pub deleted_at: Option<HookDate>,
    /// When the merge request was locked.
    pub locked_at: Option<HookDate>,
    /// The ID of the user which last updated the merge request.
    pub updated_by_id: Option<UserId>,
    /// The object ID of the commit which merged the merge request.
    pub merge_commit_sha: Option<ObjectId>,
    pub merge_error: Option<Value>, // String?
    /// The parameters for merging the merge request.
    pub merge_params: MergeRequestParams,
    /// The user which merged the merge request.
    pub merge_user_id: Option<UserId>,
    /// Whether the merge request will be merged once all builds succeed or not.
    pub merge_when_build_succeeds: bool,
    pub position: u64, // ???
    // st_commits
    // st_diffs
    /// The milestone of the merge request.
    pub milestone_id: Option<MilestoneId>,
    pub oldrev: Option<ObjectId>,
    /// The state of the merge request.
    pub state: MergeRequestState,
    /// The merge status of the merge request.
    pub merge_status: MergeStatus,
    /// The user-visible ID of the merge request.
    pub iid: u64,
    /// The description of the merge request.
    pub description: Option<String>,

    // It seems that notes miss these properties?
    /// The URL of the merge request.
    pub url: Option<String>,
    /// The type of action which caused the hook.
    pub action: Option<MergeRequestAction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A merge request hook.
pub struct MergeRequestHook {
    /// The event which occurred.
    pub object_kind: String,
    /// The user which triggered the hook.
    pub user: UserHookAttrs,
    /// The project the hook was created for.
    pub project: ProjectHookAttrs,
    /// Attributes of the merge request.
    pub object_attributes: MergeRequestHookAttrs,
    /// The assignee of the merge request.
    pub assignee: Option<UserHookAttrs>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The type of a snippet.
pub enum SnippetType {
    /// A project-owned snippet.
    Project,
    /// A user-owned snippet.
    Personal,
}
enum_serialize!(SnippetType -> "snippet type",
    Project => "ProjectSnippet",
    Personal => "PersonalSnippet",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Snippet information exposed in hooks.
pub struct SnippetHookAttrs {
    /// The title of the snippet.
    pub title: String,
    /// The content of the snippet.
    pub content: String,
    /// The author of the snippet.
    pub author_id: UserId,
    /// The project the snippet belongs to.
    pub project_id: Option<ProjectId>,
    /// When the snippet was created.
    pub created_at: HookDate,
    /// When the snippet was last updated.
    pub updated_at: HookDate,
    /// The name of the snippet.
    pub file_name: String,
    #[serde(rename="type")]
    /// The type of the snippet.
    pub type_: SnippetType,
    /// The visibility of the snippet.
    pub visibility_level: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Actions which may occur on a wiki page.
pub enum WikiPageAction {
    /// A wiki page was created.
    Create,
    /// A wiki page was updated.
    Update,
}
enum_serialize!(WikiPageAction -> "wiki page action",
    Create => "create",
    Update => "update",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Wiki information exposed in hooks.
pub struct WikiPageHookAttrs {
    /// The title of the wiki page.
    pub title: String,
    /// The content of the wiki page.
    pub content: String,
    pub format: String,
    pub message: String,
    /// The slug of the wiki page.
    pub slug: String,

    /// The URL of the wiki page.
    pub url: String,
    /// The type of action which caused the hook.
    pub action: WikiPageAction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Diff information exposed in hooks.
pub struct DiffHookAttrs {
    pub diff: String,
    /// The path on the new side of the diff.
    pub new_path: String,
    /// The path on the old side of the diff.
    pub old_path: String,
    /// The mode on the old side of the diff.
    // TODO: Create a mode type.
    pub a_mode: String,
    /// The mode on the new side of the diff.
    pub b_mode: String,
    /// Whether the diff indicates the addition of a file.
    pub new_file: bool,
    /// Whether the diff indicates the rename of a file.
    pub renamed_file: bool,
    /// Whether the diff indicates the deletion of a file.
    pub deleted_file: bool,
    pub too_large: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// FIXME: This can apparently be a string sometimes.
// https://gitlab.com/gitlab-org/gitlab-ce/issues/21467
pub struct PositionHookAttrs {
    pub base_sha: ObjectId,
    pub head_sha: ObjectId,
    pub start_sha: ObjectId,
    pub old_line: Option<u64>,
    /// The path on the old side of the diff.
    pub old_path: String,
    pub new_line: Option<u64>,
    /// The path on the new side of the diff.
    pub new_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Note (comment) information exposed in hooks.
pub struct NoteHookAttrs {
    /// The ID of the note.
    pub id: NoteId,
    /// THe content of the note.
    pub note: String,
    /// The type of entity the note is attached to.
    pub noteable_type: NoteType,
    // pub original_position: Option<PositionHookAttrs>,
    // pub position: Option<PositionHookAttrs>,
    /// The author of the note.
    pub author_id: UserId,
    /// When the note was created.
    pub created_at: HookDate,
    /// When the note was last updated.
    pub updated_at: HookDate,
    /// The ID of the user who last updated the note.
    pub updated_by_id: Option<UserId>,
    /// When the note was marked as resolved.
    pub resolved_at: Option<HookDate>,
    /// The ID of the user who marked the note as resolved.
    pub resolved_by_id: Option<UserId>,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The URL of an attachment to the note.
    pub attachment: Option<String>,
    pub line_code: Option<String>, // TODO: This is some internal format.
    pub commit_id: Option<ObjectId>, // XXX(8.11): apparently can be an empty string?
    pub discussion_id: ObjectId,
    pub original_discussion_id: Option<ObjectId>,
    noteable_id: Value, // Keep as JSON because its type depends on what `noteable_type` is.
    /// Whether the note was created by a user or in response to an external action.
    pub system: bool,
    pub st_diff: Option<DiffHookAttrs>,
    /// The URL of the note.
    pub url: String,

    #[serde(rename="type")]
    pub type_: Option<String>, // ???
    //pub is_award: bool, // seems to have been removed?
}

impl NoteHookAttrs {
    /// The ID of the object the note is for.
    pub fn noteable_id(&self) -> Option<NoteableId> {
        match self.noteable_type {
            NoteType::Commit => {
                self.noteable_id
                    .as_str()
                    .map(|id| NoteableId::Commit(ObjectId::new(id)))
            },
            NoteType::Issue => {
                self.noteable_id
                    .as_u64()
                    .map(|id| NoteableId::Issue(IssueId::new(id)))
            },
            NoteType::MergeRequest => {
                self.noteable_id
                    .as_u64()
                    .map(|id| NoteableId::MergeRequest(MergeRequestId::new(id)))
            },
            NoteType::Snippet => {
                self.noteable_id
                    .as_u64()
                    .map(|id| NoteableId::Snippet(SnippetId::new(id)))
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A note hook.
pub struct NoteHook {
    /// The event which occurred.
    pub object_kind: String,
    /// The user who triggered the hook.
    pub user: UserHookAttrs,
    /// The ID of the project the note belongs to.
    pub project_id: ProjectId,
    /// The project the note belongs to.
    pub project: ProjectHookAttrs,
    /// The attributes on the note itself.
    pub object_attributes: NoteHookAttrs,
    /// The commit the note is associated with (for commit notes).
    pub commit: Option<CommitHookAttrs>,
    /// The issue the note is associated with (for issue notes).
    pub issue: Option<IssueHookAttrs>,
    /// The merge request the note is associated with (for merge request notes).
    pub merge_request: Option<MergeRequestHookAttrs>,
    /// The snippet the note is associated with (for snippet notes).
    pub snippet: Option<SnippetHookAttrs>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Build user information exposed in hooks.
pub struct BuildUserHookAttrs {
    /// The ID of the user.
    pub id: Option<UserId>,
    /// The user's name.
    pub name: Option<String>,
    /// The user's email address.
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Build commit information exposed in hooks.
pub struct BuildCommitHookAttrs {
    pub id: String,
    /// The object ID of the commit.
    pub sha: ObjectId,
    /// The full commit message.
    pub message: String,
    /// The commit's author's name.
    pub author_name: String,
    /// The commit's author's email address.
    pub author_email: String,
    pub status: String,
    pub duration: u64,
    /// When the build started.
    pub started_at: Option<HookDate>,
    /// When the build completed.
    pub finished_at: Option<HookDate>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Project information exposed in build hooks.
pub struct BuildProjectHookAttrs {
    /// The display name of the project.
    pub name: String,
    /// The description of the project.
    pub description: Option<String>,
    /// The URL for the project's homepage.
    pub homepage: String,
    /// The URL to clone the repository over HTTPS.
    pub git_http_url: String,
    /// The URL to clone the repository over SSH.
    pub git_ssh_url: String,
    /// Integral value for the project's visibility.
    pub visibility_level: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A build hook.
pub struct BuildHook {
    /// The event which occurred.
    pub object_kind: String,
    #[serde(rename="ref")]
    /// The name of the reference that was tested.
    pub ref_: String,
    pub tag: String,
    pub before_sha: String,
    /// The object ID that was built.
    pub sha: String,
    /// The ID of the build.
    pub build_id: BuildId,
    /// The name of the build.
    pub build_name: String,
    pub build_stage: String,
    /// When the build started.
    pub build_started_at: Option<HookDate>,
    /// When the build completed.
    pub build_finished_at: Option<HookDate>,
    pub build_duration: Option<u64>,
    /// Whether the build is allowed to fail.
    pub build_allow_failure: bool,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The user which owns the build.
    pub user: BuildUserHookAttrs,
    /// The commit which was built.
    pub commit: BuildCommitHookAttrs,
    /// The repository the build is for.
    pub repository: BuildProjectHookAttrs,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A wiki page hook.
pub struct WikiPageHook {
    /// The event which occurred.
    pub object_kind: String,
    /// The user who caused the hook.
    pub user: UserHookAttrs,
    /// The project the wiki belongs to.
    pub project: ProjectHookAttrs,
    /// The wiki the page belongs to.
    pub wiki: ProjectWikiHookAttrs,
    /// The wiki page.
    pub object_attributes: WikiPageHookAttrs,
}

#[derive(Debug, Clone)]
/// A deserializable structure for all Gitlab web hooks.
pub enum WebHook {
    /// A push hook.
    Push(PushHook),
    /// An issue hook.
    Issue(IssueHook),
    /// A merge request hook.
    MergeRequest(MergeRequestHook),
    /// A note hook.
    Note(NoteHook),
    /// A build hook.
    Build(BuildHook),
    /// A wiki page hook.
    WikiPage(WikiPageHook),
}

impl Deserialize for WebHook {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        let val = <Value as Deserialize>::deserialize(deserializer)?;

        let object_kind = match val.pointer("/object_kind") {
            Some(&Value::String(ref kind)) => kind.to_string(),
            Some(_) => {
                return Err(D::Error::invalid_type(Unexpected::Other("JSON value"), &"a string"));
            },
            None => {
                return Err(D::Error::missing_field("object_kind"));
            },
        };

        let hook_res = match object_kind.as_str() {
            "push" | "tag_push" => serde_json::from_value(val).map(WebHook::Push),

            "issue" => serde_json::from_value(val).map(WebHook::Issue),

            "merge_request" => serde_json::from_value(val).map(WebHook::MergeRequest),

            "note" => serde_json::from_value(val).map(WebHook::Note),

            "build" => serde_json::from_value(val).map(WebHook::Build),

            _ => {
                return Err(D::Error::invalid_value(Unexpected::Other("object kind"),
                                                   &format!("unrecognized webhook object kind: \
                                                             {}",
                                                            object_kind)
                                                       .as_str()));
            },
        };

        hook_res.map_err(|err| {
            D::Error::invalid_value(Unexpected::Other("web hook"),
                                    &format!("{:?}", err).as_str())
        })
    }
}
