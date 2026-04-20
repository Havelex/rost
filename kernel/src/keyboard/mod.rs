use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// Raw scancode from the PS/2 controller, written by the keyboard IRQ handler.
static SCANCODE_BUFFER: AtomicU8 = AtomicU8::new(0);
/// Set to `true` by the IRQ handler when a new scancode is waiting to be read.
static SCANCODE_READY: AtomicBool = AtomicBool::new(false);
/// When `false`, the IRQ handler discards all incoming scancodes.
static KEYBOARD_ENABLED: AtomicBool = AtomicBool::new(true);

// ── Modifier state (updated in push_scancode, read in wait_for_keypress) ─────

/// Left Shift is currently held.
static LSHIFT_HELD: AtomicBool = AtomicBool::new(false);
/// Right Shift is currently held.
static RSHIFT_HELD: AtomicBool = AtomicBool::new(false);
/// AltGr (Right Alt / E0-Alt) is currently held.
static ALTGR_HELD: AtomicBool = AtomicBool::new(false);
/// The previous byte from the PS/2 controller was the E0 extended prefix.
static E0_PREFIX_SEEN: AtomicBool = AtomicBool::new(false);

// ── Layout selection ──────────────────────────────────────────────────────────

/// Active keyboard layout: 0 = EN (US QWERTY), 1 = DE (German QWERTZ).
static LAYOUT: AtomicU8 = AtomicU8::new(0);

/// Supported keyboard layouts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    En = 0,
    De = 1,
}

/// Set the active keyboard layout.
pub fn set_layout(layout: Layout) {
    LAYOUT.store(layout as u8, Ordering::Release);
}

