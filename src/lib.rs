#![warn(missing_docs)]

//! A library for communicating with Gitlab instances.

#[macro_use]
extern crate hyper;

mod error;
mod gitlab;

pub use error::Error;
pub use gitlab::Gitlab;
