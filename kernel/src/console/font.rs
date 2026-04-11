pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;
pub const GLYPH_SIZE: usize = FONT_HEIGHT;

pub static FONT: &[u8] = include_bytes!("../../res/fonts/default8x16.psf");

pub const PSF1_HEADER_SIZE: usize = 4;

pub fn glyph(c: char) -> &'static [u8] {
    let index = c as usize;
    let start = PSF1_HEADER_SIZE + index * GLYPH_SIZE;
    &FONT[start..start + GLYPH_SIZE]
}
