use crate::memory::regions::MemMap;

pub mod alloc;
mod mem;
pub mod paging;
pub mod phys;
pub mod regions;

pub fn init(mem_map: MemMap) {
    phys::init(mem_map);
}
