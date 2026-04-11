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
        self.fb.as_mut().unwrap().clear(color);
    }

    pub fn write_char(&mut self, c: char) {
        if self.fb.is_none() {
            return;
        }

        match c {
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
                return;
            }
            '\r' => {
                self.cursor_x = 0;
                return;
            }
            '\t' => {
                self.cursor_x += 4;
                return;
            }
            _ => {}
        }

        let width_chars = self.fb.as_ref().unwrap().width / 8;
        let height_chars = self.fb.as_ref().unwrap().height / 16;

        if self.cursor_x >= width_chars {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y >= height_chars {
            self.scroll_up();
            self.cursor_y = height_chars - 1;
        }

        let glyph = font::glyph(c);

        let x = self.cursor_x * 8;
        let y = self.cursor_y * 16;

        self.fb
            .as_mut()
            .unwrap()
            .draw_glyph(x, y, glyph, self.color, self.bg_color);

        self.cursor_x += 1;
    }

    fn scroll_up(&mut self) {
        let fb = match &mut self.fb {
            Some(fb) => fb,
            None => return,
        };

        let row_size = fb.pitch / 4 * 16; // 16 pixels tall
        let total_rows = fb.height / 16;

        unsafe {
            // Move all rows up by one
            for row in 1..total_rows {
                let src_offset = row * row_size;
                let dst_offset = (row - 1) * row_size;
                for col in 0..row_size {
                    let src_ptr = fb.addr.add(src_offset + col);
                    let dst_ptr = fb.addr.add(dst_offset + col);
                    core::ptr::write_volatile(dst_ptr, core::ptr::read_volatile(src_ptr));
                }
            }

            // Clear last row
            let last_row_offset = (total_rows - 1) * row_size;
            for i in 0..row_size {
                core::ptr::write_volatile(fb.addr.add(last_row_offset + i), self.bg_color);
            }
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
