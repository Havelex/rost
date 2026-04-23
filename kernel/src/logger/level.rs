//! Log levels and their display/colour mappings.
//!
//! [`LogLevel`] is used by the [`log`](crate::logger::log) function to prefix
//! each message with a colour-coded tag.

/// Severity level for a log message.
pub enum LogLevel {
    /// A successful step completed without errors.
    Ok,
    /// Informational message about ongoing progress.
    Info,
    /// Non-fatal warning that may indicate a problem.
    Warn,
    /// Recoverable error condition.
    Error,
    /// Verbose debug output (only meaningful with a debug build).
    Debug,
}

impl LogLevel {
    /// Return the short human-readable tag string for this level.
    ///
    /// # Returns
    /// A fixed-width string such as `"  OK  "` or `" ERROR"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Ok => "  OK  ",
            LogLevel::Info => " INFO ",
            LogLevel::Warn => " WARN ",
            LogLevel::Error => " ERROR",
            LogLevel::Debug => " DEBUG",
        }
    }

    /// Return the ANSI SGR escape code that sets the foreground colour for this level.
    ///
    /// # Returns
    /// A string like `"\x1b[92m"` suitable for use before the tag text.
    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Ok => "\x1b[92m",    // Green
            LogLevel::Info => "\x1b[96m",  // Cyan
            LogLevel::Warn => "\x1b[93m",  // Yellow
            LogLevel::Error => "\x1b[91m", // Red
            LogLevel::Debug => "\x1b[98m", // Orange
        }
    }
}
