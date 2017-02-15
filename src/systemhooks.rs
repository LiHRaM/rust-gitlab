// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! System hook structures
//!
//! These hooks are received from Gitlab when registered as a system hook in the administrator
//! settings. Only administrators may create such hooks.
//!
//! Gitlab does not have consistent structures for its hooks, so they often change from
//! version to version.

extern crate chrono;
use self::chrono::{DateTime, UTC};

extern crate serde;
use self::serde::{Deserialize, Deserializer, Serialize, Serializer};
use self::serde::de::{Error, Unexpected};

extern crate serde_json;
use self::serde_json::Value;

use types::{AccessLevel, GroupId, ObjectId, ProjectId, SshKeyId, UserId};
use webhooks::{CommitHookAttrs, ProjectHookAttrs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Events which occur at the project level.
pub enum ProjectEvent {
    /// A project was created.
    Create,
    /// A project was deleted.
    Destroy,
    /// A project was renamed.
    Rename,
    /// A project moved from one namespace to another.
    Transfer,
}
enum_serialize!(ProjectEvent -> "project event",
    Create => "project_create",
    Destroy => "project_destroy",
    Rename => "project_rename",
    Transfer => "project_transfer",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Visibility levels for projects.
pub enum ProjectVisibility {
    /// The project is only visible to users who are logged in.
    Internal,
    /// The project is only visible to team members.
    Private,
    /// The project is visible to everyone.
    Public,
}
enum_serialize!(ProjectVisibility -> "project visibility",
    Internal => "internal",
    Private => "private",
    Public => "public",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A hook for a project.
pub struct ProjectSystemHook {
    /// The event which occurred.
    pub event_name: ProjectEvent,
    /// When the project was created.
    pub created_at: DateTime<UTC>,
    /// When the project was last updated.
    pub updated_at: DateTime<UTC>,

    /// The display name of the project.
    pub name: String,
    /// The email address of the owner.
    pub owner_email: String,
    /// The name of the owner.
    pub owner_name: String,
    /// The path of the project (used for URLs).
    pub path: String,
    /// The namespace and path of the project.
    pub path_with_namespace: String,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The visibility level of the project.
    pub project_visibility: ProjectVisibility,
    /// The old namespace and path of the project for `Rename` and `Transfer` events.
    pub old_path_with_namespace: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Events which occur when users are added and removed from projects.
pub enum ProjectMemberEvent {
    /// A user was added to a project.
    Add,
    /// A user was removed from a project.
    Remove,
}
enum_serialize!(ProjectMemberEvent -> "project member event",
    Add => "user_add_to_team",
    Remove => "user_remove_from_team",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Access levels for groups and projects.
pub enum HumanAccessLevel {
    /// Guest access (can see the project).
    Guest,
    /// Reporter access (can open issues).
    Reporter,
    /// Developer access (can push branches, handle issues and merge requests).
    Developer,
    /// Master access (can push to protected branches).
    Master,
    /// Owner access (full rights).
    Owner,
}
enum_serialize!(HumanAccessLevel -> "human access level",
    Guest => "Guest",
    Reporter => "Reporter",
    Developer => "Developer",
    Master => "Master",
    Owner => "Owner",
);

impl From<HumanAccessLevel> for AccessLevel {
    fn from(access: HumanAccessLevel) -> Self {
        match access {
            HumanAccessLevel::Guest => AccessLevel::Guest,
            HumanAccessLevel::Reporter => AccessLevel::Reporter,
            HumanAccessLevel::Developer => AccessLevel::Developer,
            HumanAccessLevel::Master => AccessLevel::Master,
            HumanAccessLevel::Owner => AccessLevel::Owner,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A project membership hook.
pub struct ProjectMemberSystemHook {
    /// The event which occurred.
    pub event_name: ProjectMemberEvent,
    /// When the membership was created.
    pub created_at: DateTime<UTC>,
    /// When the membership was last updated.
    pub updated_at: DateTime<UTC>,
    /// The name of the project.
    pub project_name: String,
    /// The path of the project (used for URLs).
    pub project_path: String,
    /// The namespace and path of the project (used for URLs).
    pub project_path_with_namespace: String,
    /// The ID of the project.
    pub project_id: ProjectId,
    /// The username of the user added as a member.
    pub user_username: String,
    /// The name of the user added as a member.
    pub user_name: String,
    /// The email address of the user added as a member.
    pub user_email: String,
    /// The ID of the user.
    pub user_id: UserId,
    /// The access level granted to the user.
    pub access_level: HumanAccessLevel,
    /// The visibility of the project.
    pub project_visibility: ProjectVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Events which occur for user accounts.
pub enum UserEvent {
    /// The user account was created.
    Create,
    /// The user account was deleted.
    Destroy,
}
enum_serialize!(UserEvent -> "user event",
    Create => "user_create",
    Destroy => "user_destroy",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A user hook.
pub struct UserSystemHook {
    /// The event which occurred.
    pub event_name: UserEvent,
    /// When the user account was created.
    pub created_at: DateTime<UTC>,
    /// When the user account was last updated.
    pub updated_at: DateTime<UTC>,
    /// The name of the user.
    pub name: String,
    /// The email address of the user.
    pub email: String,
    /// The ID of the user.
    pub user_id: UserId,
    /// The username of the user.
    pub username: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Events which occur for SSH keys.
pub enum KeyEvent {
    /// An SSH key was uploaded.
    Create,
    /// An SSH key was deleted.
    Destroy,
}
enum_serialize!(KeyEvent -> "key event",
    Create => "key_create",
    Destroy => "key_destroy",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// An SSH key hook.
pub struct KeySystemHook {
    /// The event which occurred.
    pub event_name: KeyEvent,
    /// When the key was added.
    pub created_at: DateTime<UTC>,
    /// When the key was last updated.
    pub updated_at: DateTime<UTC>,
    /// The username of the user.
    pub username: String,
    /// The content of the key.
    pub key: String,
    /// The ID of the key.
    pub id: SshKeyId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Events which occur for groups.
pub enum GroupEvent {
    /// The group was created.
    Create,
    /// The group was deleted.
    Destroy,
}
enum_serialize!(GroupEvent -> "group event",
    Create => "group_create",
    Destroy => "group_destroy",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A group hook.
pub struct GroupSystemHook {
    /// The event which occurred.
    pub event_name: GroupEvent,
    /// When the group was created.
    pub created_at: DateTime<UTC>,
    /// When the group was last updated.
    pub updated_at: DateTime<UTC>,
    /// The name of the group.
    pub name: String,
    /// The path of the group (used for URLs).
    pub path: String,
    /// The ID of the group.
    pub group_id: GroupId,
    /// The email address of the owner of the group.
    pub owner_email: Option<String>,
    /// The name of the owner of the group.
    pub owner_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Events which occur for group memberships.
pub enum GroupMemberEvent {
    /// A user was added to the group.
    Add,
    /// A user was removed from the group.
    Remove,
}
enum_serialize!(GroupMemberEvent -> "group member event",
    Add => "user_add_to_group",
    Remove => "user_remove_from_group",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A group membership hook.
pub struct GroupMemberSystemHook {
    /// The event which occurred.
    pub event_name: GroupMemberEvent,
    /// When the group membership was added.
    pub created_at: DateTime<UTC>,
    /// When the group membership was last updated.
    pub updated_at: DateTime<UTC>,
    /// The name of the group.
    pub group_name: String,
    /// The path of the group (used for URLs).
    pub group_path: String,
    /// The ID of the group.
    pub group_id: GroupId,
    /// The username of the user.
    pub user_username: String,
    /// The name of the user.
    pub user_name: String,
    /// The email address of the user.
    pub user_email: String,
    /// The ID of the user.
    pub user_id: UserId,
    /// The access level of the user.
    pub group_access: HumanAccessLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Events which occur when a push happens.
pub enum PushEvent {
    /// A non-tag push occurred.
    Push,
    /// A tag was pushed.
    TagPush,
}
enum_serialize!(PushEvent -> "push event",
    Push => "push",
    TagPush => "tag_push",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A push hook.
pub struct PushSystemHook {
    /// The event which occurred.
    pub event_name: PushEvent,
    /// When the push occurred.
    pub created_at: DateTime<UTC>,
    /// When the push
    pub updated_at: DateTime<UTC>,
    /// The old object ID of the ref that was pushed.
    pub before: ObjectId,
    /// The new object ID of the ref that was pushed.
    pub after: ObjectId,
    #[serde(rename="ref")]
    /// The name of the reference that was pushed.
    pub ref_: String,
    /// The new object ID of the ref that was pushed.
    pub checkout_sha: ObjectId,
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
    pub commits: Vec<CommitHookAttrs>,
    /// The total number of commits pushed.
    pub total_commits_count: u64,
}

#[derive(Debug, Clone)]
/// A deserializable structure for all Gitlab system hooks.
pub enum SystemHook {
    /// A project hook.
    Project(ProjectSystemHook),
    /// A project membership hook.
    ProjectMember(ProjectMemberSystemHook),
    /// A user account hook.
    User(UserSystemHook),
    /// An SSH key hook.
    Key(KeySystemHook),
    /// A group hook.
    Group(GroupSystemHook),
    /// A group membership hook.
    GroupMember(GroupMemberSystemHook),
    /// A push hook.
    Push(PushSystemHook),
}

impl Deserialize for SystemHook {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        let val = <Value as Deserialize>::deserialize(deserializer)?;

        let event_name = match val.pointer("/event_name") {
            Some(&Value::String(ref name)) => name.to_string(),
            Some(_) => {
                return Err(D::Error::invalid_type(Unexpected::Other("JSON value"), &"a string"));
            },
            None => {
                return Err(D::Error::missing_field("event_name"));
            },
        };

        let hook_res = match event_name.as_str() {
            "project_create" |
            "project_destroy" |
            "project_rename" |
            "project_transfer" => serde_json::from_value(val).map(SystemHook::Project),

            "user_add_to_team" |
            "user_remove_from_team" => serde_json::from_value(val).map(SystemHook::ProjectMember),

            "user_create" | "user_destroy" => serde_json::from_value(val).map(SystemHook::User),

            "key_create" | "key_destroy" => serde_json::from_value(val).map(SystemHook::Key),

            "group_create" | "group_destroy" => serde_json::from_value(val).map(SystemHook::Group),

            "user_add_to_group" |
            "user_remove_from_group" => serde_json::from_value(val).map(SystemHook::GroupMember),

            "push" | "tag_push" => serde_json::from_value(val).map(SystemHook::Push),

            _ => {
                return Err(D::Error::custom(format!("unrecognized system event name: {}",
                                                    event_name)));
            },
        };

        hook_res.map_err(|err| {
            D::Error::custom(format!("failed to deserialize a system hook: {:?}", err))
        })
    }
}
