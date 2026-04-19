use crate::console::{font, framebuffer::Framebuffer};
use core::{
    ffi::c_longlong,
    fmt::{self, Write},
};
use spin::{Mutex, Once};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AnsiState {
    #[default]
    Normal,
    Escaped,    // Saw \x1b (27)
    Csi,        // Saw [
    InSequence, // Reading numeric parameters
}

pub struct Console {
    fb: Option<Framebuffer>,
    cursor_x: usize,
    cursor_y: usize,
    color: u32,
    bg_color: u32,
    ansi_state: AnsiState,
    ansi_buffer: u32, // To store numeric codes like '32'
}

impl Console {
    pub fn new(fb: Framebuffer) -> Self {
        Self {
            fb: Some(fb),
            cursor_x: 0,
            cursor_y: 0,
            color: 0xFFFFFFFF,
            bg_color: 0x00000000,
            ansi_state: AnsiState::Normal,
            ansi_buffer: 0,
        }
    }

    pub const fn empty() -> Self {
        Self {
            fb: None,
            cursor_x: 0,
            cursor_y: 0,
            color: 0xFFFFFFFF,
            bg_color: 0x00000000,
            ansi_state: AnsiState::Normal,
            ansi_buffer: 0,
        }
    }

    pub fn write_char(&mut self, c: char) {
        match self.ansi_state {
            AnsiState::Normal => {
                if c == '\x1b' {
                    self.ansi_state = AnsiState::Escaped;
                } else {
                    self.handle_raw_char(c);
                }
            }
            AnsiState::Escaped => {
                if c == '[' {
                    self.ansi_state = AnsiState::Csi;
                    self.ansi_buffer = 0;
                } else {
                    self.ansi_state = AnsiState::Normal;
                }
            }
            AnsiState::Csi => {
                match c {
                    '0'..='9' => {
                        self.ansi_buffer = self.ansi_buffer * 10 + (c as u32 - '0' as u32);
                        self.ansi_state = AnsiState::InSequence;
                    }
                    'm' => {
                        // Standard SGR end
                        self.apply_ansi_code(0);
                        self.ansi_state = AnsiState::Normal;
                    }
                    _ => self.ansi_state = AnsiState::Normal,
                }
            }
            AnsiState::InSequence => {
                match c {
                    '0'..='9' => {
                        self.ansi_buffer = self.ansi_buffer * 10 + (c as u32 - '0' as u32);
                    }
                    'm' => {
                        self.apply_ansi_code(self.ansi_buffer);
                        self.ansi_state = AnsiState::Normal;
                    }
                    ';' => {
                        // Multiple parameters (we'll just clear buffer for now)
                        self.apply_ansi_code(self.ansi_buffer);
                        self.ansi_buffer = 0;
                    }
                    _ => self.ansi_state = AnsiState::Normal,
                }
            }
        }
    }

    fn apply_ansi_code(&mut self, code: u32) {
        self.color = match code {
            00 => 0xFF__FF_FF_FF, // Reset
            30 => 0xFF__00_00_00, // Black
            31 => 0xFF__80_00_00, // Red
            32 => 0xFF__00_80_00, // Green
            33 => 0xFF__80_80_00, // Yellow
            34 => 0xFF__00_00_80, // Blue
            35 => 0xFF__80_00_80, // Magenta
            36 => 0xFF__00_80_80, // Cyan
            37 => 0xFF__80_80_80, // Gray
            38 => 0xFF__80_40_00, // Orange
            90 => 0xFF__40_40_40, // Bright Black
            91 => 0xFF__FF_00_00, // Bright Red
            92 => 0xFF__00_FF_00, // Bright Green
            93 => 0xFF__FF_FF_00, // Bright Yellow
            94 => 0xFF__00_00_FF, // Bright Blue
            95 => 0xFF__FF_00_FF, // Bright Magenta
            96 => 0xFF__00_FF_FF, // Bright Cyan
            97 => 0xFF__FF_FF_FF, // Bright White
            98 => 0xFF__FF_80_00, // Bright Orange
            _ => self.color,      // Ignore others for now
        }
    }

    fn handle_raw_char(&mut self, c: char) {
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

        // Check scrolling again for the newline case
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
            core::ptr::copy(
                fb.addr.add(shift_amount_bytes),
                fb.addr,
                total_fb_bytes - shift_amount_bytes,
            );
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
