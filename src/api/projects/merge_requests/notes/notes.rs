// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{NameOrId, SortOrder};
use crate::api::endpoint_prelude::*;
use crate::api::helpers::NoteOrderBy;

/// Query for notes on an merge request within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct MergeRequestNotes<'a> {
    /// The project to query for the merge request.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the merge request.
    merge_request: u64,

    /// Order results by a given key.
    #[builder(default)]
    order_by: Option<NoteOrderBy>,
    /// The sort order for return results.
    #[builder(default)]
    sort: Option<SortOrder>,
}

impl<'a> MergeRequestNotes<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> MergeRequestNotesBuilder<'a> {
        MergeRequestNotesBuilder::default()
    }
}

impl<'a> Endpoint for MergeRequestNotes<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/notes",
            self.project, self.merge_request,
        )
        .into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("order_by", self.order_by)
            .push_opt("sort", self.sort);

        params
    }
}

impl<'a> Pageable for MergeRequestNotes<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::merge_requests::notes::MergeRequestNotes;

    #[test]
    fn project_and_merge_request_are_necessary() {
        let err = MergeRequestNotes::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = MergeRequestNotes::builder()
            .merge_request(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn merge_request_is_necessary() {
        let err = MergeRequestNotes::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`merge_request` must be initialized");
    }

    #[test]
    fn project_and_merge_request_are_sufficient() {
        MergeRequestNotes::builder()
            .project(1)
            .merge_request(1)
            .build()
            .unwrap();
    }
}
