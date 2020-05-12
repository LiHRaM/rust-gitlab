// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project label API endpoints.
//!
//! These endpoints are used for querying project labels.

mod create;
mod label;
mod labels;

pub use self::create::CreateLabel;
pub use self::create::CreateLabelBuilder;

pub use self::label::Label;
pub use self::label::LabelBuilder;

pub use self::labels::Labels;
pub use self::labels::LabelsBuilder;
