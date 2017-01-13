// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! System hook structures
//!
//! These hooks are received from Gitlab when registered as a system hook in the administrator
//! settings. Only administrators may create such hooks.
//!
//! Gitlab does not have consistent structures for its hooks, so they often change from
//! version to version.

include!(concat!(env!("OUT_DIR"), "/systemhooks.rs"));
