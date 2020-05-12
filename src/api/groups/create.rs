// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, VisibilityLevel};
use crate::api::endpoint_prelude::*;

/// Access levels for creating a project within a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupProjectCreationAccessLevel {
    /// No new projects may be added to the group.
    NoOne,
    /// Only maintainers may add projects to the group.
    Maintainer,
    /// Developers and maintainers may add projects to the group.
    Developer,
}

impl GroupProjectCreationAccessLevel {
    fn as_str(self) -> &'static str {
        match self {
            GroupProjectCreationAccessLevel::NoOne => "noone",
            GroupProjectCreationAccessLevel::Maintainer => "maintainer",
            GroupProjectCreationAccessLevel::Developer => "developer",
        }
    }
}

/// Access levels for creating a subgroup within a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubgroupCreationAccessLevel {
    /// Owners may add new subprojects.
    Owner,
    /// Maintainers may add new subprojects.
    Maintainer,
}

impl SubgroupCreationAccessLevel {
    fn as_str(self) -> &'static str {
        match self {
            SubgroupCreationAccessLevel::Owner => "owner",
            SubgroupCreationAccessLevel::Maintainer => "maintainer",
        }
    }
}

/// Branch protection rules for groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchProtection {
    /// Developers and maintainers may push, force push, and delete branches.
    None,
    /// Developers and maintainers may push branches.
    Partial,
    /// Maintainers may push branches.
    Full,
}

impl BranchProtection {
    fn as_str(self) -> &'static str {
        match self {
            BranchProtection::None => "0",
            BranchProtection::Partial => "1",
            BranchProtection::Full => "2",
        }
    }
}

/// Create a new group on an instance.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateGroup<'a> {
    /// The name of the group.
    #[builder(setter(into))]
    name: Cow<'a, str>,
    /// The path of the group.
    #[builder(setter(into))]
    path: Cow<'a, str>,

    /// A short description for the group.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,
    /// Prevent adding members directly to projects within the group.
    #[builder(default)]
    membership_lock: Option<bool>,
    /// The visibility of the group.
    #[builder(default)]
    visibility: Option<VisibilityLevel>,
    /// Prevent sharing a project in this group with another group.
    #[builder(default)]
    share_with_group_lock: Option<bool>,
    /// Require two-factor authentication to be a member of this group.
    #[builder(default)]
    require_two_factor_authentication: Option<bool>,
    /// Time (in hours) for users to enable two-factor before enforcing it.
    #[builder(default)]
    two_factor_grace_period: Option<u64>,
    /// The access level to the group that is required to create new projects.
    #[builder(default)]
    project_creation_level: Option<GroupProjectCreationAccessLevel>,
    /// Default to Auto DevOps for new projects in the group.
    #[builder(default)]
    auto_devops_enabled: Option<bool>,
    /// The access level to the group that is required to create subgroups.
    #[builder(default)]
    subgroup_creation_level: Option<SubgroupCreationAccessLevel>,
    /// Disable email notifications from the group.
    #[builder(default)]
    emails_disabled: Option<bool>,
    // avatar   mixed   no  Image file for avatar of the group
    // avatar: ???,
    /// Disable group-wide mentions.
    #[builder(default)]
    mentions_disabled: Option<bool>,
    /// Whether `git-lfs` is enabled by default for projects within the group.
    #[builder(default)]
    lfs_enabled: Option<bool>,
    /// Whether access to the group may be requested.
    #[builder(default)]
    request_access_enabled: Option<bool>,
    /// The parent group ID (for subgroups).
    #[builder(default)]
    parent_id: Option<u64>,
    /// The default branch protection for projects within the group.
    #[builder(default)]
    default_branch_protection: Option<BranchProtection>,
    /// Pipeline quota (in minutes) for the group on shared runners.
    #[builder(default)]
    shared_runners_minutes_limit: Option<u64>,
    /// Pipeline quota excess (in minutes) for the group on shared runners.
    #[builder(default)]
    extra_shared_runners_minutes_limit: Option<u64>,
}

