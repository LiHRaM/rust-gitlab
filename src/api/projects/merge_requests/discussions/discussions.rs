// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for discussions on an merge request within a project.
#[derive(Debug, Builder)]
pub struct MergeRequestDiscussions<'a> {
    /// The project to query for the merge request.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the merge request.
    merge_request: u64,
}

impl<'a> MergeRequestDiscussions<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> MergeRequestDiscussionsBuilder<'a> {
        MergeRequestDiscussionsBuilder::default()
    }
}

impl<'a> Endpoint for MergeRequestDiscussions<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/discussions",
            self.project, self.merge_request,
        )
        .into()
    }
}

impl<'a> Pageable for MergeRequestDiscussions<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::merge_requests::discussions::MergeRequestDiscussions;

    #[test]
    fn project_and_merge_request_are_necessary() {
        let err = MergeRequestDiscussions::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = MergeRequestDiscussions::builder()
            .merge_request(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn merge_request_is_necessary() {
        let err = MergeRequestDiscussions::builder()
            .project(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`merge_request` must be initialized");
    }

    #[test]
    fn project_and_merge_request_are_sufficient() {
        MergeRequestDiscussions::builder()
            .project(1)
            .merge_request(1)
            .build()
            .unwrap();
    }
}
