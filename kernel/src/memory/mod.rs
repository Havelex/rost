//! Memory management.
//!
//! This module contains the code for managing memory in the kernel, including physical memory
//! management, paging, and memory regions.

use spin::Once;

use crate::{
    error::{KernelError, Result},
    memory::regions::MemMap,
};

pub mod alloc;
pub mod paging;
pub mod phys;
pub mod regions;

mod mem;

static MEM_MAP_ONCE: Once<MemMap> = Once::new();

/// Initializes the memory management subsystem with the given memory map.
///
/// # Parameters
/// - `mem_map`: The memory map provided by the bootloader, describing the physical memory layout.
///
/// # Returns
/// - `Ok(())` if initialization was successful.
/// - `Err(KernelError::OutOfMemory)` if initialization failed due to insufficient memory
pub fn init(mem_map: &MemMap) -> Result<()> {
    let stored = MEM_MAP_ONCE.call_once(|| *mem_map);
    phys::init(stored).map_err(|_| KernelError::OutOfMemory)
}

/// Returns a reference to the memory map recorded at boot.
///
/// # Returns
/// - A reference to the `MemMap` that was stored during initialization.
///
/// # Panics
/// - Panics if `memory::init()` has not been called yet.
pub fn mem_map() -> &'static MemMap {
    MEM_MAP_ONCE.get().expect("memory::init() not called")
}
