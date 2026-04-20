#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::console::writer::console().lock()
            .write_fmt(format_args!($($arg)*)).unwrap();
    }};
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {{
        $crate::print!("{}\n", format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! cls {
    () => {
        $crate::console::writer::console().lock().clear()
    };
}

#[macro_export]
macro_rules! draw_cursor {
    () => {
        $crate::console::writer::console().lock().draw_cursor()
    };
}

#[macro_export]
macro_rules! erase_cursor {
    () => {
        $crate::console::writer::console().lock().erase_cursor()
    };
}
