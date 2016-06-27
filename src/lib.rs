#![warn(missing_docs)]

//! A library for communicating with Gitlab instances.

#[macro_use]
extern crate hyper;

#[macro_use]
extern crate log;

mod error;
mod gitlab;
mod types;

pub use error::Error;
pub use gitlab::Gitlab;
pub use types::*;
