use spin::{Mutex, Once};

use crate::memory::{
    alloc::{FrameAllocator, MemoryFault},
    regions::{MemMap, MemoryRegionKind},
};

use core::cell::UnsafeCell;

const MAX_BITMAP_WORDS: usize = 8192;

struct BitmapStorage(UnsafeCell<[u64; MAX_BITMAP_WORDS]>);

unsafe impl Sync for BitmapStorage {}

static FRAME_BITMAP: BitmapStorage = BitmapStorage(UnsafeCell::new([0; MAX_BITMAP_WORDS]));

static FRAME_ALLOCATOR: Once<Mutex<FrameAllocator>> = Once::new();

pub fn frame_allocator() -> Result<&'static Mutex<FrameAllocator>, MemoryFault> {
    FRAME_ALLOCATOR.get().ok_or(MemoryFault::NoAllocator)
}

pub fn init(mem_map: MemMap) -> Result<(), MemoryFault> {
    let allocator =
        unsafe { FrameAllocator::new(&mut *FRAME_BITMAP.0.get(), mem_map.total_mem_size) };

    FRAME_ALLOCATOR.call_once(|| Mutex::new(allocator));

    reserve_non_usable(mem_map)
}

fn reserve_non_usable(mem_map: MemMap) -> Result<(), MemoryFault> {
    let alloc = FRAME_ALLOCATOR.get().unwrap();
    let mut alloc = alloc.lock();

    for region in mem_map.regions.iter().take(mem_map.count) {
        if !matches!(region.kind, MemoryRegionKind::Usable) {
            alloc.reserve_range(region.base, region.base + region.length - 1)?;
        }
    }

    Ok(())
}
