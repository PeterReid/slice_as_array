// Copyright 2016 Peter Reid. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides macros to convert from slices, which have lengths
//! that are stored and checked at runtime, into arrays, which have lengths
//! known at compile time. This can make types more expressive (e.g.
//! `&[u8; 32]` instead of `&[u8]`) and helps the compiler omit bounds checks.
//!
//! `slice_as_array!(xs, [u32; 4])` returns `Some(&[u32; 4])` if `xs` was
//! a slice of length 4, or `None` otherwise.
//!
//! `slice_as_array_mut!(ys, [String; 7])` returns `Some(&mut [String; 7])`
//!  if `ys` was a slice of length 7, or `None` otherwise.
//!
//! `slice_to_array_clone!(zs, [String; 4]` returns `Some([String; 4])`
//! if `zs` was a slice of length 4, or `None otherwise. The passed-in slice
//! remains intact and its elements are cloned.
//!
//! For most users, stating a dependency on this is simply:
//!
//! ```ignore
//! [dependencies]
//! slice_as_array "1.1.0"
//! ```
//! To support being called from a `#![no_std]` crate, this crate has a feature
//! named `with_std` that is on by default. A `#![no_std]` crate should use:
//!
//! ```ignore
//! [dependencies]
//! slice_as_array = { version = "1.1.0", default-features = false }
//! ```
//!
//! Example usage:
//!
//! ```ignore
//! #[macro_use] extern crate slice_as_array;
//!
//! fn slice_as_hash(xs: &[u8]) -> &[u8; 32] {
//!     slice_as_array!(xs, [u8; 32]).expect("bad hash length")
//! }
//!
//! fn mutate_chunk(xs: &mut [u32; 10]) {
//!     // ...
//! }
//!
//! fn mutate_chunks(xs: &mut [u32; 100]) {
//!     for chunk in xs.chunks_mut(10).map(|c| slice_as_array_mut!(c, [u32; 10]).unwrap() ) {
//!         mutate_chunk(chunk)
//!     }
//! }
//! ```
//!

#[doc(hidden)]
pub mod reexport {
    #[inline] pub fn clone<T: Clone>(source: T) -> T { source.clone() }
    #[inline] pub unsafe fn ptr_write<T>(dst: *mut T, src: T) { ::std::ptr::write(dst, src) }
    #[inline] pub unsafe fn ptr_read<T>(src: *const T) -> T { ::std::ptr::read(src) }
    #[inline] pub fn forget<T>(t: T) { ::std::mem::forget(t) }
    #[inline] pub unsafe fn uninitialized<T>() -> T { ::std::mem::uninitialized() }
}

#[cfg(feature="use_std")]
#[macro_export]
#[doc(hidden)]
macro_rules! slice_as_array_transmute {
    ($slice:expr) => { ::std::mem::transmute($slice) }
}

#[cfg(not(feature="use_std"))]
#[macro_export]
macro_rules! slice_as_array_transmute {
    ($slice:expr) => { core::mem::transmute($slice) }
}


// In slice_as_array[_mut], the inner function is to set the lifetime of the created array.

/// Convert a slice to an array.
/// `slice_as_array!(slice, [element_type; array_length]) -> Option<&[element_type; array_length]>`
#[macro_export]
macro_rules! slice_as_array {
    ($slice:expr, [$t:ty ; $len:expr] ) => {{
        unsafe fn this_transmute(xs: &[$t]) -> &[$t; $len] {
            slice_as_array_transmute!(xs.as_ptr())
        }

        let s: &[$t] = $slice;
        if s.len() == $len {
            Some( unsafe { this_transmute(s) } )
        } else {
            None
        }
    }}
}

/// Convert a mutable slice to a mutable array.
/// `slice_as_array_mut!(mutable_slice, [element_type; array_length]) -> Option<&mut [element_type; array_length]>`
#[macro_export]
macro_rules! slice_as_array_mut {
    ($slice:expr, [$t:ty ; $len:expr] ) => {{
        unsafe fn this_transmute(xs: &mut [$t]) -> &mut [$t; $len] {
            slice_as_array_transmute!(xs.as_mut_ptr())
        }

        let s: &mut [$t] = $slice;
        if s.len() == $len {
            Some( unsafe { this_transmute(s) } )
        } else {
            None
        }
    }}
}

