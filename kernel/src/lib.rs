#![no_std]
#![deny(missing_docs)]

//! The main kernel library, containing all the core logic and functionality of the kernel.

use crate::{
    arch::{Arch, Architecture},
    boot::BootInfo,
    cpu::Cpu,
    error::Result,
    logger::indent::{pop_indent, push_indent},
    logo::print_logo,
    memory::regions::MemMap,
    time::sleep,
};

#[macro_use]
pub(crate) mod console;
#[macro_use]
pub(crate) mod logger;
#[macro_use]
pub(crate) mod keyboard;
pub(crate) mod arch;
pub(crate) mod boot;
pub(crate) mod cpu;
pub(crate) mod error;
mod logo;
pub(crate) mod memory;
pub(crate) mod panic;
pub(crate) mod shell;
pub(crate) mod time;
pub(crate) mod vfs;

pub use boot::init as boot;

/// Initializes the kernel with the provided boot information.
/// This function is called by the architecture-specific boot code after the initial setup is
/// complete.
///
/// # Parameters
/// - `info`: The boot information provided by the bootloader.
pub fn init(info: BootInfo) -> ! {
    let fb_info = info.framebuffer.unwrap();

    console::writer::init(fb_info.into());
    log_info!("Initializing Kernel...");
    push_indent();
    init_step(
        "Initializing early architecture",
        "Early architecture initialized",
        Arch::init_early,
    )
    .unwrap();
    init_step(
        "Initializing interrupts",
        "Interrupts initialized",
        Arch::init_interrupts,
    )
    .unwrap();

    log_info!("Enabling interrupts");
    Arch::enable_interrupts();

    // ── Memory initialisation ────────────────────────────────────────────────
    let mem_map: MemMap = info.memory_map.expect("Limine memory map missing").into();
    let hhdm_offset = info.offset.expect("Limine HHDM offset missing");
    let kernel_phys_base = info
        .kernel_phys_base
        .expect("Limine kernel phys base missing");
    let kernel_virt_base = info
        .kernel_virt_base
        .expect("Limine kernel virt base missing");

    init_step(
        "Initializing physical memory",
        "Physical memory initialized",
        || memory::init(&mem_map),
    )
    .unwrap();

    // Supply arch-specific boot params through the Architecture trait.
    Arch::set_boot_params(hhdm_offset, kernel_phys_base, kernel_virt_base);

    init_step(
        "Initializing virtual memory",
        "Virtual memory initialized",
        Arch::init_memory,
    )
    .unwrap();
    // ── End memory initialisation ─────────────────────────────────────────────

    // log_info!("Disabling interrupts to upgrade to APIC");
    // Arch::enable_interrupts();
    init_step("Upgrading to APIC", "Upgraded to APIC", Arch::init_post_mem).unwrap();

    log_info!("Disabling interrupts during driver initialization");
    Arch::disable_interrupts();
    init_step(
        "Initializing drivers",
        "Drivers initialized",
        Arch::init_drivers,
    )
    .unwrap();
    log_info!("Enabling interrupts after driver initialization");
    Arch::enable_interrupts();

    print!("\nFinishing boot");

    for _ in 0..3 {
        sleep(1000);
        print!(".");
    }
    sleep(1000);
    println!("Done!");
    sleep(1000);
    cls!();
    print_logo();

    vfs::init();
    shell::run();

    loop {
        <Arch as Architecture>::Cpu::halt()
    }
}

/// Helper function to initialize a boot step with logging and error handling.
///
/// # Parameters
/// - `name`: The name of the boot step, used for logging.
/// - `succ`: The success message to log if the step completes successfully.
/// - `f`: The function that performs the boot step, returning a `Result`.
///
/// # Returns
/// - `Ok(T)` if the boot step completes successfully, or panics if it fails.
/// - 'Err(KernelError)` if the boot step fails, which will be logged and cause a kernel panic.'
pub fn init_step<T, F>(name: &'static str, succ: &'static str, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    crate::log_info!("{}...", name);
    push_indent();

    match f() {
        Ok(val) => {
            pop_indent();
            log_ok!("{}.", succ);
            Ok(val)
        }
        Err(e) => {
            pop_indent();
            log_err!("FAILED: {:?}", e);
            panic!("Critical kernel failure during boot step: {}", name);
        }
    }
}
