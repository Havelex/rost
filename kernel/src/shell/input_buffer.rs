const BUFFER_SIZE: usize = 256;

/// A fixed-size buffer that accumulates typed characters for one shell line.
///
/// Only ASCII characters are stored (one byte per char).  The buffer stops
/// accepting input once it reaches [`BUFFER_SIZE`] bytes.
pub struct InputBuffer {
    buf: [u8; BUFFER_SIZE],
    len: usize,
}

impl InputBuffer {
    /// Create a new, empty input buffer.
    pub const fn new() -> Self {
        Self {
            buf: [0u8; BUFFER_SIZE],
            len: 0,
        }
    }

    /// Append a printable ASCII character to the buffer.
    ///
    /// # Parameters
    /// - `c`: The character to append.  Only ASCII characters (0x20 to 0x7E) are accepted.
    ///
    /// # Returns
    /// - `true` if the character was successfully added to the buffer.
    /// - `false` if the buffer is full and the character was not added.
    pub fn push(&mut self, c: char) -> bool {
        if self.len < BUFFER_SIZE {
            self.buf[self.len] = c as u8;
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Remove the last character from the buffer (backspace).
    ///
    /// # Returns
    /// - `true` if a character was removed from the buffer.
    /// - `false` if the buffer was already empty and no character was removed.
    pub fn backspace(&mut self) -> bool {
        if self.len > 0 {
            self.len -= 1;
            true
        } else {
            false
        }
    }

    /// Discard all buffered input.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// View the current buffer contents as a string slice.
    ///
    /// # Returns
    /// A string slice `&str` containing the characters currently in the buffer.
    pub fn as_str(&self) -> &str {
        // Safety: only ASCII bytes are ever written into the buffer.
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}
