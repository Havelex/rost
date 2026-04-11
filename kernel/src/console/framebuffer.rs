use crate::boot::FramebufferInfo;

pub struct Framebuffer {
    pub addr: *mut u32,
    pub width: usize,
    pub height: usize,
    pub pitch: usize, // in pixels
    pub bpp: usize,
}

impl From<FramebufferInfo> for Framebuffer {
    fn from(info: FramebufferInfo) -> Self {
        Self {
            addr: info.addr as *mut u32,
            width: info.width,
            height: info.height,
            pitch: info.pitch / 4, // convert bytes to u32 pixels
            bpp: info.bpp,
        }
    }
}

impl From<&FramebufferInfo> for Framebuffer {
    fn from(info: &FramebufferInfo) -> Self {
        Self {
            addr: info.addr as *mut u32,
            width: info.width,
            height: info.height,
            pitch: info.pitch / 4, // convert bytes to u32 pixels
            bpp: info.bpp,
        }
    }
}

// SAFETY: We promise to only access framebuffer through a Mutex
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    pub fn clear(&mut self, color: u32) {
        let pixels = self.pitch * self.height;
        unsafe {
            for i in 0..pixels {
                core::ptr::write_volatile(self.addr.add(i), color);
            }
        }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        unsafe {
            let offset = y * self.pitch + x;
            core::ptr::write_volatile(self.addr.add(offset), color);
        }
    }

    pub fn draw_glyph(&mut self, x: usize, y: usize, glyph: &[u8], fg: u32, bg: u32) {
        for row in 0..16 {
            let bits = glyph[row];

            for col in 0..8 {
                let color = if (bits >> (7 - col)) & 1 == 1 { fg } else { bg };

                let px = x + col;
                let py = y + row;

                if px < self.width && py < self.height {
                    unsafe {
                        let offset = py * self.pitch + px;
                        core::ptr::write_volatile(self.addr.add(offset), color);
                    }
                }
            }
        }
    }
}
