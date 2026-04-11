#![allow(non_snake_case)]

use core::ptr;

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        ptr::copy_nonoverlapping(src, dest, n);
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(dest: *mut u8, value: i32, n: usize) -> *mut u8 {
    unsafe {
        ptr::write_bytes(dest, value as u8, n);
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let av = *a.add(i);
            let bv = *b.add(i);

            if av != bv {
                return av as i32 - bv as i32;
            }
        }
    }
    0
}
