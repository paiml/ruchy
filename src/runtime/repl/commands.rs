//! REPL Command System
//!
//! Handles REPL commands like :help, :quit, :mode, etc.

use super::state::{ReplMode, ReplState};
use anyhow::Result;

/// Result of executing a command
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Exit the REPL
    Exit,
    /// Success with optional output
    Success(String),
    /// Mode change notification
    ModeChange(ReplMode),
    /// Silent success (no output)
    Silent,
}

/// Context passed to command handlers
pub struct CommandContext<'a> {
    /// Command arguments
    pub args: Vec<&'a str>,
    /// REPL state (mutable)
    pub state: &'a mut ReplState,
    /// Evaluator for executing expressions (optional for backward compatibility)
    pub evaluator: Option<&'a mut super::evaluation::Evaluator>,
}

/// Registry of available commands
pub struct CommandRegistry {
    // No dynamic storage needed for built-in commands
}

impl CommandRegistry {
    /// Create a new command registry (complexity: 1)
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a command (complexity: 9)
    pub fn execute(&self, command: &str, context: &mut CommandContext) -> Result<CommandResult> {
        match command {
            ":help" | ":h" => Ok(CommandResult::Success(self.help_text())),
            ":quit" | ":exit" | ":q" => Ok(CommandResult::Exit),
            ":clear" => {
                context.state.clear_history();
                Ok(CommandResult::Success("History cleared".to_string()))
            }
            ":reset" => {
                context.state.clear_bindings();
                Ok(CommandResult::Success("Bindings reset".to_string()))
            }
            ":type" => {
                // Get the expression from args
                if context.args.is_empty() {
                    return Ok(CommandResult::Success(
                        "Usage: :type <expression>".to_string(),
                    ));
                }

                // Join all args to reconstruct the expression
                let expr = context.args.join(" ");

                // Return a placeholder - will be implemented in execute_type_command
                self.execute_type_command(&expr, context)
            }
            ":mode" => {
                if let Some(&mode_arg) = context.args.first() {
                    match mode_arg {
                        "normal" => {
                            context.state.set_mode(ReplMode::Normal);
                            Ok(CommandResult::ModeChange(ReplMode::Normal))
                        }
                        "debug" => {
                            context.state.set_mode(ReplMode::Debug);
                            Ok(CommandResult::ModeChange(ReplMode::Debug))
                        }
                        "ast" => {
                            context.state.set_mode(ReplMode::Ast);
                            Ok(CommandResult::ModeChange(ReplMode::Ast))
                        }
                        "transpile" => {
                            context.state.set_mode(ReplMode::Transpile);
                            Ok(CommandResult::ModeChange(ReplMode::Transpile))
                        }
                        _ => Ok(CommandResult::Success(format!("Unknown mode: {mode_arg}"))),
                    }
                } else {
                    let current = context.state.get_mode();
                    Ok(CommandResult::Success(format!("Current mode: {current:?}")))
                }
            }
            ":history" => Ok(CommandResult::Success(self.format_history(context.state))),
            ":vars" => Ok(CommandResult::Success(self.format_bindings(context.state))),
            _ => Ok(CommandResult::Success(format!(
                "Unknown command: {command}"
            ))),
        }
    }

    /// Get list of available commands (complexity: 1)
    pub fn available_commands(&self) -> Vec<&'static str> {
        vec![
            ":help", ":h", ":quit", ":exit", ":q", ":clear", ":reset", ":mode", ":history", ":vars",
        ]
    }

    /// Get help text (complexity: 1)
    fn help_text(&self) -> String {
        r"Ruchy REPL Commands:
  :help, :h          Show this help
  :quit, :exit, :q   Exit the REPL
  :clear             Clear command history
  :reset             Reset variable bindings
  :mode [mode]       Show/set REPL mode (normal, debug, ast, transpile)
  :history           Show command history
  :vars              Show variable bindings
  :type <expr>       Show type of expression

Enter expressions to evaluate them.
"
        .to_string()
    }

    /// Format command history (complexity: 3)
    fn format_history(&self, state: &ReplState) -> String {
        let history = state.get_history();
        if history.is_empty() {
            "No history".to_string()
        } else {
            history
                .iter()
                .enumerate()
                .map(|(i, cmd)| format!("{}: {}", i + 1, cmd))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    /// Format variable bindings (complexity: 3)
    fn format_bindings(&self, state: &ReplState) -> String {
        let bindings = state.get_bindings();
        if bindings.is_empty() {
            "No variables defined".to_string()
        } else {
            bindings
                .iter()
                .map(|(name, value)| format!("{name} = {value:?}"))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    /// Execute :type command to show type of expression (complexity: 6)
    fn execute_type_command(
        &self,
        expr: &str,
        context: &mut CommandContext,
    ) -> Result<CommandResult> {
        use super::EvalResult;

        // Get evaluator from context or create a new one
        let evaluator = match &mut context.evaluator {
            Some(eval) => eval,
            None => {
                return Ok(CommandResult::Success(
                    "Error: Evaluator not available".to_string(),
                ))
            }
        };

        // Evaluate the expression to get its value
        match evaluator.evaluate_line(expr, context.state) {
            Ok(EvalResult::Value(value)) => {
                // Get type name from value
                let type_name = Self::value_type_name(&value);
                Ok(CommandResult::Success(format!("Type: {type_name}")))
            }
            Ok(EvalResult::Error(msg)) => Ok(CommandResult::Success(format!(
                "Error evaluating expression: {msg}"
            ))),
            Ok(EvalResult::NeedMoreInput) => {
                Ok(CommandResult::Success("Incomplete expression".to_string()))
            }
            Err(e) => Ok(CommandResult::Success(format!("Error: {e}"))),
        }
    }

    /// Get human-readable type name from Value (complexity: 10)
    fn value_type_name(value: &super::Value) -> &'static str {
        use super::Value;
        match value {
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::Byte(_) => "Byte",
            Value::Nil => "Nil",
            Value::String(_) => "String",
            Value::Array(_) => "Array",
            Value::Tuple(_) => "Tuple",
            Value::Closure { .. } => "Function",
            Value::DataFrame { .. } => "DataFrame",
            Value::Object(_) => "Object",
            Value::ObjectMut(_) => "Object",
            Value::Range { .. } => "Range",
            Value::EnumVariant { .. } => "Enum",
            Value::BuiltinFunction(_) => "BuiltinFunction",
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_creation() {
        let registry = CommandRegistry::new();
        // Just ensure it can be created
        drop(registry);
    }

    #[test]
    fn test_help_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        match registry.execute(":help", &mut context).unwrap() {
            CommandResult::Success(help) => {
                assert!(help.contains("Commands"));
                assert!(help.contains(":help"));
                assert!(help.contains(":quit"));
            }
            result => panic!("Expected Success, got {result:?}"),
        }
    }

    #[test]
    fn test_quit_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        match registry.execute(":quit", &mut context).unwrap() {
            CommandResult::Exit => {}
            result => panic!("Expected Exit, got {result:?}"),
        }
    }

    #[test]
    fn test_mode_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["debug"],
            state: &mut state,
        };

        match registry.execute(":mode", &mut context).unwrap() {
            CommandResult::ModeChange(ReplMode::Debug) => {}
            result => panic!("Expected ModeChange(Debug), got {result:?}"),
        }

        assert!(matches!(state.get_mode(), ReplMode::Debug));
    }
}
