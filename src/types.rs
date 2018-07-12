// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! API entities
//!
//! These entities are exposed by Gitlab via its API.
//!
//! There are some places where Gitlab does not completely specify its types. This causes
//! problems when the types and names change inside of those. If found, issues should be filed
//! upstream.

use crates::chrono::{DateTime, NaiveDate, Utc};
use crates::serde::{Deserialize, Deserializer, Serialize, Serializer};
use crates::serde::de::{DeserializeOwned, Error, Unexpected};
use crates::serde_json::{self, Value};

use std::fmt::{self, Display, Formatter};

// This is only used in internal API calls.
//#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
//#[derive(Serialize, Deserialize, Debug, Clone)]
//pub struct UserSafe {
//    pub username: String,
//    pub name: String,
//}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe user ID.
pub struct UserId(u64);
impl_id!(UserId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The states a user account can be in.
pub enum UserState {
    /// The user is active and may perform actions.
    Active,
    /// Blocked from logging in.
    Blocked,
    /// Blocked from logging in via LDAP.
    LdapBlocked,
}
enum_serialize!(UserState -> "user state",
    Active => "active",
    Blocked => "blocked",
    LdapBlocked => "ldap_blocked",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Basic user information.
pub struct UserBasic {
    /// The username.
    pub username: String,
    /// The display name.
    pub name: String,
    /// The user's ID.
    pub id: UserId,
    /// The state of the user account.
    pub state: UserState,
    /// The URL of the user's avatar.
    pub avatar_url: String,
    /// The URL of the user's profile page.
    pub web_url: String,
}

/// A unifying trait for all user types.
///
/// This is used to allow (direct) user queries to return the right information because
/// administrator users receive additional information for all user queries versus
/// non-administrator users.
pub trait UserResult: DeserializeOwned {}
impl<T: DeserializeOwned + Into<UserBasic>> UserResult for T {}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// More detailed information only accessible to administrators.
pub struct User {
    /// The username.
    pub username: String,
    /// The display name.
    pub name: String,
    /// The user's ID.
    pub id: UserId,
    /// The state of the user account.
    pub state: UserState,
    /// The URL of the user's avatar.
    pub avatar_url: String,
    /// The URL of the user's profile page.
    pub web_url: String,
    /// When the account was created.
    pub created_at: DateTime<Utc>,
    /// Whether the user is an administrator or not.
    ///
    /// Only available when talking to GitLab as an admin.
    pub is_admin: Option<bool>,
    /// Self-described biography of the user.
    pub bio: Option<String>,
    /// Geographic location of the user.
    pub location: Option<String>,

    /// Skype contact information.
    pub skype: String,
    /// LinkedIn contact information.
    pub linkedin: String,
    /// Twitter contact information.
    pub twitter: String,
    /// Custom URL for the user's website.
    pub website_url: String,
    /// Organization the user belongs to.
    pub organization: Option<String>,
}

impl From<User> for UserBasic {
    fn from(user: User) -> Self {
        UserBasic {
            username: user.username,
            name: user.name,
            id: user.id,
            state: user.state,
            avatar_url: user.avatar_url,
            web_url: user.web_url,
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// External authentication tokens.
pub struct Identity {
    /// The provider of the token.
    pub provider: String,
    /// The UID for the provider.
    pub extern_uid: String,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe theme ID.
pub struct ThemeId(u64);
impl_id!(ThemeId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe color scheme ID.
pub struct ColorSchemeId(u64);
impl_id!(ColorSchemeId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Full user structure information.
pub struct UserPublic {
    /// The username.
    pub username: String,
    /// The display name.
    pub name: String,
    /// The user's ID.
    pub id: UserId,
    /// The state of the user account.
    pub state: UserState,
    /// The URL of the user's avatar.
    pub avatar_url: String,
    /// The URL of the user's profile page.
    pub web_url: String,
    /// When the account was created.
    pub created_at: DateTime<Utc>,
    /// Whether the user is an administrator or not.
    ///
    /// Only available when talking to GitLab as an admin.
    pub is_admin: Option<bool>,
    /// Self-described biography of the user.
    pub bio: Option<String>,
    /// Geographic location of the user.
    pub location: Option<String>,

    /// Skype contact information.
    pub skype: String,
    /// LinkedIn contact information.
    pub linkedin: String,
    /// Twitter contact information.
    pub twitter: String,
    /// Custom URL for the user's website.
    pub website_url: String,
    /// Organization the user belongs to.
    pub organization: Option<String>,

    /// When the user last logged in.
    pub last_sign_in_at: Option<DateTime<Utc>>,
    /// When the user last made an action.
    pub last_activity_on: Option<NaiveDate>,
    /// When the user's account was confirmed.
    pub confirmed_at: Option<DateTime<Utc>>,
    /// The primary email address for the user.
    pub email: String,

    /// The theme used by the user, if configured.
    pub theme_id: Option<ThemeId>,
    /// The color scheme used by the user.
    pub color_scheme_id: ColorSchemeId,
    /// The number of projects the user may create.
    pub projects_limit: u64,
    /// When the user's current session started.
    pub current_sign_in_at: Option<DateTime<Utc>>,

    /// List of identities associated with the user.
    pub identities: Vec<Identity>,

    /// Whether the user can create groups.
    pub can_create_group: bool,
    /// Whether the user can create a new project.
    pub can_create_project: bool,
    /// Whether the user has two-factor authentication enabled.
    pub two_factor_enabled: bool,
    /// Whether the account is externally controlled.
    pub external: bool,
}

impl From<UserPublic> for UserBasic {
    fn from(user: UserPublic) -> Self {
        UserBasic {
            username: user.username,
            name: user.name,
            id: user.id,
            state: user.state,
            avatar_url: user.avatar_url,
            web_url: user.web_url,
        }
    }
}

impl From<UserPublic> for User {
    fn from(user: UserPublic) -> Self {
        User {
            username: user.username,
            name: user.name,
            id: user.id,
            state: user.state,
            avatar_url: user.avatar_url,
            web_url: user.web_url,
            created_at: user.created_at,
            is_admin: user.is_admin,
            bio: user.bio,
            location: user.location,
            skype: user.skype,
            linkedin: user.linkedin,
            twitter: user.twitter,
            website_url: user.website_url,
            organization: user.organization,
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe email ID.
pub struct EmailId(u64);
impl_id!(EmailId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Email address.
pub struct Email {
    /// ID of the email.
    pub id: EmailId,
    /// The email address.
    pub email: String,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe hook ID.
pub struct HookId(u64);
impl_id!(HookId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A web hook to notify of events.
pub struct Hook {
    /// The ID of the hook.
    pub id: HookId,
    /// The URL to contact.
    pub url: String,
    /// When the hook was created.
    pub created_at: DateTime<Utc>,
    /// Whether the hook is contacted for push events.
    pub push_events: bool,
    /// Whether the hook is contacted for tag push events.
    pub tag_push_events: bool,
    /// Whether the communication with the hook is verified using TLS certificates.
    pub enable_ssl_verification: bool,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A web hook to notify of project events.
pub struct ProjectHook {
    /// The ID of the hook.
    pub id: HookId,
    /// The URL to contact.
    pub url: String,
    /// When the hook was created.
    pub created_at: DateTime<Utc>,
    /// The project associated with the hook.
    pub project_id: ProjectId,
    /// Whether the hook is contacted for push events.
    pub push_events: bool,
    /// Whether the hook is contacted for tag push events.
    pub tag_push_events: bool,
    /// Whether the hook is contacted for issue events.
    pub issues_events: bool,
    /// Whether the hook is contacted for confidential issue events.
    pub confidential_issues_events: Option<bool>,
    /// Whether the hook is contacted for merge request events.
    pub merge_requests_events: bool,
    /// Whether the hook is contacted for note events.
    pub note_events: bool,
    /// Whether the hook is contacted for confidential note events.
    pub confidential_note_events: Option<bool>,
    /// Whether the hook is contacted for repository update events.
    pub repository_update_events: bool,
    /// Whether the communication with the hook is verified using TLS certificates.
    pub enable_ssl_verification: bool,
    /// Whether the hook is contacted for job events.
    pub job_events: bool,
    /// Whether the hook is contacted for pipeline events.
    pub pipeline_events: bool,
    /// Whether the hook is contacted for wiki page events.
    pub wiki_page_events: bool,
}

impl From<ProjectHook> for Hook {
    fn from(hook: ProjectHook) -> Self {
        Hook {
            id: hook.id,
            url: hook.url,
            created_at: hook.created_at,
            push_events: hook.push_events,
            tag_push_events: hook.tag_push_events,
            enable_ssl_verification: hook.enable_ssl_verification,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
/// The events a webhook listener may receive.
pub struct WebhookEvents {
    /// Whether to receive job events of not.
    job: bool,
    /// Whether to receive issue events of not.
    issues: bool,
    /// Whether to receive confidential issue events of not.
    confidential_issues: bool,
    /// Whether to receive merge request events of not.
    merge_requests: bool,
    /// Whether to receive note (comment) events of not.
    note: bool,
    /// Whether to receive pipeline events of not.
    pipeline: bool,
    /// Whether to receive push events of not.
    push: bool,
    /// Whether to receive wiki events of not.
    wiki_page: bool,
}

impl WebhookEvents {
    /// Create a new, empty webhook event set.
    pub fn new() -> Self {
        WebhookEvents {
            job: false,
            issues: false,
            confidential_issues: false,
            merge_requests: false,
            note: false,
            pipeline: false,
            push: false,
            wiki_page: false,
        }
    }

    with_event!{with_job, job}
    with_event!{with_issues, issues}
    with_event!{with_confidential_issues, issues}
    with_event!{with_merge_requests, merge_requests}
    with_event!{with_note, note}
    with_event!{with_pipeline, pipeline}
    with_event!{with_push, push}
    with_event!{with_wiki_page, wiki_page}

    get_event!{job}
    get_event!{issues}
    get_event!{confidential_issues}
    get_event!{merge_requests}
    get_event!{note}
    get_event!{pipeline}
    get_event!{push}
    get_event!{wiki_page}
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe project ID.
pub struct ProjectId(u64);
impl_id!(ProjectId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Basic project information.
pub struct BasicProjectDetails {
    /// The ID of the project.
    pub id: ProjectId,
    /// The display name of the project.
    pub name: String,
    /// The display name of the project with the namespace.
    pub name_with_namespace: String,
    /// The path to the project's repository.
    pub path: String,
    /// The path to the project's repository with its namespace.
    pub path_with_namespace: String,
    /// The URL to the main page of the repository.
    pub http_url_to_repo: String,
    /// The URL to the main page of the repository.
    pub web_url: String,
}

/// Visibility levels of projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VisibilityLevel {
    /// The project is visible to anonymous users.
    Public,
    /// The project is visible to logged in users.
    Internal,
    /// The project is visible only to users with explicit access.
    Private,
}
enum_serialize!(VisibilityLevel -> "visibility level",
    Public => "public",
    Internal => "internal",
    Private => "private",
);

// TODO: enum for NotificationLevel

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Structure for a group a project has been shared with.
pub struct SharedGroup {
    /// The ID of the group.
    pub group_id: GroupId,
    /// The name of the group.
    pub group_name: String,
    /// The access level of the group.
    pub group_access_level: u64,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
/// Access information to a project.
// Called `MemberAccess` in entities.rb, but it is just a base class for `ProjectAccess` and
// `GroupAccess`. Combine them here.
pub struct MemberAccess {
    /// The access level of the membership (see `VisibilityLevel`).
    pub access_level: u64,
    /// The notification level of the current user.
    pub notification_level: Option<u64>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
/// Permissions granted to the current user to a project.
pub struct Permissions {
    /// The access granted by the project to the current user.
    pub project_access: Option<MemberAccess>,
    /// The access granted by the group to the current user.
    pub group_access: Option<MemberAccess>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// The avatar of a project's namespace.
pub struct ProjectNamespaceAvatar {
    /// The URL of the namespace avatar.
    pub url: Option<String>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectLinks {
    #[serde(rename="self")]
    /// API URL of project itself.
    self_: String,
    /// API URL of project issues, if enabled.
    issues: Option<String>,
    /// API URL of project merge requests, if enabled.
    merge_requests: Option<String>,
    /// API URL of project repository branches.
    repo_branches: String,
    /// API URL of project labels.
    labels: String,
    /// API URL of project events.
    events: String,
    /// API URL of project members.
    members: String,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Project information.
pub struct Project {
    /// The ID of the project.
    pub id: ProjectId,
    /// The description of the project.
    pub description: Option<String>,
    /// The default branch for the project.
    pub default_branch: Option<String>,
    /// A list of tags for the project.
    pub tag_list: Vec<String>,
    /// Whether the project is archived or not.
    pub archived: bool,
    /// Whether the project is public, internal, or private.
    pub visibility: VisibilityLevel,
    /// The URL to clone the repository over SSH.
    pub ssh_url_to_repo: String,
    /// The URL to clone the repository over HTTPS.
    pub http_url_to_repo: String,
    /// The URL for the project's homepage.
    pub web_url: String,
    /// The URL for the project's readme.
    pub readme_url: Option<String>,
    /// The owner of the project (`None` for a group-owned project).
    pub owner: Option<UserBasic>,
    /// The display name of the project.
    pub name: String,
    /// The display name of the project with the namespace.
    pub name_with_namespace: String,
    /// The path to the project's repository.
    pub path: String,
    /// The path to the project's repository with its namespace.
    pub path_with_namespace: String,
    /// Whether the continuous integration container registry is enabled.
    ///
    /// This is supposed to be just `bool`, but projects created before the registry was
    /// supported appear to return `null`.
    pub container_registry_enabled: Option<bool>,
    /// When the repository was created.
    pub created_at: DateTime<Utc>,
    /// When the last activity on the project occurred.
    pub last_activity_at: DateTime<Utc>,
    /// Whether continuous integration shared runners are enabled.
    pub shared_runners_enabled: bool,
    /// Whether LFS object storage is enabled.
    pub lfs_enabled: bool,
    /// The user who created the repository.
    pub creator_id: UserId,
    /// The namespace the project lives in.
    pub namespace: Namespace,
    /// If the project is a fork, details about it.
    pub forked_from_project: Option<BasicProjectDetails>,
    /// The URL to the project avatar.
    pub avatar_url: Option<String>,
    /// The path to CI config file.
    pub ci_config_path: Option<String>,
    /// Description of error if project failed to import.
    pub import_error: Option<String>,
    /// The number of stars for the project.
    pub star_count: u64,
    /// The number of forks.
    pub forks_count: u64,
    /// The number of open issues (if issues are enabled).
    pub open_issues_count: Option<u64>,
    /// The continuous integration runner token (if enabled).
    pub runners_token: Option<String>,
    /// Whether jobs are publicly visible.
    pub public_jobs: bool,
    /// Groups the project is shared with.
    pub shared_with_groups: Vec<SharedGroup>,
    /// Whether the project only enables the merge button if all pipelines are passing.
    pub only_allow_merge_if_pipeline_succeeds: Option<bool>,
    /// Whether the project only enables the merge button if all discussions are resolved.
    pub only_allow_merge_if_all_discussions_are_resolved: Option<bool>,
    /// Whether to show the link to create/view merge request when pusing from command line.
    pub printing_merge_request_link_enabled: Option<bool>,
    /// Whether access to the project may be requested.
    pub request_access_enabled: bool,
    /// Whether jobs are enabled or not.
    pub jobs_enabled: bool,
    /// Whether to automatically resolve merge request diff discussions when they become outdated,
    /// if configured.
    pub resolve_outdated_diff_discussions: Option<bool>,
    /// Whether issues are enabled or not.
    pub issues_enabled: bool,
    /// Whether merge requests are enabled or not.
    pub merge_requests_enabled: bool,
    /// Whether snippets are enabled or not.
    pub snippets_enabled: bool,
    /// Whether the project wiki is enabled or not.
    pub wiki_enabled: bool,
    /// The merge method used when merging merge request.
    pub merge_method: Option<String>,
    /// Statistics about the project.
    pub statistics: Option<ProjectStatistics>,

    /// If this is present, it is `ProjectWithAccess`, but since it is so similar, just have it be
    /// optional here.
    pub permissions: Option<Permissions>,

    /// Links to related API URLs provided by GitLab in response to
    /// direct project lookup.  We do not expose this because our
    /// clients do not need them.
    _links: Option<ProjectLinks>,
}

#[cfg(test)]
impl Project {
    pub fn has_links(&self) -> bool {
        self._links.is_some()
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
/// Statistics about a project.
pub struct ProjectStatistics {
    /// The number of commits in the repository.
    pub commit_count: u64,
    /// The size, in bytes, of the total storage required for the project.
    pub storage_size: u64,
    /// The size, in bytes, of the repository.
    pub repository_size: u64,
    /// The size, in bytes, of uploaded LFS files.
    pub lfs_objects_size: u64,
    /// The size, in bytes, of uploaded job artifacts.
    pub job_artifacts_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Access levels for groups and projects.
pub enum AccessLevel {
    /// Anonymous access.
    Anonymous,
    /// Guest access (can see the project).
    Guest,
    /// Reporter access (can open issues).
    Reporter,
    /// Developer access (can push branches, handle issues and merge requests).
    Developer,
    /// Maintainer access (can push to protected branches).
    Maintainer,
    /// Owner access (full rights).
    Owner,
}

impl From<AccessLevel> for u64 {
    fn from(access: AccessLevel) -> Self {
        match access {
            AccessLevel::Anonymous => 0,
            AccessLevel::Guest => 10,
            AccessLevel::Reporter => 20,
            AccessLevel::Developer => 30,
            AccessLevel::Maintainer => 40,
            AccessLevel::Owner => 50,
        }
    }
}

impl From<u64> for AccessLevel {
    fn from(access: u64) -> Self {
        if access >= 50 {
            AccessLevel::Owner
        } else if access >= 40 {
            AccessLevel::Maintainer
        } else if access >= 30 {
            AccessLevel::Developer
        } else if access >= 20 {
            AccessLevel::Reporter
        } else if access >= 10 {
            AccessLevel::Guest
        } else {
            AccessLevel::Anonymous
        }
    }
}

impl Display for AccessLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", Into::<u64>::into(self.clone()))
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A member with extra permissions on a project.
pub struct Member {
    /// The username.
    pub username: String,
    /// The display name.
    pub name: String,
    /// The user's ID.
    pub id: UserId,
    /// The state of the user account.
    pub state: UserState,
    /// The URL of the user's avatar.
    pub avatar_url: String,
    /// The URL of the user's profile page.
    pub web_url: String,
    /// The access level of the user.
    pub access_level: u64,
    /// When the membership expires.
    pub expires_at: Option<NaiveDate>,
}

impl From<Member> for UserBasic {
    fn from(member: Member) -> Self {
        UserBasic {
            username: member.username,
            name: member.name,
            id: member.id,
            state: member.state,
            avatar_url: member.avatar_url,
            web_url: member.web_url,
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A member with extra permissions on a project.
pub struct AccessRequester {
    /// The username.
    pub username: String,
    /// The display name.
    pub name: String,
    /// The user's ID.
    pub id: UserId,
    /// The state of the user account.
    pub state: UserState,
    /// The URL of the user's avatar.
    pub avatar_url: String,
    /// The URL of the user's profile page.
    pub web_url: String,
    /// When the membership request was created.
    pub requested_at: DateTime<Utc>,
}

impl From<AccessRequester> for UserBasic {
    fn from(member: AccessRequester) -> Self {
        UserBasic {
            username: member.username,
            name: member.name,
            id: member.id,
            state: member.state,
            avatar_url: member.avatar_url,
            web_url: member.web_url,
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe group ID.
pub struct GroupId(u64);
impl_id!(GroupId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Group information.
pub struct Group {
    /// The ID of the group.
    pub id: GroupId,
    /// The name of the group.
    pub name: String,
    /// The path to the group.
    pub path: String,
    /// The description of the group.
    pub description: Option<String>,
    /// Whether the project is public, internal, or private.
    pub visibility: VisibilityLevel,
    /// Whether LFS is enabled for the group.
    pub lfs_enabled: bool,
    /// The URL to the group avatar.
    pub avatar_url: String,
    /// The URL to the group's profile page.
    pub web_url: String,
    /// Whether membership requests are allowed for the group.
    pub request_access_enabled: bool,
    pub full_name: String,
    pub full_path: String,
    pub parent_id: Option<GroupId>,
    /// Statistics about the group.
    pub statistics: Option<GroupStatistics>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
/// Statistics about a group.
pub struct GroupStatistics {
    /// The size, in bytes, of the total storage required for the group.
    pub storage_size: u64,
    /// The size, in bytes, of all repositories in the group.
    pub repository_size: u64,
    /// The size, in bytes, of uploaded LFS files in the group.
    pub lfs_objects_size: u64,
    /// The size, in bytes, of uploaded job artifacts in the group.
    pub job_artifacts_size: u64,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Group information with a project listing.
pub struct GroupDetail {
    /// The ID of the group.
    pub id: GroupId,
    /// The name of the group.
    pub name: String,
    /// The path to the group.
    pub path: String,
    /// The description of the group.
    pub description: Option<String>,
    /// Whether the project is public, internal, or private.
    pub visibility: VisibilityLevel,
    /// Whether LFS is enabled for the group.
    pub lfs_enabled: bool,
    /// The URL to the group avatar.
    pub avatar_url: String,
    /// The URL to the group's profile page.
    pub web_url: String,
    /// The projects in a group.
    pub projects: Vec<Project>,
    /// Projects the group shares with other groups or users.
    pub shared_projects: Vec<Project>,
    /// Whether membership requests are allowed for the group.
    pub request_access_enabled: bool,
    pub full_name: String,
    pub full_path: String,
    pub parent_id: Option<GroupId>,
    /// Statistics about the group.
    pub statistics: Option<GroupStatistics>,
}

impl From<GroupDetail> for Group {
    fn from(detail: GroupDetail) -> Self {
        Group {
            id: detail.id,
            name: detail.name,
            path: detail.path,
            description: detail.description,
            visibility: detail.visibility,
            lfs_enabled: detail.lfs_enabled,
            avatar_url: detail.avatar_url,
            web_url: detail.web_url,
            request_access_enabled: detail.request_access_enabled,
            full_name: detail.full_name,
            full_path: detail.full_path,
            parent_id: detail.parent_id,
            statistics: detail.statistics,
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A branch on a repository.
pub struct RepoBranch {
    /// The name of the branch.
    pub name: String,
    /// The commit of the branch.
    pub commit: Option<RepoCommit>,
    /// Whether the branch is merged into the main branch or not.
    pub merged: Option<bool>,
    /// Whether the branch is protected or not.
    pub protected: Option<bool>,
    /// Whether the developers can push directly to the branch or not.
    pub developers_can_push: Option<bool>,
    /// Whether the developers can merge into the branch or not.
    pub developers_can_merge: Option<bool>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The ID of a git object.
pub struct ObjectId(String);

impl ObjectId {
    /// Construct a new `ObjectId`
    pub fn new<O: ToString>(oid: O) -> Self {
        ObjectId(oid.to_string())
    }

    /// The value of the id.
    pub fn value(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kinds of objects Gitlab can return.
pub enum ObjectType {
    /// A `tree` object.
    Tree,
    /// A `blob` object.
    Blob,
}
enum_serialize!(ObjectType -> "object type",
    Tree => "tree",
    Blob => "blob",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An object inside of a repository.
pub struct RepoTreeObject {
    /// The ID of the object.
    pub id: ObjectId,
    /// The name of the object.
    pub name: String,
    #[serde(rename="type")]
    /// The type of the object.
    pub type_: ObjectType,
    /// The path to the object inside of the repository.
    pub path: String,
    /// The mode of the object.
    pub mode: String,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A commit in a project.
pub struct RepoCommit {
    /// The ID of the commit.
    pub id: ObjectId,
    /// The short ID of the commit.
    pub short_id: ObjectId,
    /// The summary of the commit.
    pub title: String,
    /// The commit ID of the parents of the commit.
    pub parent_ids: Vec<ObjectId>,
    /// The commit author's name.
    pub author_name: String,
    /// The commit author's email address.
    pub author_email: String,
    /// The commit's authorship date.
    pub authored_date: DateTime<Utc>,
    /// The committer's name.
    pub committer_name: String,
    /// The committer's email address.
    pub committer_email: String,
    /// The commit's commit date.
    pub committed_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    /// The full commit message.
    pub message: String,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
/// Stats about a commit.
pub struct RepoCommitStats {
    /// The number of lines added by the commit.
    pub additions: u64,
    /// The number of lines deleted by the commit.
    pub deletions: u64,
    /// The number of lines changed by the commit.
    pub total: u64,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A commit in a project with statistics.
pub struct RepoCommitDetail {
    /// The ID of the commit.
    pub id: ObjectId,
    /// The short ID of the commit.
    pub short_id: ObjectId,
    /// The summary of the commit.
    pub title: String,
    /// The commit ID of the parents of the commit.
    pub parent_ids: Vec<ObjectId>,
    /// The commit author's name.
    pub author_name: String,
    /// The commit author's email address.
    pub author_email: String,
    /// The commit's authorship date.
    pub authored_date: DateTime<Utc>,
    /// The committer's name.
    pub committer_name: String,
    /// The committer's email address.
    pub committer_email: String,
    /// The commit's commit date.
    pub committed_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    /// The full commit message.
    pub message: String,
    /// Statistics about the commit.
    pub stats: Option<RepoCommitStats>,
    /// The last pipeline for this commit, if any.
    pub last_pipeline: Option<PipelineBasic>,
    /// The project associated with the commit.
    pub project_id: ProjectId,
    // XXX: Investigate what this is.
    /// This looks to be CI related; ignoring without better docs.
    status: Value,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe snippet ID.
pub struct SnippetId(u64);
impl_id!(SnippetId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A project-specific snippet.
pub struct ProjectSnippet {
    /// The ID of the snippet.
    pub id: SnippetId,
    /// The title of the snippet.
    pub title: String,
    /// The name of the snippet.
    pub file_name: String,
    /// The author of the snippet.
    pub author: UserBasic,
    /// When the snippet was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the snippet was created.
    pub created_at: DateTime<Utc>,
    /// When the snippet was created.
    pub expires_at: Option<DateTime<Utc>>,
    /// The URL of the snippet.
    pub web_url: String,
}

// This is just used as a common "base class" in Ruby.
//#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
//#[derive(Serialize, Deserialize, Debug, Clone)]
//pub struct ProjectEntity {
//    pub id: ProjectEntityId,
//    pub iid: ProjectEntityInternalId,
//    pub project_id: ProjectId,
//    pub title: String,
//    pub description: String,
//    pub state: ProjectEntityState,
//    pub created_at: DateTime<Utc>,
//    pub updated_at: DateTime<Utc>,
//}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A diff within a repository.
pub struct RepoDiff {
    /// The path on the old side of the diff.
    pub old_path: String,
    /// The path on the new side of the diff.
    pub new_path: String,
    /// The mode on the old side of the diff.
    pub a_mode: String,
    /// The mode on the new side of the diff.
    pub b_mode: String,
    pub diff: String,
    /// Whether the diff indicates the addition of a file.
    pub new_file: bool,
    /// Whether the diff indicates the rename of a file.
    pub renamed_file: bool,
    /// Whether the diff indicates the deletion of a file.
    pub deleted_file: bool,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DiffRefs {
    /// SHA referencing base commit in the source branch
    pub base_sha: Option<ObjectId>,
    /// SHA referencing head commit in the source branch
    pub head_sha: Option<ObjectId>,
    /// SHA referencing commit in target branch
    pub start_sha: Option<ObjectId>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe milestone ID.
pub struct MilestoneId(u64);
impl_id!(MilestoneId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe milestone internal ID (internal to a project).
pub struct MilestoneInternalId(u64);
impl_id!(MilestoneInternalId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The states a milestone may be in.
pub enum MilestoneState {
    /// The milestone is active.
    Active,
    /// The milestone has been closed.
    Closed,
}
enum_serialize!(MilestoneState -> "milestone type",
    Active => "active",
    Closed => "closed",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A milestone in a project.
pub struct Milestone {
    /// The ID of the milestone.
    pub id: MilestoneId,
    /// The user-visible ID of the milestone.
    pub iid: MilestoneInternalId,
    /// The ID of the project if this is a project milestone.
    pub project_id: Option<ProjectId>,
    /// The ID of the group if this is a group milestone.
    pub group_id: Option<GroupId>,
    /// The title of the milestone.
    pub title: String,
    /// The description of the milestone.
    pub description: String,
    /// The state of the milestone.
    pub state: MilestoneState,
    /// When the milestone was created.
    pub created_at: DateTime<Utc>,
    /// When the milestone was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the milestone is due.
    pub due_date: Option<NaiveDate>,
    /// When the milestone was started.
    pub start_date: Option<NaiveDate>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe issue ID.
pub struct IssueId(u64);
impl_id!(IssueId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe issue internal ID (internal to a project).
pub struct IssueInternalId(u64);
impl_id!(IssueInternalId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The states an issue may be in.
pub enum IssueState {
    /// The issue is open.
    Opened,
    /// The issue has been closed.
    Closed,
    /// The issue has been opened after being closed.
    Reopened,
}
enum_serialize!(IssueState -> "issue type",
    Opened => "opened",
    Closed => "closed",
    Reopened => "reopened",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct IssueLinks {
    #[serde(rename="self")]
    /// API URL of issue itself.
    self_: String,
    /// API URL of issue notes.
    notes: String,
    /// API URL of issue award emoji.
    award_emoji: String,
    /// API URL of issue project.
    project: String,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An issue on a project.
pub struct Issue {
    /// The ID of the issue.
    pub id: IssueId,
    /// The user-visible ID of the issue.
    pub iid: IssueInternalId,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The title of the issue.
    pub title: String,
    /// The description of the issue.
    pub description: Option<String>,
    /// The state of the issue.
    pub state: IssueState,
    /// When the issue was created.
    pub created_at: DateTime<Utc>,
    /// When the issue was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the issue was closed, if closed.
    pub closed_at: Option<DateTime<Utc>>,
    /// The user that closed the issue.
    pub closed_by: Option<UserBasic>,
    /// The labels attached to the issue.
    pub labels: Vec<String>,
    /// The milestone of the issue.
    pub milestone: Option<Milestone>,
    /// The author of the issue.
    pub author: UserBasic,
    /// The assignee of the issue.
    pub assignee: Option<UserBasic>,
    /// The assignees of the issue.
    pub assignees: Option<Vec<UserBasic>>,
    /// Whether the current user is subscribed or not.
    /// GitLab does not include this in responses with lists of issues but
    /// does on an individual issue.
    pub subscribed: Option<bool>,
    /// Time estimates.
    pub time_stats: IssuableTimeStats,
    /// The number of comments on the issue.
    pub user_notes_count: u64,
    /// The number of upvotes for the issue.
    pub upvotes: u64,
    /// The number of downvotes against the issue.
    pub downvotes: u64,
    /// When the issue is due.
    pub due_date: Option<NaiveDate>,
    /// Whether the issue is confidential or not.
    pub confidential: bool,
    /// Whether the discussion has been locked.
    pub discussion_locked: Option<bool>,
    /// The URL of the issue.
    pub web_url: String,

    /// Links to related API URLs provided by GitLab in response to
    /// direct issue lookup.  We do not expose this because our
    /// clients do not need them.
    _links: Option<IssueLinks>,
}

#[cfg(test)]
impl Issue {
    pub fn has_links(&self) -> bool {
        self._links.is_some()
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A time estimate on an issue or merge request.
pub struct IssuableTimeStats {
    /// The time estimate, in seconds.
    pub time_estimate: u64,
    /// The total time spent, in seconds.
    pub total_time_spent: u64,
    /// The time estimate, as a human-readable string.
    pub human_time_estimate: Option<String>,
    /// The total time spent, as a human-readable string.
    pub human_total_time_spent: Option<String>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe external issue ID.
pub struct ExternalIssueId(u64);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An external issue reference.
pub struct ExternalIssue {
    /// The ID of the issue.
    pub id: ExternalIssueId,
    /// The title of the issue.
    pub title: String,
}

#[derive(Debug, Clone)]
/// A reference to an issue.
pub enum IssueReference {
    /// A reference to an issue on the same Gitlab host.
    Internal(Issue),
    /// An external issue reference.
    External(ExternalIssue),
}

impl Serialize for IssueReference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            IssueReference::Internal(ref issue) => issue.serialize(serializer),
            IssueReference::External(ref issue) => issue.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for IssueReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        let val = <Value as Deserialize>::deserialize(deserializer)?;

        serde_json::from_value::<Issue>(val.clone())
            .map(IssueReference::Internal)
            .or_else(|_| serde_json::from_value::<ExternalIssue>(val).map(IssueReference::External))
            .map_err(|err| D::Error::custom(format!("invalid issue reference: {:?}", err)))
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe merge request ID.
pub struct MergeRequestId(u64);
impl_id!(MergeRequestId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe merge request internal ID (internal to a project).
pub struct MergeRequestInternalId(u64);
impl_id!(MergeRequestInternalId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The status of the possible merge for a merge request.
pub enum MergeStatus {
    /// The merge request has not been checked yet.
    Unchecked,
    /// The merge request may be merged.
    CanBeMerged,
    /// The merge request may not be merged yet.
    CannotBeMerged,
    /// The merge request has not been checked but previously
    /// could not be merged.
    CannotBeMergedRecheck,
}
enum_serialize!(MergeStatus -> "merge status",
    Unchecked => "unchecked",
    CanBeMerged => "can_be_merged",
    CannotBeMerged => "cannot_be_merged",
    CannotBeMergedRecheck => "cannot_be_merged_recheck",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The states a merge request may be in.
pub enum MergeRequestState {
    /// The merge request is open.
    Opened,
    /// The merge request has been closed before merging.
    Closed,
    /// The merge request has been opened after closing.
    Reopened,
    /// The merge request has been merged.
    Merged,
    /// The merge request is locked from further discussion or updates.
    Locked,
}
enum_serialize!(MergeRequestState -> "merge request state",
    Opened => "opened",
    Closed => "closed",
    Reopened => "reopened",
    Merged => "merged",
    Locked => "locked",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A merge request.
pub struct MergeRequest {
    /// The ID of the merge request.
    pub id: MergeRequestId,
    /// The user-visible ID of the merge request.
    pub iid: MergeRequestInternalId,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The title of the merge request.
    pub title: String,
    /// The description of the merge request.
    pub description: Option<String>,
    /// The state of the merge request.
    pub state: MergeRequestState,
    /// When the merge request was created.
    pub created_at: DateTime<Utc>,
    /// When the merge request was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the merge request was merged.
    pub merged_at: Option<DateTime<Utc>>,
    /// When the merge request was closed.
    pub closed_at: Option<DateTime<Utc>>,
    /// The user that merged the merge request.
    pub merged_by: Option<UserBasic>,
    /// The user that closed the merge request.
    pub closed_by: Option<UserBasic>,
    /// The target branch of the merge request.
    pub target_branch: String,
    /// The source branch of the merge request.
    pub source_branch: String,
    /// The number of upvotes for the merge request.
    pub upvotes: u64,
    /// The number of downvotes against the merge request.
    pub downvotes: u64,
    /// The author of the merge request.
    pub author: UserBasic,
    /// The assignee of the merge request.
    pub assignee: Option<UserBasic>,
    /// The assignees of the merge request.
    pub assignees: Option<Vec<UserBasic>>,
    /// The ID of the project hosting the source branch.
    pub source_project_id: ProjectId,
    /// The ID of the project hosting the target branch.
    pub target_project_id: ProjectId,
    /// The labels attached to the merge request.
    pub labels: Vec<String>,
    /// Whether the merge request is a work-in-progress or not.
    pub work_in_progress: bool,
    /// Whether the merge request allows a maintainer to collaborate.
    pub allow_collaboration: Option<bool>,
    /// Whether the merge request allows a maintainer to push (deprecated).
    pub allow_maintainer_to_push: Option<bool>,
    /// The milestone of the merge request.
    pub milestone: Option<Milestone>,
    /// Whether to squash commits on merge.
    pub squash: bool,
    /// Whether the merge request will be merged once all pipelines succeed or not.
    pub merge_when_pipeline_succeeds: bool,
    /// The status of the merge request.
    pub merge_status: MergeStatus,
    /// The object ID of the head of the source branch.
    ///
    /// This is `None` if the source branch has been deleted.
    pub sha: Option<ObjectId>,
    /// The commits used to construct the merge request diffs.
    pub diff_refs: DiffRefs,
    /// The object ID of the commit which merged the merge request.
    pub merge_commit_sha: Option<ObjectId>,
    /// Whether the current user is subscribed or not.
    /// GitLab does not include this in responses with lists of merge requests but
    /// does on an individual merge request.
    pub subscribed: Option<bool>,
    /// Time estimates.
    pub time_stats: IssuableTimeStats,
    /// The number of paths changed by the merge request.
    ///
    /// This is an integer suffixed by `+` if there are more files changed than some threshold
    /// (probably determined by a timeout).
    pub changes_count: Option<String>,
    /// The number of comments on the merge request.
    pub user_notes_count: u64,
    /// Whether the discussion has been locked.
    pub discussion_locked: Option<bool>,
    /// Whether the merge request should be deleted or not (set by the merger).
    pub should_remove_source_branch: Option<bool>,
    /// Whether the merge request should be deleted or not (set by the author).
    pub force_remove_source_branch: Option<bool>,
    /// The URL of the merge request.
    pub web_url: String,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A merge request with changes.
pub struct MergeRequestChanges {
    /// The ID of the merge request.
    pub id: MergeRequestId,
    /// The user-visible ID of the merge request.
    pub iid: MergeRequestInternalId,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The title of the merge request.
    pub title: String,
    /// The description of the merge request.
    pub description: Option<String>,
    /// The state of the merge request.
    pub state: MergeRequestState,
    /// When the merge request was created.
    pub created_at: DateTime<Utc>,
    /// When the merge request was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the merge request was merged.
    pub merged_at: Option<DateTime<Utc>>,
    /// When the merge request was closed.
    pub closed_at: Option<DateTime<Utc>>,
    /// The user that merged the merge request.
    pub merged_by: Option<UserBasic>,
    /// The user that closed the merge request.
    pub closed_by: Option<UserBasic>,
    /// The target branch of the merge request.
    pub target_branch: String,
    /// The source branch of the merge request.
    pub source_branch: String,
    /// The number of upvotes for the merge request.
    pub upvotes: u64,
    /// The number of downvotes against the merge request.
    pub downvotes: u64,
    /// The author of the merge request.
    pub author: UserBasic,
    /// The assignee of the merge request.
    pub assignee: Option<UserBasic>,
    /// The assignees of the merge request.
    pub assignees: Option<Vec<UserBasic>>,
    /// The ID of the project hosting the source branch.
    pub source_project_id: ProjectId,
    /// The ID of the project hosting the target branch.
    pub target_project_id: ProjectId,
    /// The labels attached to the merge request.
    pub labels: Vec<String>,
    /// Whether the merge request is a work-in-progress or not.
    pub work_in_progress: bool,
    /// Whether the merge request allows a maintainer to collaborate.
    pub allow_collaboration: Option<bool>,
    /// Whether the merge request allows a maintainer to push (deprecated).
    pub allow_maintainer_to_push: Option<bool>,
    /// The milestone of the merge request.
    pub milestone: Option<Milestone>,
    /// Whether to squash commits on merge.
    pub squash: bool,
    /// Whether the merge request will be merged once all jobs succeed or not.
    pub merge_when_pipeline_succeeds: bool,
    /// The status of the merge request.
    pub merge_status: MergeStatus,
    /// The object ID of the head of the source branch.
    ///
    /// This is `None` if the source branch has been deleted.
    pub sha: Option<ObjectId>,
    /// The commits used to construct the merge request diffs.
    pub diff_refs: DiffRefs,
    /// The object ID of the commit which merged the merge request.
    pub merge_commit_sha: Option<ObjectId>,
    /// GitLab does not include this in responses with lists of merge requests but
    /// does on an individual merge request.
    pub subscribed: Option<bool>,
    /// Time estimates.
    pub time_stats: IssuableTimeStats,
    /// The number of paths changed by the merge request.
    pub changes_count: Option<String>,
    /// The number of comments on the merge request.
    pub user_notes_count: u64,
    /// Whether the discussion has been locked.
    pub discussion_locked: Option<bool>,
    /// Whether the merge request should be deleted or not (set by the merger).
    pub should_remove_source_branch: Option<bool>,
    /// Whether the merge request should be deleted or not (set by the author).
    pub force_remove_source_branch: Option<bool>,
    /// The URL of the merge request.
    pub web_url: String,
    pub changes: Vec<RepoDiff>,
}

impl From<MergeRequestChanges> for MergeRequest {
    fn from(mr: MergeRequestChanges) -> Self {
        MergeRequest {
            id: mr.id,
            iid: mr.iid,
            project_id: mr.project_id,
            title: mr.title,
            description: mr.description,
            state: mr.state,
            created_at: mr.created_at,
            updated_at: mr.updated_at,
            merged_at: mr.merged_at,
            closed_at: mr.closed_at,
            merged_by: mr.merged_by,
            closed_by: mr.closed_by,
            target_branch: mr.target_branch,
            source_branch: mr.source_branch,
            upvotes: mr.upvotes,
            downvotes: mr.downvotes,
            author: mr.author,
            assignee: mr.assignee,
            assignees: mr.assignees,
            source_project_id: mr.source_project_id,
            target_project_id: mr.target_project_id,
            labels: mr.labels,
            work_in_progress: mr.work_in_progress,
            allow_collaboration: mr.allow_collaboration,
            allow_maintainer_to_push: mr.allow_maintainer_to_push,
            milestone: mr.milestone,
            squash: mr.squash,
            merge_when_pipeline_succeeds: mr.merge_when_pipeline_succeeds,
            merge_status: mr.merge_status,
            sha: mr.sha,
            diff_refs: mr.diff_refs,
            merge_commit_sha: mr.merge_commit_sha,
            subscribed: mr.subscribed,
            time_stats: mr.time_stats,
            changes_count: mr.changes_count,
            user_notes_count: mr.user_notes_count,
            discussion_locked: mr.discussion_locked,
            should_remove_source_branch: mr.should_remove_source_branch,
            force_remove_source_branch: mr.force_remove_source_branch,
            web_url: mr.web_url,
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe SSH key ID.
pub struct SshKeyId(u64);
impl_id!(SshKeyId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An uploaded SSH key.
pub struct SshKey {
    /// The ID of the SSH key.
    pub id: SshKeyId,
    /// The title of the key.
    pub title: String,
    /// The public half of the SSH key.
    pub key: String,
    /// When the key was created.
    pub created_at: DateTime<Utc>,
    /// Whether the key may push to repositories or not.
    pub can_push: bool,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An uploaded SSH key with its owner.
pub struct SshKeyWithUser {
    /// The ID of the SSH key.
    pub id: SshKeyId,
    /// The title of the key.
    pub title: String,
    /// The public half of the SSH key.
    pub key: String,
    /// When the key was created.
    pub created_at: DateTime<Utc>,
    /// The user associated with the SSH key.
    pub user: UserPublic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The entities a note may be added to.
pub enum NoteType {
    /// A note on a commit.
    Commit,
    /// A note on an issue.
    Issue,
    /// A note on a merge request.
    MergeRequest,
    /// A note on a snippet.
    Snippet,
}
enum_serialize!(NoteType -> "note type",
    Commit => "Commit",
    Issue => "Issue",
    MergeRequest => "MergeRequest",
    Snippet => "Snippet",
);

#[derive(Debug, Clone, PartialEq, Eq)]
/// The ID of an entity a note is attached to.
pub enum NoteableId {
    /// The ID of the commit for a commit note.
    Commit(ObjectId),
    /// The ID of the issue for an issue note.
    Issue(IssueId),
    /// The ID of the merge request for a merge request note.
    MergeRequest(MergeRequestId),
    /// The ID of the snippet for a snippet note.
    Snippet(SnippetId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The internal ID of an entity a note is attached to (internal to a project).
/// GitLab only has this for notes attached to issues and merge requests.
pub enum NoteableInternalId {
    /// The internal ID of the issue for an issue note.
    Issue(IssueInternalId),
    /// The internal ID of the merge request for a merge request note.
    MergeRequest(MergeRequestInternalId),
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe note (comment) ID.
pub struct NoteId(u64);
impl_id!(NoteId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A comment on an entity.
pub struct Note {
    /// The ID of the note.
    pub id: NoteId,
    /// The content of the note.
    pub body: String,
    /// The URL of an attachment to the note.
    pub attachment: Option<String>,
    /// The author of the note.
    pub author: UserBasic,
    /// When the note was created.
    pub created_at: DateTime<Utc>,
    /// When the note was last updated.
    pub updated_at: DateTime<Utc>,
    /// Whether the note can be resolved.
    pub resolvable: bool,
    /// Whether the note has been resolved.
    pub resolved: Option<bool>,
    /// The user that resolved the note.
    pub resolved_by: Option<UserBasic>,
    /// Whether the note was created by a user or in response to an external action.
    ///
    /// System notes include indications that the commit, issue, etc. was referenced elsewhere, a
    /// milestone, assignee, or label change, status chages, and so on.
    pub system: bool,
    // Keep as JSON because its type depends on what `noteable_type` is.
    noteable_id: Value,
    // Keep as JSON because its type depends on what `noteable_type` is.
    noteable_iid: Option<Value>,
    /// The type of entity the note is attached to.
    pub noteable_type: NoteType,
}

impl Note {
    /// The ID of the entity the note is attached to.
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

    /// The internal ID of the entity the note is attached to (internal to a project).
    /// This is available only for notes attached to issues and merge requests.
    pub fn noteable_iid(&self) -> Option<NoteableInternalId> {
        match self.noteable_type {
            NoteType::Commit => {
                None
            },
            NoteType::Issue => {
                self.noteable_iid
                    .as_ref()
                    .and_then(|value| value.as_u64())
                    .map(|id| NoteableInternalId::Issue(IssueInternalId::new(id)))
            },
            NoteType::MergeRequest => {
                self.noteable_iid
                    .as_ref()
                    .and_then(|value| value.as_u64())
                    .map(|id| NoteableInternalId::MergeRequest(MergeRequestInternalId::new(id)))
            },
            NoteType::Snippet => {
                None
            },
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe award ID.
pub struct AwardId(u64);
impl_id!(AwardId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// An ID of an entity which may receive an award.
pub enum AwardableId {
    /// The ID of an awarded issue.
    Issue(IssueId),
    /// The ID of an awarded merge request.
    MergeRequest(MergeRequestId),
    /// The ID of an awarded snippet.
    Snippet(SnippetId),
    /// The ID of an awarded note.
    Note(NoteId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The entities which may be awarded.
pub enum AwardableType {
    /// An award on an issue.
    Issue,
    /// An award on a merge request.
    MergeRequest,
    /// An award on a snippet.
    Snippet,
    /// An award on a note.
    Note,
}
enum_serialize!(AwardableType -> "awardable type",
    Issue => "Issue",
    MergeRequest => "MergeRequest",
    Snippet => "Snippet",
    Note => "Note",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An awarded emoji on an entity.
pub struct AwardEmoji {
    /// The ID of the award.
    pub id: AwardId,
    /// The name of the awarded emoji.
    pub name: String,
    /// The user which created the award.
    pub user: UserBasic,
    /// When the award was created.
    pub created_at: DateTime<Utc>,
    /// When the award was last updated.
    pub updated_at: DateTime<Utc>,
    awardable_id: u64,
    /// The type of entity that is awarded.
    pub awardable_type: AwardableType,
}

impl AwardEmoji {
    /// The ID of the entity the award is attached to.
    pub fn awardable_id(&self) -> AwardableId {
        match self.awardable_type {
            AwardableType::Issue => AwardableId::Issue(IssueId::new(self.awardable_id)),
            AwardableType::MergeRequest => {
                AwardableId::MergeRequest(MergeRequestId::new(self.awardable_id))
            },
            AwardableType::Snippet => AwardableId::Snippet(SnippetId::new(self.awardable_id)),
            AwardableType::Note => AwardableId::Note(NoteId::new(self.awardable_id)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The type of line commented on.
pub enum LineType {
    /// An added line was commented on.
    New,
    /// An deleted line was commented on.
    Old,
}
enum_serialize!(LineType -> "line type",
    New => "new",
    Old => "old",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A note on a commit diff.
pub struct CommitNote {
    /// The content of the note.
    pub note: String,
    /// The path of the file commented on.
    pub path: Option<String>,
    /// The line of the file commented on.
    pub line: Option<u64>,
    /// The type of the line commented on.
    pub line_type: Option<LineType>,
    /// The author of the note.
    pub author: UserBasic,
    /// When the note was created.
    pub created_at: DateTime<Utc>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe commit status ID.
pub struct CommitStatusId(u64);
impl_id!(CommitStatusId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// States for commit statuses.
pub enum StatusState {
    /// The check is queued.
    Pending,
    /// The check is currently running.
    Running,
    /// The check succeeded.
    Success,
    /// The check failed.
    Failed,
    /// The check was canceled.
    Canceled,
}
enum_serialize!(StatusState -> "status state",
    Pending => "pending",
    Running => "running",
    Success => "success",
    Failed => "failed",
    Canceled => "canceled",
);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A status of a commit.
pub struct CommitStatus {
    /// The ID of the commit status.
    pub id: CommitStatusId,
    /// The object ID of the commit this status is for.
    pub sha: ObjectId,
    #[serde(rename="ref")]
    /// The name of the reference the status was created for.
    pub ref_: Option<String>,
    /// The state of the commit status.
    pub status: StatusState,
    /// The name of the commit status.
    pub name: String,
    /// The URL associated with the commit status.
    pub target_url: Option<String>,
    /// The description of the commit status.
    pub description: Option<String>,
    /// When the commit status was created.
    pub created_at: DateTime<Utc>,
    /// When the commit status started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the commit status completed.
    pub finished_at: Option<DateTime<Utc>>,
    /// Whether the commit status is allowed to fail.
    pub allow_failure: bool,
    pub coverage: Option<u64>,
    /// The author of the commit status.
    pub author: UserBasic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The target of an event.
pub enum EventTargetType {
    /// An event targeted a commit.
    Commit,
    /// An event targeted an issue.
    Issue,
    /// An event targeted a merge request.
    MergeRequest,
    /// An event targeted a snippet.
    Snippet,
    /// An event targeted a project snippet.
    ProjectSnippet,
}
enum_serialize!(EventTargetType -> "event target type",
    Commit => "commit",
    Issue => "issue",
    MergeRequest => "merge_request",
    Snippet => "snippet",
    ProjectSnippet => "project_snippet",
);

#[derive(Debug, Clone, PartialEq, Eq)]
/// The ID of an event target.
pub enum EventTargetId {
    /// The object ID of a commit event target.
    Commit(ObjectId),
    /// The ID of an issue event target.
    Issue(IssueId),
    /// The ID of a merge request event target.
    MergeRequest(MergeRequestId),
    /// The ID of a snippet event target.
    Snippet(SnippetId),
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An event on a project.
pub struct Event {
    /// The title of the event.
    pub title: Option<String>,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The action which triggered the event.
    // FIXME: This should be an enumeration.
    pub action_name: String,
    target_id: Value,
    /// The type of the event target.
    pub target_type: EventTargetType,
    /// The ID of the author of the event.
    pub author_id: UserId,
    pub data: Option<Value>,
    /// The title of the target.
    pub target_title: String,
    /// When the event was created.
    pub created_at: DateTime<Utc>,
    pub note: Option<Note>,
    /// The author of the event.
    pub author: Option<UserBasic>,
    /// The handle of the author.
    pub author_username: Option<String>,
}

impl Event {
    /// The ID of an event's target.
    pub fn target_id(&self) -> Option<EventTargetId> {
        match self.target_type {
            EventTargetType::Commit => {
                self.target_id
                    .as_str()
                    .map(|id| EventTargetId::Commit(ObjectId(id.to_string())))
            },
            EventTargetType::Issue => {
                self.target_id
                    .as_u64()
                    .map(|id| EventTargetId::Issue(IssueId(id)))
            },
            EventTargetType::MergeRequest => {
                self.target_id
                    .as_u64()
                    .map(|id| EventTargetId::MergeRequest(MergeRequestId(id)))
            },
            EventTargetType::Snippet => {
                self.target_id
                    .as_u64()
                    .map(|id| EventTargetId::Snippet(SnippetId(id)))
            },
            EventTargetType::ProjectSnippet => {
                self.target_id
                    .as_u64()
                    .map(|id| EventTargetId::Snippet(SnippetId(id)))
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kinds of namespaces supported by Gitlab.
pub enum NamespaceKind {
    /// A user namespace.
    User,
    /// A group namespace.
    Group,
}
enum_serialize!(NamespaceKind -> "namespace kind",
    User => "user",
    Group => "group",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The ID of a namespace.
pub enum NamespaceId {
    /// A user namespace ID.
    User(UserId),
    /// A group namespace ID.
    Group(GroupId),
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An entity which can own projects.
pub struct Namespace {
    id: u64,
    /// The URL of the namespace.
    pub path: String,
    /// The name of the namespace.
    pub name: String,
    /// The kind of the namespace.
    pub kind: NamespaceKind,
    pub full_path: String,
    /// Number of members in the namespace and its descendants.
    ///
    /// Only available when talking to GitLab as a user that can admin the namespace.
    pub members_count_with_descendants: Option<u64>,
}

impl Namespace {
    /// The ID of the namespace.
    pub fn id(&self) -> NamespaceId {
        match self.kind {
            NamespaceKind::User => NamespaceId::User(UserId(self.id)),
            NamespaceKind::Group => NamespaceId::Group(GroupId(self.id)),
        }
    }
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe runner ID.
pub struct RunnerId(u64);
impl_id!(RunnerId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// A Gitlab CI runner.
pub struct Runner {
    /// The ID of the runner.
    pub id: RunnerId,
    /// The description of the runner.
    pub description: Option<String>,
    /// Whether the runner is active or not.
    pub active: bool,
    /// Whether the runner is shared or not.
    pub is_shared: bool,
    /// The name of the runner.
    pub name: Option<String>,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// An uploaded artifact from a job.
pub struct JobArtifactFile {
    /// The name of the artifact.
    pub filename: String,
    /// The size (in bytes) of the artifact.
    pub size: usize,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe job ID.
pub struct JobId(u64);
impl_id!(JobId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Information about a job in Gitlab CI.
pub struct Job {
    /// The ID of the job.
    pub id: JobId,
    /// The status of the job.
    pub status: StatusState,
    pub stage: String,
    /// The name of the job.
    pub name: String,
    #[serde(rename="ref")]
    /// The name of the reference that was tested.
    pub ref_: Option<String>,
    pub tag: bool,
    pub coverage: Option<f32>,
    /// When the job was created or marked as pending.
    pub created_at: DateTime<Utc>,
    /// When the job was started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the job completed.
    pub finished_at: Option<DateTime<Utc>>,
    /// The user which ran the job.
    pub user: Option<User>,
    /// The artifact file uploaded from the job.
    pub artifacts_file: Option<JobArtifactFile>,
    /// The commit the job tested.
    pub commit: RepoCommit,
    /// The runner which ran the job.
    pub runner: Option<Runner>,
    /// The pipeline the job belongs to.
    pub pipeline: PipelineBasic,
}

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// Type-safe pipeline ID.
pub struct PipelineId(u64);
impl_id!(PipelineId);

#[cfg_attr(feature="strict", serde(deny_unknown_fields))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Information about a pipeline in Gitlab CI.
pub struct PipelineBasic {
    /// The ID of the pipeline.
    pub id: PipelineId,
    #[serde(rename="ref")]
    /// The name of the reference that was tested.
    pub ref_: Option<String>,
    /// The object ID that was tested.
    pub sha: ObjectId,
    /// The status of the pipeline.
    pub status: StatusState,
}
