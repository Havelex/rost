#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub type Arch = X86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

use spin::Mutex;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::X86_64;
use crate::{cpu::Cpu, error::Result, memory::paging::Mapper};

pub trait Architecture {
    type Mapper: Mapper;
    type Cpu: Cpu;

    // CPU
    fn init_early() -> Result<()>;

    // interrupts
    fn init_interrupts() -> Result<()>;

    // memory
    /// Store architecture-specific boot parameters (HHDM offset, kernel physical
    /// and virtual base addresses) that `init_memory` will read later.
    fn set_boot_params(hhdm_offset: usize, kernel_phys_base: usize, kernel_virt_base: usize);
    fn init_memory() -> Result<()>;
    fn init_post_mem() -> Result<()>;
    fn init_drivers() -> Result<()>;

    // getter
    fn mapper() -> &'static Mutex<Self::Mapper>;

    fn enable_interrupts();
    fn disable_interrupts();
    fn send_eoi(irq: u8);
}
