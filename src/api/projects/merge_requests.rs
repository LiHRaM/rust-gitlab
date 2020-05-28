// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project merge request API endpoints.
//!
//! These endpoints are used for querying projects merge requests.

pub mod awards;
mod create;
mod issues_closed_by;
mod merge_request;
mod merge_requests;
pub mod notes;
mod resource_label_events;

pub use self::create::CreateMergeRequest;
pub use self::create::CreateMergeRequestBuilder;

pub use self::issues_closed_by::IssuesClosedBy;
pub use self::issues_closed_by::IssuesClosedByBuilder;

pub use self::merge_request::MergeRequest;
pub use self::merge_request::MergeRequestBuilder;

pub use self::merge_requests::MergeRequestOrderBy;
pub use self::merge_requests::MergeRequestScope;
pub use self::merge_requests::MergeRequestState;
pub use self::merge_requests::MergeRequestView;
pub use self::merge_requests::MergeRequests;
pub use self::merge_requests::MergeRequestsBuilder;

pub use self::resource_label_events::MergeRequestResourceLabelEvents;
pub use self::resource_label_events::MergeRequestResourceLabelEventsBuilder;
