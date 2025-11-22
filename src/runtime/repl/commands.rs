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
#[derive(Debug)]
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
            ":inspect" => {
                // Get the expression from args
                if context.args.is_empty() {
                    return Ok(CommandResult::Success(
                        "Usage: :inspect <expression>".to_string(),
                    ));
                }

                // Join all args to reconstruct the expression
                let expr = context.args.join(" ");

                // Execute inspect command
                self.execute_inspect_command(&expr, context)
            }
            ":ast" => {
                // Get the expression from args
                if context.args.is_empty() {
                    return Ok(CommandResult::Success(
                        "Usage: :ast <expression>".to_string(),
                    ));
                }

                // Join all args to reconstruct the expression
                let expr = context.args.join(" ");

                // Execute AST command
                self.execute_ast_command(&expr, context)
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
            ":env" => Ok(CommandResult::Success(
                self.format_environment(context.state),
            )),
            _ => Ok(CommandResult::Success(format!(
                "Unknown command: {command}"
            ))),
        }
    }

    /// Get list of available commands (complexity: 1)
    pub fn available_commands(&self) -> Vec<&'static str> {
        vec![
            ":help", ":h", ":quit", ":exit", ":q", ":clear", ":reset", ":mode", ":history",
            ":vars", ":env", ":type", ":inspect", ":ast",
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
  :env               Show comprehensive environment info
  :type <expr>       Show type of expression
  :inspect <expr>    Detailed inspection of value
  :ast <expr>        Show AST structure

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

    /// Format comprehensive environment information (complexity: 8)
    fn format_environment(&self, state: &ReplState) -> String {
        let mut output = String::new();

        // Header
        output.push_str("=== REPL Environment ===\n\n");

        // Mode information
        output.push_str(&format!("Mode: {:?}\n\n", state.get_mode()));

        // Variables section
        output.push_str("--- Variables ---\n");
        let bindings = state.get_bindings();
        if bindings.is_empty() {
            output.push_str("No variables defined\n");
        } else {
            for (name, value) in bindings {
                let type_name = Self::value_type_name(value);
                output.push_str(&format!("{name}: {type_name} = {value}\n"));
            }
        }

        // History section
        output.push_str("\n--- History ---\n");
        let history = state.get_history();
        output.push_str(&format!("Commands executed: {}\n", history.len()));

        output
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

    /// Execute :inspect command to show detailed value info (complexity: 8)
    fn execute_inspect_command(
        &self,
        expr: &str,
        context: &mut CommandContext,
    ) -> Result<CommandResult> {
        use super::EvalResult;

        // Get evaluator from context
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
                // Generate detailed inspection output
                let inspection = Self::inspect_value(&value);
                Ok(CommandResult::Success(inspection))
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

    /// Execute :ast command to show AST structure (complexity: 4)
    fn execute_ast_command(
        &self,
        expr: &str,
        _context: &mut CommandContext,
    ) -> Result<CommandResult> {
        use crate::frontend::Parser;

        // Parse the expression to get AST
        let mut parser = Parser::new(expr);
        match parser.parse() {
            Ok(ast) => {
                // Format AST as debug output
                let ast_output = format!("{ast:#?}");
                Ok(CommandResult::Success(ast_output))
            }
            Err(e) => Ok(CommandResult::Success(format!("Parse error: {e}"))),
        }
    }

    /// Generate detailed inspection output for a value (complexity: 9)
    fn inspect_value(value: &super::Value) -> String {
        use super::Value;

        let type_name = Self::value_type_name(value);
        let memory_size = Self::estimate_value_memory(value);
        let mut output = format!("Type: {type_name}\n");
        output.push_str(&format!("Memory: ~{memory_size} bytes\n"));

        match value {
            Value::Integer(n) => {
                output.push_str(&format!("Value: {n}\n"));
            }
            Value::Float(f) => {
                output.push_str(&format!("Value: {f}\n"));
            }
            Value::Bool(b) => {
                output.push_str(&format!("Value: {b}\n"));
            }
            Value::Byte(b) => {
                output.push_str(&format!("Value: {b}\n"));
                output.push_str(&format!("Character: {}\n", char::from(*b)));
            }
            Value::String(s) => {
                output.push_str(&format!("Value: \"{s}\"\n"));
                output.push_str(&format!("Length: {}\n", s.len()));
            }
            Value::Array(arr) => {
                output.push_str(&format!("Length: {}\n", arr.len()));
                output.push_str("Elements:\n");
                for (i, elem) in arr.iter().enumerate().take(10) {
                    output.push_str(&format!("  [{i}]: {elem}\n"));
                }
                if arr.len() > 10 {
                    output.push_str(&format!("  ... and {} more\n", arr.len() - 10));
                }
            }
            Value::Tuple(items) => {
                output.push_str(&format!("Length: {}\n", items.len()));
                output.push_str("Items:\n");
                for (i, item) in items.iter().enumerate() {
                    output.push_str(&format!("  [{i}]: {item}\n"));
                }
            }
            Value::Object(obj) => {
                output.push_str(&format!("Fields: {}\n", obj.len()));
                output.push_str("Properties:\n");
                for (key, val) in obj.iter() {
                    output.push_str(&format!("  {key}: {val}\n"));
                }
            }
            Value::ObjectMut(obj) => {
                let borrowed = obj.lock().expect("mutex should not be poisoned");
                output.push_str(&format!("Fields: {}\n", borrowed.len()));
                output.push_str("Properties:\n");
                for (key, val) in borrowed.iter() {
                    output.push_str(&format!("  {key}: {val}\n"));
                }
            }
            Value::DataFrame { columns } => {
                output.push_str(&format!("Columns: {}\n", columns.len()));
                if let Some(first_col) = columns.first() {
                    output.push_str(&format!("Rows: {}\n", first_col.values.len()));
                }
            }
            Value::Nil => {
                output.push_str("Value: nil\n");
            }
            Value::Closure { .. } => {
                output.push_str("Value: <function>\n");
            }
            Value::BuiltinFunction(name) => {
                output.push_str(&format!("Value: <builtin: {name}>\n"));
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                output.push_str(&format!("Start: {start}\n"));
                output.push_str(&format!("End: {end}\n"));
                output.push_str(&format!("Inclusive: {inclusive}\n"));
            }
            Value::EnumVariant { variant_name, .. } => {
                output.push_str(&format!("Variant: {variant_name}\n"));
            }
            Value::Struct { name, fields } => {
                output.push_str(&format!("Struct: {name}\n"));
                output.push_str(&format!("Fields: {}\n", fields.len()));
                output.push_str("Values:\n");
                for (key, val) in fields.iter() {
                    output.push_str(&format!("  {key}: {val}\n"));
                }
            }
            Value::Class {
                class_name,
                fields,
                methods,
            } => {
                output.push_str(&format!("Class: {class_name}\n"));
                let fields_read = fields.read().expect("rwlock should not be poisoned");
                output.push_str(&format!("Fields: {}\n", fields_read.len()));
                output.push_str(&format!("Methods: {}\n", methods.len()));
                output.push_str("Values:\n");
                for (key, val) in fields_read.iter() {
                    output.push_str(&format!("  {key}: {val}\n"));
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(_) => {
                output.push_str("Type: HtmlDocument\n");
                output.push_str("(HTML document content)\n");
            }
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(_) => {
                output.push_str("Type: HtmlElement\n");
                output.push_str("(HTML element node)\n");
            }
        }

        output
    }

    /// Estimate memory usage of a Value in bytes (complexity: 10)
    fn estimate_value_memory(value: &super::Value) -> usize {
        use super::Value;
        use std::mem::size_of;

        match value {
            Value::Integer(_) => size_of::<i64>(),
            Value::Float(_) => size_of::<f64>(),
            Value::Bool(_) => size_of::<bool>(),
            Value::Byte(_) => size_of::<u8>(),
            Value::Nil => 0,
            Value::String(s) => size_of::<String>() + s.len(),
            Value::Array(arr) => {
                size_of::<Vec<Value>>() + arr.iter().map(Self::estimate_value_memory).sum::<usize>()
            }
            Value::Tuple(items) => {
                size_of::<Vec<Value>>()
                    + items.iter().map(Self::estimate_value_memory).sum::<usize>()
            }
            Value::Object(obj) => {
                size_of::<std::collections::HashMap<String, Value>>()
                    + obj
                        .iter()
                        .map(|(k, v)| k.len() + Self::estimate_value_memory(v))
                        .sum::<usize>()
            }
            Value::ObjectMut(obj) => {
                size_of::<std::collections::HashMap<String, Value>>()
                    + obj
                        .lock()
                        .expect("mutex should not be poisoned")
                        .iter()
                        .map(|(k, v)| k.len() + Self::estimate_value_memory(v))
                        .sum::<usize>()
            }
            Value::DataFrame { columns } => columns
                .iter()
                .map(|col| {
                    col.name.len()
                        + col
                            .values
                            .iter()
                            .map(Self::estimate_value_memory)
                            .sum::<usize>()
                })
                .sum(),
            Value::Range { start, end, .. } => {
                size_of::<bool>()
                    + Self::estimate_value_memory(start)
                    + Self::estimate_value_memory(end)
            }
            Value::EnumVariant {
                variant_name, data, ..
            } => {
                variant_name.len()
                    + data
                        .as_ref()
                        .map_or(0, |vals| vals.iter().map(Self::estimate_value_memory).sum())
            }
            Value::Closure { params, .. } => {
                // RUNTIME-DEFAULT-PARAMS: Approximate params + environment overhead
                params
                    .iter()
                    .map(|(param_name, _default)| param_name.len())
                    .sum::<usize>()
                    + 128
            }
            Value::BuiltinFunction(name) => name.len() + size_of::<usize>(),
            Value::Struct { name, fields } => {
                size_of::<String>()
                    + name.len()
                    + size_of::<std::collections::HashMap<String, Value>>()
                    + fields
                        .iter()
                        .map(|(k, v)| k.len() + Self::estimate_value_memory(v))
                        .sum::<usize>()
            }
            Value::Class {
                class_name,
                fields,
                methods,
            } => {
                let fields_read = fields.read().expect("rwlock should not be poisoned");
                size_of::<String>()
                    + class_name.len()
                    + size_of::<std::collections::HashMap<String, Value>>()
                    + fields_read
                        .iter()
                        .map(|(k, v)| k.len() + Self::estimate_value_memory(v))
                        .sum::<usize>()
                    + methods.len() * 32
            }
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(_) => 128, // Estimated HTML document overhead
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(_) => 64, // Estimated HTML element overhead
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
            Value::Struct { .. } => "Struct",
            Value::Class { .. } => "Class",
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(_) => "HtmlDocument",
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(_) => "HtmlElement",
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
        let _registry = CommandRegistry::new();
        // Just ensure it can be created
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

        match registry
            .execute(":help", &mut context)
            .expect("operation should succeed in test")
        {
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

        match registry
            .execute(":quit", &mut context)
            .expect("operation should succeed in test")
        {
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

        match registry
            .execute(":mode", &mut context)
            .expect("operation should succeed in test")
        {
            CommandResult::ModeChange(ReplMode::Debug) => {}
            result => panic!("Expected ModeChange(Debug), got {result:?}"),
        }

        assert!(matches!(state.get_mode(), ReplMode::Debug));
    }
}
