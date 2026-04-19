use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// Raw scancode from the PS/2 controller, written by the keyboard IRQ handler.
static SCANCODE_BUFFER: AtomicU8 = AtomicU8::new(0);
/// Set to `true` by the IRQ handler when a new scancode is waiting to be read.
static SCANCODE_READY: AtomicBool = AtomicBool::new(false);

// ── Types ─────────────────────────────────────────────────────────────────────

/// A decoded key-press event.
pub struct KeyPress {
    /// Raw scancode received from the PS/2 controller.
    pub scancode: u8,
    /// Keycode (bits [6:0] of the scancode).
    pub keycode: u8,
    /// ASCII character, if this key maps to a printable character.
    pub ascii: Option<char>,
}

/// A keyboard event (press or release).
pub enum KeyEvent {
    /// A key was pressed; carries the decoded information.
    Pressed(KeyPress),
    /// A key was released; carries the raw scancode.
    Released(u8),
}

// ── Interrupt-safe buffer operations ─────────────────────────────────────────

/// Store a scancode in the global buffer.
///
/// Called **only** from the keyboard IRQ handler (IRQ 1).  Must not call any
/// function that acquires a mutex (e.g. `print!`).
pub fn push_scancode(scancode: u8) {
    SCANCODE_BUFFER.store(scancode, Ordering::Release);
    SCANCODE_READY.store(true, Ordering::Release);
}

// ── Detection helpers ─────────────────────────────────────────────────────────

/// Returns `true` if bit 7 of `scancode` is 0 (key pressed).
#[inline]
pub fn is_key_pressed(scancode: u8) -> bool {
    scancode & 0x80 == 0
}

/// Returns `true` if bit 7 of `scancode` is 1 (key released).
#[inline]
pub fn is_key_released(scancode: u8) -> bool {
    scancode & 0x80 != 0
}

// ── ASCII mapping ─────────────────────────────────────────────────────────────

/// Returns the ASCII character for a press scancode, or `None` for
/// non-printable / modifier keys.
pub fn get_ascii(scancode: u8) -> Option<char> {
    if (scancode as usize) < SCANCODE_TO_ASCII.len() {
        SCANCODE_TO_ASCII[scancode as usize]
    } else {
        None
    }
}

/// Print the character representation of `scancode` to the console.
/// Should only be called from non-interrupt context.
pub fn print_key(scancode: u8) {
    if let Some(c) = get_ascii(scancode) {
        crate::print!("{}", c);
    }
}

// ── Blocking wait ─────────────────────────────────────────────────────────────

/// Block until a key is **pressed** (not released).
///
/// Uses the `hlt` instruction to yield the CPU while waiting, so the timer
/// IRQ (and any other interrupt) will wake the CPU efficiently.
///
/// Returns a [`KeyPress`] describing the pressed key.
pub fn wait_for_keypress() -> KeyPress {
    loop {
        // Atomically consume the ready flag.  Using `swap` ensures we never
        // observe SCANCODE_READY = true but then miss the value because the
        // clear is not immediately visible to the interrupt handler.
        if SCANCODE_READY.swap(false, Ordering::AcqRel) {
            let sc = SCANCODE_BUFFER.load(Ordering::Acquire);

            if is_key_pressed(sc) {
                let keycode = sc & 0x7F;
                return KeyPress {
                    scancode: sc,
                    keycode,
                    ascii: get_ascii(sc),
                };
            }
            // Key release — ignore and keep waiting.
        }
        // Yield the CPU until the next interrupt.
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

// ── Macro ─────────────────────────────────────────────────────────────────────

/// Block until a key is pressed and return a [`KeyPress`].
///
/// Uses `hlt` internally to avoid busy-waiting.
///
/// # Example
/// ```
/// let key = wait_for_key!();
/// println!("Pressed keycode {:#04x}", key.keycode);
/// ```
#[macro_export]
macro_rules! wait_for_key {
    () => {
        $crate::keyboard::wait_for_keypress()
    };
}

// ── Scancode → ASCII table (US QWERTY) ────────────────────────────────────────
//
// Index = scancode (press, bit 7 = 0).  Entry is `None` for non-printable keys.
// Highest scancode in the standard set is 0x58; the table is padded to 0x80 so
// that any scancode can be used as an index without bounds issues.

const SCANCODE_TO_ASCII: [Option<char>; 0x80] = {
    let mut t: [Option<char>; 0x80] = [None; 0x80];

    // Numbers row
    t[0x02] = Some('1');
    t[0x03] = Some('2');
    t[0x04] = Some('3');
    t[0x05] = Some('4');
    t[0x06] = Some('5');
    t[0x07] = Some('6');
    t[0x08] = Some('7');
    t[0x09] = Some('8');
    t[0x0A] = Some('9');
    t[0x0B] = Some('0');

    // Top letter row (QWERTY)
    t[0x10] = Some('q');
    t[0x11] = Some('w');
    t[0x12] = Some('e');
    t[0x13] = Some('r');
    t[0x14] = Some('t');
    t[0x15] = Some('y');
    t[0x16] = Some('u');
    t[0x17] = Some('i');
    t[0x18] = Some('o');
    t[0x19] = Some('p');

    // Home row (ASDF)
    t[0x1E] = Some('a');
    t[0x1F] = Some('s');
    t[0x20] = Some('d');
    t[0x21] = Some('f');
    t[0x22] = Some('g');
    t[0x23] = Some('h');
    t[0x24] = Some('j');
    t[0x25] = Some('k');
    t[0x26] = Some('l');

    // Bottom row (ZXCV)
    t[0x2C] = Some('z');
    t[0x2D] = Some('x');
    t[0x2E] = Some('c');
    t[0x2F] = Some('v');
    t[0x30] = Some('b');
    t[0x31] = Some('n');
    t[0x32] = Some('m');

    // Special / whitespace keys
    t[0x0F] = Some('\t'); // Tab
    t[0x1C] = Some('\n'); // Enter
    t[0x39] = Some(' ');  // Space

    t
};
