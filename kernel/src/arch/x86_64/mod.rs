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

/// Boot parameters stored globally so that `init_memory` can read them without
/// changing the `Architecture` trait signature.
static HHDM_OFFSET: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);
static KERNEL_PHYS_BASE: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);
static KERNEL_VIRT_BASE: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

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

    fn set_boot_params(hhdm_offset: usize, kernel_phys_base: usize, kernel_virt_base: usize) {
        use core::sync::atomic::Ordering;
        HHDM_OFFSET.store(hhdm_offset, Ordering::Release);
        KERNEL_PHYS_BASE.store(kernel_phys_base, Ordering::Release);
        KERNEL_VIRT_BASE.store(kernel_virt_base, Ordering::Release);
    }

    fn init_memory() -> Result<()> {
        use core::sync::atomic::Ordering;
        use crate::error::KernelError;

        let hhdm_offset = HHDM_OFFSET.load(Ordering::Acquire);
        let kernel_phys_base = KERNEL_PHYS_BASE.load(Ordering::Acquire);
        let kernel_virt_base = KERNEL_VIRT_BASE.load(Ordering::Acquire);

        let mem_map = crate::memory::mem_map();

        memory::paging::init_paging(hhdm_offset, kernel_phys_base, kernel_virt_base, mem_map)
            .map_err(|_| KernelError::OutOfMemory)
    }

    fn mapper() -> &'static Mutex<Self::Mapper> {
        mapper()
    }

    fn send_eoi(irq: u8) {
        interrupts::send_eoi(irq);
    }
}
