use core::sync::atomic::{AtomicUsize, Ordering};

static TICKS: AtomicUsize = AtomicUsize::new(0);

/// Increments the system tick counter. Called from IRQ 0.
pub fn increment_ticks() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn get_ticks() -> usize {
    TICKS.load(Ordering::Relaxed)
}

/// Returns the current timer tick count.
pub fn timer_ticks() -> usize {
    TICKS.load(Ordering::Relaxed)
}

/// Resets the timer tick counter to zero.
pub fn reset_timer_ticks() {
    TICKS.store(0, Ordering::Relaxed);
}

pub fn sleep(ms: usize) {
    let start_ticks = get_ticks();
    let ticks_to_wait = ms / 10;

    while get_ticks() < start_ticks + ticks_to_wait {
        // HLT suspends the CPU until the next interrupt fires, at which point
        // the interrupt handler runs and execution resumes on the instruction
        // following HLT.  This is more reliable than PAUSE for an idle wait
        // because it correctly handles the case where the interrupt arrives
        // just before we reach HLT (IF is already set so the CPU wakes
        // immediately).
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}
