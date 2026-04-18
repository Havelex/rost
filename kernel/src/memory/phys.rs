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

pub fn init(mem_map: &MemMap) -> Result<(), MemoryFault> {
    // Size the bitmap from allocatable RAM only.  Reserved entries cover MMIO
    // ranges (PCIe BARs at ~0xB0000000, LAPIC at 0xFEE00000, etc.) that can
    // push the physical ceiling well above 2 GiB, overflowing the fixed bitmap.
    // Only Usable, BootloaderReclaimable, and KernelAndModules regions contain
    // real RAM we will ever allocate from.
    //
    // A plain loop is used instead of an iterator chain so that the call stack
    // in debug mode stays shallow (the lazy iterator adapters would add ~8
    // extra non-inlined frames that are materialised only when the consuming
    // .max() drives them, which together with the outer MemMap frames was
    // overflowing the 32 KiB boot stack and triple-faulting).
    let mut max_ram_addr: usize = 0;
    for region in mem_map.regions.iter().take(mem_map.count) {
        if matches!(
            region.kind,
            MemoryRegionKind::Usable
                | MemoryRegionKind::BootloaderReclaimable
                | MemoryRegionKind::KernelAndModules
        ) {
            let end = region.base.saturating_add(region.length);
            if end > max_ram_addr {
                max_ram_addr = end;
            }
        }
    }

    let allocator =
        unsafe { FrameAllocator::new(&mut *FRAME_BITMAP.0.get(), max_ram_addr) };

    FRAME_ALLOCATOR.call_once(|| Mutex::new(allocator));

    // Physical frame 0 (address 0x0) may not appear in any Limine memory-map
    // region, leaving it free in the bitmap even though it must never be
    // allocated: in Limine's BIOS boot environment the HHDM does not cover
    // physical address 0, so dereferencing (hhdm_offset + 0) would fault.
    // Reserve it unconditionally before scanning the rest of the map.
    {
        let mut alloc = FRAME_ALLOCATOR.get().unwrap().lock();
        let _ = alloc.reserve_range(0, 0);
    }

    reserve_non_usable(mem_map)
}

fn reserve_non_usable(mem_map: &MemMap) -> Result<(), MemoryFault> {
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
