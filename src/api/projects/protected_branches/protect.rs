// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::cmp;
use std::collections::BTreeSet;

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;
use crate::api::ParamValue;

/// Access levels for protected branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtectedAccessLevel {
    /// Developers and maintainers may perform the action.
    Developer,
    /// Maintainers may perform the action.
    Maintainer,
    /// Only administrators may perform the action.
    Admin,
    /// The action is not allowed at all.
    NoAccess,
}

impl Default for ProtectedAccessLevel {
    fn default() -> Self {
        ProtectedAccessLevel::Maintainer
    }
}

impl ProtectedAccessLevel {
    fn as_str(self) -> &'static str {
        match self {
            ProtectedAccessLevel::Developer => "30",
            ProtectedAccessLevel::Maintainer => "40",
            ProtectedAccessLevel::Admin => "60",
            ProtectedAccessLevel::NoAccess => "0",
        }
    }
}

impl ParamValue<'static> for ProtectedAccessLevel {
    fn as_value(self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// Granular protected access controls for branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectedAccess {
    /// Give a specific user access.
    User(u64),
    /// Give a group access.
    Group(u64),
    /// Give access to anyone with at least an access level.
    Level(ProtectedAccessLevel),
}

impl ProtectedAccess {
    fn add_query(self, name: &str, params: &mut FormParams) {
        match self {
            ProtectedAccess::User(user) => {
                params.push(format!("{}[][user_id]", name), user);
            },
            ProtectedAccess::Group(group) => {
                params.push(format!("{}[][group_id]", name), group);
            },
            ProtectedAccess::Level(level) => {
                params.push(format!("{}[][access_level]", name), level);
            },
        }
    }
}

impl PartialOrd for ProtectedAccess {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for ProtectedAccess {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        match (self, rhs) {
            (Self::User(l), Self::User(r)) => l.cmp(r),
            (Self::User(_), _) => cmp::Ordering::Less,
            (Self::Group(l), Self::Group(r)) => l.cmp(r),
            (Self::Group(_), Self::User(_)) => cmp::Ordering::Greater,
            (Self::Group(_), _) => cmp::Ordering::Less,
            (Self::Level(l), Self::Level(r)) => l.cmp(r),
            (Self::Level(_), _) => cmp::Ordering::Greater,
        }
    }
}

impl From<ProtectedAccessLevel> for ProtectedAccess {
    fn from(access: ProtectedAccessLevel) -> Self {
        ProtectedAccess::Level(access)
    }
}

/// Protect a branch or set of branches on a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct ProtectBranch<'a> {
    /// The project to protect a branch within.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The name or glob of the branch to protect.
    #[builder(setter(into))]
    name: Cow<'a, str>,
    /// The minimum access level required to push to the branch.
    #[builder(default)]
    push_access_level: Option<ProtectedAccessLevel>,
    /// The minimum access level required to merge into the branch.
    #[builder(default)]
    merge_access_level: Option<ProtectedAccessLevel>,
    /// The minimum access level required to unprotect the branch.
    #[builder(default)]
    unprotect_access_level: Option<ProtectedAccessLevel>,
    /// A discrete set of accesses allowed to push to the branch.
    #[builder(setter(name = "_allowed_to_push"), default, private)]
    allowed_to_push: BTreeSet<ProtectedAccess>,
    /// A discrete set of accesses allowed to merge into the branch.
    #[builder(setter(name = "_allowed_to_merge"), default, private)]
    allowed_to_merge: BTreeSet<ProtectedAccess>,
    /// A discrete set of accesses allowed to unprotect the branch.
    #[builder(setter(name = "_allowed_to_unprotect"), default, private)]
    allowed_to_unprotect: BTreeSet<ProtectedAccess>,
    /// Whether code owner approval is required to merge.
    #[builder(default)]
    code_owner_approval_required: Option<bool>,
}

impl<'a> ProtectBranch<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ProtectBranchBuilder<'a> {
        ProtectBranchBuilder::default()
    }
}

impl<'a> ProtectBranchBuilder<'a> {
    /// Add access to push to the branch.
    pub fn allowed_to_push(&mut self, access: ProtectedAccess) -> &mut Self {
        self.allowed_to_push
            .get_or_insert_with(BTreeSet::new)
            .insert(access);
        self
    }

    /// Add access to merge into the branch.
    pub fn allowed_to_merge(&mut self, access: ProtectedAccess) -> &mut Self {
        self.allowed_to_merge
            .get_or_insert_with(BTreeSet::new)
            .insert(access);
        self
    }

