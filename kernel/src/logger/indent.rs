//! Log-level indentation utilities.
//!
//! Provides a global [`INDENT_LEVEL`] counter that the logger uses to emit
//! leading whitespace proportional to the current boot-stage nesting depth.
//! Call [`push_indent`] before entering a nested init step and [`pop_indent`]
//! after it completes.

use core::sync::atomic::{AtomicUsize, Ordering};

pub(super) static INDENT_LEVEL: AtomicUsize = AtomicUsize::new(0);

/// Increase the log indent level by one step.
pub fn push_indent() {
    INDENT_LEVEL.fetch_add(1, Ordering::SeqCst);
}

/// Decrease the log indent level by one step.
pub fn pop_indent() {
    INDENT_LEVEL.fetch_sub(1, Ordering::SeqCst);
}

pub(super) fn print_indent() {
    let level = INDENT_LEVEL.load(Ordering::SeqCst);
    for _ in 0..level {
        crate::print!("  "); // Two spaces per level
    }
}
