// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project merge request API endpoints.
//!
//! These endpoints are used for querying projects merge requests.

pub mod awards;
mod issues_closed_by;
pub mod notes;
mod resource_label_events;

pub use self::issues_closed_by::IssuesClosedBy;
pub use self::issues_closed_by::IssuesClosedByBuilder;

pub use self::resource_label_events::MergeRequestResourceLabelEvents;
pub use self::resource_label_events::MergeRequestResourceLabelEventsBuilder;
