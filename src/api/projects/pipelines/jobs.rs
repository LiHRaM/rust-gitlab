// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Pipeline job API endpoints.
//!
//! These endpoints are used for querying CI jobs.

mod jobs;

pub use self::jobs::Jobs;
pub use self::jobs::JobsBuilder;
