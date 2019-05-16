// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crates::serde_json::from_str;

use webhooks::*;

#[test]
fn test_hookdate_deseriazlie() {
    let _hook: HookDate = from_str("\"2019-01-20 15:00:12 UTC\"").unwrap();
    let _hook: HookDate = from_str("\"2019-03-01T19:39:17Z\"").unwrap();
    let _hook: HookDate = from_str("\"2019-03-01T17:50:02.036-05:00\"").unwrap();
}
