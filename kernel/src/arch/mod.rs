//! Architecture abstraction layer.
//!
//! This module defines the [`Architecture`] trait that every supported target must implement,
//! and exposes the concrete [`Arch`] type alias so the rest of the kernel can refer to the
//! current platform without knowing which architecture it compiles for.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

/// Concrete architecture type for the current compilation target.
///
/// On `x86_64` this resolves to [`x86_64::X86_64`](x86_64::X86_64).
#[cfg(target_arch = "x86_64")]
pub type Arch = X86_64;

/// Architecture-specific implementation for `aarch64`.
#[cfg(target_arch = "aarch64")]
pub mod aarch64;

use spin::Mutex;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::X86_64;
use crate::{cpu::Cpu, error::Result, memory::paging::Mapper};

/// Abstraction over an architecture-specific hardware platform.
///
/// Every supported target must provide a zero-sized struct that implements this trait.
/// The kernel uses the [`Arch`] type alias to call these methods without knowing the
/// underlying hardware.
pub trait Architecture {
    /// Architecture-specific page-table mapper type.
    type Mapper: Mapper;
    /// Architecture-specific CPU operations type.
    type Cpu: Cpu;

    /// Perform the earliest architecture-specific hardware initialization.
    ///
    /// Called before interrupts are enabled.  On x86_64 this enables SSE,
    /// loads the TSS and installs the GDT.
    ///
    /// # Returns
    /// - `Ok(())` on success, or a [`KernelError`](crate::error::KernelError) on failure.
    fn init_early() -> Result<()>;

    /// Set up the interrupt descriptor table and the legacy PIC.
    ///
    /// Called once after [`init_early`](Self::init_early).  After this returns,
    /// interrupts may be enabled with [`enable_interrupts`](Self::enable_interrupts).
    ///
    /// # Returns
    /// - `Ok(())` on success, or a [`KernelError`](crate::error::KernelError) on failure.
    fn init_interrupts() -> Result<()>;

    /// Store architecture-specific boot parameters (HHDM offset, kernel physical
    /// and virtual base addresses) that `init_memory` will read later.
    ///
    /// # Parameters
    /// - `hhdm_offset`: The Higher Half Direct Map offset provided by the bootloader.
    /// - `kernel_phys_base`: The physical base address of the loaded kernel image.
    /// - `kernel_virt_base`: The virtual base address of the loaded kernel image.
    fn set_boot_params(hhdm_offset: usize, kernel_phys_base: usize, kernel_virt_base: usize);

    /// Build and activate the kernel's own page tables.
    ///
    /// Must be called after [`set_boot_params`](Self::set_boot_params) has stored the
    /// HHDM offset and kernel base addresses.
    ///
    /// # Returns
    /// - `Ok(())` on success, or a [`KernelError`](crate::error::KernelError) on failure.
    fn init_memory() -> Result<()>;

    /// Perform post-paging hardware initialization (e.g. upgrade PIC → APIC).
    ///
    /// Called once after [`init_memory`](Self::init_memory) returns, while interrupts
    /// are still enabled.
    ///
    /// # Returns
    /// - `Ok(())` on success, or a [`KernelError`](crate::error::KernelError) on failure.
    fn init_post_mem() -> Result<()>;

    /// Initialize architecture-specific hardware drivers (e.g. PIT timer).
    ///
    /// Called with interrupts **disabled** so drivers can unmask IRQs individually at
    /// the point they are ready to receive them.
    ///
    /// # Returns
    /// - `Ok(())` on success, or a [`KernelError`](crate::error::KernelError) on failure.
    fn init_drivers() -> Result<()>;

    /// Return a reference to the global page-table mapper protected by a spin-lock.
    #[allow(dead_code)]
    fn mapper() -> &'static Mutex<Self::Mapper>;

    /// Enable hardware interrupts on the current CPU (e.g. `sti` on x86_64).
    fn enable_interrupts();

    /// Disable hardware interrupts on the current CPU (e.g. `cli` on x86_64).
    fn disable_interrupts();

    /// Acknowledge the end of an interrupt to the active interrupt controller.
    ///
    /// # Parameters
    /// - `irq`: The IRQ line number (0–15) that fired.  Used by the PIC path to
    ///   decide whether a cascade EOI is also required.
    fn send_eoi(irq: u8);

    /// Read a single byte from an architecture I/O port.
    ///
    /// # Parameters
    /// - `port`: The 16-bit I/O port address to read from.
    ///
    /// # Returns
    /// The byte value read from the port.
    ///
    /// # Safety
    /// The caller must ensure that `port` is a valid I/O port and that reading
    /// from it is safe in the current context.
    unsafe fn read_port_u8(port: u16) -> u8;
}