    /// Add access to unprotect the branch.
    pub fn allowed_to_unprotect(&mut self, access: ProtectedAccess) -> &mut Self {
        self.allowed_to_unprotect
            .get_or_insert_with(BTreeSet::new)
            .insert(access);
        self
    }
}

impl<'a> Endpoint for ProtectBranch<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/protected_branches", self.project).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("name", &self.name)
            .push_opt("push_access_level", self.push_access_level)
            .push_opt("merge_access_level", self.merge_access_level)
            .push_opt("unprotect_access_level", self.unprotect_access_level)
            .push_opt(
                "code_owner_approval_required",
                self.code_owner_approval_required,
            );

        self.allowed_to_push
            .iter()
            .for_each(|value| value.add_query("allowed_to_push", &mut params));
        self.allowed_to_merge
            .iter()
            .for_each(|value| value.add_query("allowed_to_merge", &mut params));
        self.allowed_to_unprotect
            .iter()
            .for_each(|value| value.add_query("allowed_to_unprotect", &mut params));

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;

    use crate::api::projects::protected_branches::{
        ProtectBranch, ProtectedAccess, ProtectedAccessLevel,
    };

    #[test]
    fn protected_access_level_default() {
        assert_eq!(
            ProtectedAccessLevel::default(),
            ProtectedAccessLevel::Maintainer,
        );
    }

    #[test]
    fn protected_access_level_ord() {
        let items = &[
            ProtectedAccessLevel::Developer,
            ProtectedAccessLevel::Maintainer,
            ProtectedAccessLevel::Admin,
            ProtectedAccessLevel::NoAccess,
        ];

        for i in items {
            assert_eq!(*i, *i);
            assert_eq!(i.cmp(i), cmp::Ordering::Equal);

            let mut expect = cmp::Ordering::Greater;
            for j in items {
                let is_same = i == j;
                if is_same {
                    expect = cmp::Ordering::Equal;
                }
                assert_eq!(i.cmp(j), expect);
                if is_same {
                    expect = cmp::Ordering::Less;
                }
            }

            let mut expect = cmp::Ordering::Less;
            for j in items.iter().rev() {
                let is_same = i == j;
                if is_same {
                    expect = cmp::Ordering::Equal;
                }
                assert_eq!(i.cmp(j), expect);
                if is_same {
                    expect = cmp::Ordering::Greater;
                }
            }
        }
    }

    #[test]
    fn protected_access_level_as_str() {
        let items = &[
            (ProtectedAccessLevel::Developer, "30"),
            (ProtectedAccessLevel::Maintainer, "40"),
            (ProtectedAccessLevel::Admin, "60"),
            (ProtectedAccessLevel::NoAccess, "0"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn protected_access_ord() {
        let items = &[
            ProtectedAccess::User(1),
            ProtectedAccess::User(2),
            ProtectedAccess::Group(1),
            ProtectedAccess::Group(2),
            ProtectedAccessLevel::Developer.into(),
            ProtectedAccessLevel::Maintainer.into(),
            ProtectedAccessLevel::Admin.into(),
            ProtectedAccessLevel::NoAccess.into(),
        ];

        for i in items {
            assert_eq!(*i, *i);
            assert_eq!(i.cmp(i), cmp::Ordering::Equal);
            assert_eq!(i.partial_cmp(i).unwrap(), cmp::Ordering::Equal);

            let mut expect = cmp::Ordering::Greater;
            for j in items {
                let is_same = i == j;
                if is_same {
                    expect = cmp::Ordering::Equal;
                }
                assert_eq!(i.cmp(j), expect);
                assert_eq!(i.partial_cmp(j).unwrap(), expect);
                if is_same {
                    expect = cmp::Ordering::Less;
                }
            }

            let mut expect = cmp::Ordering::Less;
            for j in items.iter().rev() {
                let is_same = i == j;
                if is_same {
                    expect = cmp::Ordering::Equal;
                }
                assert_eq!(i.cmp(j), expect);
                assert_eq!(i.partial_cmp(j).unwrap(), expect);
                if is_same {
                    expect = cmp::Ordering::Greater;
                }
            }
        }
    }

    #[test]
    fn project_and_name_are_needed() {
        let err = ProtectBranch::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_required() {
        let err = ProtectBranch::builder().name("master").build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn branch_is_required() {
        let err = ProtectBranch::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`name` must be initialized");
    }

    #[test]
    fn project_and_branch_are_sufficient() {
        ProtectBranch::builder()
            .project(1)
            .name("master")
            .build()
            .unwrap();
    }
}
