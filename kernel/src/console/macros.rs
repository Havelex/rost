//! Console output macros.
//!
//! These macros forward formatted output to the global [`Console`](crate::console::writer::Console)
//! instance protected by a [`spin::Mutex`].
//!
//! # Safety
//! All macros acquire the console mutex.  They **must not** be called from
//! interrupt handlers — doing so will deadlock on a single-CPU system if the
//! main thread already holds the lock.

/// Print a formatted string to the console without a trailing newline.
///
/// Acquires the global console mutex for the duration of the write.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::console::writer::console().lock()
            .write_fmt(format_args!($($arg)*)).unwrap();
    }};
}

/// Print a formatted string to the console followed by a newline.
///
/// Acquires the global console mutex for the duration of the write.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {{
        $crate::print!("{}\n", format_args!($($arg)*));
    }};
}

/// Clear the console screen.
///
/// Fills the framebuffer with the current background colour and resets the
/// cursor to the top-left corner.
#[macro_export]
macro_rules! cls {
    () => {
        $crate::console::writer::console().lock().clear()
    };
}

/// Draw the text cursor at the current cursor position.
#[macro_export]
macro_rules! draw_cursor {
    () => {
        $crate::console::writer::console().lock().draw_cursor()
    };
}

/// Erase the text cursor at the current cursor position.
#[macro_export]
macro_rules! erase_cursor {
    () => {
        $crate::console::writer::console().lock().erase_cursor()
    };
}
