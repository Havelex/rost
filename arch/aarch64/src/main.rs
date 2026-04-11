#![no_std]
#![no_main]

use boot;
use kernel;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    kernel::init(boot::init());
}
