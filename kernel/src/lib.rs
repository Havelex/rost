#![no_std]

use crate::{arch::Architecture, boot::BootInfo};

#[macro_use]
pub(crate) mod console;
pub(crate) mod arch;
pub(crate) mod boot;
pub(crate) mod cpu;
pub(crate) mod memory;
pub(crate) mod panic;
pub(crate) mod time;

pub use boot::init as boot;

pub fn init(info: BootInfo) -> ! {
    let fb_info = info.framebuffer.expect("[!] Missing frame buffer!");
    console::writer::init(fb_info.into());
    println!("[.] Initializing Kernel...");
    println!("  [*] Console initialized.");
    println!("  [.] Initializing early architecture...");
    arch::CurrentArch::init_early();
    println!("  [*] Early architecture initialized");
    println!("  [.] Initializing interrupts...");
    arch::CurrentArch::init_interrupts();
    println!("  [*] interrupts initialized");
    // println!("  [.] Initializing physical memory...");
    // let mem_map = info.memory_map.expect("  [!] Missing memory map!");
    // memory::init(mem_map.into());
    // println!("  [*] Physical memory initiualized.");
    // println!("  [.] Initializing virtual memory...");
    // arch::CurrentArch::init_memory();
    // println!("  [*] Virtual memory initialized.");
    loop {}
}
