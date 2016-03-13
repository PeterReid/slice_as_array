// Copyright 2016 Peter Reid. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(feature = "compiletest")]

extern crate compiletest_rs;

use std::path::PathBuf;
use compiletest_rs::{default_config, run_tests};

#[test]
fn compile_fail() {
    let mut config = default_config();

    config.mode = "compile-fail".parse().ok().expect("Invalid mode");
    config.src_base = PathBuf::from("tests/compile-fail");
    config.target_rustcflags = Some("-L target/debug".to_string());

    run_tests(&config);
}
