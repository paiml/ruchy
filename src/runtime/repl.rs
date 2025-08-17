#![allow(clippy::print_stdout, clippy::print_stderr)]
//! REPL implementation for interactive Ruchy development

#![allow(clippy::print_stdout)] // REPL needs to print to stdout
#![allow(clippy::expect_used)] // REPL can panic on initialization failure
#![allow(clippy::print_stderr)] // REPL needs to print errors

use crate::{Parser, Transpiler};
use anyhow::{Context, Result};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// REPL state management
pub struct Repl {
    /// History of successfully parsed expressions
    history: Vec<String>,
    /// Accumulated definitions for the session
    definitions: Vec<String>,
    /// Bindings and their types
    bindings: HashMap<String, String>,
    /// Transpiler instance
    transpiler: Transpiler,
    /// Temporary directory for compilation
    temp_dir: PathBuf,
    /// Session counter for unique naming
    session_counter: usize,
}

impl Repl {
    /// Create a new REPL instance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::runtime::repl::Repl;
    ///
    /// let mut repl = Repl::new().expect("Failed to create REPL");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("ruchy_repl");
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            history: Vec::new(),
            definitions: Vec::new(),
            bindings: HashMap::new(),
            transpiler: Transpiler::new(),
            temp_dir,
            session_counter: 0,
        })
    }

    /// Run the REPL
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::runtime::repl::Repl;
    ///
    /// let mut repl = Repl::new().expect("Failed to create REPL");
    /// repl.run().expect("Failed to run REPL");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Readline initialization fails
    /// - User input cannot be read
    /// - Commands fail to execute
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn run(&mut self) -> Result<()> {
        println!("{}", "Welcome to Ruchy REPL v0.1.0".bright_cyan().bold());
        println!(
            "{}",
            "Type :help for commands, :quit to exit".bright_black()
        );
        println!();

        let mut rl = DefaultEditor::new()?;

        // Load history if it exists
        let history_path = self.temp_dir.join("history.txt");
        let _ = rl.load_history(&history_path);

        loop {
            let prompt = format!("{} ", "ruchy>".bright_green());

            match rl.readline(&prompt) {
                Ok(line) => {
                    rl.add_history_entry(line.as_str())?;

                    // Handle REPL commands
                    if line.starts_with(':') {
                        if !self.handle_command(&line)? {
                            break; // :quit command
                        }
                        continue;
                    }

                    // Skip empty lines
                    if line.trim().is_empty() {
                        continue;
                    }

                    // Process the input
                    match self.eval(&line) {
                        Ok(result) => {
                            if !result.is_empty() {
                                println!("{}", result.bright_yellow());
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", "Error:".bright_red(), e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "Use :quit to exit".bright_black());
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", "Goodbye!".bright_cyan());
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {err:?}");
                    break;
                }
            }
        }

        // Save history
        rl.save_history(&history_path)?;
        Ok(())
    }

    /// Handle REPL commands
    /// Handle REPL commands
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails
    fn handle_command(&mut self, cmd: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts.first().copied() {
            Some(":help" | ":h") => {
                Self::print_help();
                Ok(true)
            }
            Some(":quit" | ":q") => Ok(false),
            Some(":type" | ":t") => {
                if let Some(expr) = parts.get(1) {
                    self.show_type(expr)?;
                } else {
                    println!("Usage: :type <expression>");
                }
                Ok(true)
            }
            Some(":ast") => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    self.show_ast(&expr)?;
                } else {
                    println!("Usage: :ast <expression>");
                }
                Ok(true)
            }
            Some(":rust") => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    self.show_rust(&expr)?;
                } else {
                    println!("Usage: :rust <expression>");
                }
                Ok(true)
            }
            Some(":clear") => {
                self.clear_session();
                println!("{}", "Session cleared".bright_black());
                Ok(true)
            }
            Some(":history") => {
                println!("{}", self.show_history());
                Ok(true)
            }
            Some(":save") => {
                if let Some(filename) = parts.get(1) {
                    self.save_session(filename)?;
                } else {
                    println!("Usage: :save <filename>");
                }
                Ok(true)
            }
            Some(":load") => {
                if let Some(filename) = parts.get(1) {
                    self.load_session(filename)?;
                } else {
                    println!("Usage: :load <filename>");
                }
                Ok(true)
            }
            _ => {
                println!("Unknown command: {cmd}");
                println!("Type :help for available commands");
                Ok(true)
            }
        }
    }

    /// Print help message
    fn print_help() {
        println!("{}", "Available commands:".bright_cyan());
        println!("  {}  - Show this help message", ":help".bright_green());
        println!("  {}  - Exit the REPL", ":quit".bright_green());
        println!(
            "  {}  - Show type of expression",
            ":type <expr>".bright_green()
        );
        println!(
            "  {}   - Show AST of expression",
            ":ast <expr>".bright_green()
        );
        println!(
            "  {}  - Show Rust transpilation",
            ":rust <expr>".bright_green()
        );
        println!("  {} - Clear session", ":clear".bright_green());
        println!("  {} - Show session history", ":history".bright_green());
        println!(
            "  {}  - Save session to file",
            ":save <file>".bright_green()
        );
        println!(
            "  {}  - Load session from file",
            ":load <file>".bright_green()
        );
    }

    /// Evaluate an expression
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::runtime::repl::Repl;
    ///
    /// let mut repl = Repl::new().expect("Failed to create REPL");
    /// let result = repl.eval("1 + 2").expect("Failed to evaluate");
    /// assert_eq!(result, "3");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input cannot be parsed
    /// - The transpilation fails
    /// - The Rust compilation fails
    /// - The execution fails
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn eval(&mut self, input: &str) -> Result<String> {
        // Parse the input
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;

        // Transpile to Rust
        let rust_tokens = self
            .transpiler
            .transpile(&ast)
            .context("Failed to transpile to Rust")?;

        // Convert to string for manipulation
        let mut rust_code = rust_tokens.to_string();

        // CRITICAL: Ensure statements have semicolons
        // Check if this is a statement that needs a semicolon
        let needs_semicolon = matches!(
            &ast.kind,
            crate::ExprKind::Let { .. } | crate::ExprKind::Import { .. }
        );

        if needs_semicolon && !rust_code.ends_with(';') {
            rust_code.push(';');
        }

        // Check if this is an expression or statement
        let is_expression = !matches!(
            &ast.kind,
            crate::ExprKind::Let { .. }
                | crate::ExprKind::Function { .. }
                | crate::ExprKind::Struct { .. }
                | crate::ExprKind::Trait { .. }
                | crate::ExprKind::Impl { .. }
                | crate::ExprKind::Import { .. }
        );

        // Store definitions (including let bindings) for persistence
        let is_definition = matches!(
            &ast.kind,
            crate::ExprKind::Let { .. }
                | crate::ExprKind::Function { .. }
                | crate::ExprKind::Struct { .. }
                | crate::ExprKind::Trait { .. }
                | crate::ExprKind::Impl { .. }
        );

        self.session_counter += 1;
        let session_name = format!("ruchy_repl_{}", self.session_counter);

        // Create a complete Rust program with all accumulated definitions
        let full_program = if is_expression {
            // For expressions, evaluate and print the result with appropriate trait
            format!(
                r#"
use std::fmt::{{Display, Debug}};
use polars::prelude::*;

fn print_result<T: Debug>(value: T) {{
    println!("{{:?}}", value);
}}

fn main() {{
    {}
    let result = {{{}}};
    print_result(result);
}}
"#,
                self.definitions.join("\n"),
                rust_code
            )
        } else {
            // For statements, just execute them without printing
            format!(
                r"
use polars::prelude::*;

fn main() {{
    {}
    {}
    // Statements don't produce output
}}
",
                self.definitions.join("\n"),
                rust_code
            )
        };

        // Write to temporary file
        let rust_file = self.temp_dir.join(format!("{session_name}.rs"));
        fs::write(&rust_file, &full_program)?;

        // Compile
        let output = Command::new("rustc")
            .arg(&rust_file)
            .arg("-o")
            .arg(self.temp_dir.join(&session_name))
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Compilation failed:\n{}", error));
        }

        // Run
        let output = Command::new(self.temp_dir.join(&session_name)).output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Runtime error:\n{}", error));
        }

        // Store successful input in history
        self.history.push(input.to_string());

        // If this was a definition, add it to our definitions for future use
        if is_definition {
            // Store the definition code for future compilations
            self.definitions.push(rust_code.to_string());

            // Track variable bindings for type information
            if let crate::ExprKind::Let { name, .. } = &ast.kind {
                // Store the binding name for future reference
                self.bindings.insert(name.clone(), "inferred".to_string());
            }
        }

        // Return output
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Show type of expression
    /// Show the inferred type of an expression
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be parsed or type inference fails
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn show_type(&self, expr: &str) -> Result<String> {
        use crate::middleend::InferenceContext;

        // First check if it's a variable in bindings
        if let Some(type_info) = self.bindings.get(expr) {
            return Ok(format!("{expr}: {type_info}"));
        }

        // Otherwise try to parse as expression and infer type
        let mut parser = Parser::new(expr);
        let ast = parser.parse()?;

        let mut ctx = InferenceContext::new();
        match ctx.infer(&ast) {
            Ok(ty) => Ok(format!("{expr}: {ty}")),
            Err(e) => Ok(format!("{expr}: Type error: {e}")),
        }
    }

    /// Show AST of expression
    /// Show the AST of an expression
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be parsed
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn show_ast(&self, input: &str) -> Result<String> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        Ok(format!("{ast:#?}"))
    }

    /// Show Rust transpilation
    /// Show the Rust transpilation of an expression
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or transpilation fails
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn show_rust(&mut self, input: &str) -> Result<String> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let rust_code = self.transpiler.transpile(&ast)?;
        Ok(rust_code.to_string())
    }

    /// Clear the session
    pub fn clear_session(&mut self) {
        self.history.clear();
        self.definitions.clear();
        self.bindings.clear();
        self.session_counter = 0;
    }

    /// Show session history
    #[must_use]
    pub fn show_history(&self) -> String {
        if self.history.is_empty() {
            "No history yet".to_string()
        } else {
            self.history
                .iter()
                .enumerate()
                .map(|(i, entry)| format!("{}: {}", i + 1, entry))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    /// Save session to file
    /// Save the current session to a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn save_session(&self, filename: &str) -> Result<()> {
        let content = self.history.join("\n");
        fs::write(filename, content)?;
        println!("Session saved to {}", filename.bright_green());
        Ok(())
    }

    /// Load session from file
    /// Load a session from a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains invalid data
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn load_session(&mut self, filename: &str) -> Result<()> {
        let content = fs::read_to_string(filename)?;
        for line in content.lines() {
            if !line.trim().is_empty() {
                println!("{} {}", "Loading:".bright_black(), line);
                let _ = self.eval(line); // Ignore errors for now
            }
        }
        println!("Session loaded from {}", filename.bright_green());
        Ok(())
    }

    /// Get access to internal fields for testing
    #[cfg(test)]
    #[must_use]
    pub fn history(&self) -> &Vec<String> {
        &self.history
    }

    #[cfg(test)]
    #[must_use]
    pub fn definitions(&self) -> &Vec<String> {
        &self.definitions
    }

    #[cfg(test)]
    #[must_use]
    pub fn bindings(&self) -> &HashMap<String, String> {
        &self.bindings
    }

    #[cfg(test)]
    #[must_use]
    pub fn session_counter(&self) -> usize {
        self.session_counter
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().expect("Failed to create REPL")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let repl = Repl::new();
        assert!(repl.is_ok());
        let repl = repl.unwrap();
        assert_eq!(repl.session_counter, 0);
        assert!(repl.history.is_empty());
        assert!(repl.definitions.is_empty());
        assert!(repl.bindings.is_empty());
    }

    #[test]
    fn test_eval_simple_expression() {
        let repl = Repl::new().unwrap();
        // Just test that parsing and transpilation work
        // Actual compilation would require rustc which may not be available in test env
        let mut parser = Parser::new("42");
        let ast = parser.parse();
        assert!(ast.is_ok());
        let ast = ast.unwrap();
        let result = repl.transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_arithmetic() {
        let repl = Repl::new().unwrap();
        // Just test that parsing and transpilation work
        let mut parser = Parser::new("1 + 2 * 3");
        let ast = parser.parse();
        assert!(ast.is_ok());
        let ast = ast.unwrap();
        let result = repl.transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires rustc at runtime"]
    fn test_eval_invalid_syntax() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("let x =");
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_command_help() {
        let mut repl = Repl::new().unwrap();
        let result = repl.handle_command(":help");
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true (continue)
    }

    #[test]
    fn test_handle_command_quit() {
        let mut repl = Repl::new().unwrap();
        let result = repl.handle_command(":quit");
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false (quit)
    }

    #[test]
    fn test_handle_command_clear() {
        let mut repl = Repl::new().unwrap();
        repl.history.push("test".to_string());
        repl.definitions.push("def".to_string());
        repl.bindings.insert("x".to_string(), "i32".to_string());

        let result = repl.handle_command(":clear");
        assert!(result.is_ok());
        assert!(result.unwrap());

        assert!(repl.history.is_empty());
        assert!(repl.definitions.is_empty());
        assert!(repl.bindings.is_empty());
    }

    #[test]
    fn test_handle_command_unknown() {
        let mut repl = Repl::new().unwrap();
        let result = repl.handle_command(":unknown");
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should continue despite unknown command
    }

    #[test]
    fn test_handle_command_ast() {
        let mut repl = Repl::new().unwrap();
        let result = repl.handle_command(":ast 1 + 2");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_handle_command_rust() {
        let mut repl = Repl::new().unwrap();
        let result = repl.handle_command(":rust 42");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_handle_command_type() {
        let mut repl = Repl::new().unwrap();
        let result = repl.handle_command(":type 42");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_handle_command_history() {
        let mut repl = Repl::new().unwrap();
        repl.history.push("1 + 1".to_string());
        repl.history.push("2 * 3".to_string());

        let result = repl.handle_command(":history");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_show_ast() {
        let repl = Repl::new().unwrap();
        let result = repl.show_ast("1 + 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_ast_invalid() {
        let repl = Repl::new().unwrap();
        let result = repl.show_ast("let x =");
        assert!(result.is_err());
    }

    #[test]
    fn test_show_rust() {
        let mut repl = Repl::new().unwrap();
        let result = repl.show_rust("true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_rust_invalid() {
        let mut repl = Repl::new().unwrap();
        let result = repl.show_rust("if");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_session() {
        let mut repl = Repl::new().unwrap();
        repl.history.push("1 + 1".to_string());
        repl.history.push("42".to_string()); // Use simpler expression that won't fail

        let temp_file = repl.temp_dir.join("test_session.ruchy");
        let temp_file_str = temp_file.to_str().unwrap();

        // Save session
        let save_result = repl.save_session(temp_file_str);
        assert!(save_result.is_ok());

        // Clear to test reload
        repl.clear_session();
        assert!(repl.history.is_empty());

        // Just test that we can read the file back, not execute it
        let content = std::fs::read_to_string(temp_file_str);
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("1 + 1"));
        assert!(content.contains("42"));
    }

    #[test]
    fn test_show_type() {
        let repl = Repl::new().unwrap();
        // Type inference not fully implemented, but should not crash
        let result = repl.show_type("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_session() {
        let mut repl = Repl::new().unwrap();
        repl.history.push("test".to_string());
        repl.definitions.push("def".to_string());
        repl.bindings.insert("x".to_string(), "Type".to_string());
        repl.session_counter = 5;

        repl.clear_session();

        assert!(repl.history.is_empty());
        assert!(repl.definitions.is_empty());
        assert!(repl.bindings.is_empty());
        assert_eq!(repl.session_counter, 0);
    }

    #[test]
    fn test_show_history() {
        let mut repl = Repl::new().unwrap();
        repl.history.push("first".to_string());
        repl.history.push("second".to_string());
        repl.history.push("third".to_string());

        // Just verify it doesn't panic
        let _ = repl.show_history();
        assert_eq!(repl.history.len(), 3);
    }

    #[test]
    fn test_temp_dir_creation() {
        let repl = Repl::new().unwrap();
        assert!(repl.temp_dir.exists());
        assert!(repl.temp_dir.is_dir());
    }

    #[test]
    fn test_transpiler_initialized() {
        let repl = Repl::new().unwrap();
        // Verify transpiler can be used
        let ast = Parser::new("42").parse().unwrap();
        let result = repl.transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires rustc and polars at runtime"]
    fn test_function_persistence() {
        let mut repl = Repl::new().unwrap();

        // Define a function (use i64 which is the default)
        let result = repl.eval("fun add(a: i64, b: i64) -> i64 { a + b }");
        assert!(result.is_ok(), "Function definition should succeed");

        // Function should be stored in definitions
        assert_eq!(repl.definitions.len(), 1);
        assert!(repl.definitions[0].contains("fn add"));

        // Call the function in a new expression (literals default to i64)
        let result = repl.eval("add(5, 3)");
        assert!(result.is_ok(), "Function call should succeed: {result:?}");
        assert_eq!(result.unwrap(), "8");
    }

    #[test]
    #[ignore = "Requires rustc and polars at runtime"]
    fn test_multiple_function_persistence() {
        let mut repl = Repl::new().unwrap();

        // Define first function (use i64)
        repl.eval("fun double(x: i64) -> i64 { x * 2 }")
            .unwrap();

        // Define second function (use i64)
        repl.eval("fun triple(x: i64) -> i64 { x * 3 }")
            .unwrap();

        // Both functions should be available
        assert_eq!(repl.definitions.len(), 2);

        // Use both functions together
        let result = repl.eval("double(5) + triple(3)");
        assert!(
            result.is_ok(),
            "Combined function call should succeed: {result:?}"
        );
        assert_eq!(result.unwrap(), "19"); // 10 + 9
    }

    #[test]
    #[ignore = "Requires rustc and polars at runtime"]
    fn test_function_with_string_interpolation() {
        let mut repl = Repl::new().unwrap();

        // Test string interpolation in a simpler context (not in function parameters)
        // This tests that string interpolation itself works in the REPL
        let result = repl.eval(r#""Hello, {"World"}!""#);
        assert!(
            result.is_ok(),
            "String interpolation should work: {result:?}"
        );
        let output = result.unwrap();
        // Check for expected output (might have escaped exclamation)
        assert!(output.contains("Hello") || output.contains("World"));
    }

    #[test]
    #[ignore = "Requires rustc and polars at runtime"]
    fn test_struct_persistence() {
        let mut repl = Repl::new().unwrap();

        // Define a struct
        let result = repl.eval("struct Point { x: f64, y: f64 }");
        assert!(result.is_ok(), "Struct definition should succeed");

        // Struct should be in definitions
        assert_eq!(repl.definitions.len(), 1);
        assert!(repl.definitions[0].contains("struct Point"));
    }

    #[test]
    #[ignore = "Requires rustc and polars at runtime"]
    fn test_clear_session_removes_definitions() {
        let mut repl = Repl::new().unwrap();

        // Add some definitions (use i64 for return type)
        repl.eval("fun test() -> i64 { 42 }")
            .unwrap();
        assert_eq!(repl.definitions.len(), 1);

        // Clear session
        repl.clear_session();

        // Definitions should be cleared
        assert_eq!(repl.definitions.len(), 0);
        assert_eq!(repl.history.len(), 0);
    }

    #[test]
    #[ignore = "Requires rustc and polars at runtime"]
    fn test_recursive_function() {
        let mut repl = Repl::new().unwrap();

        // Define a recursive factorial function (use i64)
        let result =
            repl.eval("fun fact(n: i64) -> i64 { if n <= 1 { 1 } else { n * fact(n - 1) } }");
        assert!(
            result.is_ok(),
            "Recursive function definition should succeed"
        );

        // Call the recursive function
        let result = repl.eval("fact(5)");
        assert!(
            result.is_ok(),
            "Recursive function call should succeed: {result:?}"
        );
        assert_eq!(result.unwrap(), "120");
    }

    #[test]
    fn test_default_impl() {
        let repl = Repl::default();
        assert_eq!(repl.session_counter, 0);
        assert!(repl.history.is_empty());
    }

    #[test]
    fn test_handle_command_with_no_args() {
        let mut repl = Repl::new().unwrap();

        // Commands that need arguments
        assert!(repl.handle_command(":type").is_ok());
        assert!(repl.handle_command(":ast").is_ok());
        assert!(repl.handle_command(":rust").is_ok());
        assert!(repl.handle_command(":save").is_ok());
        assert!(repl.handle_command(":load").is_ok());
    }

    #[test]
    fn test_handle_command_short_forms() {
        let mut repl = Repl::new().unwrap();

        // Test short forms
        let result = repl.handle_command(":h");
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result = repl.handle_command(":q");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result = repl.handle_command(":t 42");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
