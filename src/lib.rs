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
//!
//! For most users, stating a dependency on this is simply:
//!
//! ```ignore
//! [dependencies]
//! slice_as_array "1.0.0"
//! ```
//! To support being called from a `#![no_std]` crate, this crate has a feature
//! named `with_std` that is on by default. A `#![no_std]` create should use:
//!
//! ```ignore
//! [dependencies]
//! slice_as_array = { version = "1.0.0", default-features = false }
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

// These macros define an inner function to set the lifetime of the created array
//
// To support compiling with and without `std`, calls to `transmute`
// are behind a macro to use the function from either `std` or `core`.

#[cfg(feature="use_std")]
#[macro_export]
#[doc(hidden)]
macro_rules! slice_as_array_transmute {
    ($slice:expr) => { ::std::mem::transmute($slice) }
}

#[cfg(not(feature="use_std"))]
#[macro_export]
macro_rules! slice_as_array_transmute {
    ($slice:expr) => { ::core::mem::transmute($slice) }
}

/// Convert a slice to an array.
/// `slice_to_array!(slice, [element_type; array_length]) -> Option<&[element_type; array_length]>`
#[macro_export]
macro_rules! slice_as_array {
    ($slice:expr, [$t:ident ; $len:expr] ) => {{
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
/// `slice_to_array_mut!(mutable_slice, [element_type; array_length]) -> Option<&mut [element_type; array_length]>`
#[macro_export]
macro_rules! slice_as_array_mut {
    ($slice:expr, [$t:ident ; $len:expr] ) => {{
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
}
