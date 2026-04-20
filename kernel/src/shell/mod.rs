mod command;
mod commands;
mod input_buffer;

use command::CommandResult;
use input_buffer::InputBuffer;

/// Blink period: number of timer ticks between cursor toggles.
/// The PIT fires at ~100 Hz (10 ms / tick), so 75 ticks ≈ 750 ms.
const BLINK_TICKS: usize = 75;

/// Wait for the next key-press while blinking the cursor at the current
/// console position.
///
/// The cursor is always hidden when this function returns so the caller can
/// write a character without leaving cursor artefacts.
fn wait_key_blink() -> crate::keyboard::KeyPress {
    let mut visible = true;
    draw_cursor!();
    let mut last_blink = crate::time::get_ticks();

    loop {
        if let Some(key) = crate::keyboard::try_read_keypress() {
            if visible {
                erase_cursor!();
            }
            return key;
        }

        let now = crate::time::get_ticks();
        if now.wrapping_sub(last_blink) >= BLINK_TICKS {
            last_blink = now;
            if visible {
                erase_cursor!();
                visible = false;
            } else {
                draw_cursor!();
                visible = true;
            }
        }

        // Yield CPU until the next interrupt (timer or keyboard).
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

/// Run the interactive shell until a command requests a halt.
///
/// Reads key-presses from the keyboard, echoes printable characters, handles
/// backspace, and executes commands when the user presses Enter.
///
/// Returns normally when the user runs a command that signals
/// [`CommandResult::Halt`] (e.g. `halt`).  The caller is then responsible for
/// halting the CPU.
/// Print the shell prompt: `<cwd> > `.
fn print_prompt() {
    let vfs = crate::vfs::VFS.lock();
    let (buf, len) = vfs.pwd();
    drop(vfs);
    let path = core::str::from_utf8(&buf[..len]).unwrap_or("/");
    crate::print!("{} > ", path);
}

/// Execute a single command segment, handling output redirection if present.
fn run_one(segment: &str) -> CommandResult {
    if let Some(redir_pos) = segment.find('>') {
        let cmd_part = segment[..redir_pos].trim();
        let file_part = segment[redir_pos + 1..].trim();

        if file_part.is_empty() {
            // Bare `>` with no filename – treat as a normal command.
            command::dispatch(commands::COMMANDS, segment)
        } else {
            // Capture command output then write it to the file.
            crate::console::writer::start_capture();
            let r = command::dispatch(commands::COMMANDS, cmd_part);
            let mut cap = [0u8; crate::vfs::MAX_FILE_CONTENT];
            let len = crate::console::writer::take_capture(&mut cap);
            let mut vfs = crate::vfs::VFS.lock();
            if let Err(e) = vfs.write_file(file_part, &cap[..len]) {
                drop(vfs);
                crate::println!("redirect: {}: {}", file_part, e.as_str());
            }
            r
        }
    } else {
        command::dispatch(commands::COMMANDS, segment)
    }
}

pub fn run() {
    let mut buf = InputBuffer::new();
    print_prompt();

    loop {
        let key = wait_key_blink();

        match key.ascii {
            // Enter – execute the current line.
            Some('\n') => {
                crate::println!();
                let line = buf.as_str().trim();

                // ── Command chaining: split on `&` and run each segment ───────
                let mut result = CommandResult::Continue;
                for segment in line.split('&') {
                    let segment = segment.trim();
                    if segment.is_empty() {
                        continue;
                    }
                    result = run_one(segment);
                    if let CommandResult::Halt = result {
                        break;
                    }
                }
                // ── End chaining ──────────────────────────────────────────────

                buf.clear();
                if let CommandResult::Halt = result {
                    break;
                }
                print_prompt();
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
