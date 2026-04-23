//! This module defines the panic handler for the kernel. It also defines the `KernelFault` enum,
//! which represents the different types of faults that can occur in the kernel.

use core::panic::PanicInfo;

use crate::memory::{alloc::MemoryFault, paging::PageFault};

/// A categorised kernel fault used as a diagnostic discriminant in panic messages.
#[allow(dead_code)]
#[derive(Debug)]
pub enum KernelFault {
    /// A physical memory allocation or frame-tracking error.
    Memory(MemoryFault),
    /// A virtual-memory mapping or paging error.
    Paging(PageFault),
    /// A generic kernel panic with a static description string.
    Panic(&'static str),
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log_err!("Kernel panic: {}", _info);
    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}
