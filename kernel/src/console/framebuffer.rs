//! Raw linear framebuffer abstraction.
//!
//! [`Framebuffer`] wraps the memory-mapped pixel buffer provided by the
//! bootloader and exposes pixel-level drawing primitives used by the console
//! writer.

use crate::boot::FramebufferInfo;

/// A handle to the linear framebuffer provided by the bootloader.
pub struct Framebuffer {
    /// Pointer to the first byte of the framebuffer in virtual address space.
    pub addr: *mut u8,
    /// Width of the framebuffer in pixels.
    pub width: usize,
    /// Height of the framebuffer in pixels.
    pub height: usize,
    /// Bytes per scanline (row stride). Stored in BYTES
    pub pitch: usize,
    /// Bits per pixel.
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
    /// Fill the entire framebuffer with `color`.
    ///
    /// Uses [`write_pixel`](Framebuffer::write_pixel) for every pixel to
    /// respect the scanline pitch and avoid writing into row-padding memory.
    ///
    /// # Parameters
    /// - `color`: 32-bit ARGB colour to fill with.
    pub fn clear(&mut self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_pixel(x, y, color);
            }
        }
    }

    /// Write a single pixel at `(x, y)` with the given 32-bit ARGB `color`.
    ///
    /// Out-of-bounds coordinates are silently ignored.
    ///
    /// # Parameters
    /// - `x`: Horizontal pixel coordinate (0 = leftmost column).
    /// - `y`: Vertical pixel coordinate (0 = topmost row).
    /// - `color`: 32-bit ARGB colour value to write.
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

    /// Draw a PSF glyph bitmap at character cell `(x, y)` in pixels.
    ///
    /// Each byte in `glyph` is one row; bit 7 of each byte is the leftmost
    /// pixel.  Foreground pixels are drawn with `fg`; background pixels with `bg`.
    ///
    /// # Parameters
    /// - `x`: Left pixel coordinate of the glyph.
    /// - `y`: Top pixel coordinate of the glyph.
    /// - `glyph`: Byte slice of glyph bitmap rows (one byte per row).
    /// - `fg`: 32-bit ARGB foreground colour.
    /// - `bg`: 32-bit ARGB background colour.
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
