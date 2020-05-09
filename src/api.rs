// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![warn(missing_docs)]

//! API endpoint structures
//!
//! The types in this module are meant to aid in constructing the appropriate calls using type-safe
//! Rust idioms.
//!
//! All endpoints use the builder pattern and have their members as private so that there are no
//! API implications of adding new members for additional query parameters in future GitLab
//! releases.

mod client;
mod endpoint;
mod ignore;
mod paged;
mod query;

pub mod endpoint_prelude;

pub mod projects;
pub mod users;

pub use self::client::Client;

pub use self::endpoint::Endpoint;
pub use self::endpoint::Pairs;

pub use self::ignore::ignore;
pub use self::ignore::Ignore;

pub use self::paged::paged;
pub use self::paged::LinkHeaderParseError;
pub use self::paged::Pageable;
pub use self::paged::Paged;
pub use self::paged::Pagination;

pub use self::query::Query;
