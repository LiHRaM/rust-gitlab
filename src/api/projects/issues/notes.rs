// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project issue note API endpoints.
//!
//! These endpoints are used for querying project issue notes.

mod create;

pub use self::create::CreateIssueNote;
pub use self::create::CreateIssueNoteBuilder;
