use crate::console::{font, framebuffer::Framebuffer};
use core::fmt::{self, Write};
use spin::{Mutex, Once};

pub struct Console {
    fb: Option<Framebuffer>,
    cursor_x: usize,
    cursor_y: usize,
    color: u32,
    bg_color: u32,
}

impl Console {
    pub fn new(fb: Framebuffer) -> Self {
        Self {
            fb: Some(fb),
            cursor_x: 0,
            cursor_y: 0,
            color: 0xFFFFFFFF,
            bg_color: 0x00000000,
        }
    }

    pub const fn empty() -> Self {
        Self {
            fb: None,
            cursor_x: 0,
            cursor_y: 0,
            color: 0xFFFFFFFF,
            bg_color: 0x00000000,
        }
    }

    pub fn clear(&mut self, color: u32) {
        if let Some(ref mut fb) = self.fb {
            fb.clear(color);
        }
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
            }
            '\r' => {
                self.cursor_x = 0;
            }
            '\t' => {
                self.cursor_x = (self.cursor_x + 4) & !3;
            }
            _ => {
                let (width_chars, height_chars) = if let Some(ref fb) = self.fb {
                    (fb.width / 8, fb.height / 16)
                } else {
                    return;
                };

                if self.cursor_x >= width_chars {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }

                // Check scrolling BEFORE drawing
                if self.cursor_y >= height_chars {
                    self.scroll_up();
                    self.cursor_y = height_chars - 1;
                }

                if let Some(ref mut fb) = self.fb {
                    let glyph = font::glyph(c);
                    fb.draw_glyph(
                        self.cursor_x * 8,
                        self.cursor_y * 16,
                        glyph,
                        self.color,
                        self.bg_color,
                    );
                }
                self.cursor_x += 1;
            }
        }

        // Final check for the newline case
        if let Some(ref fb) = self.fb {
            let height_chars = fb.height / 16;
            if self.cursor_y >= height_chars {
                self.scroll_up();
                self.cursor_y = height_chars - 1;
            }
        }
    }

    fn scroll_up(&mut self) {
        let fb = match &mut self.fb {
            Some(fb) => fb,
            None => return,
        };

        let bytes_per_row = fb.pitch;
        let char_height = 16;
        let shift_amount_bytes = char_height * bytes_per_row;
        let total_fb_bytes = fb.height * bytes_per_row;

        unsafe {
            // Move rows up
            core::ptr::copy(
                fb.addr.add(shift_amount_bytes),
                fb.addr,
                total_fb_bytes - shift_amount_bytes,
            );

            // Clear the new bottom line (sets bytes to 0/Black)
            let last_row_ptr = fb.addr.add(total_fb_bytes - shift_amount_bytes);
            core::ptr::write_bytes(last_row_ptr, 0, shift_amount_bytes);
        }
    }
}

unsafe impl Sync for Console {}
unsafe impl Send for Console {}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

static CONSOLE: Once<Mutex<Console>> = Once::new();

pub fn init(fb: Framebuffer) -> &'static Mutex<Console> {
    CONSOLE.call_once(|| Mutex::new(Console::new(fb)))
}

pub fn console() -> &'static Mutex<Console> {
    CONSOLE.get().expect("Console not initialized")
}
