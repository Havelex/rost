//! Kernel logger.
//!
//! Provides the [`log`] function that formats and writes a colour-coded
//! log line to the console, and re-exports the [`LogLevel`] enum.
//! Use the [`log_ok!`], [`log_info!`], [`log_warn!`], [`log_err!`], and
//! [`log_dbug!`] macros rather than calling [`log`] directly.

pub mod indent;
/// Log-level enum and formatting helpers.
pub mod level;
#[macro_use]
/// Logging macros (`log_info!`, `log_warn!`, etc.).
pub mod macros;

pub use self::level::LogLevel;
use crate::logger::indent::print_indent;

/// Write a single log line to the console.
///
/// Prints `[<color><level><reset>] <indent><message>\n`.
///
/// # Parameters
/// - `level`: Severity of the message, controls the colour and tag label.
/// - `message`: Pre-formatted arguments to display after the tag and indent.
pub fn log(level: LogLevel, message: core::fmt::Arguments) {
    let reset = "\x1b[0m";
    // 1. Print the status tag
    crate::print!("[{}{}{}] ", level.color_code(), level.as_str(), reset);

    // 2. Print the indentation
    print_indent();

    // 3. Print the actual message
    crate::println!("{}", message);
}
