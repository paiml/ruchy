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
            Value::Atom(s) => {
                output.push_str(&format!("Atom: :{s}\n"));
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
            Value::Atom(s) => std::mem::size_of::<Value>() + s.len(),
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
            Value::Atom(_) => "Atom",
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
    use crate::runtime::Value;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, RwLock};

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

    // CommandResult tests
    #[test]
    fn test_command_result_debug() {
        let exit = CommandResult::Exit;
        assert!(format!("{:?}", exit).contains("Exit"));

        let success = CommandResult::Success("test".to_string());
        assert!(format!("{:?}", success).contains("Success"));

        let mode_change = CommandResult::ModeChange(ReplMode::Normal);
        assert!(format!("{:?}", mode_change).contains("ModeChange"));

        let silent = CommandResult::Silent;
        assert!(format!("{:?}", silent).contains("Silent"));
    }

    #[test]
    fn test_command_result_clone() {
        let exit = CommandResult::Exit;
        let _exit2 = exit.clone();

        let success = CommandResult::Success("test".to_string());
        let success2 = success.clone();
        assert!(matches!(success2, CommandResult::Success(s) if s == "test"));

        let silent = CommandResult::Silent;
        let _silent2 = silent.clone();
    }

    // Help command aliases
    #[test]
    fn test_help_command_alias_h() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":h", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(_)));
    }

    // Exit command aliases
    #[test]
    fn test_exit_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":exit", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Exit));
    }

    #[test]
    fn test_q_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":q", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Exit));
    }

    // Clear command
    #[test]
    fn test_clear_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        state.add_to_history("test command".to_string());
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":clear", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("cleared")));
    }

    // Reset command
    #[test]
    fn test_reset_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        state.set_variable("x".to_string(), Value::Integer(42));
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":reset", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("reset")));
    }

    // Type command - no args
    #[test]
    fn test_type_command_no_args() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":type", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("Usage")));
    }

    // Type command - no evaluator
    #[test]
    fn test_type_command_no_evaluator() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["42"],
            state: &mut state,
        };

        let result = registry
            .execute(":type", &mut context)
            .expect("should succeed");
        assert!(
            matches!(result, CommandResult::Success(s) if s.contains("Evaluator not available"))
        );
    }

    // Inspect command - no args
    #[test]
    fn test_inspect_command_no_args() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":inspect", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("Usage")));
    }

    // Inspect command - no evaluator
    #[test]
    fn test_inspect_command_no_evaluator() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["42"],
            state: &mut state,
        };

        let result = registry
            .execute(":inspect", &mut context)
            .expect("should succeed");
        assert!(
            matches!(result, CommandResult::Success(s) if s.contains("Evaluator not available"))
        );
    }

    // AST command - no args
    #[test]
    fn test_ast_command_no_args() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":ast", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("Usage")));
    }

    // AST command - valid expression
    #[test]
    fn test_ast_command_valid_expr() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["1", "+", "2"],
            state: &mut state,
        };

        let result = registry
            .execute(":ast", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(_)));
    }

    // AST command - invalid expression
    #[test]
    fn test_ast_command_invalid_expr() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["{{{"],
            state: &mut state,
        };

        let result = registry
            .execute(":ast", &mut context)
            .expect("should succeed");
        assert!(
            matches!(result, CommandResult::Success(s) if s.contains("error") || s.contains("Error"))
        );
    }

    // Mode commands - all modes
    #[test]
    fn test_mode_command_normal() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["normal"],
            state: &mut state,
        };

        let result = registry
            .execute(":mode", &mut context)
            .expect("should succeed");
        assert!(matches!(
            result,
            CommandResult::ModeChange(ReplMode::Normal)
        ));
    }

    #[test]
    fn test_mode_command_ast() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["ast"],
            state: &mut state,
        };

        let result = registry
            .execute(":mode", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::ModeChange(ReplMode::Ast)));
    }

    #[test]
    fn test_mode_command_transpile() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["transpile"],
            state: &mut state,
        };

        let result = registry
            .execute(":mode", &mut context)
            .expect("should succeed");
        assert!(matches!(
            result,
            CommandResult::ModeChange(ReplMode::Transpile)
        ));
    }

    #[test]
    fn test_mode_command_unknown() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec!["invalid"],
            state: &mut state,
        };

        let result = registry
            .execute(":mode", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("Unknown mode")));
    }

    #[test]
    fn test_mode_command_no_args() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":mode", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("Current mode")));
    }

    // History command
    #[test]
    fn test_history_command_empty() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":history", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("No history")));
    }

    #[test]
    fn test_history_command_with_items() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        state.add_to_history("let x = 1".to_string());
        state.add_to_history("let y = 2".to_string());
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":history", &mut context)
            .expect("should succeed");
        if let CommandResult::Success(s) = result {
            assert!(s.contains("1:"));
            assert!(s.contains("let x = 1"));
        } else {
            panic!("Expected Success");
        }
    }

    // Vars command
    #[test]
    fn test_vars_command_empty() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":vars", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("No variables")));
    }

    #[test]
    fn test_vars_command_with_items() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        state.set_variable("x".to_string(), Value::Integer(42));
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":vars", &mut context)
            .expect("should succeed");
        if let CommandResult::Success(s) = result {
            assert!(s.contains("x"));
        } else {
            panic!("Expected Success");
        }
    }

    // Env command
    #[test]
    fn test_env_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":env", &mut context)
            .expect("should succeed");
        if let CommandResult::Success(s) = result {
            assert!(s.contains("REPL Environment"));
            assert!(s.contains("Mode:"));
            assert!(s.contains("Variables"));
            assert!(s.contains("History"));
        } else {
            panic!("Expected Success");
        }
    }

    #[test]
    fn test_env_command_with_data() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        state.set_variable("x".to_string(), Value::Integer(42));
        state.add_to_history("let x = 42".to_string());
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":env", &mut context)
            .expect("should succeed");
        if let CommandResult::Success(s) = result {
            assert!(s.contains("x: Integer"));
        } else {
            panic!("Expected Success");
        }
    }

    // Unknown command
    #[test]
    fn test_unknown_command() {
        let registry = CommandRegistry::new();
        let mut state = ReplState::new();
        let mut context = CommandContext {
            evaluator: None,
            args: vec![],
            state: &mut state,
        };

        let result = registry
            .execute(":unknown", &mut context)
            .expect("should succeed");
        assert!(matches!(result, CommandResult::Success(s) if s.contains("Unknown command")));
    }

    // available_commands test
    #[test]
    fn test_available_commands() {
        let registry = CommandRegistry::new();
        let commands = registry.available_commands();
        assert!(commands.contains(&":help"));
        assert!(commands.contains(&":h"));
        assert!(commands.contains(&":quit"));
        assert!(commands.contains(&":exit"));
        assert!(commands.contains(&":q"));
        assert!(commands.contains(&":clear"));
        assert!(commands.contains(&":reset"));
        assert!(commands.contains(&":mode"));
        assert!(commands.contains(&":history"));
        assert!(commands.contains(&":vars"));
        assert!(commands.contains(&":env"));
        assert!(commands.contains(&":type"));
        assert!(commands.contains(&":inspect"));
        assert!(commands.contains(&":ast"));
    }

    // Default impl
    #[test]
    fn test_command_registry_default() {
        let registry = CommandRegistry::default();
        let commands = registry.available_commands();
        assert!(!commands.is_empty());
    }

    // CommandRegistry Debug
    #[test]
    fn test_command_registry_debug() {
        let registry = CommandRegistry::new();
        let debug = format!("{:?}", registry);
        assert!(debug.contains("CommandRegistry"));
    }

    // value_type_name tests
    #[test]
    fn test_value_type_name_integer() {
        assert_eq!(
            CommandRegistry::value_type_name(&Value::Integer(42)),
            "Integer"
        );
    }

    #[test]
    fn test_value_type_name_float() {
        assert_eq!(
            CommandRegistry::value_type_name(&Value::Float(3.14)),
            "Float"
        );
    }

    #[test]
    fn test_value_type_name_bool() {
        assert_eq!(CommandRegistry::value_type_name(&Value::Bool(true)), "Bool");
    }

    #[test]
    fn test_value_type_name_byte() {
        assert_eq!(CommandRegistry::value_type_name(&Value::Byte(65)), "Byte");
    }

    #[test]
    fn test_value_type_name_nil() {
        assert_eq!(CommandRegistry::value_type_name(&Value::Nil), "Nil");
    }

    #[test]
    fn test_value_type_name_string() {
        assert_eq!(
            CommandRegistry::value_type_name(&Value::String("test".into())),
            "String"
        );
    }

    #[test]
    fn test_value_type_name_array() {
        assert_eq!(
            CommandRegistry::value_type_name(&Value::Array(vec![].into())),
            "Array"
        );
    }

    #[test]
    fn test_value_type_name_tuple() {
        assert_eq!(
            CommandRegistry::value_type_name(&Value::Tuple(vec![].into())),
            "Tuple"
        );
    }

    #[test]
    fn test_value_type_name_object() {
        let map = HashMap::new();
        assert_eq!(
            CommandRegistry::value_type_name(&Value::Object(Arc::new(map))),
            "Object"
        );
    }

    #[test]
    fn test_value_type_name_object_mut() {
        let map = HashMap::new();
        assert_eq!(
            CommandRegistry::value_type_name(&Value::ObjectMut(Arc::new(Mutex::new(map)))),
            "Object"
        );
    }

    #[test]
    fn test_value_type_name_range() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };
        assert_eq!(CommandRegistry::value_type_name(&range), "Range");
    }

    #[test]
    fn test_value_type_name_enum_variant() {
        let variant = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: None,
        };
        assert_eq!(CommandRegistry::value_type_name(&variant), "Enum");
    }

    #[test]
    fn test_value_type_name_builtin_function() {
        assert_eq!(
            CommandRegistry::value_type_name(&Value::BuiltinFunction("print".to_string())),
            "BuiltinFunction"
        );
    }

    #[test]
    fn test_value_type_name_struct() {
        let s = Value::Struct {
            name: "Point".to_string(),
            fields: Arc::new(HashMap::new()),
        };
        assert_eq!(CommandRegistry::value_type_name(&s), "Struct");
    }

    #[test]
    fn test_value_type_name_class() {
        let c = Value::Class {
            class_name: "MyClass".to_string(),
            fields: Arc::new(RwLock::new(HashMap::new())),
            methods: Arc::new(HashMap::new()),
        };
        assert_eq!(CommandRegistry::value_type_name(&c), "Class");
    }

    #[test]
    fn test_value_type_name_atom() {
        assert_eq!(
            CommandRegistry::value_type_name(&Value::Atom("ok".to_string())),
            "Atom"
        );
    }

    // estimate_value_memory tests
    #[test]
    fn test_estimate_value_memory_integer() {
        let size = CommandRegistry::estimate_value_memory(&Value::Integer(42));
        assert_eq!(size, std::mem::size_of::<i64>());
    }

    #[test]
    fn test_estimate_value_memory_float() {
        let size = CommandRegistry::estimate_value_memory(&Value::Float(3.14));
        assert_eq!(size, std::mem::size_of::<f64>());
    }

    #[test]
    fn test_estimate_value_memory_bool() {
        let size = CommandRegistry::estimate_value_memory(&Value::Bool(true));
        assert_eq!(size, std::mem::size_of::<bool>());
    }

    #[test]
    fn test_estimate_value_memory_byte() {
        let size = CommandRegistry::estimate_value_memory(&Value::Byte(65));
        assert_eq!(size, std::mem::size_of::<u8>());
    }

    #[test]
    fn test_estimate_value_memory_nil() {
        let size = CommandRegistry::estimate_value_memory(&Value::Nil);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_estimate_value_memory_string() {
        let expected = std::mem::size_of::<String>() + 5;
        let size = CommandRegistry::estimate_value_memory(&Value::String("hello".into()));
        assert_eq!(size, expected);
    }

    #[test]
    fn test_estimate_value_memory_array() {
        let arr = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
        let size = CommandRegistry::estimate_value_memory(&arr);
        assert!(size > std::mem::size_of::<Vec<Value>>());
    }

    #[test]
    fn test_estimate_value_memory_tuple() {
        let tuple = Value::Tuple(vec![Value::Integer(1)].into());
        let size = CommandRegistry::estimate_value_memory(&tuple);
        assert!(size > std::mem::size_of::<Vec<Value>>());
    }

    #[test]
    fn test_estimate_value_memory_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(1));
        let obj = Value::Object(Arc::new(map));
        let size = CommandRegistry::estimate_value_memory(&obj);
        assert!(size > 0);
    }

    #[test]
    fn test_estimate_value_memory_object_mut() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(1));
        let obj = Value::ObjectMut(Arc::new(Mutex::new(map)));
        let size = CommandRegistry::estimate_value_memory(&obj);
        assert!(size > 0);
    }

    #[test]
    fn test_estimate_value_memory_range() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };
        let size = CommandRegistry::estimate_value_memory(&range);
        assert!(size >= std::mem::size_of::<bool>() + 2 * std::mem::size_of::<i64>());
    }

    #[test]
    fn test_estimate_value_memory_enum_variant() {
        let variant = Value::EnumVariant {
            enum_name: "Opt".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        let size = CommandRegistry::estimate_value_memory(&variant);
        assert!(size > 0);
    }

    #[test]
    fn test_estimate_value_memory_enum_variant_no_data() {
        let variant = Value::EnumVariant {
            enum_name: "Opt".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };
        let size = CommandRegistry::estimate_value_memory(&variant);
        assert!(size >= "None".len());
    }

    #[test]
    fn test_estimate_value_memory_builtin_function() {
        let size =
            CommandRegistry::estimate_value_memory(&Value::BuiltinFunction("print".to_string()));
        assert!(size > 0);
    }

    #[test]
    fn test_estimate_value_memory_struct() {
        let s = Value::Struct {
            name: "Point".to_string(),
            fields: Arc::new(HashMap::new()),
        };
        let size = CommandRegistry::estimate_value_memory(&s);
        assert!(size > 0);
    }

    #[test]
    fn test_estimate_value_memory_class() {
        let c = Value::Class {
            class_name: "Cls".to_string(),
            fields: Arc::new(RwLock::new(HashMap::new())),
            methods: Arc::new(HashMap::new()),
        };
        let size = CommandRegistry::estimate_value_memory(&c);
        assert!(size > 0);
    }

    #[test]
    fn test_estimate_value_memory_atom() {
        let size = CommandRegistry::estimate_value_memory(&Value::Atom("ok".to_string()));
        assert!(size > 0);
    }

    // inspect_value tests
    #[test]
    fn test_inspect_value_integer() {
        let output = CommandRegistry::inspect_value(&Value::Integer(42));
        assert!(output.contains("Type: Integer"));
        assert!(output.contains("Value: 42"));
    }

    #[test]
    fn test_inspect_value_float() {
        let output = CommandRegistry::inspect_value(&Value::Float(3.14));
        assert!(output.contains("Type: Float"));
        assert!(output.contains("3.14"));
    }

    #[test]
    fn test_inspect_value_bool() {
        let output = CommandRegistry::inspect_value(&Value::Bool(true));
        assert!(output.contains("Type: Bool"));
        assert!(output.contains("true"));
    }

    #[test]
    fn test_inspect_value_byte() {
        let output = CommandRegistry::inspect_value(&Value::Byte(65));
        assert!(output.contains("Type: Byte"));
        assert!(output.contains("65"));
        assert!(output.contains("Character:"));
    }

    #[test]
    fn test_inspect_value_string() {
        let output = CommandRegistry::inspect_value(&Value::String("hello".into()));
        assert!(output.contains("Type: String"));
        assert!(output.contains("hello"));
        assert!(output.contains("Length: 5"));
    }

    #[test]
    fn test_inspect_value_nil() {
        let output = CommandRegistry::inspect_value(&Value::Nil);
        assert!(output.contains("Type: Nil"));
        assert!(output.contains("nil"));
    }

    #[test]
    fn test_inspect_value_array_small() {
        let arr = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
        let output = CommandRegistry::inspect_value(&arr);
        assert!(output.contains("Type: Array"));
        assert!(output.contains("Length: 2"));
        assert!(output.contains("[0]:"));
        assert!(output.contains("[1]:"));
    }

    #[test]
    fn test_inspect_value_array_large() {
        let items: Vec<Value> = (0..15).map(Value::Integer).collect();
        let arr = Value::Array(items.into());
        let output = CommandRegistry::inspect_value(&arr);
        assert!(output.contains("Length: 15"));
        assert!(output.contains("... and 5 more"));
    }

    #[test]
    fn test_inspect_value_tuple() {
        let tuple = Value::Tuple(vec![Value::Integer(1), Value::String("a".into())].into());
        let output = CommandRegistry::inspect_value(&tuple);
        assert!(output.contains("Type: Tuple"));
        assert!(output.contains("Length: 2"));
    }

    #[test]
    fn test_inspect_value_object() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), Value::Integer(1));
        let obj = Value::Object(Arc::new(map));
        let output = CommandRegistry::inspect_value(&obj);
        assert!(output.contains("Type: Object"));
        assert!(output.contains("Fields: 1"));
        assert!(output.contains("x:"));
    }

    #[test]
    fn test_inspect_value_object_mut() {
        let mut map = HashMap::new();
        map.insert("y".to_string(), Value::Integer(2));
        let obj = Value::ObjectMut(Arc::new(Mutex::new(map)));
        let output = CommandRegistry::inspect_value(&obj);
        assert!(output.contains("Fields: 1"));
        assert!(output.contains("y:"));
    }

    #[test]
    fn test_inspect_value_range() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(10)),
            inclusive: true,
        };
        let output = CommandRegistry::inspect_value(&range);
        assert!(output.contains("Type: Range"));
        assert!(output.contains("Start:"));
        assert!(output.contains("End:"));
        assert!(output.contains("Inclusive: true"));
    }

    #[test]
    fn test_inspect_value_enum_variant() {
        let variant = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };
        let output = CommandRegistry::inspect_value(&variant);
        assert!(output.contains("Type: Enum"));
        assert!(output.contains("Variant: None"));
    }

    #[test]
    fn test_inspect_value_builtin_function() {
        let output = CommandRegistry::inspect_value(&Value::BuiltinFunction("print".to_string()));
        assert!(output.contains("Type: BuiltinFunction"));
        assert!(output.contains("builtin: print"));
    }

    #[test]
    fn test_inspect_value_struct() {
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Value::Integer(10));
        let s = Value::Struct {
            name: "Point".to_string(),
            fields: Arc::new(fields),
        };
        let output = CommandRegistry::inspect_value(&s);
        assert!(output.contains("Type: Struct"));
        assert!(output.contains("Struct: Point"));
        assert!(output.contains("Fields: 1"));
    }

    #[test]
    fn test_inspect_value_class() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String("test".into()));
        let c = Value::Class {
            class_name: "MyClass".to_string(),
            fields: Arc::new(RwLock::new(fields)),
            methods: Arc::new(HashMap::new()),
        };
        let output = CommandRegistry::inspect_value(&c);
        assert!(output.contains("Type: Class"));
        assert!(output.contains("Class: MyClass"));
        assert!(output.contains("Fields: 1"));
        assert!(output.contains("Methods: 0"));
    }

    #[test]
    fn test_inspect_value_atom() {
        let output = CommandRegistry::inspect_value(&Value::Atom("ok".to_string()));
        assert!(output.contains("Type: Atom"));
        assert!(output.contains(":ok"));
    }

    #[test]
    fn test_inspect_value_dataframe() {
        use crate::runtime::DataFrameColumn;
        let columns = vec![DataFrameColumn {
            name: "col1".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2)],
        }];
        let df = Value::DataFrame { columns };
        let output = CommandRegistry::inspect_value(&df);
        assert!(output.contains("Columns: 1"));
        assert!(output.contains("Rows: 2"));
    }

    #[test]
    fn test_inspect_value_closure() {
        use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
        use std::cell::RefCell;
        use std::rc::Rc;

        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Arc::new(Expr::new(
                ExprKind::Literal(Literal::Integer(0, None)),
                Span::default(),
            )),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        let output = CommandRegistry::inspect_value(&closure);
        assert!(output.contains("Type: Function"));
        assert!(output.contains("<function>"));
    }

    // estimate_value_memory for closure
    #[test]
    fn test_estimate_value_memory_closure() {
        use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
        use std::cell::RefCell;
        use std::rc::Rc;

        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Arc::new(Expr::new(
                ExprKind::Literal(Literal::Integer(0, None)),
                Span::default(),
            )),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        let size = CommandRegistry::estimate_value_memory(&closure);
        assert!(size >= 1 + 128); // "x".len() + 128 base overhead
    }

    // estimate_value_memory for dataframe
    #[test]
    fn test_estimate_value_memory_dataframe() {
        use crate::runtime::DataFrameColumn;
        let columns = vec![DataFrameColumn {
            name: "col1".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let df = Value::DataFrame { columns };
        let size = CommandRegistry::estimate_value_memory(&df);
        assert!(size > 0);
    }

    // value_type_name for dataframe
    #[test]
    fn test_value_type_name_dataframe() {
        let df = Value::DataFrame { columns: vec![] };
        assert_eq!(CommandRegistry::value_type_name(&df), "DataFrame");
    }

    // value_type_name for closure
    #[test]
    fn test_value_type_name_closure() {
        use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
        use std::cell::RefCell;
        use std::rc::Rc;

        let closure = Value::Closure {
            params: vec![],
            body: Arc::new(Expr::new(
                ExprKind::Literal(Literal::Integer(0, None)),
                Span::default(),
            )),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        assert_eq!(CommandRegistry::value_type_name(&closure), "Function");
    }
}
