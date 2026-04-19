use core::panic::PanicInfo;

use crate::memory::{alloc::MemoryFault, paging::PageFault};

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
