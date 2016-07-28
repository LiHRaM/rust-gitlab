// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate serde_codegen;

use std::env;
use std::fs;
use std::path::Path;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let paths = [
        "types",
        "webhooks",
    ];

    for path in &paths {
        let src = format!("src/{}.rs.in", path);
        let dst = format!("{}.rs", path);
        let src_path = Path::new(&src);
        let dst_path = Path::new(&out_dir).join(&dst);

        fs::create_dir_all(dst_path.parent().unwrap()).unwrap();

        serde_codegen::expand(&src_path, &dst_path).unwrap();
    }
}
