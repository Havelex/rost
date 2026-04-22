use crate::shell::command::{Command, CommandResult};
use crate::vfs::{InodeKind, VFS, VfsError};

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

// ── File-system helpers ───────────────────────────────────────────────────────

fn vfs_err(cmd: &str, path: &str, e: VfsError) -> CommandResult {
    crate::println!("{}: {}: {}", cmd, path, e.as_str());
    CommandResult::Continue
}

// ── File-system command handlers ──────────────────────────────────────────────

fn cmd_ls(args: &[&str]) -> CommandResult {
    let path = args.first().copied().unwrap_or(".");
    let vfs = VFS.lock();

    let idx = match vfs.resolve_path(path) {
        Ok(i) => i,
        Err(e) => {
            drop(vfs);
            return vfs_err("ls", path, e);
        }
    };

    if vfs.inodes[idx].kind != InodeKind::Directory {
        drop(vfs);
        return vfs_err("ls", path, VfsError::NotADirectory);
    }

    let child_count = vfs.inodes[idx].child_count;
    for i in 0..child_count {
        let child_idx = vfs.inodes[idx].children[i];
        let child = &vfs.inodes[child_idx];
        let kind_char = if child.kind == InodeKind::Directory {
            'd'
        } else {
            '-'
        };
        crate::println!(
            "{} {:6} {}",
            kind_char,
            child.display_size(),
            child.name_str()
        );
    }

    CommandResult::Continue
}

fn cmd_cat(args: &[&str]) -> CommandResult {
    let path = match args.first().copied() {
        Some(p) => p,
        None => {
            crate::println!("cat: missing file operand");
            return CommandResult::Continue;
        }
    };

    let vfs = VFS.lock();
    match vfs.read_file(path) {
        Ok(data) => {
            match core::str::from_utf8(data) {
                Ok(s) => crate::print!("{}", s),
                Err(_) => {
                    for &b in data {
                        if b.is_ascii_graphic() || b == b' ' || b == b'\n' || b == b'\t' {
                            crate::print!("{}", b as char);
                        } else {
                            crate::print!(".");
                        }
                    }
                }
            }
            // Ensure output ends on a new line.
            if data.last() != Some(&b'\n') {
                crate::println!();
            }
        }
        Err(e) => {
            drop(vfs);
            return vfs_err("cat", path, e);
        }
    }

    CommandResult::Continue
}

fn cmd_touch(args: &[&str]) -> CommandResult {
    let path = match args.first().copied() {
        Some(p) => p,
        None => {
            crate::println!("touch: missing file operand");
            return CommandResult::Continue;
        }
    };

    let mut vfs = VFS.lock();
    if let Err(e) = vfs.touch(path) {
        drop(vfs);
        return vfs_err("touch", path, e);
    }

    CommandResult::Continue
}

fn cmd_mkdir(args: &[&str]) -> CommandResult {
    let path = match args.first().copied() {
        Some(p) => p,
        None => {
            crate::println!("mkdir: missing directory operand");
            return CommandResult::Continue;
        }
    };

    let mut vfs = VFS.lock();
    if let Err(e) = vfs.mkdir(path) {
        drop(vfs);
        return vfs_err("mkdir", path, e);
    }

    CommandResult::Continue
}

fn cmd_cd(args: &[&str]) -> CommandResult {
    let path = args.first().copied().unwrap_or("/");
    let mut vfs = VFS.lock();
    if let Err(e) = vfs.cd(path) {
        drop(vfs);
        return vfs_err("cd", path, e);
    }
    CommandResult::Continue
}

fn cmd_pwd(_args: &[&str]) -> CommandResult {
    let vfs = VFS.lock();
    let (buf, len) = vfs.pwd();
    drop(vfs);
    let path = core::str::from_utf8(&buf[..len]).unwrap_or("/");
    crate::println!("{}", path);
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
    Command {
        name: "ls",
        description: "List directory contents (ls [path])",
        handler: cmd_ls,
    },
    Command {
        name: "cat",
        description: "Print file contents (cat <file>)",
        handler: cmd_cat,
    },
    Command {
        name: "touch",
        description: "Create an empty file (touch <file>)",
        handler: cmd_touch,
    },
    Command {
        name: "mkdir",
        description: "Create a directory (mkdir <dir>)",
        handler: cmd_mkdir,
    },
    Command {
        name: "cd",
        description: "Change current directory (cd <path>)",
        handler: cmd_cd,
    },
    Command {
        name: "pwd",
        description: "Print current working directory",
        handler: cmd_pwd,
    },
];
