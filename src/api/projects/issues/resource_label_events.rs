// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for resource label events for an issue.
#[derive(Debug, Builder)]
pub struct IssueResourceLabelEvents<'a> {
    /// The project to query for the issue.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the issue.
    issue: u64,
}

impl<'a> IssueResourceLabelEvents<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> IssueResourceLabelEventsBuilder<'a> {
        IssueResourceLabelEventsBuilder::default()
    }
}

impl<'a> Endpoint for IssueResourceLabelEvents<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/issues/{}/resource_label_events",
            self.project, self.issue,
        )
        .into()
    }
}

impl<'a> Pageable for IssueResourceLabelEvents<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::issues::IssueResourceLabelEvents;

    #[test]
    fn project_and_issue_are_needed() {
        let err = IssueResourceLabelEvents::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = IssueResourceLabelEvents::builder()
            .issue(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn issue_is_needed() {
        let err = IssueResourceLabelEvents::builder()
            .project(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`issue` must be initialized");
    }

    #[test]
    fn project_and_issue_are_sufficient() {
        IssueResourceLabelEvents::builder()
            .project(1)
            .issue(1)
            .build()
            .unwrap();
    }
}
