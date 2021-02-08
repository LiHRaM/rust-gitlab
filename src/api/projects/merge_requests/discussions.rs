// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project merge request discussion API endpoints.
//!
//! These endpoints are used for querying project merge request discussions.

mod create;
mod discussions;

pub use self::create::CreateMergeRequestDiscussion;
pub use self::create::CreateMergeRequestDiscussionBuilder;
pub use self::create::ImagePosition;
pub use self::create::ImagePositionBuilder;
pub use self::create::LineCode;
pub use self::create::LineRange;
pub use self::create::LineRangeBuilder;
pub use self::create::LineType;
pub use self::create::Position;
pub use self::create::PositionBuilder;
pub use self::create::TextPosition;
pub use self::create::TextPositionBuilder;

pub use self::discussions::MergeRequestDiscussions;
pub use self::discussions::MergeRequestDiscussionsBuilder;
