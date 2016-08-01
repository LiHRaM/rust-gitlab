// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate serde;
use self::serde::{Deserialize, Deserializer};
use self::serde::de::Error;

extern crate serde_json;
use self::serde_json::Value;

use super::systemhooks::SystemHook;
use super::webhooks::WebHook;

#[derive(Debug)]
pub enum GitlabHook {
    System(SystemHook),
    Web(WebHook),
}

impl Deserialize for GitlabHook {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        let val = try!(Value::deserialize(deserializer));

        // Look for `object_kind` first because some web hooks also have `event_name` which would
        // cause a false match here.
        if let Some(_) = val.pointer("/object_kind") {
            Ok(GitlabHook::Web(try!(WebHook::deserialize(deserializer))))
        } else if let Some(_) = val.pointer("/event_name") {
            Ok(GitlabHook::System(try!(SystemHook::deserialize(deserializer))))
        } else {
            Err(D::Error::missing_field("either object_kind or event_name"))
        }
    }
}
