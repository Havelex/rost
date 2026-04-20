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

fn cmd_keymap(args: &[&str]) -> CommandResult {
    match args.first().copied() {
        Some("en") => {
            crate::keyboard::set_layout(crate::keyboard::Layout::En);
            crate::println!("Keyboard layout set to EN (US QWERTY)");
        }
        Some("de") => {
            crate::keyboard::set_layout(crate::keyboard::Layout::De);
            crate::println!("Keyboard layout set to DE (German QWERTZ)");
        }
        Some(other) => {
            crate::println!("Unknown layout '{}'. Supported layouts: en, de", other);
        }
        None => {
            let name = match crate::keyboard::get_layout() {
                crate::keyboard::Layout::En => "en (US QWERTY)",
                crate::keyboard::Layout::De => "de (German QWERTZ)",
            };
            crate::println!("Current keyboard layout: {}", name);
            crate::println!("Usage: keymap <en|de>");
        }
    }
    CommandResult::Continue
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
    Command {
        name: "keymap",
        description: "Get or set the keyboard layout (en, de)",
        handler: cmd_keymap,
    },
];
