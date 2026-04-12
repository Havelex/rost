#![no_std]

use crate::{arch::Architecture, boot::BootInfo};

#[macro_use]
pub(crate) mod console;
pub(crate) mod arch;
pub(crate) mod boot;
pub(crate) mod memory;
pub(crate) mod panic;

pub use boot::init as boot;

pub fn init(info: BootInfo) -> ! {
    let fb = info.framebuffer.expect("  [!] Missing frame buffer!");
    console::writer::init(fb.into());
    println!("[.] Initializing Kernel...");
    // println!("  [*] Console initialized.");
    // arch::CurrentArch::init_early();
    // println!("  [.] Initializing physical memory...");
    // let mem_map = info.memory_map.expect("  [!] Missing memory map!");
    // memory::init(mem_map.into());
    // println!("  [*] Physical memory initiualized.");
    // println!("  [.] Initializing virtual memory...");
    // arch::CurrentArch::init_memory();
    // println!("  [*] Virtual memory initialized.");
    loop {}
}
