// These macros define an inner function to set the lifetime of the created array

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
