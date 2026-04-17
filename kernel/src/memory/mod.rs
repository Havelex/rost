use spin::Once;

use crate::{
    error::{KernelError, Result},
    memory::regions::MemMap,
};

pub mod alloc;
mod mem;
pub mod paging;
pub mod phys;
pub mod regions;

static MEM_MAP_ONCE: Once<MemMap> = Once::new();

pub fn init(mem_map: MemMap) -> Result<()> {
    // Store the MemMap in static storage and use the returned reference to call
    // phys::init.  Passing the &'static MemMap reference (8 bytes) instead of
    // the value (4 KiB) eliminates one large stack copy from the deepest call
    // frames, keeping the kernel well inside the 32 KiB boot stack.
    let stored = MEM_MAP_ONCE.call_once(|| mem_map);
    phys::init(stored).map_err(|_| KernelError::OutOfMemory)
}

/// Returns a reference to the memory map recorded at boot.
///
/// Panics if `memory::init()` has not been called yet.
pub fn mem_map() -> &'static MemMap {
    MEM_MAP_ONCE.get().expect("memory::init() not called")
}
