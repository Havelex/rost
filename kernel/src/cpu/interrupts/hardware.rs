use core::sync::atomic::{AtomicU64, Ordering};

use crate::{
    arch::{Arch, Architecture},
    time::increment_ticks,
};

static IRQ0_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn handle_hardware_interrupt(irq: u8) {
    match irq {
        0 => {
            increment_ticks();
            let count = IRQ0_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 100 == 0 {
                log_debug!("[timer] IRQ0 received, ticks={}", crate::time::timer_ticks());
            }
        }
        1 => { /* Keyboard logic */ }
        _ => {
            println!("Received hardware IRQ: {}", irq);
        }
    }

    <Arch as Architecture>::send_eoi(irq);
}
