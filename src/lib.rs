#![warn(missing_docs)]

//! A library for communicating with Gitlab instances.

#[macro_use]
extern crate hyper;

mod gitlab;

pub use gitlab::Error;
pub use gitlab::Gitlab;
