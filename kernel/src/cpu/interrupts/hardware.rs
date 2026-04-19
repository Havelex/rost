use crate::{
    arch::{Arch, Architecture},
    keyboard,
    time::increment_ticks,
};

/// PS/2 keyboard data port — read to obtain the scancode.
const KEYBOARD_DATA_PORT: u16 = 0x60;
/// PS/2 keyboard status/command port.
#[allow(dead_code)]
const KEYBOARD_STATUS_PORT: u16 = 0x64;

pub fn handle_hardware_interrupt(irq: u8) {
    match irq {
        0 => {
            increment_ticks();
        }
        1 => {
            // Read the scancode from the PS/2 data port and push it into the
            // keyboard buffer.  Must NOT call print!/log_* here — those macros
            // acquire a spin::Mutex and will deadlock if the main thread holds
            // the console lock when this IRQ fires.
            let scancode = unsafe { <Arch as Architecture>::read_port_u8(KEYBOARD_DATA_PORT) };
            keyboard::push_scancode(scancode);
        }
        _ => {}
    }

    <Arch as Architecture>::send_eoi(irq);
}
