#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub type CurrentArch = X86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

use spin::Mutex;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::X86_64;
use crate::{cpu::Cpu, memory::paging::Mapper};

pub trait Architecture {
    type Mapper: Mapper;
    type Cpu: Cpu;

    // CPU
    fn init_early();

    // intterupts
    fn init_interrupts();

    // memory
    fn init_memory();

    // getter
    fn mapper() -> &'static Mutex<Self::Mapper>;
}
