pub mod indent;
pub mod level;
#[macro_use]
pub mod macros;

pub use self::level::LogLevel;
use crate::logger::indent::print_indent;

pub fn log(level: LogLevel, message: core::fmt::Arguments) {
    let reset = "\x1b[0m";
    // 1. Print the status tag
    crate::print!("[{}{}{}] ", level.color_code(), level.as_str(), reset);

    // 2. Print the indentation
    print_indent();

    // 3. Print the actual message
    crate::println!("{}", message);
}
