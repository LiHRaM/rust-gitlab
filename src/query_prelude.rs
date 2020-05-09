// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use std::borrow::Cow;

pub use reqwest::Method;
pub use serde::de::DeserializeOwned;

pub use crate::api::Client;
pub use crate::api::Pageable;

pub use crate::gitlab::GitlabError;

pub use crate::query::Pairs;
pub use crate::query::Query;
pub use crate::query::SingleQuery;

pub use crate::query_common::EnableState;
pub use crate::query_common::SortOrder;
