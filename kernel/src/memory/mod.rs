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
    // Cache the MemMap so the architecture layer can retrieve it for paging.
    MEM_MAP_ONCE.call_once(|| mem_map);
    phys::init(mem_map).map_err(|_| KernelError::OutOfMemory)
}

/// Returns a reference to the memory map recorded at boot.
///
/// Panics if `memory::init()` has not been called yet.
pub fn mem_map() -> &'static MemMap {
    MEM_MAP_ONCE.get().expect("memory::init() not called")
}
