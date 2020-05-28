// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{NameOrId, SortOrder};
use crate::api::endpoint_prelude::*;
use crate::api::ParamValue;

/// Keys note results may be ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteOrderBy {
    /// Sort by creation date.
    CreatedAt,
    /// Sort by last updated date.
    UpdatedAt,
}

impl Default for NoteOrderBy {
    fn default() -> Self {
        NoteOrderBy::CreatedAt
    }
}

impl NoteOrderBy {
    fn as_str(self) -> &'static str {
        match self {
            NoteOrderBy::CreatedAt => "created_at",
            NoteOrderBy::UpdatedAt => "updated_at",
        }
    }
}

impl ParamValue<'static> for NoteOrderBy {
    fn as_value(self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// Query for notes on an issue within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct IssueNotes<'a> {
    /// The project to query for the issue.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the issue.
    issue: u64,

    /// Order results by a given key.
    #[builder(default)]
    order_by: Option<NoteOrderBy>,
    /// The sort order for return results.
    #[builder(default)]
    sort: Option<SortOrder>,
}

impl<'a> IssueNotes<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> IssueNotesBuilder<'a> {
        IssueNotesBuilder::default()
    }
}

impl<'a> Endpoint for IssueNotes<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/notes", self.project, self.issue).into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("order_by", self.order_by)
            .push_opt("sort", self.sort);

        params
    }
}

impl<'a> Pageable for IssueNotes<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::issues::notes::{IssueNotes, NoteOrderBy};

    #[test]
    fn note_order_by_default() {
        assert_eq!(NoteOrderBy::default(), NoteOrderBy::CreatedAt);
    }

    #[test]
    fn note_order_by_as_str() {
        let items = &[
            (NoteOrderBy::CreatedAt, "created_at"),
            (NoteOrderBy::UpdatedAt, "updated_at"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn project_and_issue_are_necessary() {
        let err = IssueNotes::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = IssueNotes::builder().issue(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn issue_is_necessary() {
        let err = IssueNotes::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`issue` must be initialized");
    }

    #[test]
    fn project_and_issue_are_sufficient() {
        IssueNotes::builder().project(1).issue(1).build().unwrap();
    }
}
