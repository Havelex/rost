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
    loop {}
}
