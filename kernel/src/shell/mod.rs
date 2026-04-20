mod command;
mod commands;
mod input_buffer;

use command::CommandResult;
use input_buffer::InputBuffer;

/// Run the interactive shell until a command requests a halt.
///
/// Reads key-presses from the keyboard, echoes printable characters, handles
/// backspace, and executes commands when the user presses Enter.
///
/// Returns normally when the user runs a command that signals
/// [`CommandResult::Halt`] (e.g. `halt`).  The caller is then responsible for
/// halting the CPU.
pub fn run() {
    let mut buf = InputBuffer::new();
    crate::print!("> ");

    loop {
        let key = crate::wait_for_key!();

        match key.ascii {
            // Enter – execute the current line.
            Some('\n') => {
                crate::println!();
                let result = command::dispatch(commands::COMMANDS, buf.as_str().trim());
                buf.clear();
                if let CommandResult::Halt = result {
                    break;
                }
                crate::print!("> ");
            }

            // Backspace – remove the last character from the buffer and
            // erase it from the screen.
            Some('\x08') => {
                if buf.backspace() {
                    crate::print!("\x08");
                }
            }

            // Printable character – add to buffer and echo.
            Some(c) if !c.is_control() => {
                if buf.push(c) {
                    crate::print!("{}", c);
                }
            }

            _ => {}
        }
    }
}
