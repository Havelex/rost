//! Compiler-required C memory intrinsics.
//!
//! Rust's `compiler_builtins` crate usually provides these, but a bare-metal
//! `no_std` kernel without the standard library needs them defined explicitly
//! so that the linker can resolve calls emitted by LLVM for struct copies,
//! zeroing, and comparisons.

#![allow(dead_code)]

/// Copy `n` bytes from `src` to `dest`.  Regions must not overlap.
///
/// # Safety
/// - `dest` and `src` must be valid for `n` bytes.
/// - The byte ranges `dest..dest+n` and `src..src+n` must not overlap.
#[unsafe(no_mangle)]
unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe {
            *dest.add(i) = *src.add(i);
        }
        i += 1;
    }
    dest
}

/// Fill `n` bytes at `s` with the byte value `c`.
///
/// # Safety
/// - `s` must be valid for writes of `n` bytes.
#[unsafe(no_mangle)]
unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe {
            *s.add(i) = c as u8;
        }
        i += 1;
    }
    s
}

/// Copy `n` bytes from `src` to `dest`, handling overlapping regions correctly.
///
/// # Safety
/// - `dest` and `src` must be valid for `n` bytes.
#[unsafe(no_mangle)]
unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest < src as *mut u8 {
        unsafe { memcpy(dest, src, n) }
    } else {
        let mut i = n;
        while i != 0 {
            i -= 1;
            unsafe {
                *dest.add(i) = *src.add(i);
            }
        }
        dest
    }
}

/// Compare `n` bytes of `s1` and `s2`.
///
/// # Returns
/// - `0` if the regions are equal.
/// - A positive value if the first differing byte of `s1` is greater.
/// - A negative value if the first differing byte of `s1` is less.
///
/// # Safety
/// - Both `s1` and `s2` must be valid for reads of `n` bytes.
#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        unsafe {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b {
                return a as i32 - b as i32;
            }
        }
    }
    0
}
