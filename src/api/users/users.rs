// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use derive_builder::Builder;

use crate::api::endpoint_prelude::*;

/// Keys user results may be ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserOrderBy {
    /// Order by the user ID.
    Id,
    /// Order by the user display name.
    Name,
    /// Order by the username.
    Username,
    /// Order by the creation date of the user.
    CreatedAt,
    /// Order by the last updated date of the project.
    UpdatedAt,
}

impl Default for UserOrderBy {
    fn default() -> Self {
        UserOrderBy::Id
    }
}

impl UserOrderBy {
    /// The ordering as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            UserOrderBy::Id => "id",
            UserOrderBy::Name => "name",
            UserOrderBy::Username => "username",
            UserOrderBy::CreatedAt => "created_at",
            UserOrderBy::UpdatedAt => "updated_at",
        }
    }
}

impl fmt::Display for UserOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Representation of a user provided by an external service.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct ExternalProvider<'a> {
    /// The ID of the user on the service.
    pub id: u64,
    /// The name of the service.
    #[builder(setter(into))]
    pub name: Cow<'a, str>,
}

impl<'a> ExternalProvider<'a> {
    /// Create a builder for the external provider.
    pub fn builder() -> ExternalProviderBuilder<'a> {
        ExternalProviderBuilder::default()
    }
}

/// Query for users on an instance.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Users<'a> {
    /// Search for users using a query string.
    ///
    /// The search query will be escaped automatically.
    #[builder(setter(into), default)]
    search: Option<Cow<'a, str>>,

    /// Get a user with a given username.
    #[builder(setter(into), default)]
    username: Option<Cow<'a, str>>,
    /// Return only active users.
    #[builder(default)]
    active: Option<()>,
    /// Return only blocked users.
    #[builder(default)]
    blocked: Option<()>,

    /// Search for a user with a given external provider identity.
    #[builder(default)]
    external_provider: Option<ExternalProvider<'a>>,
    /// Whether to return external users or not.
    #[builder(default)]
    external: Option<bool>,

    /// Return users created before a given date.
    #[builder(default)]
    created_before: Option<DateTime<Utc>>,
    /// Return users created after a given date.
    #[builder(default)]
    created_after: Option<DateTime<Utc>>,

    /// Search for users with a given custom attribute set.
    #[builder(setter(name = "_custom_attributes"), default, private)]
    custom_attributes: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
    /// Search for users with custom attributes.
    #[builder(default)]
    with_custom_attributes: Option<bool>,

    /// Order results by a given key.
    #[builder(default)]
    order_by: Option<UserOrderBy>,
    /// The sort order for return results.
    #[builder(default)]
    sort: Option<SortOrder>,
    /// Return users with a two-factor enabled or not.
    #[builder(default)]
    two_factor: Option<EnableState>,
    /// If set to `true`, filter out users without any projects.
    #[builder(default)]
    without_projects: Option<bool>,
}

impl<'a> Users<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> UsersBuilder<'a> {
        UsersBuilder::default()
    }
}

impl<'a> UsersBuilder<'a> {
    /// Clear custom attribute search parameters.
    pub fn clear_custom_attributes(&mut self) -> &mut Self {
        self.custom_attributes = None;
        self
    }

    /// Add a custom attribute search parameter.
    pub fn custom_attribute<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.custom_attributes
            .get_or_insert_with(Default::default)
            .insert(key.into(), value.into());
        self
    }

    /// Add multiple custom attribute search parameters.
    pub fn custom_attributes<I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = (K, V)>,
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.custom_attributes
            .get_or_insert_with(Default::default)
            .extend(iter.map(|(k, v)| (k.into(), v.into())));
        self
    }
}

fn bool_as_str(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

impl<'a> Endpoint for Users<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        "users".into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.search
            .as_ref()
            .map(|value| pairs.append_pair("search", value));
        self.username
            .as_ref()
            .map(|value| pairs.append_pair("username", value));
        self.active.map(|_| pairs.append_pair("active", "true"));
        self.blocked.map(|_| pairs.append_pair("blocked", "true"));
        if let Some(value) = self.external_provider.as_ref() {
            pairs.append_pair("extern_uid", &format!("{}", value.id));
            pairs.append_pair("provider", &value.name);
        }
        self.external
            .map(|value| pairs.append_pair("external", bool_as_str(value)));
        self.created_before
            .map(|value| pairs.append_pair("created_before", &value.to_rfc3339()));
        self.created_after
            .map(|value| pairs.append_pair("created_before", &value.to_rfc3339()));

        pairs.extend_pairs(
            self.custom_attributes
                .iter()
                .map(|(key, value)| (format!("custom_attribute[{}]", key), value)),
        );
        self.with_custom_attributes
            .map(|value| pairs.append_pair("with_custom_attributes", bool_as_str(value)));

        self.order_by
            .map(|value| pairs.append_pair("order_by", value.as_str()));
        self.sort
            .map(|value| pairs.append_pair("sort", value.as_str()));
        self.two_factor
            .map(|value| pairs.append_pair("two_factor", value.as_str()));
        self.without_projects
            .map(|value| pairs.append_pair("without_projects", bool_as_str(value)));
    }
}

impl<'a> Pageable for Users<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::users::Users;

    #[test]
    fn defaults_are_sufficient() {
        Users::builder().build().unwrap();
    }
}
