use crate::{
    arch::{Arch, Architecture},
    time::increment_ticks,
};

pub fn handle_hardware_interrupt(irq: u8) {
    match irq {
        0 => {
            increment_ticks();
        }
        1 => { /* Keyboard logic */ }
        _ => {
            println!("Received hardware IRQ: {}", irq);
        }
    }

    <Arch as Architecture>::send_eoi(irq);
}
