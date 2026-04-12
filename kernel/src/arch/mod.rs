#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub type CurrentArch = X86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

use spin::Mutex;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::X86_64;
use crate::memory::paging::Mapper;

pub trait Architecture {
    type Mapper: Mapper;

    // CPU
    fn init_early();
    fn halt();

    // intterupts
    fn init_interrupts();
    fn disable_interupts();
    fn enable_interrupts();

    // memory
    fn init_memory();
    fn mapper() -> &'static Mutex<Self::Mapper>;
}
