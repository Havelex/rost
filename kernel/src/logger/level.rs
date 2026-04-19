pub enum LogLevel {
    Ok,
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Ok => "  OK  ",
            LogLevel::Info => " INFO ",
            LogLevel::Warn => " WARN ",
            LogLevel::Error => " ERROR",
            LogLevel::Debug => " DEBUG",
        }
    }

    // Assuming you have a VGA color mapping or ANSI support
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
