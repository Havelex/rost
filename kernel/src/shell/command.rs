/// Maximum number of whitespace-separated tokens parsed from a single line.
const MAX_ARGS: usize = 16;

/// Outcome returned by every command handler.
pub enum CommandResult {
    /// Keep the shell running and show the next prompt.
    Continue,
    /// Exit the shell loop (e.g. the `halt` command).
    Halt,
}

/// A single shell command entry.
pub struct Command {
    /// Short name typed by the user (e.g. `"echo"`).
    pub name: &'static str,
    /// One-line description shown by the `help` command.
    pub description: &'static str,
    /// Handler called with the arguments that follow the command name.
    pub handler: fn(args: &[&str]) -> CommandResult,
}

/// Parse `line` into tokens, look up the first token in `commands`, and
/// execute the matching handler.
///
/// Returns [`CommandResult::Continue`] for unknown commands (after printing an
/// error) and for empty lines.
pub fn dispatch(commands: &[Command], line: &str) -> CommandResult {
    // argv is initialised to empty strings; only indices 0..argc are valid and
    // are the only indices ever read below.
    let mut argv = [""; MAX_ARGS];
    let mut argc = 0usize;

    for word in line.split_whitespace() {
        if argc < MAX_ARGS {
            argv[argc] = word;
            argc += 1;
        }
    }

    if argc == 0 {
        return CommandResult::Continue;
    }

    let cmd_name = argv[0];
    let cmd_args = &argv[1..argc];

    for cmd in commands {
        if cmd.name == cmd_name {
            return (cmd.handler)(cmd_args);
        }
    }

    crate::println!(
        "Unknown command: '{}'. Type 'help' for a list of commands.",
        cmd_name
    );
    CommandResult::Continue
}
