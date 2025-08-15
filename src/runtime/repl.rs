use crate::{Parser, Transpiler};
use anyhow::{Context, Result};
use colored::*;
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
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history
        rl.save_history(&history_path)?;
        Ok(())
    }

    /// Handle REPL commands
    fn handle_command(&mut self, cmd: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts.first().copied() {
            Some(":help") | Some(":h") => {
                self.print_help();
                Ok(true)
            }
            Some(":quit") | Some(":q") => Ok(false),
            Some(":type") | Some(":t") => {
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
                self.show_history();
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
                println!("Unknown command: {}", cmd);
                println!("Type :help for available commands");
                Ok(true)
            }
        }
    }

    /// Print help message
    fn print_help(&self) {
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
    pub fn eval(&mut self, input: &str) -> Result<String> {
        // Parse the input
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;

        // Transpile to Rust
        let rust_code = self
            .transpiler
            .transpile(&ast)
            .context("Failed to transpile to Rust")?;

        // For now, just compile and run simple expressions
        // In a real implementation, we'd handle definitions separately
        self.session_counter += 1;
        let session_name = format!("ruchy_repl_{}", self.session_counter);

        // Create a complete Rust program
        let full_program = format!(
            r#"
fn main() {{
    {}
    let result = {{{}}};
    println!("{{:?}}", result);
}}
"#,
            self.definitions.join("\n"),
            rust_code
        );

        // Write to temporary file
        let rust_file = self.temp_dir.join(format!("{}.rs", session_name));
        fs::write(&rust_file, full_program)?;

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

        // Return output
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Show type of expression (placeholder)
    fn show_type(&self, expr: &str) -> Result<()> {
        // In a real implementation, we'd have type inference
        println!(
            "{}: {}",
            expr,
            "<type inference not yet implemented>".bright_black()
        );
        Ok(())
    }

    /// Show AST of expression
    fn show_ast(&self, input: &str) -> Result<()> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        println!("{:#?}", ast);
        Ok(())
    }

    /// Show Rust transpilation
    fn show_rust(&self, input: &str) -> Result<()> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let rust_code = self.transpiler.transpile_to_string(&ast)?;
        println!("{}", rust_code.bright_blue());
        Ok(())
    }

    /// Clear the session
    fn clear_session(&mut self) {
        self.history.clear();
        self.definitions.clear();
        self.bindings.clear();
        self.session_counter = 0;
    }

    /// Show session history
    fn show_history(&self) {
        if self.history.is_empty() {
            println!("{}", "No history yet".bright_black());
        } else {
            for (i, entry) in self.history.iter().enumerate() {
                println!("{}: {}", i + 1, entry);
            }
        }
    }

    /// Save session to file
    fn save_session(&self, filename: &str) -> Result<()> {
        let content = self.history.join("\n");
        fs::write(filename, content)?;
        println!("Session saved to {}", filename.bright_green());
        Ok(())
    }

    /// Load session from file
    fn load_session(&mut self, filename: &str) -> Result<()> {
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
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().expect("Failed to create REPL")
    }
}
