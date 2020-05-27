// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use derive_builder::Builder;

use crate::api::common::{AccessLevel, SortOrder, VisibilityLevel};
use crate::api::endpoint_prelude::*;
use crate::api::ParamValue;

/// Keys project results may be ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectOrderBy {
    /// Order by the project ID.
    Id,
    /// Order by the name of the project.
    Name,
    /// Order by the full path of the project.
    Path,
    /// Order by the creation date of the project.
    CreatedAt,
    /// Order by the last updated date of the project.
    UpdatedAt,
    /// Order by the last activity date of the project.
    LastActivityAt,
}

impl Default for ProjectOrderBy {
    fn default() -> Self {
        ProjectOrderBy::CreatedAt
    }
}

impl ProjectOrderBy {
    fn use_keyset_pagination(self) -> bool {
        self == ProjectOrderBy::Id
    }

    /// The ordering as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            ProjectOrderBy::Id => "id",
            ProjectOrderBy::Name => "name",
            ProjectOrderBy::Path => "path",
            ProjectOrderBy::CreatedAt => "created_at",
            ProjectOrderBy::UpdatedAt => "updated_at",
            ProjectOrderBy::LastActivityAt => "last_activity_at",
        }
    }
}

impl ParamValue<'static> for ProjectOrderBy {
    fn as_value(self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// Query for projects on an instance.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Projects<'a> {
    /// Search for projects using a query string.
    ///
    /// The search query will be escaped automatically.
    #[builder(setter(into), default)]
    search: Option<Cow<'a, str>>,

    /// Filter projects by its archived state.
    #[builder(default)]
    archived: Option<bool>,
    /// Filter projects by its visibility.
    #[builder(default)]
    visibility: Option<VisibilityLevel>,
    /// Search  ancestor namespaces when matching filters.
    ///
    /// Defaults to `false`.
    #[builder(default)]
    search_namespaces: Option<bool>,
    /// Return only simple fields for search results.
    #[builder(default)]
    simple: Option<bool>,
    /// Filter projects by those owned by the API caller.
    #[builder(default)]
    owned: Option<bool>,
    /// Filter projects by those the API caller is a member of.
    #[builder(default)]
    membership: Option<bool>,
    /// Filter projects by those the API caller has starred.
    #[builder(default)]
    starred: Option<bool>,
    /// Include project statistics in the results.
    #[builder(default)]
    statistics: Option<bool>,

    /// Filter projects by whether issues are enabled.
    #[builder(default)]
    with_issues_enabled: Option<bool>,
    /// Filter projects by whether merge requests are enabled.
    #[builder(default)]
    with_merge_requests_enabled: Option<bool>,
    /// Filter projects by programming language.
    #[builder(setter(into), default)]
    with_programming_language: Option<Cow<'a, str>>,
    /// Filter projects by those with a failing wiki checksum.
    #[builder(default)]
    wiki_checksum_failed: Option<bool>,
    /// Filter projects by those with a failing repository checksum.
    #[builder(default)]
    repository_checksum_failed: Option<bool>,
    /// Filter projects by those where the API caller has a minimum access level.
    #[builder(default)]
    min_access_level: Option<AccessLevel>,

    /// Search for projects with a given custom attribute set.
    #[builder(setter(name = "_custom_attributes"), default, private)]
    custom_attributes: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
    /// Search for projects with custom attributes.
    #[builder(default)]
    with_custom_attributes: Option<bool>,

    /// Filter projects by those with at least this ID.
    #[builder(default)]
    id_after: Option<u64>,
    /// Filter projects by those with at most this ID.
    #[builder(default)]
    id_before: Option<u64>,
    /// Filter projects by those with activity after this date.
    #[builder(default)]
    last_activity_after: Option<DateTime<Utc>>,
    /// Filter projects by those without activity before this date.
    #[builder(default)]
    last_activity_before: Option<DateTime<Utc>>,

    /// Order results by a given key.
    #[builder(default)]
    order_by: Option<ProjectOrderBy>,
    /// The sort order for return results.
    #[builder(default)]
    sort: Option<SortOrder>,
}

impl<'a> Projects<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ProjectsBuilder<'a> {
        ProjectsBuilder::default()
    }
}

impl<'a> ProjectsBuilder<'a> {
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

impl<'a> Endpoint for Projects<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        "projects".into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("search", self.search.as_ref())
            .push_opt("archived", self.archived)
            .push_opt("visibility", self.visibility)
            .push_opt("search_namespaces", self.search_namespaces)
            .push_opt("simple", self.simple)
            .push_opt("owned", self.owned)
            .push_opt("membership", self.membership)
            .push_opt("starred", self.starred)
            .push_opt("statistics", self.statistics)
            .push_opt("with_issues_enabled", self.with_issues_enabled)
            .push_opt(
                "with_merge_requests_enabled",
                self.with_merge_requests_enabled,
            )
            .push_opt(
                "with_programming_language",
                self.with_programming_language.as_ref(),
            )
            .push_opt("wiki_checksum_failed", self.wiki_checksum_failed)
            .push_opt(
                "repository_checksum_failed",
                self.repository_checksum_failed,
            )
            .push_opt(
                "min_access_level",
                self.min_access_level.map(|level| level.as_u64()),
            )
            .push_opt("id_after", self.id_after)
            .push_opt("id_before", self.id_before)
            .push_opt("last_activity_after", self.last_activity_after)
            .push_opt("last_activity_before", self.last_activity_before)
            .extend(
                self.custom_attributes
                    .iter()
                    .map(|(key, value)| (format!("custom_attributes[{}]", key), value)),
            )
            .push_opt("with_custom_attributes", self.with_custom_attributes)
            .push_opt("order_by", self.order_by)
            .push_opt("sort", self.sort);

        params
    }
}

impl<'a> Pageable for Projects<'a> {
    fn use_keyset_pagination(&self) -> bool {
        self.order_by
            .map_or(false, |order_by| order_by.use_keyset_pagination())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::{ProjectOrderBy, Projects};

    #[test]
    fn order_by_default() {
        assert_eq!(ProjectOrderBy::default(), ProjectOrderBy::CreatedAt);
    }

    #[test]
    fn order_by_as_str() {
        let items = &[
            (ProjectOrderBy::Id, "id"),
            (ProjectOrderBy::Name, "name"),
            (ProjectOrderBy::Path, "path"),
            (ProjectOrderBy::CreatedAt, "created_at"),
            (ProjectOrderBy::UpdatedAt, "updated_at"),
            (ProjectOrderBy::LastActivityAt, "last_activity_at"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn defaults_are_sufficient() {
        Projects::builder().build().unwrap();
    }
}
