#![no_std]

use crate::{
    arch::{Architecture, CurrentArch, x86_64::drivers},
    boot::BootInfo,
    error::Result,
    logger::indent::{pop_indent, push_indent},
};

#[macro_use]
pub(crate) mod console;
#[macro_use]
pub(crate) mod logger;
pub(crate) mod arch;
pub(crate) mod boot;
pub(crate) mod cpu;
pub(crate) mod error;
pub(crate) mod memory;
pub(crate) mod panic;
pub(crate) mod time;

pub use boot::init as boot;

pub fn init(info: BootInfo) -> ! {
    let fb_info = info.framebuffer.unwrap();
    console::writer::init(fb_info.into());
    log_info!("Initializing Kernel...");
    push_indent();
    init_step("Initializing early architecture", CurrentArch::init_early).unwrap();
    init_step("Initializing interrupts...", CurrentArch::init_interrupts).unwrap();
    init_step("Initializing drivers...", drivers::init).unwrap();

    // println!("  [.] Initializing physical memory...");
    // let mem_map = info.memory_map.expect("  [!] Missing memory map!");
    // memory::init(mem_map.into());
    // println!("  [*] Physical memory initiualized.");
    // println!("  [.] Initializing virtual memory...");
    // arch::CurrentArch::init_memory();
    // println!("  [*] Virtual memory initialized.");
    loop {}
}

pub fn init_step<T, F>(name: &'static str, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    crate::log_info!("{}...", name);
    push_indent();

    match f() {
        Ok(val) => {
            pop_indent();
            log_ok!("{}", name);
            Ok(val)
        }
        Err(e) => {
            pop_indent();
            log_err!("FAILED: {:?}", e);
            panic!("Critical kernel failure during boot step: {}", name);
        }
    }
}
