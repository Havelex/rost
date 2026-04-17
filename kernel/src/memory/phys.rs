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
    // Size the bitmap from actual physical RAM only.  Framebuffer, Bad Memory,
    // and Unknown entries can have very high physical addresses (e.g. the VESA
    // framebuffer is often at 0xFD000000) that would overflow the fixed-size
    // bitmap and trigger the capacity assert.
    let max_ram_addr = mem_map
        .regions
        .iter()
        .take(mem_map.count)
        .filter(|r| {
            !matches!(
                r.kind,
                MemoryRegionKind::Framebuffer
                    | MemoryRegionKind::BadMemory
                    | MemoryRegionKind::Unknown(_)
            )
        })
        .map(|r| r.base + r.length)
        .max()
        .unwrap_or(0);

    let allocator =
        unsafe { FrameAllocator::new(&mut *FRAME_BITMAP.0.get(), max_ram_addr) };

    FRAME_ALLOCATOR.call_once(|| Mutex::new(allocator));

    reserve_non_usable(mem_map)
}

fn reserve_non_usable(mem_map: MemMap) -> Result<(), MemoryFault> {
    let alloc = FRAME_ALLOCATOR.get().unwrap();
    let mut alloc = alloc.lock();

    for region in mem_map.regions.iter().take(mem_map.count) {
        if !matches!(region.kind, MemoryRegionKind::Usable) {
            let end = region.base.saturating_add(region.length).saturating_sub(1);
            match alloc.reserve_range(region.base, end) {
                Ok(()) => {}
                // Region lies beyond the RAM ceiling (e.g. MMIO / framebuffer).
                // It does not need to be tracked in the bitmap.
                Err(MemoryFault::FrameIndexOutOfBounds { .. }) => {}
                // Overlapping non-usable regions – skip the duplicate frames.
                Err(MemoryFault::DoubleAllocation { .. }) => {}
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}
