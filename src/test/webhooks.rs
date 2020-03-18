// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{TimeZone, Utc};
use serde_json::from_str;

use crate::webhooks::*;

#[test]
fn test_hookdate_deserialize() {
    let hook: HookDate = from_str("\"2019-01-20 15:00:12 UTC\"").unwrap();
    assert_eq!(
        *hook.as_ref(),
        Utc.ymd(2019, 1, 20).and_hms_milli(15, 00, 12, 0),
    );
    let hook: HookDate = from_str("\"2019-03-01T19:39:17Z\"").unwrap();
    assert_eq!(
        *hook.as_ref(),
        Utc.ymd(2019, 3, 1).and_hms_milli(19, 39, 17, 0),
    );
    let hook: HookDate = from_str("\"2019-03-01T17:50:02.036-05:00\"").unwrap();
    assert_eq!(
        *hook.as_ref(),
        Utc.ymd(2019, 3, 1).and_hms_milli(22, 50, 2, 36),
    );
}
