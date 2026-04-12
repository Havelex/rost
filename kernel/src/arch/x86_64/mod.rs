use spin::Mutex;

use crate::{
    arch::{
        Architecture, Cpu,
        x86_64::{
            cpu::{X86Cpu, interrupts},
            interrupts::pic,
            memory::paging::{X86Mapper, mapper},
        },
    },
    error::Result,
    init_step,
};

mod cpu;
pub mod drivers;
mod gdt;
mod memory;
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
        interrupts::init();
        init_step("Remapping PIC", || {
            unsafe { pic::remap_pic(32, 40) };
            Ok(())
        })?;
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
}
