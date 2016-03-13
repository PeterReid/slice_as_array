// Copyright 2016 Peter Reid. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use] extern crate slice_as_array;

fn main() {
    let mut xs = [0u32; 50];
    let xs_prefix = slice_as_array_mut!(&mut xs[0..20], [u32; 20]);

    xs[0] = 5; //~error: cannot assign to `xs[..]` because it is borrowed
}
