use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::{
    arch::{Arch, Architecture},
    time::increment_ticks,
};

/// Set to `true` the first time IRQ0 fires, so we can emit a one-shot
/// diagnostic without printing on every tick (which would risk the console
/// mutex being held by the main thread).
static IRQ0_FIRST_SEEN: AtomicBool = AtomicBool::new(false);

/// Last tick value at which we emitted a periodic diagnostic.
static IRQ0_LAST_LOGGED: AtomicUsize = AtomicUsize::new(0);

pub fn handle_hardware_interrupt(irq: u8) {
    match irq {
        0 => {
            increment_ticks();

            // Emit a one-shot diagnostic the very first time IRQ0 arrives.
            if !IRQ0_FIRST_SEEN.swap(true, Ordering::Relaxed) {
                // DIAGNOSTIC NOTE: Calling println! from an interrupt handler risks
                // a single-CPU deadlock if the main thread holds the console spin-lock
                // when the IRQ fires (the ISR would spin on the mutex forever).
                // This print is guarded to run at most once.  At that point the main
                // thread is either in sleep() (a busy-wait spin with no lock held) or
                // still in the early boot sequence where the console is idle.
                // If the system hangs here, it means IRQ0 fired while the main thread
                // was holding the console lock; remove this print to recover.
                crate::println!("[irq0] first timer interrupt received");
            }

            // Periodic diagnostic: log tick count every 10 ticks.
            let ticks = crate::time::get_ticks();
            let last = IRQ0_LAST_LOGGED.load(Ordering::Relaxed);
            if ticks >= last + 10 {
                IRQ0_LAST_LOGGED.store(ticks, Ordering::Relaxed);
                crate::println!(
                    "[irq0] tick={} (EOI via {})",
                    ticks,
                    <Arch as Architecture>::active_controller()
                );
            }
        }
        1 => { /* Keyboard logic */ }
        _ => {
            println!("Received hardware IRQ: {}", irq);
        }
    }

    <Arch as Architecture>::send_eoi(irq);
}
