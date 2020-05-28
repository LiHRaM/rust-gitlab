// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Create a new award on a note on a merge request on a project.
#[derive(Debug, Builder)]
pub struct CreateMergeRequestNoteAward<'a> {
    /// The project the merge request belongs to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The merge request to add the note to.
    merge_request: u64,
    /// The ID of the note.
    note: u64,
    /// The award to give to the note (without colons).
    #[builder(setter(into))]
    name: Cow<'a, str>,
}

impl<'a> CreateMergeRequestNoteAward<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateMergeRequestNoteAwardBuilder<'a> {
        CreateMergeRequestNoteAwardBuilder::default()
    }
}

impl<'a> Endpoint for CreateMergeRequestNoteAward<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/notes/{}/award_emoji",
            self.project, self.merge_request, self.note,
        )
        .into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params.push("name", self.name.as_ref());

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::merge_requests::notes::awards::CreateMergeRequestNoteAward;

    #[test]
    fn project_merge_request_note_and_name_are_necessary() {
        let err = CreateMergeRequestNoteAward::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateMergeRequestNoteAward::builder()
            .merge_request(1)
            .note(1)
            .name("award")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn merge_request_is_necessary() {
        let err = CreateMergeRequestNoteAward::builder()
            .project(1)
            .note(1)
            .name("award")
            .build()
            .unwrap_err();
        assert_eq!(err, "`merge_request` must be initialized");
    }

    #[test]
    fn note_is_necessary() {
        let err = CreateMergeRequestNoteAward::builder()
            .project(1)
            .merge_request(1)
            .name("award")
            .build()
            .unwrap_err();
        assert_eq!(err, "`note` must be initialized");
    }

    #[test]
    fn name_is_necessary() {
        let err = CreateMergeRequestNoteAward::builder()
            .project(1)
            .merge_request(1)
            .note(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`name` must be initialized");
    }

    #[test]
    fn project_merge_request_note_and_name_are_sufficient() {
        CreateMergeRequestNoteAward::builder()
            .project(1)
            .merge_request(1)
            .note(1)
            .name("award")
            .build()
            .unwrap();
    }
}
