#![no_std]
#![no_main]

use core::arch::asm;
use kernel;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let info = kernel::boot();
    kernel::init(info);
}
