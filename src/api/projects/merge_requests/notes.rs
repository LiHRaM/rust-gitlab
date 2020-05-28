// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project merge request note API endpoints.
//!
//! These endpoints are used for querying project merge request notes.

mod create;
mod notes;

pub use self::create::CreateMergeRequestNote;
pub use self::create::CreateMergeRequestNoteBuilder;

pub use self::notes::MergeRequestNotes;
pub use self::notes::MergeRequestNotesBuilder;
pub use crate::api::projects::issues::notes::NoteOrderBy;
