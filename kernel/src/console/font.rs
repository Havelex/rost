//! PSF (PC Screen Font) bitmap font data and glyph lookup.
//!
//! Embeds the default 8×16 PSF font and provides the [`glyph`] function that
//! returns a pointer to the 16-byte (PSF1) or variable-length (PSF2) glyph
//! bitmap for any Unicode codepoint.

#[allow(dead_code)]
/// Width of each font glyph in pixels.
pub const FONT_WIDTH: usize = 8;
/// Height of each font glyph in pixels (rows of bitmap data per glyph).
pub const FONT_HEIGHT: usize = 16;
#[allow(dead_code)]
/// Number of bytes that make up a single glyph in PSF1 format.
pub const GLYPH_SIZE: usize = FONT_HEIGHT;

/// The embedded default 8×16 PSF bitmap font.
pub static FONT: &[u8] = include_bytes!("../../res/fonts/default8x16.psf");

#[allow(dead_code)]
/// Byte size of a PSF1 file header.
pub const PSF1_HEADER_SIZE: usize = 4;

/// Return the bitmap data for character `c` from the embedded font.
///
/// Supports both PSF1 and PSF2 formats.  If `c` falls outside the font's
/// glyph table, the `'?'` glyph is returned; if that is also absent, glyph 0
/// is returned.
///
/// # Parameters
/// - `c`: The Unicode character whose glyph bitmap is requested.
///
/// # Returns
/// A byte slice of length equal to the glyph height containing the row bitmaps.
/// Each byte represents one row; bit 7 is the leftmost pixel.
pub fn glyph(c: char) -> &'static [u8] {
    let index = c as usize;

    if FONT[0..4] == [0x72, 0xb5, 0x4a, 0x86] {
        // PSF2 format
        let header_size = u32::from_le_bytes(FONT[8..12].try_into().unwrap()) as usize;
        let glyph_count = u32::from_le_bytes(FONT[16..20].try_into().unwrap()) as usize;
        let glyph_size = u32::from_le_bytes(FONT[24..28].try_into().unwrap()) as usize;
        // Fall back to '?' for characters outside the font's glyph table.
        let safe_index = if index < glyph_count {
            index
        } else {
            let q = b'?' as usize;
            if q < glyph_count { q } else { 0 }
        };
        let start = header_size + (safe_index * glyph_size);
        &FONT[start..start + glyph_size]
    } else {
        // PSF1 format: 256 glyphs (512 if mode bit 0 is set).
        let glyph_count: usize = if FONT[1] & 0x01 != 0 { 512 } else { 256 };
        let safe_index = if index < glyph_count {
            index
        } else {
            let q = b'?' as usize;
            if q < glyph_count { q } else { 0 }
        };
        let start = 4 + (safe_index * 16);
        &FONT[start..start + 16]
    }
}
