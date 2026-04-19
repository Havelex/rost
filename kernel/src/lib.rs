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
#[macro_use]
pub(crate) mod keyboard;
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

    println!();
    println!("Done!");
    println!("Keyboard input active — press keys to log them (Esc to stop):");

    loop {
        let key = wait_for_key!();

        // Scancode 0x01 = Escape — stop logging as a demonstration.
        if key.keycode == 0x01 {
            println!("[keyboard] Escape pressed, halting.");
            break;
        }

        match key.ascii {
            Some('\n') => println!("[keyboard] Enter"),
            Some('\t') => println!("[keyboard] Tab"),
            Some(' ')  => println!("[keyboard] Space"),
            Some(c)    => println!("[keyboard] '{}' (scancode={:#04x})", c, key.scancode),
            None       => println!("[keyboard] scancode={:#04x}", key.scancode),
        }
    }

    loop {
        <Arch as Architecture>::Cpu::halt()
    }
}

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
