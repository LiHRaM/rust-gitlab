// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! API types common to many endpoints.
//!
//! Usually these are enumerations or other simple wrappers around structures present in
//! GitLab's REST API.

use std::borrow::Cow;
use std::fmt;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// Access levels for groups and projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    /// Admin access (full rights).
    Admin,
}

impl AccessLevel {
    /// The string representation of the access level.
    pub fn as_str(self) -> &'static str {
        match self {
            AccessLevel::Admin => "admin",
            AccessLevel::Owner => "owner",
            AccessLevel::Maintainer => "maintainer",
            AccessLevel::Developer => "developer",
            AccessLevel::Reporter => "reporter",
            AccessLevel::Guest => "guest",
            AccessLevel::Anonymous => "anonymous",
        }
    }

    /// The integer representation of the access level.
    pub fn as_u64(self) -> u64 {
        match self {
            AccessLevel::Admin => 60,
            AccessLevel::Owner => 50,
            AccessLevel::Maintainer => 40,
            AccessLevel::Developer => 30,
            AccessLevel::Reporter => 20,
            AccessLevel::Guest => 10,
            AccessLevel::Anonymous => 0,
        }
    }
}

/// Orderings for sorted results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Values should be sorted with "higher" values after "lower" values.
    Ascending,
    /// Values should be sorted with "lower" values after "higher" values.
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Descending
    }
}

impl SortOrder {
    /// The string representation of the sort order.
    pub fn as_str(self) -> &'static str {
        match self {
            SortOrder::Ascending => "asc",
            SortOrder::Descending => "desc",
        }
    }
}

/// States for features or flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnableState {
    /// The feature or flag is enabled.
    Enabled,
    /// The feature or flag is disabled.
    Disabled,
}

impl EnableState {
    /// The string representation of the enabled state.
    pub fn as_str(self) -> &'static str {
        match self {
            EnableState::Enabled => "enabled",
            EnableState::Disabled => "disabled",
        }
    }
}

/// A strucutre for storing a name or ID where either is allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameOrId<'a> {
    /// The name of the entity.
    ///
    /// Note that numeric names are ambiguous to GitLab. There is nothing done with this crate
    /// which attempts to resolve this ambiguity.
    Name(Cow<'a, str>),
    /// The ID of the entity.
    Id(u64),
}

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

impl<'a> fmt::Display for NameOrId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NameOrId::Name(name) => {
                let encoded = utf8_percent_encode(name, PATH_SEGMENT_ENCODE_SET);
                write!(f, "{}", encoded)
            },
            NameOrId::Id(id) => write!(f, "{}", id),
        }
    }
}

impl<'a> From<u64> for NameOrId<'a> {
    fn from(id: u64) -> Self {
        NameOrId::Id(id)
    }
}

impl<'a> From<&'a str> for NameOrId<'a> {
    fn from(name: &'a str) -> Self {
        NameOrId::Name(name.into())
    }
}

impl<'a> From<String> for NameOrId<'a> {
    fn from(name: String) -> Self {
        NameOrId::Name(name.into())
    }
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

impl VisibilityLevel {
    /// The string representation of the visibility level.
    pub fn as_str(&self) -> &str {
        match self {
            VisibilityLevel::Public => "public",
            VisibilityLevel::Internal => "internal",
            VisibilityLevel::Private => "private",
        }
    }
}
