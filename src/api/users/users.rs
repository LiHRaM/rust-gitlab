// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use derive_builder::Builder;

use crate::api::common::{EnableState, SortOrder};
use crate::api::endpoint_prelude::*;
use crate::api::ParamValue;

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

impl ParamValue<'static> for UserOrderBy {
    fn as_value(self) -> Cow<'static, str> {
        self.as_str().into()
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
    #[builder(setter(into), default)]
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
    /// Add a custom attribute search parameter.
    pub fn custom_attribute<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.custom_attributes
            .get_or_insert_with(BTreeMap::new)
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
            .get_or_insert_with(BTreeMap::new)
            .extend(iter.map(|(k, v)| (k.into(), v.into())));
        self
    }
}

impl<'a> Endpoint for Users<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        "users".into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("search", self.search.as_ref())
            .push_opt("username", self.username.as_ref())
            .push_opt("active", self.active.map(|()| true))
            .push_opt("blocked", self.blocked.map(|()| true))
            .push_opt("external", self.external)
            .push_opt("created_before", self.created_before)
            .push_opt("created_after", self.created_after)
            .extend(
                self.custom_attributes
                    .iter()
                    .map(|(key, value)| (format!("custom_attributes[{}]", key), value)),
            )
            .push_opt("with_custom_attributes", self.with_custom_attributes)
            .push_opt("order_by", self.order_by)
            .push_opt("sort", self.sort)
            .push_opt("two_factor", self.two_factor)
            .push_opt("without_projects", self.without_projects);

        if let Some(value) = self.external_provider.as_ref() {
            params
                .push("extern_uid", value.id)
                .push("provider", &value.name);
        }

        params
    }
}

impl<'a> Pageable for Users<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::users::{ExternalProvider, UserOrderBy, Users};

    #[test]
    fn order_by_default() {
        assert_eq!(UserOrderBy::default(), UserOrderBy::Id);
    }

    #[test]
    fn order_by_as_str() {
        let items = &[
            (UserOrderBy::Id, "id"),
            (UserOrderBy::Name, "name"),
            (UserOrderBy::Username, "username"),
            (UserOrderBy::CreatedAt, "created_at"),
            (UserOrderBy::UpdatedAt, "updated_at"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn external_provider_id_and_name_are_necessary() {
        let err = ExternalProvider::builder().build().unwrap_err();
        assert_eq!(err, "`id` must be initialized");
    }

    #[test]
    fn external_provider_id_is_necessary() {
        let err = ExternalProvider::builder()
            .name("name")
            .build()
            .unwrap_err();
        assert_eq!(err, "`id` must be initialized");
    }

    #[test]
    fn external_provider_name_is_necessary() {
        let err = ExternalProvider::builder().id(1).build().unwrap_err();
        assert_eq!(err, "`name` must be initialized");
    }

    #[test]
    fn external_provider_id_and_name_are_sufficient() {
        ExternalProvider::builder()
            .id(1)
            .name("name")
            .build()
            .unwrap();
    }

    #[test]
    fn defaults_are_sufficient() {
        Users::builder().build().unwrap();
    }
}