impl<'a> CreateGroup<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateGroupBuilder<'a> {
        CreateGroupBuilder::default()
    }
}

impl<'a> Endpoint for CreateGroup<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        "groups".into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        pairs.append_pair("name", &self.name);
        pairs.append_pair("path", &self.path);

        self.description
            .as_ref()
            .map(|value| pairs.append_pair("description", value));
        self.membership_lock
            .map(|value| pairs.append_pair("membership_lock", common::bool_str(value)));
        self.visibility
            .map(|value| pairs.append_pair("visibility", value.as_str()));
        self.share_with_group_lock
            .map(|value| pairs.append_pair("share_with_group_lock", common::bool_str(value)));
        self.require_two_factor_authentication.map(|value| {
            pairs.append_pair("require_two_factor_authentication", common::bool_str(value))
        });
        self.two_factor_grace_period
            .map(|value| pairs.append_pair("two_factor_grace_period", &format!("{}", value)));
        self.project_creation_level
            .map(|value| pairs.append_pair("project_creation_level", value.as_str()));
        self.auto_devops_enabled
            .map(|value| pairs.append_pair("auto_devops_enabled", common::bool_str(value)));
        self.subgroup_creation_level
            .map(|value| pairs.append_pair("subgroup_creation_level", value.as_str()));
        self.emails_disabled
            .map(|value| pairs.append_pair("emails_disabled", common::bool_str(value)));
        self.mentions_disabled
            .map(|value| pairs.append_pair("mentions_disabled", common::bool_str(value)));
        self.lfs_enabled
            .map(|value| pairs.append_pair("lfs_enabled", common::bool_str(value)));
        self.request_access_enabled
            .map(|value| pairs.append_pair("request_access_enabled", common::bool_str(value)));
        self.parent_id
            .map(|value| pairs.append_pair("parent_id", &format!("{}", value)));
        self.default_branch_protection
            .map(|value| pairs.append_pair("default_branch_protection", value.as_str()));
        self.shared_runners_minutes_limit
            .map(|value| pairs.append_pair("shared_runners_minutes_limit", &format!("{}", value)));
        self.extra_shared_runners_minutes_limit.map(|value| {
            pairs.append_pair("extra_shared_runners_minutes_limit", &format!("{}", value))
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::api::groups::{
        BranchProtection, CreateGroup, GroupProjectCreationAccessLevel, SubgroupCreationAccessLevel,
    };

    #[test]
    fn group_project_creation_access_level_as_str() {
        let items = &[
            (GroupProjectCreationAccessLevel::NoOne, "noone"),
            (GroupProjectCreationAccessLevel::Maintainer, "maintainer"),
            (GroupProjectCreationAccessLevel::Developer, "developer"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn subgroup_creation_access_level_as_str() {
        let items = &[
            (SubgroupCreationAccessLevel::Owner, "owner"),
            (SubgroupCreationAccessLevel::Maintainer, "maintainer"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn branch_protection_as_str() {
        let items = &[
            (BranchProtection::None, "0"),
            (BranchProtection::Partial, "1"),
            (BranchProtection::Full, "2"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn name_and_path_are_necessary() {
        let err = CreateGroup::builder().build().unwrap_err();
        assert_eq!(err, "`name` must be initialized");
    }

    #[test]
    fn name_is_necessary() {
        let err = CreateGroup::builder().path("path").build().unwrap_err();
        assert_eq!(err, "`name` must be initialized");
    }

    #[test]
    fn path_is_necessary() {
        let err = CreateGroup::builder().name("name").build().unwrap_err();
        assert_eq!(err, "`path` must be initialized");
    }

    #[test]
    fn name_and_path_are_sufficient() {
        CreateGroup::builder()
            .name("name")
            .path("path")
            .build()
            .unwrap();
    }
}
