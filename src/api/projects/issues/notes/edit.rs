// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Edit an issue note on a project.
#[derive(Debug, Builder)]
pub struct EditIssueNote<'a> {
    /// The project to add the issue to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The internal IID of the issue.
    issue: u64,
    /// The ID of the note.
    note: u64,

    /// The content of the note.
    #[builder(setter(into))]
    body: Cow<'a, str>,
}

impl<'a> EditIssueNote<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> EditIssueNoteBuilder<'a> {
        EditIssueNoteBuilder::default()
    }
}

impl<'a> Endpoint for EditIssueNote<'a> {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/issues/{}/notes/{}",
            self.project, self.issue, self.note,
        )
        .into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params.push("body", self.body.as_ref());

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::issues::notes::EditIssueNote;

    #[test]
    fn project_issue_note_and_body_are_necessary() {
        let err = EditIssueNote::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = EditIssueNote::builder()
            .issue(1)
            .note(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn issue_is_necessary() {
        let err = EditIssueNote::builder()
            .project(1)
            .note(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`issue` must be initialized");
    }

    #[test]
    fn note_is_necessary() {
        let err = EditIssueNote::builder()
            .project(1)
            .issue(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`note` must be initialized");
    }

    #[test]
    fn body_is_necessary() {
        let err = EditIssueNote::builder()
            .project(1)
            .issue(1)
            .note(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`body` must be initialized");
    }

    #[test]
    fn project_issue_note_and_body_are_sufficient() {
        EditIssueNote::builder()
            .project(1)
            .issue(1)
            .note(1)
            .body("body")
            .build()
            .unwrap();
    }
}
