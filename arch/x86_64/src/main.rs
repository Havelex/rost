#![no_std]
#![no_main]

use kernel;

/// Architecture entry point invoked by the bootloader.
///
/// Collects boot information via [`kernel::boot`] and transfers control to
/// [`kernel::init`].
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    kernel::init(kernel::boot());
}