/// Get the active keyboard layout.
pub fn get_layout() -> Layout {
    match LAYOUT.load(Ordering::Acquire) {
        1 => Layout::De,
        _ => Layout::En,
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// Active modifier keys at the time of a key-press.
#[derive(Clone, Copy)]
pub struct Modifiers {
    pub shift: bool,
    pub altgr: bool,
}

/// A decoded key-press event.
pub struct KeyPress {
    /// Raw scancode received from the PS/2 controller.
    pub scancode: u8,
    /// Keycode (bits [6:0] of the scancode).
    pub keycode: u8,
    /// ASCII character after applying active modifiers, if printable.
    pub ascii: Option<char>,
    /// Modifier keys that were active when this key was pressed.
    pub modifiers: Modifiers,
}

/// A keyboard event (press or release).
pub enum KeyEvent {
    /// A key was pressed; carries the decoded information.
    Pressed(KeyPress),
    /// A key was released; carries the raw scancode.
    Released(u8),
}

// ── Interrupt-safe buffer operations ─────────────────────────────────────────

/// Store a scancode in the global buffer and update modifier state.
///
/// Called **only** from the keyboard IRQ handler (IRQ 1).  Must not call any
/// function that acquires a mutex (e.g. `print!`).
/// Scancodes are silently dropped when the keyboard is disabled.
pub fn push_scancode(scancode: u8) {
    if !KEYBOARD_ENABLED.load(Ordering::Acquire) {
        return;
    }

    // Handle E0 extended prefix (AltGr, arrow keys, etc.).
    if scancode == 0xE0 {
        E0_PREFIX_SEEN.store(true, Ordering::Release);
        return;
    }

    let extended = E0_PREFIX_SEEN.swap(false, Ordering::AcqRel);

    if extended {
        // Only extended scancode we care about is AltGr (E0 + 0x38 / 0xB8).
        match scancode {
            0x38 => ALTGR_HELD.store(true, Ordering::Release),
            0xB8 => ALTGR_HELD.store(false, Ordering::Release),
            _ => {}
        }
        // Extended scancodes are not queued as regular keypresses.
        return;
    }

    // Track modifier key presses and releases (non-extended path).
    match scancode {
        0x2A => { LSHIFT_HELD.store(true, Ordering::Release); return; }  // L-Shift press
        0xAA => { LSHIFT_HELD.store(false, Ordering::Release); return; } // L-Shift release
        0x36 => { RSHIFT_HELD.store(true, Ordering::Release); return; }  // R-Shift press
        0xB6 => { RSHIFT_HELD.store(false, Ordering::Release); return; } // R-Shift release
        _ => {}
    }

    // Queue the scancode for the consumer (press *or* release for all other keys).
    SCANCODE_BUFFER.store(scancode, Ordering::Release);
    SCANCODE_READY.store(true, Ordering::Release);
}

// ── Enable / disable ──────────────────────────────────────────────────────────

/// Enable keyboard input. Scancodes from IRQ 1 will be stored in the buffer.
pub fn enable() {
    KEYBOARD_ENABLED.store(true, Ordering::Release);
}

/// Disable keyboard input. Scancodes from IRQ 1 are discarded until [`enable`]
/// is called. Any scancode already in the buffer is left untouched.
pub fn disable() {
    KEYBOARD_ENABLED.store(false, Ordering::Release);
}

/// Returns `true` if the keyboard is currently enabled.
pub fn is_enabled() -> bool {
    KEYBOARD_ENABLED.load(Ordering::Acquire)
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

/// Returns the ASCII character for a press scancode given the active modifiers,
/// or `None` for non-printable / modifier keys.
pub fn get_ascii(scancode: u8) -> Option<char> {
    get_ascii_with_mods(scancode, false, false)
}

/// Returns the character for `scancode` applying `shift` and `altgr`, using
/// the currently active keyboard layout.
pub fn get_ascii_with_mods(scancode: u8, shift: bool, altgr: bool) -> Option<char> {
    let (base, shifted, altgr_table) = match get_layout() {
        Layout::En => (
            &SCANCODE_TO_ASCII_EN,
            &SCANCODE_TO_ASCII_EN_SHIFTED,
            &SCANCODE_TO_ASCII_EN_ALTGR,
        ),
        Layout::De => (
            &SCANCODE_TO_ASCII_DE,
            &SCANCODE_TO_ASCII_DE_SHIFTED,
            &SCANCODE_TO_ASCII_DE_ALTGR,
        ),
    };

    let idx = scancode as usize;
    if altgr && idx < altgr_table.len() {
        if let Some(c) = altgr_table[idx] {
            return Some(c);
        }
    }
    if shift && idx < shifted.len() {
        return shifted[idx];
    }
    if idx < base.len() {
        base[idx]
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

/// Read the current modifier state.
fn current_modifiers() -> Modifiers {
    Modifiers {
        shift: LSHIFT_HELD.load(Ordering::Acquire) || RSHIFT_HELD.load(Ordering::Acquire),
        altgr: ALTGR_HELD.load(Ordering::Acquire),
    }
}

/// Attempt to read one key-press from the buffer without blocking.
///
/// Returns `Some(KeyPress)` if a scancode was waiting and it represents a key
/// press, or `None` if the buffer was empty or the waiting event was a release.
pub fn try_read_keypress() -> Option<KeyPress> {
    if SCANCODE_READY.swap(false, Ordering::AcqRel) {
        let sc = SCANCODE_BUFFER.load(Ordering::Acquire);
        if is_key_pressed(sc) {
            let keycode = sc & 0x7F;
            let mods = current_modifiers();
            return Some(KeyPress {
                scancode: sc,
                keycode,
                ascii: get_ascii_with_mods(sc, mods.shift, mods.altgr),
                modifiers: mods,
            });
        }
        // Release event — swallowed; caller gets None.
    }
    None
}

/// Block until a key is **pressed** (not released).
///
/// Uses the `hlt` instruction to yield the CPU while waiting, so the timer
/// IRQ (and any other interrupt) will wake the CPU efficiently.
///
/// Returns a [`KeyPress`] describing the pressed key.
pub fn wait_for_keypress() -> KeyPress {
    loop {
        if let Some(kp) = try_read_keypress() {
            return kp;
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

// ── Scancode → character tables ───────────────────────────────────────────────
//
// Index = scancode (press, bit 7 = 0).  Entry is `None` for non-printable keys.
// Tables are padded to 0x80 so any standard scancode can be used as an index.

// ── EN (US QWERTY) ────────────────────────────────────────────────────────────

const SCANCODE_TO_ASCII_EN: [Option<char>; 0x80] = {
    let mut t: [Option<char>; 0x80] = [None; 0x80];

    t[0x02] = Some('1');  t[0x03] = Some('2');  t[0x04] = Some('3');
    t[0x05] = Some('4');  t[0x06] = Some('5');  t[0x07] = Some('6');
    t[0x08] = Some('7');  t[0x09] = Some('8');  t[0x0A] = Some('9');
    t[0x0B] = Some('0');  t[0x0C] = Some('-');  t[0x0D] = Some('=');

    t[0x10] = Some('q');  t[0x11] = Some('w');  t[0x12] = Some('e');
    t[0x13] = Some('r');  t[0x14] = Some('t');  t[0x15] = Some('y');
    t[0x16] = Some('u');  t[0x17] = Some('i');  t[0x18] = Some('o');
    t[0x19] = Some('p');  t[0x1A] = Some('[');  t[0x1B] = Some(']');

    t[0x1E] = Some('a');  t[0x1F] = Some('s');  t[0x20] = Some('d');
    t[0x21] = Some('f');  t[0x22] = Some('g');  t[0x23] = Some('h');
    t[0x24] = Some('j');  t[0x25] = Some('k');  t[0x26] = Some('l');
    t[0x27] = Some(';');  t[0x28] = Some('\''); t[0x2B] = Some('\\');

    t[0x2C] = Some('z');  t[0x2D] = Some('x');  t[0x2E] = Some('c');
    t[0x2F] = Some('v');  t[0x30] = Some('b');  t[0x31] = Some('n');
    t[0x32] = Some('m');  t[0x33] = Some(',');  t[0x34] = Some('.');
    t[0x35] = Some('/');

    t[0x0E] = Some('\x08'); // Backspace
    t[0x0F] = Some('\t');   // Tab
    t[0x1C] = Some('\n');   // Enter
    t[0x39] = Some(' ');    // Space

    t
};

const SCANCODE_TO_ASCII_EN_SHIFTED: [Option<char>; 0x80] = {
    let mut t: [Option<char>; 0x80] = [None; 0x80];

    t[0x02] = Some('!');  t[0x03] = Some('@');  t[0x04] = Some('#');
    t[0x05] = Some('$');  t[0x06] = Some('%');  t[0x07] = Some('^');
    t[0x08] = Some('&');  t[0x09] = Some('*');  t[0x0A] = Some('(');
    t[0x0B] = Some(')');  t[0x0C] = Some('_');  t[0x0D] = Some('+');

    t[0x10] = Some('Q');  t[0x11] = Some('W');  t[0x12] = Some('E');
    t[0x13] = Some('R');  t[0x14] = Some('T');  t[0x15] = Some('Y');
    t[0x16] = Some('U');  t[0x17] = Some('I');  t[0x18] = Some('O');
    t[0x19] = Some('P');  t[0x1A] = Some('{');  t[0x1B] = Some('}');

    t[0x1E] = Some('A');  t[0x1F] = Some('S');  t[0x20] = Some('D');
    t[0x21] = Some('F');  t[0x22] = Some('G');  t[0x23] = Some('H');
    t[0x24] = Some('J');  t[0x25] = Some('K');  t[0x26] = Some('L');
    t[0x27] = Some(':');  t[0x28] = Some('"');  t[0x2B] = Some('|');

    t[0x2C] = Some('Z');  t[0x2D] = Some('X');  t[0x2E] = Some('C');
    t[0x2F] = Some('V');  t[0x30] = Some('B');  t[0x31] = Some('N');
    t[0x32] = Some('M');  t[0x33] = Some('<');  t[0x34] = Some('>');
    t[0x35] = Some('?');

    t[0x0E] = Some('\x08');
    t[0x0F] = Some('\t');
    t[0x1C] = Some('\n');
    t[0x39] = Some(' ');

    t
};

const SCANCODE_TO_ASCII_EN_ALTGR: [Option<char>; 0x80] = {
    let mut t: [Option<char>; 0x80] = [None; 0x80];

    t[0x03] = Some('@');   // AltGr+2 → @
    t[0x04] = Some('#');   // AltGr+3 → #
    t[0x05] = Some('$');   // AltGr+4 → $
    t[0x07] = Some('^');   // AltGr+6 → ^
    t[0x10] = Some('@');   // AltGr+Q → @
    t[0x12] = Some('€');   // AltGr+E → €
    t[0x1A] = Some('[');
    t[0x1B] = Some(']');
    t[0x2B] = Some('\\');
    t[0x28] = Some('`');
    t[0x35] = Some('~');
    t[0x1E] = Some('{');
    t[0x27] = Some('}');

    t
};

// ── DE (German QWERTZ) ────────────────────────────────────────────────────────
//
// Physical key differences from US QWERTY:
//   • y ↔ z swapped
//   • ü  at [  (0x1A)
//   • +  at ]  (0x1B)
//   • ö  at ;  (0x27)
//   • ä  at '  (0x28)
//   • #  at \  (0x2B)
//   • ß  at -  (0x0C)
//   • -  at /  (0x35)
//   • ´  at =  (0x0D)  — represented as backtick here (no dead key support)

const SCANCODE_TO_ASCII_DE: [Option<char>; 0x80] = {
    let mut t: [Option<char>; 0x80] = [None; 0x80];

    t[0x02] = Some('1');  t[0x03] = Some('2');  t[0x04] = Some('3');
    t[0x05] = Some('4');  t[0x06] = Some('5');  t[0x07] = Some('6');
    t[0x08] = Some('7');  t[0x09] = Some('8');  t[0x0A] = Some('9');
    t[0x0B] = Some('0');
    t[0x0C] = Some('ß');
    t[0x0D] = Some('`');    // ´ → backtick (dead-key not supported)

    t[0x10] = Some('q');  t[0x11] = Some('w');  t[0x12] = Some('e');
    t[0x13] = Some('r');  t[0x14] = Some('t');
    t[0x15] = Some('z');  // z is where y sits on QWERTY
    t[0x16] = Some('u');  t[0x17] = Some('i');  t[0x18] = Some('o');
    t[0x19] = Some('p');
    t[0x1A] = Some('ü');
    t[0x1B] = Some('+');

    t[0x1E] = Some('a');  t[0x1F] = Some('s');  t[0x20] = Some('d');
    t[0x21] = Some('f');  t[0x22] = Some('g');  t[0x23] = Some('h');
    t[0x24] = Some('j');  t[0x25] = Some('k');  t[0x26] = Some('l');
    t[0x27] = Some('ö');
    t[0x28] = Some('ä');
    t[0x2B] = Some('#');

    t[0x2C] = Some('y');  // y is where z sits on QWERTY
    t[0x2D] = Some('x');  t[0x2E] = Some('c');  t[0x2F] = Some('v');
    t[0x30] = Some('b');  t[0x31] = Some('n');  t[0x32] = Some('m');
    t[0x33] = Some(',');  t[0x34] = Some('.');
    t[0x35] = Some('-');
    t[0x56] = Some('<');  // Extra key (left of Y on DE keyboard)

    t[0x0E] = Some('\x08'); // Backspace
    t[0x0F] = Some('\t');   // Tab
    t[0x1C] = Some('\n');   // Enter
    t[0x39] = Some(' ');    // Space

    t
};

const SCANCODE_TO_ASCII_DE_SHIFTED: [Option<char>; 0x80] = {
    let mut t: [Option<char>; 0x80] = [None; 0x80];

    t[0x02] = Some('!');
    t[0x03] = Some('"');
    t[0x04] = Some('§');
    t[0x05] = Some('$');
    t[0x06] = Some('%');
    t[0x07] = Some('&');
    t[0x08] = Some('/');
    t[0x09] = Some('(');
    t[0x0A] = Some(')');
    t[0x0B] = Some('=');
    t[0x0C] = Some('?');
    t[0x0D] = Some('`');

    t[0x10] = Some('Q');  t[0x11] = Some('W');  t[0x12] = Some('E');
    t[0x13] = Some('R');  t[0x14] = Some('T');
    t[0x15] = Some('Z');
    t[0x16] = Some('U');  t[0x17] = Some('I');  t[0x18] = Some('O');
    t[0x19] = Some('P');
    t[0x1A] = Some('Ü');
    t[0x1B] = Some('*');

    t[0x1E] = Some('A');  t[0x1F] = Some('S');  t[0x20] = Some('D');
    t[0x21] = Some('F');  t[0x22] = Some('G');  t[0x23] = Some('H');
    t[0x24] = Some('J');  t[0x25] = Some('K');  t[0x26] = Some('L');
    t[0x27] = Some('Ö');
    t[0x28] = Some('Ä');
    t[0x2B] = Some('\'');

    t[0x2C] = Some('Y');
    t[0x2D] = Some('X');  t[0x2E] = Some('C');  t[0x2F] = Some('V');
    t[0x30] = Some('B');  t[0x31] = Some('N');  t[0x32] = Some('M');
    t[0x33] = Some(';');  t[0x34] = Some(':');
    t[0x35] = Some('_');
    t[0x56] = Some('>');  // Extra key shifted

    t[0x0E] = Some('\x08');
    t[0x0F] = Some('\t');
    t[0x1C] = Some('\n');
    t[0x39] = Some(' ');

    t
};

const SCANCODE_TO_ASCII_DE_ALTGR: [Option<char>; 0x80] = {
    let mut t: [Option<char>; 0x80] = [None; 0x80];

    t[0x03] = Some('²'); // ² (AltGr+2)
    t[0x04] = Some('³'); // ³ (AltGr+3)
    t[0x08] = Some('{');    // AltGr+7
    t[0x09] = Some('[');    // AltGr+8
    t[0x0A] = Some(']');    // AltGr+9
    t[0x0B] = Some('}');    // AltGr+0
    t[0x0C] = Some('\\');   // AltGr+ß → backslash
    t[0x12] = Some('€');    // AltGr+E → €
    t[0x56] = Some('|');    // AltGr+< → | (the extra key left of Z on DE keyboards)
    t[0x1B] = Some('~');    // AltGr++ → ~
    t[0x10] = Some('@');    // AltGr+Q → @

    t
};

