#[macro_export]
macro_rules! log_ok {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Ok, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Info, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Warn, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Error, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_dbug {
    ($($arg:tt)*) => ($crate::logger::log($crate::logger::LogLevel::Debug, format_args!($($arg)*)));
}
