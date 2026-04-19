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
            // Cannot log here — acquiring the console Mutex from an interrupt
            // handler risks deadlock on a single-CPU kernel (stored memory
            // constraint: interrupt handlers must never call print!/log_info!).
        }
    }

    <Arch as Architecture>::send_eoi(irq);
}
