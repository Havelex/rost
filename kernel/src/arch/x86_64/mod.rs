use spin::Mutex;

use crate::{
    arch::{
        Architecture, Cpu,
        x86_64::{
            cpu::{X86Cpu, interrupts},
            memory::paging::{X86Mapper, mapper},
        },
    },
    error::Result,
    init_step,
};

/// Low-level x86_64 assembly helpers (port I/O and control-register access).
pub(crate) mod asm;
/// x86_64 CPU implementation and interrupt entry plumbing.
pub mod cpu;
/// x86_64 platform drivers (PIT/HPET and related setup).
pub mod drivers;
/// Global Descriptor Table setup.
mod gdt;
/// x86_64 paging and memory-region helpers.
mod memory;
/// Model Specific Register (MSR) access helpers.
pub mod msr;
/// Task State Segment setup and backing storage.
mod tss;

/// Zero-sized marker type that implements [`Architecture`] for the x86_64 target.
///
/// All methods are called as associated functions (no `self` receiver); the
/// struct itself is never instantiated at runtime.
pub struct X86_64;

/// Boot parameters stored globally so that `init_memory` can read them without
/// changing the `Architecture` trait signature.
static HHDM_OFFSET: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
static KERNEL_PHYS_BASE: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
static KERNEL_VIRT_BASE: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

impl Architecture for X86_64 {
    type Mapper = X86Mapper;
    type Cpu = X86Cpu;

    fn init_early() -> Result<()> {
        use cpu::X86CpuExt;
        X86Cpu::enable_sse();
        init_step("Initializing TSS", "TSS initialized", tss::init)?;
        init_step("Initializing GDT", "GDT initialized", gdt::init)?;
        Ok(())
    }

    fn init_interrupts() -> Result<()> {
        interrupts::init()?;
        Ok(())
    }

    /// Attempt to upgrade from PIC to APIC after virtual memory is active.
    ///
    /// The default implementation is a no-op; architectures that support APIC
    /// should override this to try x2APIC / xAPIC initialization.
    fn init_post_mem() -> Result<()> {
        interrupts::init_apic_post_paging();
        Ok(())
    }

    fn set_boot_params(hhdm_offset: usize, kernel_phys_base: usize, kernel_virt_base: usize) {
        use core::sync::atomic::Ordering;
        HHDM_OFFSET.store(hhdm_offset, Ordering::Release);
        KERNEL_PHYS_BASE.store(kernel_phys_base, Ordering::Release);
        KERNEL_VIRT_BASE.store(kernel_virt_base, Ordering::Release);
    }

    fn init_memory() -> Result<()> {
        use crate::error::KernelError;
        use core::sync::atomic::Ordering;

        let hhdm_offset = HHDM_OFFSET.load(Ordering::Acquire);
        let kernel_phys_base = KERNEL_PHYS_BASE.load(Ordering::Acquire);
        let kernel_virt_base = KERNEL_VIRT_BASE.load(Ordering::Acquire);

        let mem_map = crate::memory::mem_map();

        memory::paging::init_paging(hhdm_offset, kernel_phys_base, kernel_virt_base, mem_map)
            .map_err(|_| KernelError::OutOfMemory)
    }

    fn init_drivers() -> Result<()> {
        init_step("Initializing drivers", "Drivers initialized", drivers::init)?;
        Ok(())
    }

    fn mapper() -> &'static Mutex<Self::Mapper> {
        mapper()
    }

    fn enable_interrupts() {
        X86Cpu::enable_interrupts();
    }

    fn disable_interrupts() {
        X86Cpu::disable_interrupts();
    }

    fn send_eoi(irq: u8) {
        interrupts::send_eoi(irq);
    }

    unsafe fn read_port_u8(port: u16) -> u8 {
        unsafe { crate::arch::x86_64::asm::inb(port) }
    }
}
