use core::sync::atomic::{AtomicU8, Ordering};

use crate::{
    arch::{Arch, Architecture},
    cpu::Cpu,
    time::increment_ticks,
};

/// PS/2 keyboard data port — read to obtain the scancode.
#[allow(dead_code)]
const KEYBOARD_DATA_PORT: u16 = 0x60;
/// PS/2 keyboard status/command port.
#[allow(dead_code)]
const KEYBOARD_STATUS_PORT: u16 = 0x64;

/// One-slot scancode buffer.  Zero means "empty"; PS/2 Set 1 scancodes never
/// produce `0x00` for a real keypress, so it is a safe sentinel value.
static SCANCODE: AtomicU8 = AtomicU8::new(0);

pub fn handle_hardware_interrupt(irq: u8) {
    match irq {
        0 => {
            increment_ticks();
        }
        1 => {
            let code = <Arch as Architecture>::read_keyboard_scancode();
            if code != 0 {
                // Overwrite any unread scancode with the most-recent one.
                SCANCODE.store(code, Ordering::Release);
            }
        }
        _ => {}
    }

    <Arch as Architecture>::send_eoi(irq);
}

/// Block until a keyboard key is pressed, then return its PS/2 Set 1 scancode.
///
/// Uses the `hlt` instruction to yield the CPU while waiting, avoiding a
/// pure busy-wait loop.
///
/// # Note on timing
/// A keyboard interrupt that fires in the narrow window between the empty-check
/// and `hlt` is not lost — the scancode is stored in the atomic and will be
/// returned on the very next loop iteration, which is triggered at the latest
/// by the next PIT tick (≤10 ms).
pub fn wait_for_key() -> u8 {
    loop {
        let code = SCANCODE.swap(0, Ordering::AcqRel);
        if code != 0 {
            return code;
        }
        <Arch as Architecture>::Cpu::wait_for_interrupt();
    }
}
