//! This module defines the panic handler for the kernel. It also defines the `KernelFault` enum,
//! which represents the different types of faults that can occur in the kernel.

use core::panic::PanicInfo;

use crate::memory::{alloc::MemoryFault, paging::PageFault};

/// An Enum representing the type of fault that occurred in the kernel.,
#[allow(dead_code)]
#[derive(Debug)]
pub enum KernelFault {
    Memory(MemoryFault),
    Paging(PageFault),
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
