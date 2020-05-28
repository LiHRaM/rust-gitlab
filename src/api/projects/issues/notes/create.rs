// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Create a new note on an issue on a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateIssueNote<'a> {
    /// The project the issue belongs to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The issue to add the note to.
    issue: u64,
    /// The content of the note.
    #[builder(setter(into))]
    body: Cow<'a, str>,

    /// The creation date of the note.
    ///
    /// Requires administrator or owner permissions.
    #[builder(default)]
    created_at: Option<DateTime<Utc>>,
}

impl<'a> CreateIssueNote<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateIssueNoteBuilder<'a> {
        CreateIssueNoteBuilder::default()
    }
}

impl<'a> Endpoint for CreateIssueNote<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/notes", self.project, self.issue).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("body", self.body.as_ref())
            .push_opt("created_at", self.created_at);

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::issues::notes::CreateIssueNote;

    #[test]
    fn project_issue_and_body_are_necessary() {
        let err = CreateIssueNote::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateIssueNote::builder()
            .issue(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn issue_is_necessary() {
        let err = CreateIssueNote::builder()
            .project(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`issue` must be initialized");
    }

    #[test]
    fn body_is_necessary() {
        let err = CreateIssueNote::builder()
            .project(1)
            .issue(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`body` must be initialized");
    }

    #[test]
    fn project_issue_and_body_are_sufficient() {
        CreateIssueNote::builder()
            .project(1)
            .issue(1)
            .body("body")
            .build()
            .unwrap();
    }
}
