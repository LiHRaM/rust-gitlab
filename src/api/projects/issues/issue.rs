// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for a issue within a project.
#[derive(Debug, Builder)]
pub struct Issue<'a> {
    /// The project to query for the issue.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the issue.
    issue: u64,
}

impl<'a> Issue<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> IssueBuilder<'a> {
        IssueBuilder::default()
    }
}

impl<'a> Endpoint for Issue<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}", self.project, self.issue).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::issues::Issue;

    #[test]
    fn project_and_issue_are_needed() {
        let err = Issue::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = Issue::builder().issue(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn issue_is_needed() {
        let err = Issue::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`issue` must be initialized");
    }

    #[test]
    fn project_and_issue_are_sufficient() {
        Issue::builder().project(1).issue(1).build().unwrap();
    }
}
