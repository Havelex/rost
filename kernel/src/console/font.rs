pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;
pub const GLYPH_SIZE: usize = FONT_HEIGHT;

pub static FONT: &[u8] = include_bytes!("../../res/fonts/default8x16.psf");

pub const PSF1_HEADER_SIZE: usize = 4;

pub fn glyph(c: char) -> &'static [u8] {
    let index = c as usize;

    if FONT[0..4] == [0x72, 0xb5, 0x4a, 0x86] {
        // PSF2
        let header_size = u32::from_le_bytes(FONT[8..12].try_into().unwrap()) as usize;
        let glyph_size = u32::from_le_bytes(FONT[24..28].try_into().unwrap()) as usize;
        let num_glyphs = u32::from_le_bytes(FONT[16..20].try_into().unwrap()) as usize;
        // Fall back to '?' for any codepoint beyond the font's glyph table.
        let safe_index = if index < num_glyphs { index } else { b'?' as usize };
        let start = header_size + (safe_index * glyph_size);
        &FONT[start..start + glyph_size]
    } else {
        // PSF1: 256 glyphs, or 512 when mode bit 0 is set.
        let num_glyphs: usize = if FONT[2] & 0x01 != 0 { 512 } else { 256 };
        // Fall back to '?' for any codepoint beyond the font's glyph table.
        let safe_index = if index < num_glyphs { index } else { b'?' as usize };
        let start = 4 + (safe_index * 16);
        &FONT[start..start + 16]
    }
}
