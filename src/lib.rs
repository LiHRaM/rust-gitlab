// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// TODO: Document API entities.
// #![warn(missing_docs)]

//! A library for communicating with Gitlab instances.

#[macro_use]
extern crate log;

mod crates {
    // public
    pub extern crate chrono;
    pub extern crate serde;
    pub extern crate serde_json;

    // private
    pub extern crate itertools;
    pub extern crate log;
    pub extern crate percent_encoding;
    pub extern crate reqwest;
    // pub extern crate rustversion;
    pub extern crate thiserror;
}

#[macro_use]
mod macros;
mod gitlab;

pub mod hooks;
pub mod systemhooks;
pub mod types;
pub mod webhooks;

pub use gitlab::CommitStatusInfo;
pub use gitlab::Gitlab;
pub use gitlab::GitlabBuilder;
pub use gitlab::GitlabError;
pub use gitlab::MergeRequestStateFilter;
pub use gitlab::TokenError;
pub use types::*;

#[cfg(test)]
mod test;
