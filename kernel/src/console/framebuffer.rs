use crate::boot::FramebufferInfo;

pub struct Framebuffer {
    pub addr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize, // Stored in BYTES
    pub bpp: usize,
}

impl From<FramebufferInfo> for Framebuffer {
    fn from(info: FramebufferInfo) -> Self {
        (&info).into()
    }
}

impl From<&FramebufferInfo> for Framebuffer {
    fn from(info: &FramebufferInfo) -> Self {
        Self {
            addr: info.addr,
            width: info.width,
            height: info.height,
            pitch: if info.pitch == 0 {
                info.width * (info.bpp / 8)
            } else {
                info.pitch
            },
            bpp: info.bpp,
        }
    }
}

// SAFETY: We promise to only access framebuffer through a Mutex
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    /// Fills the screen with a color. Uses write_pixel to ensure
    /// we respect the pitch and don't draw into padding memory.
    pub fn clear(&mut self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_pixel(x, y, color);
            }
        }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        unsafe {
            let bytes_per_pixel = self.bpp / 8;
            let byte_offset = (y * self.pitch) + (x * bytes_per_pixel);

            // Get the pointer to the specific byte, then cast to u32 for the write
            let pixel_ptr = self.addr.add(byte_offset) as *mut u32;

            // Volatile write ensures the compiler doesn't optimize this away
            core::ptr::write_volatile(pixel_ptr, color);
        }
    }

    pub fn draw_glyph(&mut self, x: usize, y: usize, glyph: &[u8], fg: u32, bg: u32) {
        for row in 0..16 {
            let bits = glyph[row];

            for col in 0..8 {
                // PSF fonts store the leftmost pixel in the highest bit (MSB)
                let color = if (bits >> (7 - col)) & 1 == 1 { fg } else { bg };

                // Leverage our fixed write_pixel for all coordinate math
                self.write_pixel(x + col, y + row, color);
            }
        }
    }
}
