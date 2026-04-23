//! Logging macros.
//!
//! Each macro formats its arguments and delegates to [`crate::logger::log`]
//! with the appropriate [`LogLevel`](crate::logger::LogLevel).
//!
//! # Safety
//! All macros ultimately call [`print!`] which acquires the console mutex.
//! They **must not** be called from interrupt handlers.

/// Log a message at the [`Ok`](crate::logger::LogLevel::Ok) level.
#[macro_export]
macro_rules! log_ok {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Ok, format_args!($($arg)*)));
}

/// Log a message at the [`Info`](crate::logger::LogLevel::Info) level.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Info, format_args!($($arg)*)));
}

/// Log a message at the [`Warn`](crate::logger::LogLevel::Warn) level.
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Warn, format_args!($($arg)*)));
}

/// Log a message at the [`Error`](crate::logger::LogLevel::Error) level.
#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Error, format_args!($($arg)*)));
}

/// Log a message at the [`Debug`](crate::logger::LogLevel::Debug) level.
#[macro_export]
macro_rules! log_dbug {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Debug, format_args!($($arg)*)));
}
