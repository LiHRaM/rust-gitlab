// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project job API endpoints.
//!
//! These endpoints are used for querying CI jobs.

mod cancel;
mod erase;
mod job;
mod jobs;
mod play;
mod retry;
mod trace;

pub use self::cancel::CancelJob;
pub use self::cancel::CancelJobBuilder;

pub use self::erase::EraseJob;
pub use self::erase::EraseJobBuilder;

pub use self::job::Job;
pub use self::job::JobBuilder;

pub use self::jobs::JobScope;
pub use self::jobs::Jobs;
pub use self::jobs::JobsBuilder;

pub use self::play::PlayJob;
pub use self::play::PlayJobBuilder;

pub use self::retry::RetryJob;
pub use self::retry::RetryJobBuilder;

pub use self::trace::JobTrace;
pub use self::trace::JobTraceBuilder;
