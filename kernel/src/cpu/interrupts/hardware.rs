use crate::{
    arch::{Arch, Architecture},
    time::{get_ticks, increment_ticks},
};

pub fn handle_hardware_interrupt(irq: u8) {
    match irq {
        0 => {
            increment_ticks();
            let ticks = get_ticks();
            if ticks % 100 == 0 {
                log_info!("[timer] tick={}", ticks);
            }
        }
        1 => { /* Keyboard logic */ }
        _ => {
            println!("Received hardware IRQ: {}", irq);
        }
    }

    <Arch as Architecture>::send_eoi(irq);
}
