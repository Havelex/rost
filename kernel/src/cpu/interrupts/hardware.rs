use crate::{
    arch::{Arch, Architecture},
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
            #[cfg(target_arch = "x86_64")]
            {
                let scancode =
                    unsafe { crate::arch::x86_64::asm::inb(KEYBOARD_DATA_PORT) };
                log_info!("[keyboard] scancode: {:#04x}", scancode);
            }
        }
        _ => {
            println!("Received hardware IRQ: {}", irq);
        }
    }

    <Arch as Architecture>::send_eoi(irq);
}
