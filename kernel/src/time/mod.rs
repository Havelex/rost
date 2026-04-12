use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

/// Increments the system tick counter. Called from IRQ 0.
pub fn increment_ticks() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
