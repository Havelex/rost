use spin::Mutex;

use crate::{
    arch::{
        Architecture, Cpu,
        x86_64::{
            cpu::{
                X86Cpu,
                interrupts,
            },
            memory::paging::{X86Mapper, mapper},
        },
    },
    error::Result,
    init_step,
};

mod asm;
pub mod cpu;
pub mod drivers;
mod gdt;
mod memory;
pub mod msr;
mod tss;

pub struct X86_64;

impl Architecture for X86_64 {
    type Mapper = X86Mapper;
    type Cpu = X86Cpu;

    fn init_early() -> Result<()> {
        init_step("Initializing TSS...", tss::init)?;
        init_step("Initializing GDT...", gdt::init)?;
        Ok(())
    }

    fn init_interrupts() -> Result<()> {
        interrupts::init()?;
        init_step("Initializing drivers", drivers::init)?;
        init_step("Enabling interrupts", || {
            Self::Cpu::enable_interrupts();
            Ok(())
        })?;
        Ok(())
    }

    fn init_memory() -> Result<()> {
        println!("  [.] Initializing paging...");

        let pml4 = memory::paging::allocate_pml4();
        memory::paging::init(pml4.expect("  [!] Fault at paging initialization"));

        println!("  [*] Paging initialized.");
        Ok(())
    }

    fn mapper() -> &'static Mutex<Self::Mapper> {
        mapper()
    }

    fn send_eoi(irq: u8) {
        interrupts::send_eoi(irq);
    }
}
