#![no_std]

use crate::{
    arch::{Arch, Architecture},
    boot::BootInfo,
    cpu::Cpu,
    error::Result,
    logger::indent::{pop_indent, push_indent},
    memory::regions::MemMap,
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

    // ── Memory initialisation ────────────────────────────────────────────────
    let mem_map: MemMap = info.memory_map.expect("Limine memory map missing").into();
    let hhdm_offset = info.offset.expect("Limine HHDM offset missing");
    let kernel_phys_base = info
        .kernel_phys_base
        .expect("Limine kernel phys base missing");
    let kernel_virt_base = info
        .kernel_virt_base
        .expect("Limine kernel virt base missing");

    init_step("Initializing physical memory", || memory::init(&mem_map)).unwrap();

    // Supply arch-specific boot params through the Architecture trait.
    Arch::set_boot_params(hhdm_offset, kernel_phys_base, kernel_virt_base);

    init_step("Initializing virtual memory (paging)", Arch::init_memory).unwrap();
    // ── End memory initialisation ─────────────────────────────────────────────

    init_step("Upgrading to APIC (post-paging)", Arch::init_apic_post_paging).unwrap();

    println!("\nFinishing boot");

    for i in 0..3 {
        sleep(1000);
        log_info!("Timer ticks at T+{}s: {}", i + 1, crate::time::get_ticks());
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
