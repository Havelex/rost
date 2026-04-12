use crate::boot::limine_helpers::{FB_REQUEST, HDDM_REQUEST, MEM_MAP_REQUEST};
use spin::Once;

pub const MAX_REGIONS: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct MemoryRegionInfo {
    pub base: usize,
    pub length: usize,
    pub kind: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemMapInfo {
    pub regions: &'static [MemoryRegionInfo],
}

static mut REGIONS: [MemoryRegionInfo; MAX_REGIONS] = [MemoryRegionInfo {
    base: 0,
    length: 0,
    kind: 0,
}; MAX_REGIONS];

static MEMORY_MAP_INFO: Once<&'static [MemoryRegionInfo]> = Once::new();

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FramebufferInfo {
    pub addr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bpp: usize,
}

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: Option<FramebufferInfo>,
    pub memory_map: Option<MemMapInfo>,
    pub offset: Option<usize>,
}

impl BootInfo {
    pub fn new() -> BootInfo {
        let hddm_response = HDDM_REQUEST
            .response()
            .expect("No HDDM was provided by Limine");

        let hhdm_offset = hddm_response.offset as usize;

        let fb_response = FB_REQUEST
            .response()
            .expect("Limine framebuffer response missing");

        // FIX: Change parameter type to *mut () to match fb.address()
        let make_virt = |phys_ptr: *mut ()| -> *mut u8 {
            let addr_val = phys_ptr as usize;
            // If Limine gives us an address already in the higher half, don't double-offset
            if addr_val >= 0xFFFF_8000_0000_0000 {
                addr_val as *mut u8
            } else {
                (addr_val + hhdm_offset) as *mut u8
            }
        };

        let fb_info = match fb_response.framebuffers_rev1() {
            Some(f) => {
                let fb = f.first().expect("No framebuffer provided");
                Some(FramebufferInfo {
                    addr: make_virt(fb.address()), // Now passes *mut () correctly
                    width: fb.width as usize,
                    height: fb.height as usize,
                    pitch: fb.pitch as usize,
                    bpp: fb.bpp as usize,
                })
            }
            None => {
                let fb = fb_response
                    .framebuffers()
                    .first()
                    .expect("No framebuffer provided");

                Some(FramebufferInfo {
                    addr: make_virt(fb.address()), // Now passes *mut () correctly
                    width: fb.width as usize,
                    height: fb.height as usize,
                    pitch: fb.pitch as usize,
                    bpp: fb.bpp as usize,
                })
            }
        };

        let mem_map_response = MEM_MAP_REQUEST
            .response()
            .expect("Limine memory map response missing");

        let entries = mem_map_response.entries();
        let mut count = 0;

        for (i, entry) in entries.iter().enumerate().take(MAX_REGIONS) {
            unsafe {
                REGIONS[i] = MemoryRegionInfo {
                    base: entry.base as usize,
                    length: entry.length as usize,
                    // FIX: entry.type_ is likely an enum, cast it to u64
                    kind: entry.type_ as u64,
                };
            }
            count += 1;
        }

        let mem_map = MEMORY_MAP_INFO.call_once(|| unsafe { &REGIONS[..count] });

        BootInfo {
            framebuffer: fb_info,
            memory_map: Some(MemMapInfo { regions: mem_map }),
            offset: Some(hhdm_offset),
        }
    }
}