/// Convert a slice to an array by cloning each element.
/// `slice_to_array_clone!(slice, [element_type; array_length]) -> Option<[element_type; array_length]>`
#[macro_export]
macro_rules! slice_to_array_clone {
    ($slice:expr, [$t:ty ; $len:expr] ) => {{
        struct SafeArrayInitialization {
            array: Option<[$t; $len]>,
            count: usize,
        }
        impl SafeArrayInitialization {
            fn new() -> Self {
                SafeArrayInitialization { array: Some(unsafe { $crate::reexport::uninitialized() }), count: 0 }
            }
            fn init_from_slice(mut self, slice: &[$t]) -> Option<[$t; $len]> {
                {
                    let array_mut: &mut [$t] = self.array.as_mut().unwrap().as_mut();
                    if slice.len() != array_mut.len() {
                        return None;
                    }
                    debug_assert_eq!(self.count, 0);
                    for (val, ptr) in slice.iter().zip(array_mut.iter_mut()) {
                        let val = $crate::reexport::clone(*val);
                        unsafe { $crate::reexport::ptr_write(ptr, val) };
                        self.count += 1;
                    }
                }
                self.array.take()
            }
        }
        impl Drop for SafeArrayInitialization {
            fn drop(&mut self) {
                if let Some(mut array) = self.array.take() {
                    let count = self.count;
                    {
                        for ptr in array.as_mut()[..count].iter_mut() {
                            unsafe { $crate::reexport::ptr_read(ptr) };
                        }
                    }
                    $crate::reexport::forget(array);
                }
            }
        }

        SafeArrayInitialization::new().init_from_slice($slice)
    }}
}

#[cfg(test)]
mod test {
    #[test]
    fn correct_length() {
        let xs: [u32; 6] = [1, 2, 4, 8, 16, 32];
        let xs_prefix: &[u32; 3] = slice_as_array!(&xs[1..4], [u32; 3]).expect("Length mismatch");
        assert_eq!(xs_prefix, &[2, 4, 8]);
    }

    #[test]
    fn incorrect_length() {
        let xs: [u32; 6] = [1, 2, 4, 8, 16, 32];
        let xs_prefix: Option<&[u32; 8]> = slice_as_array!(&xs[1..4], [u32; 8]);
        assert_eq!(xs_prefix, None);
    }

    #[test]
    #[should_panic]
    fn overlong_length() {
        let xs: [u32; 6] = [1, 2, 4, 8, 16, 32];
        let xs_prefix: Option<&[u32; 8]> = slice_as_array!(&xs[0..8], [u32; 8]);
        assert_eq!(xs_prefix, None);
    }

    #[test]
    fn zero_length() {
        let xs: [u32; 6] = [1, 2, 4, 8, 16, 32];
        let xs_prefix: &[u32; 0] = slice_as_array!(&xs[1..1], [u32; 0]).unwrap();
        assert_eq!(xs_prefix, &[]);
    }

    #[test]
    fn array_of_arrays() {
        let xs: [[u8; 4]; 3] = [
            [10,11,12,13],
            [20,21,22,23],
            [30,31,32,33],
        ];

        let xs_suffix: &[[u8;4]; 2] = slice_as_array!(&xs[1..], [[u8; 4]; 2]).unwrap();
        assert_eq!(xs_suffix[0][0], 20);
        assert_eq!(xs_suffix[1][3], 33);
    }

    #[test]
    fn clone_correct() {
        let xs: [u32; 6] = [1, 2, 4, 8, 16, 32];
        let xs_middle: [u32; 3] = slice_to_array_clone!(&xs[1..4], [u32; 3]).expect("Length mismatch");
        assert_eq!(xs_middle, [2, 4, 8]);
    }

    #[test]
    fn clone_wrong_length() {
        let xs: [u32; 6] = [1, 2, 4, 8, 16, 32];
        let xs_middle: Option<[u32; 3]> = slice_to_array_clone!(&xs[1..5], [u32; 3]);
        assert_eq!(xs_middle, None);
    }
}
