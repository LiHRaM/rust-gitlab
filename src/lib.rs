// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// TODO: Document API entities.
// #![warn(missing_docs)]

//! A library for communicating with Gitlab instances.

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate log;

#[macro_use]
mod macros;
mod gitlab;

pub mod hooks;
pub mod systemhooks;
pub mod types;
pub mod webhooks;

pub use crate::gitlab::CommitStatusInfo;
pub use crate::gitlab::Gitlab;
pub use crate::gitlab::GitlabBuilder;
pub use crate::gitlab::GitlabError;
pub use crate::gitlab::MergeRequestStateFilter;
pub use crate::gitlab::TokenError;
pub use crate::types::*;

#[cfg(test)]
mod test;
