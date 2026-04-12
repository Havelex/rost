use crate::boot::limine_helpers::{FB_REQUEST, MEM_MAP_REQUEST};
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
    pub addr: *mut u32,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bpp: usize,
}

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: Option<FramebufferInfo>,
    pub memory_map: Option<MemMapInfo>,
}

impl BootInfo {
    pub fn new() -> BootInfo {
        let fb_response = FB_REQUEST
            .response()
            .expect("Limine framebuffer response missing");

        let fbv1 = fb_response.framebuffers_rev1();

        let fb_info = match fbv1 {
            Some(f) => {
                let fb = f.first().expect("No framebuffer was provided by limine");
                let mode = fb.modes().first().expect("No modes");
                Some(FramebufferInfo {
                    addr: fb.address() as *mut u32,
                    width: mode.width as usize,
                    height: mode.height as usize,
                    pitch: mode.pitch as usize,
                    bpp: mode.bpp as usize,
                })
            }
            None => {
                let fb = fb_response
                    .framebuffers()
                    .first()
                    .expect("No framebuffer was provided by limine");

                Some(FramebufferInfo {
                    addr: fb.address() as *mut u32,
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

        if entries.len() == 0 {
            panic!("No memory regions returned by Limine");
        }

        let mut count = 0;

        for (i, entry) in entries.iter().enumerate().take(MAX_REGIONS) {
            unsafe {
                REGIONS[i] = MemoryRegionInfo {
                    base: entry.base as usize,
                    length: entry.length as usize,
                    kind: entry.type_,
                };
            }
            count += 1;
        }

        let mem_map = MEMORY_MAP_INFO.call_once(|| unsafe { &REGIONS[..count] });

        BootInfo {
            framebuffer: fb_info,
            memory_map: Some(MemMapInfo { regions: mem_map }),
        }
    }
}
