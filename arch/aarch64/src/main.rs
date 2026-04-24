#![no_std]
#![no_main]

/// Architecture entry point invoked by the bootloader on `aarch64`.
///
/// This target is currently a stub and loops forever.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {}
}
