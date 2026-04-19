pub enum LogLevel {
    Ok,
    Info,
    Debug,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Ok => "  OK  ",
            LogLevel::Info => " INFO ",
            LogLevel::Debug => " DEBUG",
            LogLevel::Warn => " WARN ",
            LogLevel::Error => " ERROR",
        }
    }

    // Assuming you have a VGA color mapping or ANSI support
    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Ok => "\x1b[32m",    // Green
            LogLevel::Info => "\x1b[36m",  // Cyan
            LogLevel::Debug => "\x1b[35m", // Magenta
            LogLevel::Warn => "\x1b[33m",  // Yellow
            LogLevel::Error => "\x1b[31m", // Red
        }
    }
}
