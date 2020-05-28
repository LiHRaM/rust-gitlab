// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Edit a merge request note on a project.
#[derive(Debug, Builder)]
pub struct EditMergeRequestNote<'a> {
    /// The project to add the merge request to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The internal IID of the merge request.
    merge_request: u64,
    /// The ID of the note.
    note: u64,

    /// The content of the note.
    #[builder(setter(into))]
    body: Cow<'a, str>,
}

impl<'a> EditMergeRequestNote<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> EditMergeRequestNoteBuilder<'a> {
        EditMergeRequestNoteBuilder::default()
    }
}

impl<'a> Endpoint for EditMergeRequestNote<'a> {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/notes/{}",
            self.project, self.merge_request, self.note,
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
    use crate::api::projects::merge_requests::notes::EditMergeRequestNote;

    #[test]
    fn project_merge_request_note_and_body_are_necessary() {
        let err = EditMergeRequestNote::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = EditMergeRequestNote::builder()
            .merge_request(1)
            .note(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn merge_request_is_necessary() {
        let err = EditMergeRequestNote::builder()
            .project(1)
            .note(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`merge_request` must be initialized");
    }

    #[test]
    fn note_is_necessary() {
        let err = EditMergeRequestNote::builder()
            .project(1)
            .merge_request(1)
            .body("body")
            .build()
            .unwrap_err();
        assert_eq!(err, "`note` must be initialized");
    }

    #[test]
    fn body_is_necessary() {
        let err = EditMergeRequestNote::builder()
            .project(1)
            .merge_request(1)
            .note(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`body` must be initialized");
    }

    #[test]
    fn project_merge_request_note_and_body_are_sufficient() {
        EditMergeRequestNote::builder()
            .project(1)
            .merge_request(1)
            .note(1)
            .body("body")
            .build()
            .unwrap();
    }
}
