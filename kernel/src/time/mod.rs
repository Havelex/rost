//! Timer module for the operating system. Provides functionality for tracking system ticks and
//! sleeping.

use core::sync::atomic::{AtomicUsize, Ordering};

static TICKS: AtomicUsize = AtomicUsize::new(0);

/// Increments the system tick counter. Called from IRQ 0.
pub fn increment_ticks() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn get_ticks() -> usize {
    TICKS.load(Ordering::Relaxed)
}

/// Retrieves the current tick count. This is a wrapper around the atomic load operation.
///
/// # Returns
/// - The current number of ticks since the system started.
#[allow(dead_code)]
pub fn timer_ticks() -> usize {
    TICKS.load(Ordering::Relaxed)
}

/// Resets the timer tick counter to zero.
#[allow(dead_code)]
pub fn reset_timer_ticks() {
    TICKS.store(0, Ordering::Relaxed);
}

/// Sleeps for the specified number of milliseconds by busy-waiting on the tick count.
///
/// # Parameters
/// - `ms`: The number of milliseconds to sleep.
pub fn sleep(ms: usize) {
    let start_ticks = get_ticks();
    let ticks_to_wait = ms / 10;

    while get_ticks() < start_ticks + ticks_to_wait {
        core::hint::spin_loop();
    }
}
