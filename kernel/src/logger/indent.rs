use core::sync::atomic::{AtomicUsize, Ordering};

pub(super) static INDENT_LEVEL: AtomicUsize = AtomicUsize::new(0);

pub fn push_indent() {
    INDENT_LEVEL.fetch_add(1, Ordering::SeqCst);
}
pub fn pop_indent() {
    INDENT_LEVEL.fetch_sub(1, Ordering::SeqCst);
}

pub(super) fn print_indent() {
    let level = INDENT_LEVEL.load(Ordering::SeqCst);
    for _ in 0..level {
        crate::print!("  "); // Two spaces per level
    }
}
