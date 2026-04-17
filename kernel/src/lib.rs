#![no_std]

use crate::{
    arch::{Arch, Architecture},
    boot::BootInfo,
    cpu::Cpu,
    error::Result,
    logger::indent::{pop_indent, push_indent},
    time::sleep,
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
    init_step("Initializing early architecture", Arch::init_early).unwrap();
    init_step("Initializing interrupts...", Arch::init_interrupts).unwrap();

    print!("\nFinishing boot");
    // unsafe {
    //     core::arch::asm!("int $32");
    // }
    unsafe {
        log_info!(
            "PIC IRR: {:#04x}",
            crate::arch::x86_64::cpu::interrupts::pic::pic_get_irr()
        );
        log_info!(
            "PIC ISR: {:#04x}",
            crate::arch::x86_64::cpu::interrupts::pic::pic_get_isr()
        );
    }
    for _ in 0..3 {
        sleep(1000);
        print!(".");
    }

    println!();
    println!("Done!");

    loop {
        <Arch as Architecture>::Cpu::halt()
    }
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
