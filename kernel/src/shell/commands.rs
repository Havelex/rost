use crate::shell::command::{Command, CommandResult};

// ── Built-in handlers ─────────────────────────────────────────────────────────

fn cmd_help(_args: &[&str]) -> CommandResult {
    crate::println!("Available commands:");
    for cmd in COMMANDS {
        crate::println!("  {:10} - {}", cmd.name, cmd.description);
    }
    CommandResult::Continue
}

fn cmd_echo(args: &[&str]) -> CommandResult {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            crate::print!(" ");
        }
        crate::print!("{}", arg);
    }
    crate::println!();
    CommandResult::Continue
}

fn cmd_clear(_args: &[&str]) -> CommandResult {
    crate::cls!();
    CommandResult::Continue
}

fn cmd_halt(_args: &[&str]) -> CommandResult {
    crate::println!("Halting system...");
    CommandResult::Halt
}

// ── Command registry ──────────────────────────────────────────────────────────

/// All built-in commands recognised by the shell.
///
/// Add a new entry here to make a command available to the user.
pub static COMMANDS: &[Command] = &[
    Command {
        name: "help",
        description: "List available commands",
        handler: cmd_help,
    },
    Command {
        name: "echo",
        description: "Print arguments to the screen",
        handler: cmd_echo,
    },
    Command {
        name: "clear",
        description: "Clear the screen",
        handler: cmd_clear,
    },
    Command {
        name: "halt",
        description: "Halt the system",
        handler: cmd_halt,
    },
];
