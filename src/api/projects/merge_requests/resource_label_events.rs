// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for resource label events for a merge request.
#[derive(Debug, Builder)]
pub struct MergeRequestResourceLabelEvents<'a> {
    /// The project to query for the merge request.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the merge request.
    merge_request: u64,
}

impl<'a> MergeRequestResourceLabelEvents<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> MergeRequestResourceLabelEventsBuilder<'a> {
        MergeRequestResourceLabelEventsBuilder::default()
    }
}

impl<'a> Endpoint for MergeRequestResourceLabelEvents<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/resource_label_events",
            self.project, self.merge_request,
        )
        .into()
    }
}

impl<'a> Pageable for MergeRequestResourceLabelEvents<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::merge_requests::MergeRequestResourceLabelEvents;

    #[test]
    fn project_and_merge_request_are_needed() {
        let err = MergeRequestResourceLabelEvents::builder()
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = MergeRequestResourceLabelEvents::builder()
            .merge_request(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn merge_request_is_needed() {
        let err = MergeRequestResourceLabelEvents::builder()
            .project(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`merge_request` must be initialized");
    }

    #[test]
    fn project_and_merge_request_are_sufficient() {
        MergeRequestResourceLabelEvents::builder()
            .project(1)
            .merge_request(1)
            .build()
            .unwrap();
    }
}
