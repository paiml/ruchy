//! REPL command handling module
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use super::state::ReplMode;
use std::collections::HashMap;

/// Command handler for REPL special commands
pub struct CommandHandler {
    commands: HashMap<String, Command>,
}

/// A REPL command with metadata
struct Command {
    name: String,
    description: String,
    usage: String,
    handler: CommandFunction,
}

/// Command handler function type
type CommandFunction = fn(&[String]) -> CommandResult;

/// Result of command execution
pub enum CommandResult {
    /// Command executed successfully with optional output
    Success(Option<String>),
    /// Command failed with error message
    Error(String),
    /// Switch to a different REPL mode
    SwitchMode(ReplMode),
    /// Exit the REPL
    Exit,
    /// Clear the screen
    Clear,
    /// Show help
    Help,
    /// Reset the REPL state
    Reset,
}

impl CommandHandler {
    /// Create a new command handler with built-in commands
    pub fn new() -> Self {
        let mut handler = Self {
            commands: HashMap::new(),
        };
        handler.register_builtin_commands();
        handler
    }

    /// Register built-in commands
    fn register_builtin_commands(&mut self) {
        self.register(
            "help",
            "Display help information",
            ":help [command]",
            cmd_help,
        );

        self.register(
            "exit",
            "Exit the REPL",
            ":exit",
            cmd_exit,
        );

        self.register(
            "quit",
            "Exit the REPL",
            ":quit",
            cmd_exit,
        );

        self.register(
            "clear",
            "Clear the screen",
            ":clear",
            cmd_clear,
        );

        self.register(
            "reset",
            "Reset the REPL state",
            ":reset",
            cmd_reset,
        );

        self.register(
            "mode",
            "Switch REPL mode",
            ":mode [normal|debug|tutorial|benchmark]",
            cmd_mode,
        );

        self.register(
            "load",
            "Load a file",
            ":load <filename>",
            cmd_load,
        );

        self.register(
            "save",
            "Save session to file",
            ":save <filename>",
            cmd_save,
        );

        self.register(
            "history",
            "Show command history",
            ":history [n]",
            cmd_history,
        );

        self.register(
            "bindings",
            "Show current bindings",
            ":bindings [pattern]",
            cmd_bindings,
        );
    }

    /// Register a new command
    fn register(
        &mut self,
        name: &str,
        description: &str,
        usage: &str,
        handler: CommandFunction,
    ) {
        self.commands.insert(
            name.to_string(),
            Command {
                name: name.to_string(),
                description: description.to_string(),
                usage: usage.to_string(),
                handler,
            },
        );
    }

    /// Process a command line
    pub fn process(&self, line: &str) -> Option<CommandResult> {
        if !line.starts_with(':') {
            return None;
        }

        let parts: Vec<String> = line[1..]
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if parts.is_empty() {
            return Some(CommandResult::Error("Empty command".to_string()));
        }

        let cmd_name = &parts[0];
        let args = &parts[1..];

        match self.commands.get(cmd_name) {
            Some(cmd) => Some((cmd.handler)(args)),
            None => Some(CommandResult::Error(format!("Unknown command: {}", cmd_name))),
        }
    }

    /// Get list of available commands
    pub fn list_commands(&self) -> Vec<String> {
        let mut commands: Vec<String> = self.commands.keys().cloned().collect();
        commands.sort();
        commands
    }

    /// Get command help
    pub fn get_help(&self, command: Option<&str>) -> String {
        match command {
            Some(name) => {
                match self.commands.get(name) {
                    Some(cmd) => format!(
                        "{}: {}\nUsage: {}",
                        cmd.name, cmd.description, cmd.usage
                    ),
                    None => format!("Unknown command: {}", name),
                }
            }
            None => {
                let mut help = String::from("Available commands:\n");
                for name in self.list_commands() {
                    if let Some(cmd) = self.commands.get(&name) {
                        help.push_str(&format!("  {:10} - {}\n", name, cmd.description));
                    }
                }
                help
            }
        }
    }
}

// Command implementations (complexity: ≤10)

fn cmd_help(args: &[String]) -> CommandResult {
    if args.is_empty() {
        CommandResult::Help
    } else {
        CommandResult::Success(Some(format!("Help for: {}", args[0])))
    }
}

fn cmd_exit(_args: &[String]) -> CommandResult {
    CommandResult::Exit
}

fn cmd_clear(_args: &[String]) -> CommandResult {
    CommandResult::Clear
}

fn cmd_reset(_args: &[String]) -> CommandResult {
    CommandResult::Reset
}

fn cmd_mode(args: &[String]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("Mode name required".to_string());
    }

    match args[0].as_str() {
        "normal" => CommandResult::SwitchMode(ReplMode::Normal),
        "debug" => CommandResult::SwitchMode(ReplMode::Debug),
        "tutorial" => CommandResult::SwitchMode(ReplMode::Tutorial),
        "benchmark" => CommandResult::SwitchMode(ReplMode::Benchmark),
        _ => CommandResult::Error(format!("Unknown mode: {}", args[0])),
    }
}

fn cmd_load(args: &[String]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("Filename required".to_string());
    }

    match std::fs::read_to_string(&args[0]) {
        Ok(content) => CommandResult::Success(Some(content)),
        Err(e) => CommandResult::Error(format!("Failed to load file: {}", e)),
    }
}

fn cmd_save(args: &[String]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("Filename required".to_string());
    }

    CommandResult::Success(Some(format!("Saving session to {}", args[0])))
}

fn cmd_history(args: &[String]) -> CommandResult {
    let n = if args.is_empty() {
        10
    } else {
        match args[0].parse::<usize>() {
            Ok(num) => num,
            Err(_) => return CommandResult::Error("Invalid number".to_string()),
        }
    };

    CommandResult::Success(Some(format!("Showing last {} commands", n)))
}

fn cmd_bindings(args: &[String]) -> CommandResult {
    let pattern = if args.is_empty() {
        None
    } else {
        Some(args[0].as_str())
    };

    match pattern {
        Some(p) => CommandResult::Success(Some(format!("Bindings matching '{}'", p))),
        None => CommandResult::Success(Some("All bindings".to_string())),
    }
}