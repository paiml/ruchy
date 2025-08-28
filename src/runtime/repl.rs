//! REPL implementation for interactive Ruchy development
//!
//! Production-grade REPL with resource bounds, error recovery, and grammar coverage
#![allow(clippy::cast_sign_loss)]
//!
//! # Examples
//!
//! ```
//! use ruchy::runtime::Repl;
//!
//! let mut repl = Repl::new().unwrap();
//!
//! // Evaluate arithmetic
//! let result = repl.eval("2 + 2").unwrap();
//! assert_eq!(result, "4");
//!
//! // Define variables
//! repl.eval("let x = 10").unwrap();
//! let result = repl.eval("x * 2").unwrap();
//! assert_eq!(result, "20");
//! ```
//!
//! # One-liner evaluation
//!
//! ```
//! use ruchy::runtime::Repl;
//! use std::time::{Duration, Instant};
//!
//! let mut repl = Repl::new().unwrap();
//! let deadline = Some(Instant::now() + Duration::from_millis(100));
//!
//! let value = repl.evaluate_expr_str("5 + 3", deadline).unwrap();
//! assert_eq!(value.to_string(), "8");
//! ```

#![allow(clippy::print_stdout)] // REPL needs to print to stdout
#![allow(clippy::print_stderr)] // REPL needs to print errors
#![allow(clippy::expect_used)] // REPL can panic on initialization failure

use crate::frontend::ast::{
    BinaryOp, Expr, ExprKind, ImportItem, Literal, MatchArm, Pattern, PipelineStage, Span, UnaryOp,
};
use crate::{Parser, Transpiler};
use anyhow::{bail, Context, Result};
use colored::Colorize;

mod display;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, EditMode, Helper};
use std::collections::{HashMap, HashSet};
use std::fmt;
#[allow(unused_imports)]
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// Runtime value for evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    DataFrame {
        columns: Vec<DataFrameColumn>,
    },
    Object(HashMap<String, Value>),
    HashMap(HashMap<Value, Value>),
    HashSet(HashSet<Value>),
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    EnumVariant {
        enum_name: String,
        variant_name: String,
        data: Option<Vec<Value>>,
    },
    Unit,
}

/// `DataFrame` column representation for pretty printing
#[derive(Debug, Clone, PartialEq)]
pub struct DataFrameColumn {
    pub name: String,
    pub values: Vec<Value>,
}

// Manual Eq implementation for Value
impl Eq for Value {}

// Manual Hash implementation for Value
impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Int(n) => n.hash(state),
            Value::Float(f) => {
                // Hash floats by their bit representation to handle NaN properly
                f.to_bits().hash(state);
            },
            Value::String(s) => s.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Char(c) => c.hash(state),
            Value::List(items) => {
                for item in items {
                    item.hash(state);
                }
            },
            Value::Tuple(items) => {
                for item in items {
                    item.hash(state);
                }
            },
            // Functions, DataFrames, Objects, HashMaps, and HashSets are not hashable
            // We'll just hash their discriminant
            Value::Function { name, .. } => name.hash(state),
            Value::Lambda { .. } => "lambda".hash(state),
            Value::DataFrame { .. } => "dataframe".hash(state),
            Value::Object(_) => "object".hash(state),
            Value::HashMap(_) => "hashmap".hash(state),
            Value::HashSet(_) => "hashset".hash(state),
            Value::Range { start, end, inclusive } => {
                start.hash(state);
                end.hash(state);
                inclusive.hash(state);
            },
            Value::EnumVariant { enum_name, variant_name, data } => {
                enum_name.hash(state);
                variant_name.hash(state);
                if let Some(d) = data {
                    for item in d {
                        item.hash(state);
                    }
                }
            },
            Value::Unit => "unit".hash(state),
        }
    }
}

// Display implementations moved to repl_display.rs

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Char(c) => write!(f, "'{c}'"),
            Value::List(items) => Self::fmt_list(f, items),
            Value::Tuple(items) => Self::fmt_tuple(f, items),
            Value::Function { name, params, .. } => {
                write!(f, "fn {}({})", name, params.join(", "))
            }
            Value::Lambda { params, .. } => {
                write!(f, "|{}| <closure>", params.join(", "))
            }
            Value::DataFrame { columns } => Self::format_dataframe(f, columns),
            Value::Object(map) => Self::fmt_object(f, map),
            Value::HashMap(map) => Self::fmt_hashmap(f, map),
            Value::HashSet(set) => Self::fmt_hashset(f, set),
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    write!(f, "{start}..={end}")
                } else {
                    write!(f, "{start}..{end}")
                }
            }
            Value::EnumVariant {
                enum_name,
                variant_name,
                data,
            } => Self::fmt_enum_variant(f, enum_name, variant_name, data.as_deref()),
            Value::Unit => write!(f, "()"),
        }
    }
}

/// REPL mode determines how input is processed
#[derive(Debug, Clone, PartialEq)]
pub enum ReplMode {
    Normal,  // Standard Ruchy evaluation
    Shell,   // Execute everything as shell commands
    Pkg,     // Package management mode
    Help,    // Help documentation mode
    Sql,     // SQL query mode
    Math,    // Mathematical expression mode
    Debug,   // Debug mode with extra info
    Time,    // Time mode showing execution timing
}

impl ReplMode {
    fn prompt(&self) -> String {
        match self {
            ReplMode::Normal => "ruchy> ".to_string(),
            ReplMode::Shell => "shell> ".to_string(),
            ReplMode::Pkg => "pkg> ".to_string(),
            ReplMode::Help => "help> ".to_string(),
            ReplMode::Sql => "sql> ".to_string(),
            ReplMode::Math => "math> ".to_string(),
            ReplMode::Debug => "debug> ".to_string(),
            ReplMode::Time => "time> ".to_string(),
        }
    }
}

/// Debug information for post-mortem analysis
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// The expression that caused the error
    pub expression: String,
    /// The error message
    pub error_message: String,
    /// Stack trace at time of error
    pub stack_trace: Vec<String>,
    /// Variable bindings at time of error
    pub bindings_snapshot: HashMap<String, Value>,
    /// Timestamp when error occurred
    pub timestamp: std::time::SystemTime,
}

/// REPL configuration
pub struct ReplConfig {
    /// Maximum memory for evaluation (default: 10MB)
    pub max_memory: usize,
    /// Timeout for evaluation (default: 100ms)
    pub timeout: Duration,
    /// Maximum stack depth (default: 1000)
    pub max_depth: usize,
    /// Enable debug mode
    pub debug: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            max_memory: 10 * 1024 * 1024, // 10MB
            timeout: Duration::from_millis(100),
            max_depth: 256, // Balance between safety and usability
            debug: false,
        }
    }
}

/// Tab completion helper for Ruchy REPL
#[derive(Default)]
struct RuchyCompleter {
    keywords: Vec<String>,
    builtin_functions: Vec<String>,
    list_methods: Vec<String>,
    string_methods: Vec<String>,
    commands: Vec<String>,
    // HashSets for O(1) lookups in highlighting
    keywords_set: HashSet<String>,
    builtin_functions_set: HashSet<String>,
}

impl RuchyCompleter {
    fn new() -> Self {
        let keywords = vec![
            "let".to_string(),
            "fn".to_string(),
            "if".to_string(),
            "else".to_string(),
            "match".to_string(),
            "for".to_string(),
            "while".to_string(),
            "loop".to_string(),
            "break".to_string(),
            "continue".to_string(),
            "return".to_string(),
            "true".to_string(),
            "false".to_string(),
            "struct".to_string(),
            "enum".to_string(),
            "trait".to_string(),
            "impl".to_string(),
            "mod".to_string(),
            "use".to_string(),
            "pub".to_string(),
            "mut".to_string(),
            "actor".to_string(),
            "receive".to_string(),
            "spawn".to_string(),
            "send".to_string(),
            "ask".to_string(),
            "df".to_string(),
            "Ok".to_string(),
            "Err".to_string(),
            "Some".to_string(),
            "None".to_string(),
        ];
        let builtin_functions = vec![
            "println".to_string(),
            "print".to_string(),
            "input".to_string(),
            "readline".to_string(),
            "assert".to_string(),
            "assert_eq".to_string(),
            "assert_ne".to_string(),
            "curry".to_string(),
            "uncurry".to_string(),
            "read_file".to_string(),
            "write_file".to_string(),
            "append_file".to_string(),
            "file_exists".to_string(),
            "delete_file".to_string(),
            "current_dir".to_string(),
            "env".to_string(),
            "set_env".to_string(),
            "args".to_string(),
            "HashMap".to_string(),
            "HashSet".to_string(),
            "Some".to_string(),
            "None".to_string(),
            "Option".to_string(),
            "Ok".to_string(),
            "Err".to_string(),
            "Result".to_string(),
        ];

        // Create HashSets for O(1) lookups
        let keywords_set = keywords.iter().cloned().collect();
        let builtin_functions_set = builtin_functions.iter().cloned().collect();

        Self {
            keywords,
            builtin_functions,
            list_methods: vec![
                "map".to_string(),
                "filter".to_string(),
                "reduce".to_string(),
                "len".to_string(),
                "length".to_string(),
                "head".to_string(),
                "first".to_string(),
                "tail".to_string(),
                "rest".to_string(),
                "last".to_string(),
                "reverse".to_string(),
                "sum".to_string(),
                "push".to_string(),
                "pop".to_string(),
            ],
            string_methods: vec![
                "len".to_string(),
                "length".to_string(),
                "upper".to_string(),
                "to_upper".to_string(),
                "lower".to_string(),
                "to_lower".to_string(),
                "trim".to_string(),
                "split".to_string(),
                "contains".to_string(),
                "starts_with".to_string(),
                "ends_with".to_string(),
                "replace".to_string(),
                "substring".to_string(),
                "repeat".to_string(),
                "chars".to_string(),
                "reverse".to_string(),
            ],
            commands: vec![
                ":help".to_string(),
                ":h".to_string(),
                ":quit".to_string(),
                ":q".to_string(),
                ":history".to_string(),
                ":clear".to_string(),
                ":reset".to_string(),
                ":bindings".to_string(),
                ":env".to_string(),
                ":type".to_string(),
                ":ast".to_string(),
                ":compile".to_string(),
                ":load".to_string(),
                ":save".to_string(),
                ":search".to_string(),
            ],
            keywords_set,
            builtin_functions_set,
        }
    }

    fn get_completions(
        &self,
        line: &str,
        pos: usize,
        bindings: &HashMap<String, Value>,
    ) -> Vec<String> {
        use std::collections::HashSet;

        let text_before_cursor = &line[..pos];

        // Extract the word being completed
        let word_start = text_before_cursor
            .rfind(|c: char| c.is_whitespace() || "()[]{},.;".contains(c))
            .map_or(0, |i| i + 1);
        let partial_word = &text_before_cursor[word_start..];

        // Complete commands (starting with :)
        if partial_word.starts_with(':') {
            return self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(partial_word))
                .cloned()
                .collect();
        }

        // Complete method calls (after a dot)
        if let Some(dot_pos) = text_before_cursor.rfind('.') {
            let object_name = &text_before_cursor[..dot_pos];
            let object_part = &text_before_cursor[..=dot_pos];
            let method_partial = &text_before_cursor[dot_pos + 1..];
            // Use HashSet to avoid O(n²) duplicates check
            let mut seen = HashSet::new();
            let mut completions = Vec::new();

            // Check if we have an object variable with fields
            if let Some(value) = bindings.get(object_name) {
                if let Value::Object(map) = value {
                    // Add object fields
                    for field_name in map.keys() {
                        if field_name.starts_with(method_partial) {
                            let full_completion = format!("{}{}", object_part, field_name);
                            if seen.insert(full_completion.clone()) {
                                completions.push(full_completion);
                            }
                        }
                    }
                }
            }

            // Add list methods
            for method in &self.list_methods {
                if method.starts_with(method_partial) {
                    let full_completion = format!("{}{}", object_part, method);
                    if seen.insert(full_completion.clone()) {
                        completions.push(full_completion);
                    }
                }
            }
            // Add string methods
            for method in &self.string_methods {
                if method.starts_with(method_partial) {
                    let full_completion = format!("{}{}", object_part, method);
                    if seen.insert(full_completion.clone()) {
                        completions.push(full_completion);
                    }
                }
            }
            completions.sort();
            return completions;
        }

        // Complete keywords, functions, and variables
        let mut completions = Vec::new();
        let partial_lower = partial_word.to_lowercase();

        completions.extend(
            self.keywords
                .iter()
                .filter(|kw| kw.to_lowercase().starts_with(&partial_lower))
                .cloned(),
        );

        completions.extend(
            self.builtin_functions
                .iter()
                .filter(|func| func.to_lowercase().starts_with(&partial_lower))
                .cloned(),
        );

        completions.extend(
            bindings
                .keys()
                .filter(|var| var.to_lowercase().starts_with(&partial_lower))
                .cloned(),
        );

        completions.sort();
        completions
    }

    /// Highlight Ruchy syntax with colors
    fn highlight_ruchy_syntax(&self, line: &str) -> String {
        let mut result = String::new();
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => self.highlight_string(&mut result, &mut chars),
                '\'' => self.highlight_char(&mut result, &mut chars),
                '0'..='9' => self.highlight_number(&mut result, &mut chars, ch),
                'a'..='z' | 'A'..='Z' | '_' => self.highlight_identifier(&mut result, &mut chars, ch),
                '/' if chars.peek() == Some(&'/') => {
                    self.highlight_comment(&mut result, &mut chars);
                    break;
                }
                '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' => {
                    result.push_str(&ch.to_string().bright_red().to_string());
                }
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    result.push_str(&ch.to_string().bright_white().bold().to_string());
                }
                ',' | ';' | ':' | '.' => {
                    result.push_str(&ch.to_string().bright_black().to_string());
                }
                _ => result.push(ch),
            }
        }

        result
    }

    fn highlight_string(&self, result: &mut String, chars: &mut std::iter::Peekable<std::str::Chars>) {
        result.push_str(&"\"".bright_green().to_string());
        while let Some(string_ch) = chars.next() {
            if string_ch == '"' {
                result.push_str(&"\"".bright_green().to_string());
                break;
            } else if string_ch == '\\' {
                result.push_str(&"\\".bright_green().to_string());
                if let Some(escaped) = chars.next() {
                    result.push_str(&escaped.to_string().bright_green().to_string());
                }
            } else {
                result.push_str(&string_ch.to_string().bright_green().to_string());
            }
        }
    }

    fn highlight_char(&self, result: &mut String, chars: &mut std::iter::Peekable<std::str::Chars>) {
        result.push_str(&"'".bright_yellow().to_string());
        if let Some(char_ch) = chars.next() {
            result.push_str(&char_ch.to_string().bright_yellow().to_string());
            if let Some(quote) = chars.next() {
                if quote == '\'' {
                    result.push_str(&"'".bright_yellow().to_string());
                } else {
                    result.push(quote);
                }
            }
        }
    }

    fn highlight_number(&self, result: &mut String, chars: &mut std::iter::Peekable<std::str::Chars>, first_char: char) {
        let mut number = String::new();
        number.push(first_char);

        while let Some(&next_ch) = chars.peek() {
            if next_ch.is_ascii_digit() || next_ch == '.' || next_ch == '_' {
                number.push(chars.next().expect("Digit continuation expected"));
            } else {
                break;
            }
        }
        result.push_str(&number.bright_blue().to_string());
    }

    fn highlight_identifier(&self, result: &mut String, chars: &mut std::iter::Peekable<std::str::Chars>, first_char: char) {
        let mut identifier = String::new();
        identifier.push(first_char);

        while let Some(&next_ch) = chars.peek() {
            if next_ch.is_alphanumeric() || next_ch == '_' {
                identifier.push(chars.next().expect("Identifier continuation expected"));
            } else {
                break;
            }
        }

        let highlighted = if self.keywords_set.contains(&identifier) {
            identifier.bright_magenta().bold().to_string()
        } else if self.builtin_functions_set.contains(&identifier) {
            identifier.bright_cyan().to_string()
        } else {
            identifier
        };

        result.push_str(&highlighted);
    }

    fn highlight_comment(&self, result: &mut String, chars: &mut std::iter::Peekable<std::str::Chars>) {
        result.push_str(&"//".bright_black().to_string());
        chars.next(); // consume second '/'
        
        for comment_ch in chars.by_ref() {
            result.push_str(&comment_ch.to_string().bright_black().to_string());
        }
    }
}

impl Completer for RuchyCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        // For now, complete without variable bindings (basic completion only)
        let empty_bindings = HashMap::new();
        let completions = self.get_completions(line, pos, &empty_bindings);

        // Find the start position of the word being completed
        let text_before_cursor = &line[..pos];
        let word_start = text_before_cursor
            .rfind(|c: char| c.is_whitespace() || "()[]{},.;".contains(c))
            .map_or(0, |i| i + 1);

        Ok((word_start, completions))
    }
}

impl Helper for RuchyCompleter {}
impl Hinter for RuchyCompleter {
    type Hint = String;
}

impl Highlighter for RuchyCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        use std::borrow::Cow;

        // Simple syntax highlighting for Ruchy
        let highlighted = self.highlight_ruchy_syntax(line);
        Cow::Owned(highlighted)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        // Enable character-by-character highlighting
        true
    }
}

impl Validator for RuchyCompleter {}

/// Memory tracker for bounded allocation
struct MemoryTracker {
    max_size: usize,
    current: usize,
}

impl MemoryTracker {
    fn new(max_size: usize) -> Self {
        Self {
            max_size,
            current: 0,
        }
    }

    fn try_alloc(&mut self, size: usize) -> Result<()> {
        if self.current + size > self.max_size {
            bail!(
                "Memory limit exceeded: {} + {} > {}",
                self.current,
                size,
                self.max_size
            );
        }
        self.current += size;
        Ok(())
    }

    fn reset(&mut self) {
        self.current = 0;
    }
}

/// REPL state management with resource bounds
pub struct Repl {
    /// History of successfully parsed expressions
    history: Vec<String>,
    /// History of evaluation results (for _ and _n variables)
    result_history: Vec<Value>,
    /// Accumulated definitions for the session
    definitions: Vec<String>,
    /// Bindings and their types/values
    bindings: HashMap<String, Value>,
    /// Mutability tracking for bindings
    binding_mutability: HashMap<String, bool>,
    /// Impl methods: `Type::method` -> (params, body)
    impl_methods: HashMap<String, (Vec<String>, Box<Expr>)>,
    /// Enum definitions: `EnumName` -> list of variant names
    enum_definitions: HashMap<String, Vec<String>>,
    /// Transpiler instance
    transpiler: Transpiler,
    /// Temporary directory for compilation
    temp_dir: PathBuf,
    /// Session counter for unique naming
    session_counter: usize,
    /// Configuration
    config: ReplConfig,
    /// Memory tracker
    memory: MemoryTracker,
    /// O(1) in-memory module cache: path -> parsed functions
    /// Guarantees O(1) performance regardless of storage backend (EFS, NFS, etc)
    module_cache: HashMap<String, HashMap<String, Value>>,
    /// Current REPL mode
    mode: ReplMode,
    /// Debug information from last error
    last_error_debug: Option<DebugInfo>,
}

impl Repl {
    /// Create a new REPL instance with default config
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::Repl;
    ///
    /// let repl = Repl::new();
    /// assert!(repl.is_ok());
    /// ```
    pub fn new() -> Result<Self> {
        Self::with_config(ReplConfig::default())
    }

    /// Create a new REPL instance with custom config
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created
    pub fn with_config(config: ReplConfig) -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("ruchy_repl");
        fs::create_dir_all(&temp_dir)?;

        let memory = MemoryTracker::new(config.max_memory);

        let mut repl = Self {
            history: Vec::new(),
            result_history: Vec::new(),
            definitions: Vec::new(),
            bindings: HashMap::new(),
            binding_mutability: HashMap::new(),
            impl_methods: HashMap::new(),
            enum_definitions: HashMap::new(),
            transpiler: Transpiler::new(),
            temp_dir,
            session_counter: 0,
            config,
            memory,
            module_cache: HashMap::new(),
            mode: ReplMode::Normal,
            last_error_debug: None,
        };

        // Initialize built-in types
        repl.init_builtins();

        Ok(repl)
    }

    /// Initialize built-in enum types (Option, Result)
    fn init_builtins(&mut self) {
        // Register Option enum
        self.enum_definitions.insert(
            "Option".to_string(),
            vec!["None".to_string(), "Some".to_string()],
        );

        // Register Result enum
        self.enum_definitions.insert(
            "Result".to_string(),
            vec!["Ok".to_string(), "Err".to_string()],
        );

        // Add Option and Result to definitions for transpiler
        self.definitions
            .push("enum Option<T> { None, Some(T) }".to_string());
        self.definitions
            .push("enum Result<T, E> { Ok(T), Err(E) }".to_string());
    }

    /// Evaluate an expression string and return the Value
    ///
    /// This is used for one-liner evaluation from CLI
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or evaluation fails
    pub fn evaluate_expr_str(&mut self, input: &str, deadline: Option<Instant>) -> Result<Value> {
        // Reset memory tracker for fresh evaluation
        self.memory.reset();

        // Track input memory
        self.memory.try_alloc(input.len())?;

        // Use provided deadline or default timeout
        let deadline = deadline.unwrap_or_else(|| Instant::now() + self.config.timeout);

        // Parse the input
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;

        // Check memory for AST
        self.memory.try_alloc(std::mem::size_of_val(&ast))?;

        // Evaluate the expression
        let value = self.evaluate_expr(&ast, deadline, 0)?;

        // Handle let bindings specially (for backward compatibility)
        if let ExprKind::Let { name, type_annotation: _, is_mutable, .. } = &ast.kind {
            self.create_binding(name.clone(), value.clone(), *is_mutable);
        }

        Ok(value)
    }

    /// Evaluate an expression with resource bounds
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Memory limit is exceeded
    /// - Timeout is reached
    /// - Stack depth limit is exceeded
    /// - Parse or evaluation fails
    pub fn eval(&mut self, input: &str) -> Result<String> {
        // Reset memory tracker for fresh evaluation
        self.memory.reset();

        // Track input memory
        self.memory.try_alloc(input.len())?;

        // Check for magic commands
        let trimmed = input.trim();
        
        // Handle mode-specific evaluation
        match self.mode {
            ReplMode::Shell if !trimmed.starts_with(':') => {
                // In shell mode, execute everything as shell commands unless it's a colon command
                return self.execute_shell_command(trimmed);
            }
            ReplMode::Pkg if !trimmed.starts_with(':') => {
                // In pkg mode, handle package commands
                return self.handle_pkg_command(trimmed);
            }
            ReplMode::Help if !trimmed.starts_with(':') => {
                // In help mode, show help for keywords
                return self.handle_help_command(trimmed);
            }
            ReplMode::Sql if !trimmed.starts_with(':') => {
                // In SQL mode, execute SQL queries
                return Ok(format!("SQL mode not yet implemented: {}", trimmed));
            }
            ReplMode::Math if !trimmed.starts_with(':') => {
                // In math mode, enhanced math evaluation
                return self.handle_math_command(trimmed);
            }
            ReplMode::Debug if !trimmed.starts_with(':') => {
                // In debug mode, evaluate with extra info
                return self.handle_debug_evaluation(trimmed);
            }
            ReplMode::Time if !trimmed.starts_with(':') => {
                // In time mode, evaluate with timing
                return self.handle_timed_evaluation(trimmed);
            }
            _ => {} // Normal mode or colon command - continue with regular processing
        }
        if trimmed.starts_with('%') {
            return self.handle_magic_command(trimmed);
        }

        // Check for REPL commands
        if trimmed.starts_with(':') {
            // Handle command and return result as string
            let (should_quit, output) = self.handle_command_with_output(trimmed)?;
            if should_quit {
                return Ok("Exiting REPL...".to_string());
            }
            return Ok(output);
        }
        
        // Check for shell commands
        if trimmed.starts_with('!') {
            return self.execute_shell_command(&trimmed[1..]);
        }
        
        // Check for introspection commands
        if trimmed.starts_with("??") {
            // Double question mark - detailed introspection
            let target = trimmed[2..].trim();
            return self.detailed_introspection(target);
        } else if trimmed.starts_with('?') && !trimmed.starts_with("?:") {
            // Single question mark - basic introspection (but not ternary operator)
            let target = trimmed[1..].trim();
            return self.basic_introspection(target);
        }

        // Handle shell substitution in let bindings (let x = !command)
        if trimmed.starts_with("let ") {
            if let Some(bang_pos) = trimmed.find(" = !") {
                // Extract the variable name and command
                let var_part = &trimmed[4..bang_pos];
                let command_part = &trimmed[bang_pos + 4..];
                
                // Execute the shell command
                let result = self.execute_shell_command(command_part)?;
                
                // Create a let binding with the result
                let modified_input = format!("let {} = \"{}\"", var_part, result.replace('"', "\\\""));
                
                // Parse and evaluate the modified input
                let deadline = Instant::now() + self.config.timeout;
                let mut parser = Parser::new(&modified_input);
                let ast = parser.parse().context("Failed to parse shell substitution")?;
                self.memory.try_alloc(std::mem::size_of_val(&ast))?;
                let value = self.evaluate_expr(&ast, deadline, 0)?;
                self.history.push(input.to_string());
                self.result_history.push(value.clone());
                self.update_history_variables();
                // Let bindings return empty string in REPL
                return Ok(String::new());
            }
        }
        
        // Set evaluation deadline
        let deadline = Instant::now() + self.config.timeout;

        // Parse the input
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;

        // Check memory for AST
        self.memory.try_alloc(std::mem::size_of_val(&ast))?;

        // Evaluate the expression with debug capture
        let value = match self.evaluate_expr(&ast, deadline, 0) {
            Ok(value) => {
                // Clear debug info on successful evaluation
                self.last_error_debug = None;
                value
            }
            Err(e) => {
                // Capture debug information
                self.last_error_debug = Some(DebugInfo {
                    expression: input.to_string(),
                    error_message: e.to_string(),
                    stack_trace: self.generate_stack_trace(&e),
                    bindings_snapshot: self.bindings.clone(),
                    timestamp: std::time::SystemTime::now(),
                });
                return Err(e);
            }
        };

        // Store successful evaluation
        self.history.push(input.to_string());
        self.result_history.push(value.clone());

        // Update history variables
        self.update_history_variables();

        // Let bindings are handled in evaluate_expr, no need to duplicate here

        Ok(value.to_string())
    }

    /// Get tab completions for the given input at the cursor position
    pub fn complete(&self, input: &str) -> Vec<String> {
        let pos = input.len();
        let completer = RuchyCompleter::new();
        completer.get_completions(input, pos, &self.bindings)
    }

    /// Get the current REPL mode
    pub fn get_mode(&self) -> &str {
        match self.mode {
            ReplMode::Normal => "normal",
            ReplMode::Shell => "shell",
            ReplMode::Pkg => "pkg",
            ReplMode::Help => "help",
            ReplMode::Sql => "sql",
            ReplMode::Math => "math",
            ReplMode::Debug => "debug",
            ReplMode::Time => "time",
        }
    }

    /// Get the current prompt
    pub fn get_prompt(&self) -> String {
        self.mode.prompt()
    }
    
    /// Create a new binding (for let/var) - handles shadowing
    fn create_binding(&mut self, name: String, value: Value, is_mutable: bool) {
        self.bindings.insert(name.clone(), value);
        self.binding_mutability.insert(name, is_mutable);
    }
    
    /// Try to update an existing binding (for assignment)
    fn update_binding(&mut self, name: &str, value: Value) -> Result<()> {
        if !self.bindings.contains_key(name) {
            bail!("Cannot assign to undefined variable '{}'. Declare it first with 'let' or 'var'", name)
        }
        
        let is_mutable = self.binding_mutability.get(name).copied().unwrap_or(false);
        if !is_mutable {
            bail!("Cannot assign to immutable binding '{}'. Use 'var' for mutable bindings or shadow with 'let'", name)
        }
        
        self.bindings.insert(name.to_string(), value);
        Ok(())
    }
    
    /// Get the value of a binding
    fn get_binding(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }
    
    /// Check if a binding exists
    fn has_binding(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Evaluate an expression to a value
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cognitive_complexity)]
    fn evaluate_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        // Check resource bounds
        if Instant::now() > deadline {
            bail!("Evaluation timeout exceeded");
        }
        if depth > self.config.max_depth {
            bail!("Maximum recursion depth {} exceeded", self.config.max_depth);
        }

        // COMPLEXITY REDUCTION: Dispatcher pattern by expression category
        match &expr.kind {
            // Basic expressions (literals, identifiers, binaries, unaries)
            ExprKind::Literal(_) | ExprKind::Binary { .. } | ExprKind::Unary { .. } 
            | ExprKind::Identifier(_) | ExprKind::QualifiedName { .. } => {
                self.evaluate_basic_expr(expr, deadline, depth)
            }
            
            // Control flow expressions
            ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::For { .. } 
            | ExprKind::While { .. } | ExprKind::IfLet { .. } | ExprKind::WhileLet { .. }
            | ExprKind::Loop { .. } | ExprKind::Break { .. } | ExprKind::Continue { .. } => {
                self.evaluate_control_flow_expr(expr, deadline, depth)
            }
            
            // Data structure expressions
            ExprKind::List(_) | ExprKind::Tuple(_) | ExprKind::ObjectLiteral { .. }
            | ExprKind::Range { .. } | ExprKind::FieldAccess { .. } | ExprKind::IndexAccess { .. } 
            | ExprKind::Slice { .. } => {
                self.evaluate_data_structure_expr(expr, deadline, depth)
            }
            
            // Function and call expressions
            ExprKind::Function { .. } | ExprKind::Lambda { .. } | ExprKind::Call { .. }
            | ExprKind::MethodCall { .. } => {
                self.evaluate_function_expr(expr, deadline, depth)
            }
            
            // Advanced language features
            _ => self.evaluate_advanced_expr(expr, deadline, depth)
        }
    }

    // COMPLEXITY REDUCTION: Basic expressions dispatcher  
    fn evaluate_basic_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.evaluate_literal(lit),
            ExprKind::Binary { left, op, right } => {
                self.evaluate_binary_expr(left, *op, right, deadline, depth)
            }
            ExprKind::Unary { op, operand } => {
                self.evaluate_unary_expr(*op, operand, deadline, depth)
            }
            ExprKind::Identifier(name) => self.evaluate_identifier(name),
            ExprKind::QualifiedName { module, name } => {
                Ok(Self::evaluate_qualified_name(module, name))
            }
            _ => bail!("Non-basic expression in basic dispatcher"),
        }
    }

    // COMPLEXITY REDUCTION: Control flow expressions dispatcher
    fn evaluate_control_flow_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        match &expr.kind {
            ExprKind::If { condition, then_branch, else_branch } => {
                self.evaluate_if(condition, then_branch, else_branch.as_deref(), deadline, depth)
            }
            ExprKind::Match { expr: match_expr, arms } => {
                self.evaluate_match(match_expr, arms, deadline, depth)
            }
            ExprKind::For { var, pattern, iter, body } => {
                self.evaluate_for_loop(var, pattern.as_ref(), iter, body, deadline, depth)
            }
            ExprKind::While { condition, body } => {
                self.evaluate_while_loop(condition, body, deadline, depth)
            }
            ExprKind::IfLet { pattern, expr, then_branch, else_branch } => {
                self.evaluate_if_let(pattern, expr, then_branch, else_branch.as_deref(), deadline, depth)
            }
            ExprKind::WhileLet { pattern, expr, body } => {
                self.evaluate_while_let(pattern, expr, body, deadline, depth)
            }
            ExprKind::Loop { body } => self.evaluate_loop(body, deadline, depth),
            ExprKind::Break { .. } => Err(anyhow::anyhow!("break")),
            ExprKind::Continue { .. } => Err(anyhow::anyhow!("continue")),
            _ => bail!("Non-control-flow expression in control flow dispatcher"),
        }
    }

    // COMPLEXITY REDUCTION: Data structure expressions dispatcher
    fn evaluate_data_structure_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        match &expr.kind {
            ExprKind::List(elements) => self.evaluate_list_literal(elements, deadline, depth),
            ExprKind::Tuple(elements) => self.evaluate_tuple_literal(elements, deadline, depth),
            ExprKind::ObjectLiteral { fields } => self.evaluate_object_literal(fields, deadline, depth),
            ExprKind::Range { start, end, inclusive } => {
                self.evaluate_range_literal(start, end, *inclusive, deadline, depth)
            }
            ExprKind::FieldAccess { object, field } => {
                self.evaluate_field_access(object, field, deadline, depth)
            }
            ExprKind::IndexAccess { object, index } => {
                self.evaluate_index_access(object, index, deadline, depth)
            }
            ExprKind::Slice { object, start, end } => {
                self.evaluate_slice(object, start.as_deref(), end.as_deref(), deadline, depth)
            }
            _ => bail!("Non-data-structure expression in data structure dispatcher"),
        }
    }

    // COMPLEXITY REDUCTION: Function expressions dispatcher
    fn evaluate_function_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        match &expr.kind {
            ExprKind::Function { name, params, body, .. } => {
                Ok(self.evaluate_function_definition(name, params, body))
            }
            ExprKind::Lambda { params, body } => Ok(Self::evaluate_lambda_expression(params, body)),
            ExprKind::Call { func, args } => self.evaluate_call(func, args, deadline, depth),
            ExprKind::MethodCall { receiver, method, args } => {
                let receiver_val = self.evaluate_expr(receiver, deadline, depth + 1)?;
                match receiver_val {
                    Value::List(items) => {
                        self.evaluate_list_methods(items, method, args, deadline, depth)
                    }
                    Value::String(s) => {
                        Self::evaluate_string_methods(&s, method, args, deadline, depth)
                    }
                    Value::Int(n) => Self::evaluate_int_methods(n, method),
                    Value::Float(f) => Self::evaluate_float_methods(f, method),
                    Value::Object(obj) => {
                        Self::evaluate_object_methods(obj, method, args, deadline, depth)
                    }
                    Value::HashMap(map) => {
                        self.evaluate_hashmap_methods(map, method, args, deadline, depth)
                    }
                    Value::HashSet(set) => {
                        self.evaluate_hashset_methods(set, method, args, deadline, depth)
                    }
                    Value::EnumVariant { .. } => {
                        self.evaluate_enum_methods(receiver_val, method, args, deadline, depth)
                    }
                    _ => bail!("Method {} not supported on this type", method),
                }
            }
            _ => bail!("Non-function expression in function dispatcher"),
        }
    }

    // COMPLEXITY REDUCTION: Advanced expressions dispatcher
    fn evaluate_advanced_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        match &expr.kind {
            ExprKind::Module { name: _name, body } => {
                self.evaluate_expr(body, deadline, depth + 1)
            }
            ExprKind::Let { name, type_annotation: _, value, body, is_mutable } => {
                self.evaluate_let_binding(name, value, body, *is_mutable, deadline, depth)
            }
            ExprKind::Block(exprs) => self.evaluate_block(exprs, deadline, depth),
            ExprKind::Assign { target, value } => {
                self.evaluate_assignment(target, value, deadline, depth)
            }
            ExprKind::DataFrame { columns } => {
                self.evaluate_dataframe_literal(columns, deadline, depth)
            }
            ExprKind::DataFrameOperation { .. } => Self::evaluate_dataframe_operation(),
            ExprKind::Pipeline { expr, stages } => {
                self.evaluate_pipeline(expr, stages, deadline, depth)
            }
            ExprKind::StringInterpolation { parts } => {
                self.evaluate_string_interpolation(parts, deadline, depth)
            }
            ExprKind::Ok { value } => self.evaluate_result_ok(value, deadline, depth),
            ExprKind::Err { error } => self.evaluate_result_err(error, deadline, depth),
            ExprKind::Some { value } => self.evaluate_option_some(value, deadline, depth),
            ExprKind::None => Ok(Self::evaluate_option_none()),
            ExprKind::Try { expr } => self.evaluate_try_operator(expr, deadline, depth),
            ExprKind::Await { expr } => self.evaluate_await_expr(expr, deadline, depth),
            ExprKind::AsyncBlock { body } => self.evaluate_async_block(body, deadline, depth),
            ExprKind::Enum { name, variants, .. } => {
                Ok(self.evaluate_enum_definition(name, variants))
            }
            ExprKind::Struct { name, fields, .. } => {
                Ok(Self::evaluate_struct_definition(name, fields))
            }
            ExprKind::StructLiteral { name: _, fields } => {
                self.evaluate_struct_literal(fields, deadline, depth)
            }
            ExprKind::Trait { name, methods, .. } => {
                Ok(Self::evaluate_trait_definition(name, methods))
            }
            ExprKind::Impl { for_type, methods, .. } => {
                Ok(self.evaluate_impl_block(for_type, methods))
            }
            ExprKind::Return { value } => {
                if let Some(val) = value {
                    let result = self.evaluate_expr(val, deadline, depth + 1)?;
                    Err(anyhow::anyhow!("return:{}", result))
                } else {
                    Err(anyhow::anyhow!("return:()"))
                }
            }
            ExprKind::Command { program, args, env: _, working_dir: _ } => {
                Self::evaluate_command(program, args, deadline, depth)
            }
            ExprKind::Macro { name, args } => {
                self.evaluate_macro(name, args, deadline, depth)
            }
            ExprKind::Import { path, items } => {
                self.evaluate_import(path, items)
            }
            ExprKind::Export { items } => {
                self.evaluate_export(items)
            }
            _ => bail!("Expression type not yet implemented: {:?}", expr.kind),
        }
    }

    // ========================================================================
    // Helper methods extracted to reduce evaluate_expr complexity
    // Following Toyota Way: Each function has single responsibility
    // Target: All functions < 50 cyclomatic complexity
    // ========================================================================

    /// Handle method calls on list values (complexity < 20)
    fn evaluate_list_methods(
        &mut self,
        items: Vec<Value>,
        method: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match method {
            "map" => self.evaluate_list_map(items, args, deadline, depth),
            "filter" => self.evaluate_list_filter(items, args, deadline, depth),
            "reduce" => self.evaluate_list_reduce(items, args, deadline, depth),
            "len" | "length" => {
                let len = items.len();
                i64::try_from(len)
                    .map(Value::Int)
                    .map_err(|_| anyhow::anyhow!("List length too large to represent as i64"))
            }
            "head" | "first" => items
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Empty list")),
            "last" => items
                .last()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Empty list")),
            "tail" | "rest" => {
                if items.is_empty() {
                    Ok(Value::List(Vec::new()))
                } else {
                    Ok(Value::List(items[1..].to_vec()))
                }
            }
            "reverse" => {
                let mut reversed = items;
                reversed.reverse();
                Ok(Value::List(reversed))
            }
            "sum" => {
                let mut sum = 0i64;
                for item in &items {
                    if let Value::Int(n) = item {
                        sum += n;
                    } else {
                        bail!("sum requires all integers");
                    }
                }
                Ok(Value::Int(sum))
            }
            "push" => {
                if args.len() != 1 {
                    bail!("push requires exactly 1 argument");
                }
                let mut new_items = items;
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                new_items.push(value);
                Ok(Value::List(new_items))
            }
            "pop" => {
                if !args.is_empty() {
                    bail!("pop requires no arguments");
                }
                let mut new_items = items;
                if let Some(popped) = new_items.pop() {
                    Ok(popped)
                } else {
                    bail!("Cannot pop from empty list")
                }
            }
            "append" => {
                if args.len() != 1 {
                    bail!("append requires exactly 1 argument");
                }
                let mut new_items = items;
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                if let Value::List(other_items) = value {
                    new_items.extend(other_items);
                    Ok(Value::List(new_items))
                } else {
                    bail!("append requires a list argument");
                }
            }
            "insert" => {
                if args.len() != 2 {
                    bail!("insert requires exactly 2 arguments (index, value)");
                }
                let mut new_items = items;
                let index = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let value = self.evaluate_expr(&args[1], deadline, depth + 1)?;
                if let Value::Int(idx) = index {
                    if idx < 0 || idx as usize > new_items.len() {
                        bail!("Insert index out of bounds");
                    }
                    new_items.insert(idx as usize, value);
                    Ok(Value::List(new_items))
                } else {
                    bail!("Insert index must be an integer");
                }
            }
            "remove" => {
                if args.len() != 1 {
                    bail!("remove requires exactly 1 argument (index)");
                }
                let mut new_items = items;
                let index = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                if let Value::Int(idx) = index {
                    if idx < 0 || idx as usize >= new_items.len() {
                        bail!("Remove index out of bounds");
                    }
                    let removed = new_items.remove(idx as usize);
                    Ok(removed)
                } else {
                    bail!("Remove index must be an integer");
                }
            }
            "slice" => {
                if args.len() != 2 {
                    bail!("slice requires exactly 2 arguments (start, end)");
                }
                let start_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let end_val = self.evaluate_expr(&args[1], deadline, depth + 1)?;
                
                if let (Value::Int(start), Value::Int(end)) = (start_val, end_val) {
                    let start = start as usize;
                    let end = end as usize;
                    
                    if start > items.len() || end > items.len() || start > end {
                        Ok(Value::List(Vec::new())) // Return empty for out of bounds
                    } else {
                        Ok(Value::List(items[start..end].to_vec()))
                    }
                } else {
                    bail!("slice arguments must be integers");
                }
            }
            "concat" => {
                if args.len() != 1 {
                    bail!("concat requires exactly 1 argument");
                }
                let other_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                
                if let Value::List(other_items) = other_val {
                    let mut result = items;
                    result.extend(other_items);
                    Ok(Value::List(result))
                } else {
                    bail!("concat argument must be a list");
                }
            }
            "flatten" => {
                if !args.is_empty() {
                    bail!("flatten requires no arguments");
                }
                let mut result = Vec::new();
                for item in items {
                    if let Value::List(inner_items) = item {
                        result.extend(inner_items);
                    } else {
                        result.push(item);
                    }
                }
                Ok(Value::List(result))
            }
            "unique" => {
                if !args.is_empty() {
                    bail!("unique requires no arguments");
                }
                use std::collections::HashSet;
                let mut seen = HashSet::new();
                let mut result = Vec::new();
                
                for item in items {
                    // Use string representation for hashing since Value doesn't implement Hash
                    let key = format!("{:?}", item);
                    if seen.insert(key) {
                        result.push(item);
                    }
                }
                Ok(Value::List(result))
            }
            "join" => {
                if args.len() != 1 {
                    bail!("join requires exactly 1 argument (separator)");
                }
                let sep_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                
                if let Value::String(separator) = sep_val {
                    let strings: Result<Vec<String>, _> = items.iter().map(|item| {
                        if let Value::String(s) = item {
                            Ok(s.clone())
                        } else {
                            bail!("join requires a list of strings");
                        }
                    }).collect();
                    
                    match strings {
                        Ok(string_vec) => Ok(Value::String(string_vec.join(&separator))),
                        Err(e) => Err(e),
                    }
                } else {
                    bail!("join separator must be a string");
                }
            }
            _ => bail!("Unknown list method: {}", method),
        }
    }

    /// Evaluate `list.map()` operation (complexity: 8)
    fn evaluate_list_map(
        &mut self,
        items: Vec<Value>,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("map expects 1 argument");
        }

        if let ExprKind::Lambda { params, body } = &args[0].kind {
            if params.len() != 1 {
                bail!("map lambda must take exactly 1 parameter");
            }

            let saved_bindings = self.bindings.clone();
            let mut results = Vec::new();

            for item in items {
                self.bindings.insert(params[0].name(), item);
                let result = self.evaluate_expr(body, deadline, depth + 1)?;
                results.push(result);
            }

            self.bindings = saved_bindings;
            Ok(Value::List(results))
        } else {
            bail!("map currently only supports lambda expressions");
        }
    }

    /// Evaluate `list.filter()` operation (complexity: 9)
    fn evaluate_list_filter(
        &mut self,
        items: Vec<Value>,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("filter expects 1 argument");
        }

        if let ExprKind::Lambda { params, body } = &args[0].kind {
            if params.len() != 1 {
                bail!("filter lambda must take exactly 1 parameter");
            }

            let saved_bindings = self.bindings.clone();
            let mut results = Vec::new();

            for item in items {
                self.bindings.insert(params[0].name(), item.clone());
                let predicate_result = self.evaluate_expr(body, deadline, depth + 1)?;

                if let Value::Bool(true) = predicate_result {
                    results.push(item);
                }
            }

            self.bindings = saved_bindings;
            Ok(Value::List(results))
        } else {
            bail!("filter currently only supports lambda expressions");
        }
    }

    /// Evaluate `list.reduce()` operation (complexity: 10)
    fn evaluate_list_reduce(
        &mut self,
        items: Vec<Value>,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 2 {
            bail!("reduce expects 2 arguments: initial value and lambda");
        }

        let mut accumulator = self.evaluate_expr(&args[0], deadline, depth + 1)?;

        if let ExprKind::Lambda { params, body } = &args[1].kind {
            if params.len() != 2 {
                bail!("reduce lambda must take exactly 2 parameters");
            }

            let saved_bindings = self.bindings.clone();

            for item in items {
                self.bindings.insert(params[0].name(), accumulator);
                self.bindings.insert(params[1].name(), item);
                accumulator = self.evaluate_expr(body, deadline, depth + 1)?;
            }

            self.bindings = saved_bindings;
            Ok(accumulator)
        } else {
            bail!("reduce currently only supports lambda expressions");
        }
    }

    /// Handle method calls on string values (complexity < 10)
    fn evaluate_string_methods(
        s: &str,
        method: &str,
        args: &[Expr],
        _deadline: Instant,
        _depth: usize,
    ) -> Result<Value> {
        match method {
            "len" | "length" => {
                let len = s.len();
                i64::try_from(len)
                    .map(Value::Int)
                    .map_err(|_| anyhow::anyhow!("String length too large to represent as i64"))
            }
            "upper" | "to_upper" | "to_uppercase" => Ok(Value::String(s.to_uppercase())),
            "lower" | "to_lower" | "to_lowercase" => Ok(Value::String(s.to_lowercase())),
            "trim" => Ok(Value::String(s.trim().to_string())),
            "split" => {
                if args.len() != 1 {
                    bail!("split expects 1 argument");
                }
                // For now, handle simple string literal separators
                if let ExprKind::Literal(Literal::String(sep)) = &args[0].kind {
                    let parts: Vec<Value> = s
                        .split(sep)
                        .map(|p| Value::String(p.to_string()))
                        .collect();
                    Ok(Value::List(parts))
                } else {
                    bail!("split separator must be a string literal");
                }
            }
            "contains" => {
                if args.len() != 1 {
                    bail!("contains expects 1 argument");
                }
                if let ExprKind::Literal(Literal::String(needle)) = &args[0].kind {
                    Ok(Value::Bool(s.contains(needle)))
                } else {
                    bail!("contains argument must be a string literal");
                }
            }
            "starts_with" => {
                if args.len() != 1 {
                    bail!("starts_with expects 1 argument");
                }
                if let ExprKind::Literal(Literal::String(prefix)) = &args[0].kind {
                    Ok(Value::Bool(s.starts_with(prefix)))
                } else {
                    bail!("starts_with argument must be a string literal");
                }
            }
            "ends_with" => {
                if args.len() != 1 {
                    bail!("ends_with expects 1 argument");
                }
                if let ExprKind::Literal(Literal::String(suffix)) = &args[0].kind {
                    Ok(Value::Bool(s.ends_with(suffix)))
                } else {
                    bail!("ends_with argument must be a string literal");
                }
            }
            "replace" => {
                if args.len() != 2 {
                    bail!("replace expects 2 arguments (from, to)");
                }
                if let (ExprKind::Literal(Literal::String(from)), ExprKind::Literal(Literal::String(to))) = 
                    (&args[0].kind, &args[1].kind) {
                    Ok(Value::String(s.replace(from, to)))
                } else {
                    bail!("replace arguments must be string literals");
                }
            }
            "substring" | "substr" => {
                if args.len() == 2 {
                    // substring(start, end)
                    if let (ExprKind::Literal(Literal::Integer(start)), ExprKind::Literal(Literal::Integer(end))) =
                        (&args[0].kind, &args[1].kind) {
                        let start_idx = (*start as usize).min(s.len());
                        let end_idx = (*end as usize).min(s.len());
                        if start_idx <= end_idx {
                            Ok(Value::String(s[start_idx..end_idx].to_string()))
                        } else {
                            Ok(Value::String(String::new()))
                        }
                    } else {
                        bail!("substring arguments must be integers");
                    }
                } else if args.len() == 1 {
                    // substring(start) - to end of string
                    if let ExprKind::Literal(Literal::Integer(start)) = &args[0].kind {
                        let start_idx = (*start as usize).min(s.len());
                        Ok(Value::String(s[start_idx..].to_string()))
                    } else {
                        bail!("substring argument must be an integer");
                    }
                } else {
                    bail!("substring expects 1 or 2 arguments");
                }
            }
            "repeat" => {
                if args.len() != 1 {
                    bail!("repeat expects 1 argument");
                }
                if let ExprKind::Literal(Literal::Integer(count)) = &args[0].kind {
                    if *count < 0 {
                        bail!("repeat count cannot be negative");
                    }
                    Ok(Value::String(s.repeat(*count as usize)))
                } else {
                    bail!("repeat argument must be an integer");
                }
            }
            "chars" => {
                // Return array of single-character strings
                let chars: Vec<Value> = s
                    .chars()
                    .map(|c| Value::String(c.to_string()))
                    .collect();
                Ok(Value::List(chars))
            }
            "reverse" => {
                let reversed: String = s.chars().rev().collect();
                Ok(Value::String(reversed))
            }
            _ => bail!("Unknown string method: {}", method),
        }
    }

    /// Handle method calls on integer values (complexity < 10)
    fn evaluate_int_methods(n: i64, method: &str) -> Result<Value> {
        match method {
            "abs" => Ok(Value::Int(n.abs())),
            #[allow(clippy::cast_precision_loss)]
            "sqrt" => Ok(Value::Float((n as f64).sqrt())),
            #[allow(clippy::cast_precision_loss)]
            "sin" => Ok(Value::Float((n as f64).sin())),
            #[allow(clippy::cast_precision_loss)]
            "cos" => Ok(Value::Float((n as f64).cos())),
            #[allow(clippy::cast_precision_loss)]
            "tan" => Ok(Value::Float((n as f64).tan())),
            #[allow(clippy::cast_precision_loss)]
            "log" => Ok(Value::Float((n as f64).ln())),
            #[allow(clippy::cast_precision_loss)]
            "log10" => Ok(Value::Float((n as f64).log10())),
            #[allow(clippy::cast_precision_loss)]
            "exp" => Ok(Value::Float((n as f64).exp())),
            "to_string" => Ok(Value::String(n.to_string())),
            _ => bail!("Unknown integer method: {}", method),
        }
    }

    /// Handle method calls on float values (complexity < 10)
    fn evaluate_float_methods(f: f64, method: &str) -> Result<Value> {
        match method {
            "abs" => Ok(Value::Float(f.abs())),
            "sqrt" => Ok(Value::Float(f.sqrt())),
            "sin" => Ok(Value::Float(f.sin())),
            "cos" => Ok(Value::Float(f.cos())),
            "tan" => Ok(Value::Float(f.tan())),
            "log" => Ok(Value::Float(f.ln())),
            "log10" => Ok(Value::Float(f.log10())),
            "exp" => Ok(Value::Float(f.exp())),
            "floor" => Ok(Value::Float(f.floor())),
            "ceil" => Ok(Value::Float(f.ceil())),
            "round" => Ok(Value::Float(f.round())),
            _ => bail!("Unknown float method: {}", method),
        }
    }

    /// Handle method calls on object values (complexity < 10)
    fn evaluate_object_methods(
        obj: HashMap<String, Value>,
        method: &str,
        _args: &[Expr],
        _deadline: Instant,
        _depth: usize,
    ) -> Result<Value> {
        match method {
            "items" => {
                // Return list of (key, value) tuples
                let mut items = Vec::new();
                for (key, value) in obj {
                    let tuple = Value::Tuple(vec![Value::String(key), value]);
                    items.push(tuple);
                }
                Ok(Value::List(items))
            }
            "keys" => {
                // Return list of keys
                let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
                Ok(Value::List(keys))
            }
            "values" => {
                // Return list of values
                let values: Vec<Value> = obj.values().cloned().collect();
                Ok(Value::List(values))
            }
            "len" => {
                // Return length of object
                Ok(Value::Int(obj.len() as i64))
            }
            "has_key" => {
                // This would need args handling - simplified for now
                bail!("has_key method requires arguments")
            }
            _ => bail!("Unknown object method: {}", method),
        }
    }

    /// Handle method calls on `HashMap` values (complexity < 10)
    fn evaluate_hashmap_methods(
        &mut self,
        mut map: HashMap<Value, Value>,
        method: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match method {
            "insert" => {
                if args.len() != 2 {
                    bail!("insert requires exactly 2 arguments (key, value)");
                }
                let key = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let value = self.evaluate_expr(&args[1], deadline, depth + 1)?;
                map.insert(key, value);
                Ok(Value::HashMap(map))
            }
            "get" => {
                if args.len() != 1 {
                    bail!("get requires exactly 1 argument (key)");
                }
                let key = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                match map.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Ok(Value::Unit), // Could return Option::None in future
                }
            }
            "contains_key" => {
                if args.len() != 1 {
                    bail!("contains_key requires exactly 1 argument (key)");
                }
                let key = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                Ok(Value::Bool(map.contains_key(&key)))
            }
            "remove" => {
                if args.len() != 1 {
                    bail!("remove requires exactly 1 argument (key)");
                }
                let key = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let removed_value = map.remove(&key);
                match removed_value {
                    Some(value) => Ok(Value::Tuple(vec![Value::HashMap(map), value])),
                    None => Ok(Value::Tuple(vec![Value::HashMap(map), Value::Unit])),
                }
            }
            "len" => Ok(Value::Int(map.len() as i64)),
            "is_empty" => Ok(Value::Bool(map.is_empty())),
            "clear" => {
                map.clear();
                Ok(Value::HashMap(map))
            }
            _ => bail!("Unknown HashMap method: {}", method),
        }
    }

    /// Handle method calls on `HashSet` values (complexity < 10)
    fn evaluate_hashset_methods(
        &mut self,
        mut set: HashSet<Value>,
        method: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match method {
            "insert" => {
                if args.len() != 1 {
                    bail!("insert requires exactly 1 argument (value)");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let was_new = set.insert(value);
                Ok(Value::Tuple(vec![Value::HashSet(set), Value::Bool(was_new)]))
            }
            "contains" => {
                if args.len() != 1 {
                    bail!("contains requires exactly 1 argument (value)");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                Ok(Value::Bool(set.contains(&value)))
            }
            "remove" => {
                if args.len() != 1 {
                    bail!("remove requires exactly 1 argument (value)");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let was_present = set.remove(&value);
                Ok(Value::Tuple(vec![Value::HashSet(set), Value::Bool(was_present)]))
            }
            "len" => Ok(Value::Int(set.len() as i64)),
            "is_empty" => Ok(Value::Bool(set.is_empty())),
            "clear" => {
                set.clear();
                Ok(Value::HashSet(set))
            }
            "union" => {
                if args.len() != 1 {
                    bail!("union requires exactly 1 argument (other set)");
                }
                let other_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                if let Value::HashSet(other_set) = other_val {
                    let union_set = set.union(&other_set).cloned().collect();
                    Ok(Value::HashSet(union_set))
                } else {
                    bail!("union argument must be a HashSet");
                }
            }
            "intersection" => {
                if args.len() != 1 {
                    bail!("intersection requires exactly 1 argument (other set)");
                }
                let other_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                if let Value::HashSet(other_set) = other_val {
                    let intersection_set = set.intersection(&other_set).cloned().collect();
                    Ok(Value::HashSet(intersection_set))
                } else {
                    bail!("intersection argument must be a HashSet");
                }
            }
            "difference" => {
                if args.len() != 1 {
                    bail!("difference requires exactly 1 argument (other set)");
                }
                let other_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                if let Value::HashSet(other_set) = other_val {
                    let difference_set = set.difference(&other_set).cloned().collect();
                    Ok(Value::HashSet(difference_set))
                } else {
                    bail!("difference argument must be a HashSet");
                }
            }
            _ => bail!("Unknown HashSet method: {}", method),
        }
    }

    // ========================================================================
    // Additional helper methods to further reduce evaluate_expr complexity
    // Phase 2: Control flow extraction (Target: < 50 total complexity)
    // ========================================================================

    /// Evaluate for loop (complexity: 10)
    fn evaluate_for_loop(
        &mut self,
        var: &str,
        pattern: Option<&Pattern>,
        iter: &Expr,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        // Evaluate the iterable
        let iterable = self.evaluate_expr(iter, deadline, depth + 1)?;

        // Save the previous value of the loop variable (if any)
        let saved_loop_var = self.bindings.get(var).cloned();
        
        // If we have a pattern, save all variables it will bind
        let saved_pattern_vars = if let Some(pat) = pattern {
            self.save_pattern_variables(pat)
        } else {
            HashMap::new()
        };

        // Execute the loop based on iterable type
        let result = match iterable {
            Value::List(items) => {
                if let Some(pat) = pattern {
                    self.iterate_list_with_pattern(pat, items, body, deadline, depth)
                } else {
                    self.iterate_list(var, items, body, deadline, depth)
                }
            },
            Value::Range {
                start,
                end,
                inclusive,
            } => self.iterate_range(var, start, end, inclusive, body, deadline, depth),
            Value::String(s) => self.iterate_string(var, &s, body, deadline, depth),
            _ => bail!(
                "For loops only support lists, ranges, and strings, got: {:?}",
                iterable
            ),
        };

        // Restore the loop variable
        if let Some(prev_value) = saved_loop_var {
            self.bindings.insert(var.to_string(), prev_value);
        } else {
            self.bindings.remove(var);
        }
        
        // Restore pattern variables
        for (name, value) in saved_pattern_vars {
            if let Some(val) = value {
                self.bindings.insert(name, val);
            } else {
                self.bindings.remove(&name);
            }
        }

        result
    }

    /// Helper: Iterate over a list (complexity: 4)
    fn iterate_list(
        &mut self,
        var: &str,
        items: Vec<Value>,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut result = Value::Unit;
        for item in items {
            self.bindings.insert(var.to_string(), item);
            match self.evaluate_expr(body, deadline, depth + 1) {
                Ok(value) => result = value,
                Err(e) if e.to_string() == "break" => break,
                Err(e) if e.to_string() == "continue" => {},
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }

    /// Helper: Iterate over a range (complexity: 5)
    #[allow(clippy::too_many_arguments)]
    fn iterate_range(
        &mut self,
        var: &str,
        start: i64,
        end: i64,
        inclusive: bool,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut result = Value::Unit;
        let actual_end = if inclusive { end + 1 } else { end };
        for i in start..actual_end {
            self.bindings.insert(var.to_string(), Value::Int(i));
            match self.evaluate_expr(body, deadline, depth + 1) {
                Ok(value) => result = value,
                Err(e) if e.to_string() == "break" => break,
                Err(e) if e.to_string() == "continue" => {},
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }

    /// Helper: Iterate over a string (as characters)
    fn iterate_string(
        &mut self,
        var: &str,
        s: &str,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut result = Value::Unit;
        for ch in s.chars() {
            self.bindings.insert(var.to_string(), Value::String(ch.to_string()));
            match self.evaluate_expr(body, deadline, depth + 1) {
                Ok(value) => result = value,
                Err(e) if e.to_string() == "break" => break,
                Err(e) if e.to_string() == "continue" => {},
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }

    /// Helper: Save pattern variables for restoration
    fn save_pattern_variables(&self, pattern: &Pattern) -> HashMap<String, Option<Value>> {
        let mut saved = HashMap::new();
        self.collect_pattern_vars(pattern, &mut saved);
        saved
    }
    
    /// Helper: Collect all variables from a pattern
    fn collect_pattern_vars(&self, pattern: &Pattern, saved: &mut HashMap<String, Option<Value>>) {
        match pattern {
            Pattern::Identifier(name) => {
                saved.insert(name.clone(), self.bindings.get(name).cloned());
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.collect_pattern_vars(p, saved);
                }
            }
            _ => {} // Other patterns don't bind variables
        }
    }
    
    /// Helper: Iterate over a list with pattern destructuring
    fn iterate_list_with_pattern(
        &mut self,
        pattern: &Pattern,
        items: Vec<Value>,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut result = Value::Unit;
        for item in items {
            // Bind the pattern variables
            self.bind_pattern(pattern, &item)?;
            
            match self.evaluate_expr(body, deadline, depth + 1) {
                Ok(value) => result = value,
                Err(e) if e.to_string() == "break" => break,
                Err(e) if e.to_string() == "continue" => {},
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }
    
    /// Helper: Bind pattern variables from a value
    fn bind_pattern(&mut self, pattern: &Pattern, value: &Value) -> Result<()> {
        match (pattern, value) {
            (Pattern::Identifier(name), val) => {
                self.bindings.insert(name.clone(), val.clone());
                Ok(())
            }
            (Pattern::Tuple(patterns), Value::Tuple(values)) => {
                if patterns.len() != values.len() {
                    bail!("Pattern tuple has {} elements but value has {}", patterns.len(), values.len());
                }
                for (p, v) in patterns.iter().zip(values.iter()) {
                    self.bind_pattern(p, v)?;
                }
                Ok(())
            }
            _ => bail!("Pattern does not match value")
        }
    }

    /// Evaluate while loop (complexity: 7)
    /// 
    /// While loops always return Unit, regardless of body expression.
    /// 
    /// # Example
    /// ```
    /// use ruchy::runtime::Repl;
    /// let mut repl = Repl::new().unwrap();
    /// 
    /// // While loops return Unit, not the last body value
    /// let result = repl.eval("let i = 0; while i < 3 { i = i + 1 }; i").unwrap();
    /// assert_eq!(result.to_string(), "3"); // i is 3 after loop
    /// 
    /// // While loop doesn't return body value
    /// let result = repl.eval("let i = 0; while i < 1 { i = i + 1; 42 }").unwrap();
    /// assert_eq!(result.to_string(), "()"); // Returns Unit, not 42
    /// ```
    fn evaluate_while_loop(
        &mut self,
        condition: &Expr,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let max_iterations = 1000; // Prevent infinite loops in REPL
        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                bail!(
                    "While loop exceeded maximum iterations ({})",
                    max_iterations
                );
            }

            // Evaluate condition
            let cond_val = self.evaluate_expr(condition, deadline, depth + 1)?;
            match cond_val {
                Value::Bool(true) => {
                    // Execute body but don't save result - while loops return Unit
                    self.evaluate_expr(body, deadline, depth + 1)?;
                    iterations += 1;
                }
                Value::Bool(false) => break,
                _ => bail!("While condition must be boolean, got: {:?}", cond_val),
            }
        }
        // While loops always return Unit
        Ok(Value::Unit)
    }

    /// Evaluate loop expression (complexity: 6)
    fn evaluate_loop(&mut self, body: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        let mut result = Value::Unit;
        let max_iterations = 1000; // Prevent infinite loops in REPL
        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                bail!("Loop exceeded maximum iterations ({})", max_iterations);
            }

            // Evaluate body, catching break
            match self.evaluate_expr(body, deadline, depth + 1) {
                Ok(val) => {
                    result = val;
                    iterations += 1;
                }
                Err(e) if e.to_string() == "break" => {
                    break;
                }
                Err(e) if e.to_string() == "continue" => {
                    iterations += 1;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    }

    /// Evaluate block expression (complexity: 4)
    fn evaluate_block(&mut self, exprs: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if exprs.is_empty() {
            return Ok(Value::Unit);
        }

        let mut result = Value::Unit;
        for expr in exprs {
            result = self.evaluate_expr(expr, deadline, depth + 1)?;
        }
        Ok(result)
    }

    /// Evaluate list literal (complexity: 4)
    fn evaluate_list_literal(
        &mut self,
        elements: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut results = Vec::new();
        for elem in elements {
            let val = self.evaluate_expr(elem, deadline, depth + 1)?;
            results.push(val);
        }
        Ok(Value::List(results))
    }

    /// Evaluate tuple literal (complexity: 4)
    fn evaluate_tuple_literal(
        &mut self,
        elements: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut results = Vec::new();
        for elem in elements {
            let val = self.evaluate_expr(elem, deadline, depth + 1)?;
            results.push(val);
        }
        Ok(Value::Tuple(results))
    }

    /// Evaluate range literal (complexity: 5)
    fn evaluate_range_literal(
        &mut self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let start_val = self.evaluate_expr(start, deadline, depth + 1)?;
        let end_val = self.evaluate_expr(end, deadline, depth + 1)?;

        match (start_val, end_val) {
            (Value::Int(s), Value::Int(e)) => Ok(Value::Range {
                start: s,
                end: e,
                inclusive,
            }),
            _ => bail!("Range endpoints must be integers"),
        }
    }

    /// Evaluate assignment expression (complexity: 5)
    fn evaluate_assignment(
        &mut self,
        target: &Expr,
        value: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let val = self.evaluate_expr(value, deadline, depth + 1)?;

        // For now, only support simple variable assignment
        if let ExprKind::Identifier(name) = &target.kind {
            // Use update_binding which checks mutability
            self.update_binding(name, val.clone())?;
            Ok(val)
        } else {
            bail!(
                "Only simple variable assignment is supported, got: {:?}",
                target.kind
            );
        }
    }

    /// Evaluate let binding (complexity: 5)
    fn evaluate_let_binding(
        &mut self,
        name: &str,
        value: &Expr,
        body: &Expr,
        is_mutable: bool,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let val = self.evaluate_expr(value, deadline, depth + 1)?;
        self.create_binding(name.to_string(), val.clone(), is_mutable);

        // If there's a body, evaluate it; otherwise return the value
        match &body.kind {
            ExprKind::Literal(Literal::Unit) => Ok(val),
            _ => self.evaluate_expr(body, deadline, depth + 1),
        }
    }

    /// Evaluate string interpolation (complexity: 7)
    fn evaluate_string_interpolation(
        &mut self,
        parts: &[crate::frontend::ast::StringPart],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        use crate::frontend::ast::StringPart;

        let mut result = String::new();
        for part in parts {
            match part {
                StringPart::Text(text) => result.push_str(text),
                StringPart::Expr(expr) => {
                    let value = self.evaluate_expr(expr, deadline, depth + 1)?;
                    // Format the value for interpolation (without quotes for strings)
                    match value {
                        Value::String(s) => result.push_str(&s),
                        Value::Char(c) => result.push(c),
                        other => result.push_str(&other.to_string()),
                    }
                }
                StringPart::ExprWithFormat { expr, format_spec } => {
                    let value = self.evaluate_expr(expr, deadline, depth + 1)?;
                    // Apply format specifier for REPL
                    let formatted = Self::format_value_with_spec(&value, format_spec);
                    result.push_str(&formatted);
                }
            }
        }
        Ok(Value::String(result))
    }

    /// Format a value with a format specifier like :.2 for floats
    fn format_value_with_spec(value: &Value, spec: &str) -> String {
        // Parse format specifier (e.g., ":.2" -> precision 2)
        if let Some(stripped) = spec.strip_prefix(":.") {
            if let Ok(precision) = stripped.parse::<usize>() {
                match value {
                    Value::Float(f) => return format!("{f:.precision$}"),
                    Value::Int(i) => return format!("{:.precision$}", *i as f64, precision = precision),
                    _ => {}
                }
            }
        }
        // Default formatting if spec doesn't match or isn't supported
        value.to_string()
    }

    /// Evaluate function definition (complexity: 5)
    fn evaluate_function_definition(
        &mut self,
        name: &str,
        params: &[crate::frontend::ast::Param],
        body: &Expr,
    ) -> Value {
        let param_names: Vec<String> = params
            .iter()
            .map(crate::frontend::ast::Param::name)
            .collect();
        let func_value = Value::Function {
            name: name.to_string(),
            params: param_names,
            body: Box::new(body.clone()),
        };

        // Store the function in bindings
        self.bindings.insert(name.to_string(), func_value.clone());
        func_value
    }

    /// Evaluate lambda expression (complexity: 3)
    fn evaluate_lambda_expression(params: &[crate::frontend::ast::Param], body: &Expr) -> Value {
        let param_names: Vec<String> = params
            .iter()
            .map(crate::frontend::ast::Param::name)
            .collect();
        Value::Lambda {
            params: param_names,
            body: Box::new(body.clone()),
        }
    }

    /// Evaluate `DataFrame` literal (complexity: 6)
    fn evaluate_dataframe_literal(
        &mut self,
        columns: &[crate::frontend::ast::DataFrameColumn],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut df_columns = Vec::new();
        for col in columns {
            let mut values = Vec::new();
            for val_expr in &col.values {
                let val = self.evaluate_expr(val_expr, deadline, depth + 1)?;
                values.push(val);
            }
            df_columns.push(DataFrameColumn {
                name: col.name.clone(),
                values,
            });
        }
        Ok(Value::DataFrame {
            columns: df_columns,
        })
    }


    /// Evaluate `Result::Ok` constructor (complexity: 3)
    fn evaluate_result_ok(
        &mut self,
        value: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let val = self.evaluate_expr(value, deadline, depth + 1)?;
        Ok(Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Ok".to_string(),
            data: Some(vec![val]),
        })
    }

    /// Evaluate `Result::Err` constructor (complexity: 3)
    fn evaluate_result_err(
        &mut self,
        error: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let err = self.evaluate_expr(error, deadline, depth + 1)?;
        Ok(Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Err".to_string(),
            data: Some(vec![err]),
        })
    }

    /// Evaluate `Option::Some` constructor (complexity: 3)
    fn evaluate_option_some(
        &mut self,
        value: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let val = self.evaluate_expr(value, deadline, depth + 1)?;
        Ok(Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![val]),
        })
    }

    /// Evaluate `Option::None` constructor (complexity: 1)
    fn evaluate_option_none() -> Value {
        Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        }
    }
    
    /// Evaluate try operator (?) - early return on Err or None
    fn evaluate_try_operator(
        &mut self,
        expr: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let val = self.evaluate_expr(expr, deadline, depth + 1)?;
        
        // Check if it's a Result::Err or Option::None and propagate
        if let Value::EnumVariant { enum_name, variant_name, data } = &val {
            if enum_name == "Result" && variant_name == "Err" {
                // For Result::Err, propagate the error
                return Ok(val.clone());
            } else if enum_name == "Option" && variant_name == "None" {
                // For Option::None, propagate None
                return Ok(val.clone());
            } else if enum_name == "Result" && variant_name == "Ok" {
                // For Result::Ok, unwrap the value
                if let Some(values) = data {
                    if !values.is_empty() {
                        return Ok(values[0].clone());
                    }
                }
            } else if enum_name == "Option" && variant_name == "Some" {
                // For Option::Some, unwrap the value
                if let Some(values) = data {
                    if !values.is_empty() {
                        return Ok(values[0].clone());
                    }
                }
            }
        }
        
        // If not a Result or Option, return as-is (this might be an error case)
        Ok(val)
    }

    /// Evaluate methods on enum variants (Result/Option types)
    #[allow(clippy::too_many_lines)]
    /// Evaluate enum methods with complexity <10
    /// 
    /// Delegates to specialized handlers for each enum type
    /// 
    /// Example Usage:
    /// 
    /// Handles methods on Result and Option enums:
    /// - `Result::unwrap()` - Returns Ok value or panics on Err
    /// - `Result::unwrap_or(default)` - Returns Ok value or default
    /// - `Option::unwrap()` - Returns Some value or panics on None  
    /// - `Option::unwrap_or(default)` - Returns Some value or default
    fn evaluate_enum_methods(
        &mut self,
        receiver: Value,
        method: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let Value::EnumVariant { enum_name, variant_name, data } = receiver {
            match enum_name.as_str() {
                "Result" => self.evaluate_result_methods(&variant_name, method, data.as_ref(), args, deadline, depth),
                "Option" => self.evaluate_option_methods(&variant_name, method, data.as_ref(), args, deadline, depth),
                "Vec" => self.evaluate_vec_methods(&variant_name, method, data.as_ref(), args, deadline, depth),
                _ => bail!("Method {} not supported on {}", method, enum_name),
            }
        } else {
            bail!("evaluate_enum_methods called on non-enum variant")
        }
    }

    /// Handle Result enum methods (unwrap, expect, map, `and_then`)
    /// 
    /// # Example Usage
    /// Evaluates methods on enum variants like `Some(x).unwrap()` or `Ok(v).is_ok()`.
    /// 
    /// use `ruchy::runtime::{Repl`, Value};
    /// use `std::time::{Duration`, Instant};
    /// 
    /// let mut repl = `Repl::new().unwrap()`;
    /// let deadline = `Instant::now()` + `Duration::from_secs(1)`;
    /// let data = Some(vec![`Value::Int(42)`]);
    /// 
    /// // Test Ok unwrap
    /// let result = `repl.evaluate_result_methods("Ok`", "unwrap", &data, &[], deadline, `0).unwrap()`;
    /// `assert_eq!(result`, `Value::Int(42)`);
    /// ```
    fn evaluate_result_methods(
        &mut self,
        variant_name: &str,
        method: &str,
        data: Option<&Vec<Value>>,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match (variant_name, method) {
            ("Ok", "unwrap" | "expect") if args.is_empty() || args.len() == 1 => {
                self.extract_value_or_unit(data)
            }
            ("Err", "unwrap") if args.is_empty() => {
                let error_msg = self.format_error_message("Result::unwrap()", "Err", data);
                bail!(error_msg)
            }
            ("Err", "expect") if args.len() == 1 => {
                let custom_msg = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let msg = self.value_to_string(custom_msg);
                bail!(msg)
            }
            ("Ok", "map") if args.len() == 1 => {
                self.apply_function_to_value("Result", "Ok", data, &args[0], deadline, depth)
            }
            ("Err", "map") if args.len() == 1 => {
                Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: variant_name.to_string(),
                    data: data.cloned(),
                })
            }
            ("Ok", "and_then") if args.len() == 1 => {
                self.apply_function_and_flatten(data, &args[0], deadline, depth)
            }
            ("Err", "and_then") if args.len() == 1 => {
                Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: variant_name.to_string(),
                    data: data.cloned(),
                })
            }
            _ => bail!("Method {} not supported on Result::{}", method, variant_name),
        }
    }

    /// Handle Option enum methods (unwrap, expect, map, `and_then`)
    fn evaluate_option_methods(
        &mut self,
        variant_name: &str,
        method: &str,
        data: Option<&Vec<Value>>,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match (variant_name, method) {
            ("Some", "unwrap" | "expect") if args.is_empty() || args.len() == 1 => {
                self.extract_value_or_unit(data)
            }
            ("None", "unwrap") if args.is_empty() => {
                bail!("called `Option::unwrap()` on a `None` value")
            }
            ("None", "expect") if args.len() == 1 => {
                let custom_msg = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let msg = self.value_to_string(custom_msg);
                bail!(msg)
            }
            ("Some", "map") if args.len() == 1 => {
                self.apply_function_to_value("Option", "Some", data, &args[0], deadline, depth)
            }
            ("None", "map" | "and_then") if args.len() == 1 => {
                Ok(Value::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant_name: variant_name.to_string(),
                    data: data.cloned(),
                })
            }
            ("Some", "and_then") if args.len() == 1 => {
                self.apply_function_and_flatten(data, &args[0], deadline, depth)
            }
            _ => bail!("Method {} not supported on Option::{}", method, variant_name),
        }
    }

    /// Handle Vec enum methods (placeholder for future Vec methods)
    fn evaluate_vec_methods(
        &mut self,
        variant_name: &str,
        method: &str,
        data: Option<&Vec<Value>>,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match method {
            "len" => Ok(Value::Int(data.as_ref().map_or(0, |v| v.len() as i64))),
            "push" if args.len() == 1 => {
                let new_elem = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let mut vec_data = data.cloned().unwrap_or_default();
                vec_data.push(new_elem);
                Ok(Value::EnumVariant {
                    enum_name: "Vec".to_string(),
                    variant_name: variant_name.to_string(),
                    data: Some(vec_data),
                })
            }
            _ => bail!("Method {} not supported on Vec", method),
        }
    }

    /// Extract value from enum data or return Unit
    fn extract_value_or_unit(&self, data: Option<&Vec<Value>>) -> Result<Value> {
        if let Some(values) = data {
            if !values.is_empty() {
                return Ok(values[0].clone());
            }
        }
        Ok(Value::Unit)
    }

    /// Format error message for unwrap operations
    fn format_error_message(&self, method: &str, variant: &str, data: Option<&Vec<Value>>) -> String {
        if let Some(values) = data {
            if values.is_empty() {
                format!("called `{method}` on an `{variant}` value")
            } else {
                format!("called `{}` on an `{}` value: {}", method, variant, values[0])
            }
        } else {
            format!("called `{method}` on an `{variant}` value")
        }
    }

    /// Convert Value to string representation
    fn value_to_string(&self, value: Value) -> String {
        match value {
            Value::String(s) => s,
            other => format!("{other}"),
        }
    }

    /// Apply function to enum value (for map operations)
    fn apply_function_to_value(
        &mut self,
        enum_name: &str,
        variant_name: &str,
        data: Option<&Vec<Value>>,
        func_arg: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let Some(values) = data {
            if !values.is_empty() {
                let call_expr = self.create_function_call(func_arg, &values[0]);
                let mapped_value = self.evaluate_expr(&call_expr, deadline, depth + 1)?;
                return Ok(Value::EnumVariant {
                    enum_name: enum_name.to_string(),
                    variant_name: variant_name.to_string(),
                    data: Some(vec![mapped_value]),
                });
            }
        }
        Ok(Value::EnumVariant {
            enum_name: enum_name.to_string(),
            variant_name: variant_name.to_string(),
            data: Some(vec![Value::Unit]),
        })
    }

    /// Apply function and flatten result (for `and_then` operations)
    fn apply_function_and_flatten(
        &mut self,
        data: Option<&Vec<Value>>,
        func_arg: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let Some(values) = data {
            if !values.is_empty() {
                let call_expr = self.create_function_call(func_arg, &values[0]);
                return self.evaluate_expr(&call_expr, deadline, depth + 1);
            }
        }
        Ok(Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Ok".to_string(),
            data: Some(vec![Value::Unit]),
        })
    }

    /// Create function call expression for enum combinators
    fn create_function_call(&self, func_arg: &Expr, value: &Value) -> Expr {
        Expr::new(
            ExprKind::Call {
                func: Box::new(func_arg.clone()),
                args: vec![Expr::new(
                    ExprKind::Literal(crate::frontend::ast::Literal::from_value(value)),
                    Span { start: 0, end: 0 },
                )],
            },
            Span { start: 0, end: 0 },
        )
    }

    /// Evaluate object literal (complexity: 10)
    fn evaluate_object_literal(
        &mut self,
        fields: &[crate::frontend::ast::ObjectField],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        use crate::frontend::ast::ObjectField;
        let mut map = HashMap::new();

        for field in fields {
            match field {
                ObjectField::KeyValue { key, value } => {
                    let val = self.evaluate_expr(value, deadline, depth + 1)?;
                    map.insert(key.clone(), val);
                }
                ObjectField::Spread { expr } => {
                    let spread_val = self.evaluate_expr(expr, deadline, depth + 1)?;
                    if let Value::Object(spread_map) = spread_val {
                        map.extend(spread_map);
                    } else {
                        bail!("Spread operator can only be used with objects");
                    }
                }
            }
        }

        Ok(Value::Object(map))
    }

    /// Evaluate enum definition (complexity: 4)
    fn evaluate_enum_definition(
        &mut self,
        name: &str,
        variants: &[crate::frontend::ast::EnumVariant],
    ) -> Value {
        let variant_names: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
        self.enum_definitions
            .insert(name.to_string(), variant_names);
        println!("Defined enum {} with {} variants", name, variants.len());
        Value::Unit
    }

    /// Evaluate struct definition (complexity: 3)
    fn evaluate_struct_definition(
        name: &str,
        fields: &[crate::frontend::ast::StructField],
    ) -> Value {
        println!("Defined struct {} with {} fields", name, fields.len());
        Value::Unit
    }

    /// Evaluate struct literal (complexity: 5)
    fn evaluate_struct_literal(
        &mut self,
        fields: &[(String, Expr)],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut map = HashMap::new();
        for (field_name, field_expr) in fields {
            let field_value = self.evaluate_expr(field_expr, deadline, depth + 1)?;
            map.insert(field_name.clone(), field_value);
        }
        Ok(Value::Object(map))
    }

    /// Evaluate field access (complexity: 4)
    fn evaluate_field_access(
        &mut self,
        object: &Expr,
        field: &str,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let obj_val = self.evaluate_expr(object, deadline, depth + 1)?;
        match obj_val {
            Value::Object(map) => map
                .get(field)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Field '{}' not found", field)),
            _ => bail!("Field access on non-object value"),
        }
    }

    /// Evaluate index access (complexity: 5)
    fn evaluate_index_access(
        &mut self,
        object: &Expr,
        index: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let obj_val = self.evaluate_expr(object, deadline, depth + 1)?;
        let index_val = self.evaluate_expr(index, deadline, depth + 1)?;

        // Check for range indexing first
        if let Value::Range { start, end, inclusive } = index_val {
            return self.handle_range_indexing(obj_val, start, end, inclusive);
        }

        // Handle single index access
        self.handle_single_index_access(obj_val, index_val)
    }

    /// Handle range-based indexing for lists and strings
    /// 
    /// Example Usage:
    /// 
    /// Handles range indexing for lists and strings:
    /// - list[0..2] returns a sublist with elements at indices 0 and 1
    /// - string[1..3] returns substring from index 1 to 2
    /// - list[0..=2] returns elements at indices 0, 1, and 2 (inclusive)
    fn handle_range_indexing(&self, obj_val: Value, start: i64, end: i64, inclusive: bool) -> Result<Value> {
        match obj_val {
            Value::List(list) => {
                let (start_idx, end_idx) = self.calculate_slice_bounds(start, end, inclusive, list.len())?;
                Ok(Value::List(list[start_idx..end_idx].to_vec()))
            }
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                let (start_idx, end_idx) = self.calculate_slice_bounds(start, end, inclusive, chars.len())?;
                Ok(Value::String(chars[start_idx..end_idx].iter().collect()))
            }
            _ => bail!("Cannot slice into {:?}", obj_val),
        }
    }

    /// Handle single index access for various data types
    /// 
    /// # Example Usage
    /// Handles range-based indexing for strings and arrays like arr[0..3] or str[1..].
    /// # use `ruchy::runtime::repl::Repl`;
    /// # use `ruchy::runtime::repl::Value`;
    /// let mut repl = `Repl::new().unwrap()`;
    /// let list = `Value::List(vec`![`Value::Int(42)`]);
    /// let result = `repl.handle_single_index_access(list`, `Value::Int(0)).unwrap()`;
    /// `assert_eq!(result`, `Value::Int(42)`);
    /// ```
    fn handle_single_index_access(&self, obj_val: Value, index_val: Value) -> Result<Value> {
        match (obj_val, index_val) {
            (Value::List(list), Value::Int(idx)) => {
                let idx = self.validate_array_index(idx, list.len())?;
                Ok(list[idx].clone())
            }
            (Value::String(s), Value::Int(idx)) => {
                let chars: Vec<char> = s.chars().collect();
                let idx = self.validate_array_index(idx, chars.len())?;
                Ok(Value::String(chars[idx].to_string()))
            }
            (Value::Object(obj), Value::String(key)) => {
                obj.get(&key)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Key '{}' not found in object", key))
            }
            (obj_val, index_val) => bail!("Cannot index into {:?} with index {:?}", obj_val, index_val),
        }
    }

    /// Calculate slice bounds and validate them
    /// 
    /// # Example Usage
    /// Calculates and validates slice bounds for array indexing operations.
    /// Converts indices to valid array bounds and handles inclusive/exclusive ranges.
    fn calculate_slice_bounds(&self, start: i64, end: i64, inclusive: bool, len: usize) -> Result<(usize, usize)> {
        let start_idx = usize::try_from(start)
            .map_err(|_| anyhow::anyhow!("Invalid start index: {}", start))?;
        
        let end_idx = if inclusive {
            usize::try_from(end + 1)
                .map_err(|_| anyhow::anyhow!("Invalid end index: {}", end + 1))?
        } else {
            usize::try_from(end)
                .map_err(|_| anyhow::anyhow!("Invalid end index: {}", end))?
        };

        if start_idx > len || end_idx > len {
            bail!("Slice indices out of bounds");
        }
        if start_idx > end_idx {
            bail!("Invalid slice range: start > end");
        }

        Ok((start_idx, end_idx))
    }

    /// Validate array index and convert to usize
    /// 
    /// # Example Usage
    /// Calculates and validates slice bounds for array indexing operations.
    /// # use `ruchy::runtime::repl::Repl`;
    /// let repl = `Repl::new().unwrap()`;
    /// let idx = `repl.validate_array_index(2`, `5).unwrap()`;
    /// `assert_eq!(idx`, 2);
    /// ```
    fn validate_array_index(&self, idx: i64, len: usize) -> Result<usize> {
        let idx = usize::try_from(idx)
            .map_err(|_| anyhow::anyhow!("Invalid index: {}", idx))?;
        
        if idx >= len {
            bail!("Index {} out of bounds for length {}", idx, len);
        }
        
        Ok(idx)
    }

    fn evaluate_slice(
        &mut self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let obj_val = self.evaluate_expr(object, deadline, depth + 1)?;
        
        // Evaluate start and end indices if provided
        let start_idx = if let Some(start_expr) = start {
            match self.evaluate_expr(start_expr, deadline, depth + 1)? {
                Value::Int(idx) => Some(usize::try_from(idx)
                    .map_err(|_| anyhow::anyhow!("Invalid start index: {}", idx))?),
                _ => return Err(anyhow::anyhow!("Slice indices must be integers")),
            }
        } else {
            None
        };
        
        let end_idx = if let Some(end_expr) = end {
            match self.evaluate_expr(end_expr, deadline, depth + 1)? {
                Value::Int(idx) => Some(usize::try_from(idx)
                    .map_err(|_| anyhow::anyhow!("Invalid end index: {}", idx))?),
                _ => return Err(anyhow::anyhow!("Slice indices must be integers")),
            }
        } else {
            None
        };
        
        match obj_val {
            Value::List(list) => {
                let start = start_idx.unwrap_or(0);
                let end = end_idx.unwrap_or(list.len());
                
                if start > list.len() || end > list.len() {
                    return Err(anyhow::anyhow!("Slice indices out of bounds"));
                }
                if start > end {
                    return Err(anyhow::anyhow!("Invalid slice range: start > end"));
                }
                
                Ok(Value::List(list[start..end].to_vec()))
            }
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                let start = start_idx.unwrap_or(0);
                let end = end_idx.unwrap_or(chars.len());
                
                if start > chars.len() || end > chars.len() {
                    return Err(anyhow::anyhow!("Slice indices out of bounds"));
                }
                if start > end {
                    return Err(anyhow::anyhow!("Invalid slice range: start > end"));
                }
                
                let sliced: String = chars[start..end].iter().collect();
                Ok(Value::String(sliced))
            }
            _ => Err(anyhow::anyhow!("Cannot slice value of type {:?}", obj_val)),
        }
    }

    /// Evaluate trait definition (complexity: 3)
    fn evaluate_trait_definition(
        name: &str,
        methods: &[crate::frontend::ast::TraitMethod],
    ) -> Value {
        println!("Defined trait {} with {} methods", name, methods.len());
        Value::Unit
    }

    /// Evaluate impl block (complexity: 12)
    fn evaluate_impl_block(
        &mut self,
        for_type: &str,
        methods: &[crate::frontend::ast::ImplMethod],
    ) -> Value {
        for method in methods {
            let qualified_name = format!("{}::{}", for_type, method.name);

            let param_names: Vec<String> = method
                .params
                .iter()
                .filter_map(|p| {
                    let name = p.name();
                    if name != "self" && name != "&self" {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();

            self.impl_methods
                .insert(qualified_name, (param_names, method.body.clone()));
        }

        println!(
            "Defined impl for {} with {} methods",
            for_type,
            methods.len()
        );
        Value::Unit
    }

    /// Evaluate binary expression (complexity: 3)
    fn evaluate_binary_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let lhs = self.evaluate_expr(left, deadline, depth + 1)?;
        let rhs = self.evaluate_expr(right, deadline, depth + 1)?;
        Self::evaluate_binary(&lhs, op, &rhs)
    }

    /// Evaluate unary expression (complexity: 2)
    fn evaluate_unary_expr(
        &mut self,
        op: UnaryOp,
        operand: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let val = self.evaluate_expr(operand, deadline, depth + 1)?;
        Self::evaluate_unary(op, &val)
    }

    /// Evaluate identifier (complexity: 2)
    fn evaluate_identifier(&self, name: &str) -> Result<Value> {
        self.get_binding(name)
            .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))
    }

    /// Evaluate qualified name (complexity: 2)
    fn evaluate_qualified_name(module: &str, name: &str) -> Value {
        Value::EnumVariant {
            enum_name: module.to_string(),
            variant_name: name.to_string(),
            data: None,
        }
    }

    /// Evaluate await expression (complexity: 1)
    fn evaluate_await_expr(
        &mut self,
        expr: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        // For now, await just evaluates the expression
        // In a full async implementation, this would handle Future resolution
        self.evaluate_expr(expr, deadline, depth + 1)
    }

    /// Evaluate async block (complexity: 1)
    fn evaluate_async_block(
        &mut self,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        // For REPL purposes, evaluate the async block body synchronously
        // In a full async implementation, this would return a Future
        self.evaluate_expr(body, deadline, depth + 1)
    }

    /// Evaluate try operator (?) (complexity: 1)
    /// Evaluate `DataFrame` operation (complexity: 1)
    fn evaluate_dataframe_operation() -> Result<Value> {
        // DataFrame operations not yet implemented in REPL
        bail!("DataFrame operations not yet implemented in REPL")
    }

    /// Check if a pattern matches a value and return bindings
    ///
    /// Returns Some(bindings) if pattern matches, None if it doesn't
    fn pattern_matches(value: &Value, pattern: &Pattern) -> Result<Option<HashMap<String, Value>>> {
        let mut bindings = HashMap::new();

        if Self::pattern_matches_recursive(value, pattern, &mut bindings)? {
            Ok(Some(bindings))
        } else {
            Ok(None)
        }
    }

    /// Recursive pattern matching helper
    fn pattern_matches_recursive(
        value: &Value,
        pattern: &Pattern,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<bool> {
        match (value, pattern) {
            // Wildcard matches everything and literal Unit
            (_, Pattern::Wildcard) | (Value::Unit, Pattern::Literal(Literal::Unit)) => Ok(true),

            // Literal patterns
            (Value::Int(v), Pattern::Literal(Literal::Integer(p))) => Ok(v == p),
            (Value::Float(v), Pattern::Literal(Literal::Float(p))) => {
                Ok((v - p).abs() < f64::EPSILON)
            }
            (Value::String(v), Pattern::Literal(Literal::String(p))) => Ok(v == p),
            (Value::Bool(v), Pattern::Literal(Literal::Bool(p))) => Ok(v == p),

            // Identifier patterns (bind to variable)
            (value, Pattern::Identifier(name)) => {
                bindings.insert(name.clone(), value.clone());
                Ok(true)
            }

            // List patterns
            (Value::List(values), Pattern::List(patterns)) => {
                if values.len() != patterns.len() {
                    return Ok(false);
                }

                for (value, pattern) in values.iter().zip(patterns.iter()) {
                    if !Self::pattern_matches_recursive(value, pattern, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            // Tuple patterns (treat as list for now)
            (Value::List(values), Pattern::Tuple(patterns)) => {
                if values.len() != patterns.len() {
                    return Ok(false);
                }

                for (value, pattern) in values.iter().zip(patterns.iter()) {
                    if !Self::pattern_matches_recursive(value, pattern, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            // OR patterns - try each alternative
            (value, Pattern::Or(patterns)) => {
                for pattern in patterns {
                    let mut temp_bindings = HashMap::new();
                    if Self::pattern_matches_recursive(value, pattern, &mut temp_bindings)? {
                        // Merge bindings
                        for (name, val) in temp_bindings {
                            bindings.insert(name, val);
                        }
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            // Range patterns (simplified implementation)
            (
                Value::Int(v),
                Pattern::Range {
                    start,
                    end,
                    inclusive,
                },
            ) => {
                // For simplicity, only handle integer literal patterns in ranges
                if let (
                    Pattern::Literal(Literal::Integer(start_val)),
                    Pattern::Literal(Literal::Integer(end_val)),
                ) = (start.as_ref(), end.as_ref())
                {
                    if *inclusive {
                        Ok(*start_val <= *v && *v <= *end_val)
                    } else {
                        Ok(*start_val <= *v && *v < *end_val)
                    }
                } else {
                    bail!("Complex range patterns not yet supported");
                }
            }

            // Result patterns (Ok/Err)
            (value, Pattern::Ok(inner_pattern)) => {
                // Check if value is a Result::Ok variant
                if let Some(ok_value) = Self::extract_result_ok(value) {
                    Self::pattern_matches_recursive(&ok_value, inner_pattern, bindings)
                } else {
                    Ok(false)
                }
            }
            (value, Pattern::Err(inner_pattern)) => {
                // Check if value is a Result::Err variant
                if let Some(err_value) = Self::extract_result_err(value) {
                    Self::pattern_matches_recursive(&err_value, inner_pattern, bindings)
                } else {
                    Ok(false)
                }
            }

            // Option patterns (Some/None)
            (value, Pattern::Some(inner_pattern)) => {
                // Check if value is an Option::Some variant
                if let Some(some_value) = Self::extract_option_some(value) {
                    Self::pattern_matches_recursive(&some_value, inner_pattern, bindings)
                } else {
                    Ok(false)
                }
            }
            (value, Pattern::None) => {
                // Check if value is an Option::None variant
                Ok(Self::is_option_none(value))
            }

            // Struct patterns
            (Value::Object(obj_fields), Pattern::Struct { name: _struct_name, fields: pattern_fields, has_rest: _ }) => {
                // For now, we don't check struct name since objects are generic
                // Check all pattern fields match
                for pattern_field in pattern_fields {
                    let field_name = &pattern_field.name;
                    
                    // Find the corresponding field in the object
                    if let Some(field_value) = obj_fields.get(field_name) {
                        // Check if pattern matches (if specified)
                        if let Some(pattern) = &pattern_field.pattern {
                            if !Self::pattern_matches_recursive(field_value, pattern, bindings)? {
                                return Ok(false);
                            }
                        } else {
                            // Shorthand pattern ({ x } instead of { x: x })
                            // This creates a binding: x => field_value
                            bindings.insert(field_name.clone(), field_value.clone());
                        }
                    } else {
                        // Required field not found in struct
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            // Qualified name patterns (like Status::Ok, Ordering::Less)
            (Value::EnumVariant { enum_name, variant_name, data: _ }, Pattern::QualifiedName(path)) => {
                // Match if qualified name matches enum variant  
                if path.len() >= 2 {
                    let pattern_enum = &path[path.len() - 2];
                    let pattern_variant = &path[path.len() - 1];
                    Ok(enum_name == pattern_enum && variant_name == pattern_variant)
                } else {
                    Ok(false)
                }
            }
            
            // Qualified name patterns should also match qualified name expressions
            (value, Pattern::QualifiedName(path)) => {
                // Convert value to string and compare with pattern path
                let value_str = format!("{value}");
                let pattern_str = path.join("::");
                Ok(value_str == pattern_str)
            }

            // Type mismatches
            _ => Ok(false),
        }
    }

    /// Extract value from `Result::Ok` variant (complexity: 4)
    fn extract_result_ok(value: &Value) -> Option<Value> {
        match value {
            Value::EnumVariant { enum_name, variant_name, data } => {
                if enum_name == "Result" && variant_name == "Ok" {
                    data.as_ref()?.first().cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Extract value from `Result::Err` variant (complexity: 4)
    fn extract_result_err(value: &Value) -> Option<Value> {
        match value {
            Value::EnumVariant { enum_name, variant_name, data } => {
                if enum_name == "Result" && variant_name == "Err" {
                    data.as_ref()?.first().cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Extract value from `Option::Some` variant (complexity: 4)
    fn extract_option_some(value: &Value) -> Option<Value> {
        match value {
            Value::EnumVariant { enum_name, variant_name, data } => {
                if enum_name == "Option" && variant_name == "Some" {
                    data.as_ref()?.first().cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if value is `Option::None` variant (complexity: 3)
    fn is_option_none(value: &Value) -> bool {
        match value {
            Value::EnumVariant { enum_name, variant_name, data: _ } => {
                enum_name == "Option" && variant_name == "None"
            }
            _ => false,
        }
    }

    /// Evaluate binary operations
    fn evaluate_binary(lhs: &Value, op: BinaryOp, rhs: &Value) -> Result<Value> {
        use Value::{Bool, Float, Int};

        match (lhs, op, rhs) {
            // Integer arithmetic with overflow checking
            (Int(a), BinaryOp::Add, Int(b)) => a
                .checked_add(*b)
                .map(Int)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in addition: {} + {}", a, b)),
            (Int(a), BinaryOp::Subtract, Int(b)) => a
                .checked_sub(*b)
                .map(Int)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in subtraction: {} - {}", a, b)),
            (Int(a), BinaryOp::Multiply, Int(b)) => a.checked_mul(*b).map(Int).ok_or_else(|| {
                anyhow::anyhow!("Integer overflow in multiplication: {} * {}", a, b)
            }),
            (Int(a), BinaryOp::Divide, Int(b)) => {
                if *b == 0 {
                    bail!("Division by zero");
                }
                Ok(Int(a / b))
            }
            (Int(a), BinaryOp::Modulo, Int(b)) => {
                if *b == 0 {
                    bail!("Modulo by zero");
                }
                Ok(Int(a % b))
            }
            (Int(a), BinaryOp::Power, Int(b)) => {
                if *b < 0 {
                    bail!("Negative integer powers not supported in integer context");
                }
                let exp =
                    u32::try_from(*b).map_err(|_| anyhow::anyhow!("Power exponent too large"))?;
                a.checked_pow(exp)
                    .map(Int)
                    .ok_or_else(|| anyhow::anyhow!("Integer overflow in power: {} ^ {}", a, b))
            }

            // Float arithmetic
            (Float(a), BinaryOp::Add, Float(b)) => Ok(Float(a + b)),
            (Float(a), BinaryOp::Subtract, Float(b)) => Ok(Float(a - b)),
            (Float(a), BinaryOp::Multiply, Float(b)) => Ok(Float(a * b)),
            (Float(a), BinaryOp::Divide, Float(b)) => {
                if *b == 0.0 {
                    bail!("Division by zero");
                }
                Ok(Float(a / b))
            }
            (Float(a), BinaryOp::Power, Float(b)) => Ok(Float(a.powf(*b))),

            // String operations
            (Value::String(a), BinaryOp::Add, Value::String(b)) => {
                Ok(Value::String(format!("{a}{b}")))
            }

            // Comparisons - Integers
            (Int(a), BinaryOp::Less, Int(b)) => Ok(Bool(a < b)),
            (Int(a), BinaryOp::LessEqual, Int(b)) => Ok(Bool(a <= b)),
            (Int(a), BinaryOp::Greater, Int(b)) => Ok(Bool(a > b)),
            (Int(a), BinaryOp::GreaterEqual, Int(b)) => Ok(Bool(a >= b)),
            (Int(a), BinaryOp::Equal, Int(b)) => Ok(Bool(a == b)),
            (Int(a), BinaryOp::NotEqual, Int(b)) => Ok(Bool(a != b)),

            // Comparisons - Strings
            (Value::String(a), BinaryOp::Equal, Value::String(b)) => Ok(Bool(a == b)),
            (Value::String(a), BinaryOp::NotEqual, Value::String(b)) => Ok(Bool(a != b)),

            // Comparisons - Booleans
            (Bool(a), BinaryOp::Equal, Bool(b)) => Ok(Bool(a == b)),
            (Bool(a), BinaryOp::NotEqual, Bool(b)) => Ok(Bool(a != b)),

            // Boolean logic
            (Bool(a), BinaryOp::And, Bool(b)) => Ok(Bool(*a && *b)),
            (Bool(a), BinaryOp::Or, Bool(b)) => Ok(Bool(*a || *b)),

            // Bitwise operations on integers
            (Int(a), BinaryOp::BitwiseAnd, Int(b)) => Ok(Int(a & b)),
            (Int(a), BinaryOp::BitwiseOr, Int(b)) => Ok(Int(a | b)),
            (Int(a), BinaryOp::BitwiseXor, Int(b)) => Ok(Int(a ^ b)),
            (Int(a), BinaryOp::LeftShift, Int(b)) => Ok(Int(a << b)),

            _ => bail!(
                "Type mismatch in binary operation: {:?} {:?} {:?}",
                lhs,
                op,
                rhs
            ),
        }
    }

    /// Evaluate unary operations
    fn evaluate_unary(op: UnaryOp, val: &Value) -> Result<Value> {
        use Value::{Bool, Float, Int};

        match (op, val) {
            (UnaryOp::Negate, Int(n)) => Ok(Int(-n)),
            (UnaryOp::Negate, Float(f)) => Ok(Float(-f)),
            (UnaryOp::Not, Bool(b)) => Ok(Bool(!b)),
            (UnaryOp::BitwiseNot, Int(n)) => Ok(Int(!n)),
            (UnaryOp::Reference, v) => {
                // References in the REPL context just return the value
                // In a real implementation, this would create a reference/pointer
                // For now, we'll just return the value as references are primarily
                // useful for the transpiled code, not the interpreted REPL
                Ok(v.clone())
            }
            _ => bail!("Type mismatch in unary operation: {:?} {:?}", op, val),
        }
    }

    /// Run the interactive REPL
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Readline initialization fails
    /// - User input cannot be read
    /// - Commands fail to execute
    pub fn run(&mut self) -> Result<()> {
        // Welcome message is printed in bin/ruchy.rs, not here
        println!();

        // Configure rustyline with enhanced features
        let config = Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)?
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();

        let mut rl = rustyline::Editor::<RuchyCompleter, DefaultHistory>::with_config(config)?;

        // Set up tab completion
        let completer = RuchyCompleter::new();
        rl.set_helper(Some(completer));

        // Load history if it exists
        let history_path = self.temp_dir.join("history.txt");
        let _ = rl.load_history(&history_path);

        let mut multiline_buffer = String::new();
        let mut in_multiline = false;

        loop {
            let prompt = if in_multiline {
                format!("{} ", "   ...".bright_black())
            } else {
                format!("{} ", "ruchy>".bright_green())
            };
            let readline = rl.readline(&prompt);

            match readline {
                Ok(line) => {
                    // Skip empty lines unless we're in multiline mode
                    if line.trim().is_empty() && !in_multiline {
                        continue;
                    }

                    // Handle commands (only when not in multiline mode)
                    if !in_multiline && line.starts_with(':') {
                        let (should_quit, output) = self.handle_command_with_output(&line)?;
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                        if should_quit {
                            break; // :quit command
                        }
                        continue;
                    }

                    // Check if this starts a multiline expression
                    if !in_multiline && Self::needs_continuation(&line) {
                        multiline_buffer.clone_from(&line);
                        in_multiline = true;
                        continue;
                    }

                    // If in multiline mode, accumulate lines
                    if in_multiline {
                        multiline_buffer.push('\n');
                        multiline_buffer.push_str(&line);

                        // Check if we have a complete expression
                        if !Self::needs_continuation(&multiline_buffer) {
                            // Add complete expression to history
                            let _ = rl.add_history_entry(multiline_buffer.as_str());

                            // Evaluate the complete expression
                            match self.eval(&multiline_buffer) {
                                Ok(result) => {
                                    println!("{}", result.bright_white());
                                }
                                Err(e) => {
                                    eprintln!("{}: {}", "Error".bright_red().bold(), e);
                                }
                            }

                            // Reset multiline mode
                            multiline_buffer.clear();
                            in_multiline = false;
                        }
                    } else {
                        // Single line expression
                        let _ = rl.add_history_entry(line.as_str());

                        // Evaluate the expression
                        match self.eval(&line) {
                            Ok(result) => {
                                println!("{}", result.bright_white());
                            }
                            Err(e) => {
                                eprintln!("{}: {}", "Error".bright_red().bold(), e);
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("\nUse :quit to exit");
                }
                Err(ReadlineError::Eof) => {
                    println!("\nGoodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {err:?}");
                    break;
                }
            }
        }

        // Save history
        let _ = rl.save_history(&history_path);
        Ok(())
    }

    /// Handle REPL commands and return output as string (for testing)
    fn handle_command_with_output(&mut self, command: &str) -> Result<(bool, String)> {
        let mut output = String::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        let should_quit = match parts.first().copied() {
            Some(":quit" | ":q") => {
                if self.mode != ReplMode::Normal {
                    // In a special mode, :quit returns to normal
                    self.mode = ReplMode::Normal;
                    output = "Returned to normal mode".to_string();
                    false
                } else {
                    // In normal mode, :quit exits REPL
                    true
                }
            }
            Some(":history") => {
                if self.history.is_empty() {
                    output = "No history".to_string();
                } else {
                    for (i, item) in self.history.iter().enumerate() {
                        output.push_str(&format!("{}: {}\n", i + 1, item));
                    }
                }
                false
            }
            Some(":clear") => {
                self.history.clear();
                self.definitions.clear();
                self.bindings.clear();
                self.result_history.clear();
                output = "Session cleared".to_string();
                false
            }
            Some(":bindings" | ":env") => {
                if self.bindings.is_empty() {
                    output = "No bindings".to_string();
                } else {
                    for (name, value) in &self.bindings {
                        output.push_str(&format!("{}: {}\n", name, value));
                    }
                }
                false
            }
            Some(":compile") => {
                match self.compile_session() {
                    Ok(()) => output = "Session compiled successfully".to_string(),
                    Err(e) => output = format!("Compilation failed: {}", e),
                }
                false
            }
            Some(":load") if parts.len() == 2 => {
                match self.load_file(parts[1]) {
                    Ok(()) => output = format!("Loaded file: {}", parts[1]),
                    Err(e) => output = format!("Failed to load file: {}", e),
                }
                false
            }
            Some(":load") => {
                output = "Usage: :load <filename>".to_string();
                false
            }
            Some(":save") if parts.len() >= 2 => {
                let filename = command.strip_prefix(":save").unwrap_or("").trim();
                match self.save_session(filename) {
                    Ok(()) => output = format!("Session saved to {}", filename),
                    Err(e) => output = format!("Failed to save session: {}", e),
                }
                false
            }
            Some(":save") => {
                output = "Usage: :save <filename>".to_string();
                false
            }
            Some(":type") => {
                let expr = command.strip_prefix(":type").unwrap_or("").trim();
                if expr.is_empty() {
                    output = "Usage: :type <expression>".to_string();
                } else {
                    output = self.get_type_info_with_bindings(expr);
                }
                false
            }
            Some(":ast") => {
                let expr = command.strip_prefix(":ast").unwrap_or("").trim();
                if expr.is_empty() {
                    output = "Usage: :ast <expression>".to_string();
                } else {
                    output = Self::get_ast_info(expr);
                }
                false
            }
            Some(":reset") => {
                self.history.clear();
                self.definitions.clear();
                self.bindings.clear();
                self.result_history.clear();
                self.memory.reset();
                output = "REPL reset to initial state".to_string();
                false
            }
            Some(":search") => {
                let query = command.strip_prefix(":search").unwrap_or("").trim();
                if query.is_empty() {
                    output = "Usage: :search <query>\nSearch through command history with fuzzy matching".to_string();
                } else {
                    output = self.get_search_results(query);
                }
                false
            }
            // Mode switching commands
            Some(":normal") => {
                self.mode = ReplMode::Normal;
                output = "Switched to normal mode".to_string();
                false
            }
            Some(":shell") => {
                self.mode = ReplMode::Shell;
                output = "Switched to shell mode - all input will be executed as shell commands".to_string();
                false
            }
            Some(":pkg") => {
                self.mode = ReplMode::Pkg;
                output = "Switched to package mode - use 'search', 'install', 'list' commands".to_string();
                false
            }
            Some(":help" | ":h") if parts.len() == 1 => {
                // No argument - switch to help mode
                self.mode = ReplMode::Help;
                output = "Switched to help mode - type any keyword for documentation\nUse :normal to exit".to_string();
                false
            }
            Some(":help") if parts.len() > 1 => {
                // :help with argument shows help for specific topic
                let topic = parts[1];
                output = self.handle_help_command(topic)?;
                false
            }
            Some(":sql") => {
                self.mode = ReplMode::Sql;
                output = "Switched to SQL mode - execute SQL queries".to_string();
                false
            }
            Some(":math") => {
                self.mode = ReplMode::Math;
                output = "Switched to math mode - enhanced mathematical expressions".to_string();
                false
            }
            Some(":debug") => {
                self.mode = ReplMode::Debug;
                output = "Switched to debug mode - extra information shown".to_string();
                false
            }
            Some(":time") => {
                self.mode = ReplMode::Time;
                output = "Switched to time mode - execution timing shown".to_string();
                false
            }
            Some(":exit") => {
                self.mode = ReplMode::Normal;
                output = "Exited to normal mode".to_string();
                false
            }
            Some(":modes") => {
                output = "Available modes:\n".to_string();
                output.push_str("  normal - Standard Ruchy evaluation\n");
                output.push_str("  shell  - Execute shell commands\n");
                output.push_str("  pkg    - Package management\n");
                output.push_str("  help   - Interactive help\n");
                output.push_str("  sql    - SQL queries\n");
                output.push_str("  math   - Mathematical expressions\n");
                output.push_str("  debug  - Debug information\n");
                output.push_str("  time   - Execution timing\n");
                output.push_str("\nUse :mode_name to switch modes, :normal or :exit to return");
                false
            }
            _ => {
                output = format!("Unknown command: {}\nType :help for available commands", command);
                false
            }
        };
        
        Ok((should_quit, output))
    }

    /// Handle REPL commands (public for testing)
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails
    pub fn handle_command(&mut self, command: &str) -> Result<bool> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        match parts.first().copied() {
            Some(":quit" | ":q") => Ok(true),
            Some(":help" | ":h") => {
                Self::print_help();
                Ok(false)
            }
            Some(":history") => {
                for (i, item) in self.history.iter().enumerate() {
                    println!("{}: {}", i + 1, item);
                }
                Ok(false)
            }
            Some(":clear") => {
                self.history.clear();
                self.definitions.clear();
                self.bindings.clear();
                println!("Session cleared");
                Ok(false)
            }
            Some(":bindings" | ":env") => {
                if self.bindings.is_empty() {
                    println!("No bindings");
                } else {
                    for (name, value) in &self.bindings {
                        println!("{name}: {value}");
                    }
                }
                Ok(false)
            }
            Some(":compile") => {
                self.compile_session()?;
                Ok(false)
            }
            Some(":load") if parts.len() == 2 => {
                self.load_file(parts[1])?;
                Ok(false)
            }
            Some(":type") => {
                // Get the rest of the line after :type
                let expr = command.strip_prefix(":type").unwrap_or("").trim();
                if expr.is_empty() {
                    println!("Usage: :type <expression>");
                } else {
                    Self::show_type(expr);
                }
                Ok(false)
            }
            Some(":ast") => {
                // Get the rest of the line after :ast
                let expr = command.strip_prefix(":ast").unwrap_or("").trim();
                if expr.is_empty() {
                    println!("Usage: :ast <expression>");
                } else {
                    Self::show_ast(expr);
                }
                Ok(false)
            }
            Some(":reset") => {
                // Full reset - clear everything and restart
                self.history.clear();
                self.definitions.clear();
                self.bindings.clear();
                self.memory.reset();
                println!("REPL reset to initial state");
                Ok(false)
            }
            Some(cmd) if cmd.starts_with(":search") => {
                let query = cmd.strip_prefix(":search").unwrap_or("").trim();
                if query.is_empty() {
                    println!("Usage: :search <query>");
                    println!("Search through command history with fuzzy matching");
                } else {
                    self.search_history(query);
                }
                Ok(false)
            }
            Some(cmd) if cmd.starts_with(":save") => {
                let filename = cmd.strip_prefix(":save").unwrap_or("").trim();
                if filename.is_empty() {
                    println!("Usage: :save <filename>");
                    println!("Save current session to a file");
                } else {
                    match self.save_session(filename) {
                        Ok(()) => println!("Session saved to {}", filename.bright_green()),
                        Err(e) => eprintln!("Failed to save session: {e}"),
                    }
                }
                Ok(false)
            }
            _ => {
                eprintln!("Unknown command: {command}");
                Self::print_help();
                Ok(false)
            }
        }
    }

    /// Get help text as string
    fn get_help_text() -> String {
        let mut help = String::new();
        help.push_str("Available commands:\n");
        help.push_str("  :help, :h       - Show this help message\n");
        help.push_str("  :quit, :q       - Exit the REPL\n");
        help.push_str("  :history        - Show evaluation history\n");
        help.push_str("  :search <query> - Search history with fuzzy matching\n");
        help.push_str("  :clear          - Clear definitions and history\n");
        help.push_str("  :reset          - Full reset to initial state\n");
        help.push_str("  :bindings, :env - Show current variable bindings\n");
        help.push_str("  :type <expr>    - Show type of expression\n");
        help.push_str("  :ast <expr>     - Show AST of expression\n");
        help.push_str("  :compile        - Compile and run the session\n");
        help.push_str("  :load <file>    - Load and evaluate a file\n");
        help.push_str("  :save <file>    - Save session to file\n");
        help
    }
    
    /// Print help message
    fn print_help() {
        println!("{}", Self::get_help_text());
    }
    
    /// Get type information as string  
    fn get_type_info(expr: &str) -> String {
        match Parser::new(expr).parse() {
            Ok(ast) => {
                // Create an inference context for type checking
                let mut ctx = crate::middleend::InferenceContext::new();
                
                // Infer the type
                match ctx.infer(&ast) {
                    Ok(ty) => format!("Type: {}", ty),
                    Err(e) => format!("Type inference error: {}", e),
                }
            }
            Err(e) => format!("Parse error: {}", e),
        }
    }
    
    /// Get type information with REPL bindings context
    fn get_type_info_with_bindings(&self, expr: &str) -> String {
        // If the expression is a simple identifier, check bindings first
        if let Ok(_) = Parser::new(expr).parse() {
            if let Some(value) = self.bindings.get(expr) {
                // Infer type from the value
                let type_name = match value {
                    Value::Int(_) => "Integer",
                    Value::Float(_) => "Float",  
                    Value::String(_) => "String",
                    Value::Bool(_) => "Bool",
                    Value::List(_) => "List",
                    Value::Function { .. } => "Function",
                    Value::Lambda { .. } => "Lambda",
                    Value::Object(_) => "Object",
                    Value::Tuple(_) => "Tuple",
                    Value::Char(_) => "Char",
                    Value::DataFrame { .. } => "DataFrame",
                    Value::HashMap(_) => "HashMap",
                    Value::HashSet(_) => "HashSet",
                    Value::Range { .. } => "Range",
                    Value::EnumVariant { enum_name, variant_name, .. } => {
                        &format!("{}::{}", enum_name, variant_name)
                    }
                    Value::Unit => "Unit"
                };
                return format!("Type: {}", type_name);
            }
        }
        
        // Fall back to regular type inference
        Self::get_type_info(expr)
    }
    
    /// Get AST information as string
    fn get_ast_info(expr: &str) -> String {
        match Parser::new(expr).parse() {
            Ok(ast) => format!("{:#?}", ast),
            Err(e) => format!("Parse error: {}", e),
        }
    }
    
    /// Get search results as string
    fn get_search_results(&self, query: &str) -> String {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        for (i, item) in self.history.iter().enumerate() {
            if item.to_lowercase().contains(&query_lower) {
                results.push(format!("{}: {}", i + 1, item));
            }
        }
        
        if results.is_empty() {
            format!("No matches found for '{}'", query)
        } else {
            results.join("\n")
        }
    }
    
    /// Execute a shell command and return its output
    fn execute_shell_command(&self, command: &str) -> Result<String> {
        use std::process::Command;
        
        // Execute command through shell
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .context(format!("Failed to execute shell command: {}", command))?;
        
        // Combine stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !output.status.success() {
            // If command failed, return error with stderr
            if !stderr.is_empty() {
                bail!("Shell command failed: {}", stderr);
            } else {
                bail!("Shell command failed with exit code: {:?}", output.status.code());
            }
        }
        
        // Return stdout (stderr is usually empty for successful commands)
        Ok(stdout.trim_end().to_string())
    }
    
    /// Basic introspection with single ?
    fn basic_introspection(&self, target: &str) -> Result<String> {
        // Check if target exists in bindings
        if let Some(value) = self.bindings.get(target) {
            let type_name = self.get_value_type_name(value);
            let value_str = self.format_value_brief(value);
            return Ok(format!("Type: {}\nValue: {}", type_name, value_str));
        }
        
        // Check if it's a builtin function
        if self.is_builtin_function(target) {
            return Ok(format!("Type: Builtin Function\nName: {}", target));
        }
        
        // Try to evaluate the expression and introspect result
        if let Ok(ast) = Parser::new(target).parse() {
            // Try to get type information
            let mut ctx = crate::middleend::InferenceContext::new();
            if let Ok(ty) = ctx.infer(&ast) {
                return Ok(format!("Type: {}", ty));
            }
        }
        
        bail!("'{}' is not defined or cannot be introspected", target)
    }
    
    /// Detailed introspection with double ??
    fn detailed_introspection(&self, target: &str) -> Result<String> {
        // Check if target exists in bindings  
        if let Some(value) = self.bindings.get(target) {
            return Ok(self.format_detailed_introspection(target, value));
        }
        
        // Check if it's a builtin function
        if self.is_builtin_function(target) {
            return Ok(self.format_builtin_help(target));
        }
        
        bail!("'{}' is not defined or cannot be introspected", target)
    }
    
    /// Check if a name is a builtin function
    fn is_builtin_function(&self, name: &str) -> bool {
        matches!(name, "println" | "print" | "len" | "push" | "pop" | "insert" | 
                       "remove" | "clear" | "contains" | "index_of" | "slice" |
                       "split" | "join" | "trim" | "to_upper" | "to_lower" |
                       "replace" | "starts_with" | "ends_with" | "parse" |
                       "type" | "str" | "int" | "float" | "bool" |
                       "sqrt" | "pow" | "abs" | "min" | "max" | "floor" | "ceil" | "round")
    }
    
    /// Get type name for a value
    fn get_value_type_name(&self, value: &Value) -> &str {
        match value {
            Value::Int(_) => "Integer",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Bool(_) => "Bool",
            Value::Char(_) => "Char",
            Value::List(_) => "List",
            Value::Tuple(_) => "Tuple",
            Value::Function { .. } => "Function",
            Value::Lambda { .. } => "Lambda",
            Value::Object(_) => "Object",
            Value::HashMap(_) => "HashMap",
            Value::HashSet(_) => "HashSet",
            Value::Range { .. } => "Range",
            Value::DataFrame { .. } => "DataFrame",
            Value::EnumVariant { enum_name, variant_name, .. } => {
                // Return a static str by leaking - safe for REPL lifetime
                Box::leak(format!("{}::{}", enum_name, variant_name).into_boxed_str())
            }
            Value::Unit => "Unit",
        }
    }
    
    /// Format value briefly for introspection
    fn format_value_brief(&self, value: &Value) -> String {
        match value {
            Value::List(items) => format!("[{} items]", items.len()),
            Value::Object(fields) => {
                let field_names: Vec<_> = fields.keys().cloned().collect();
                format!("{{{}}}",field_names.join(", "))
            }
            Value::Function { name, params, .. } => {
                format!("fn {}({})", name, params.join(", "))
            }
            Value::Lambda { params, .. } => {
                format!("|{}| -> ...", params.join(", "))
            }
            _ => value.to_string(),
        }
    }
    
    /// Format detailed introspection output
    fn format_detailed_introspection(&self, name: &str, value: &Value) -> String {
        let mut output = String::new();
        output.push_str(&format!("Name: {}\n", name));
        output.push_str(&format!("Type: {}\n", self.get_value_type_name(value)));
        
        match value {
            Value::Function { name: fn_name, params, body } => {
                output.push_str(&format!("Source: fn {}({}) {{\n", fn_name, params.join(", ")));
                output.push_str(&format!("  {}\n", self.format_expr_source(body)));
                output.push_str("}\n");
                output.push_str(&format!("Parameters: {}\n", params.join(", ")));
            }
            Value::Lambda { params, body } => {
                output.push_str(&format!("Source: |{}| {{\n", params.join(", ")));
                output.push_str(&format!("  {}\n", self.format_expr_source(body)));
                output.push_str("}\n");
                output.push_str(&format!("Parameters: {}\n", params.join(", ")));
            }
            Value::Object(fields) => {
                output.push_str("Fields:\n");
                for (key, val) in fields {
                    output.push_str(&format!("  {}: {}\n", key, self.get_value_type_name(val)));
                }
            }
            Value::List(items) => {
                output.push_str(&format!("Length: {}\n", items.len()));
                if !items.is_empty() {
                    output.push_str(&format!("First: {}\n", items[0]));
                    if items.len() > 1 {
                        output.push_str(&format!("Last: {}\n", items[items.len() - 1]));
                    }
                }
            }
            _ => {
                output.push_str(&format!("Value: {}\n", value));
            }
        }
        
        output
    }
    
    /// Format expression source code
    fn format_expr_source(&self, expr: &Expr) -> String {
        // Format the expression in a more readable way
        self.expr_to_source_string(expr, 0)
    }
    
    /// Convert expression to source string
    fn expr_to_source_string(&self, expr: &Expr, indent: usize) -> String {
        use crate::frontend::ast::ExprKind;
        let indent_str = "  ".repeat(indent);
        
        match &expr.kind {
            ExprKind::Binary { left, op, right } => {
                format!("{} {} {}", 
                    self.expr_to_source_string(left, 0),
                    op,
                    self.expr_to_source_string(right, 0))
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                let mut s = format!("if {} {{\n{}{}\n{}}}", 
                    self.expr_to_source_string(condition, 0),
                    "  ".repeat(indent + 1),
                    self.expr_to_source_string(then_branch, indent + 1),
                    indent_str);
                if let Some(else_b) = else_branch {
                    s.push_str(&format!(" else {{\n{}{}\n{}}}",
                        "  ".repeat(indent + 1),
                        self.expr_to_source_string(else_b, indent + 1),
                        indent_str));
                }
                s
            }
            ExprKind::Call { func, args } => {
                if let ExprKind::Identifier(name) = &func.kind {
                    format!("{}({})", name, 
                        args.iter()
                            .map(|a| self.expr_to_source_string(a, 0))
                            .collect::<Vec<_>>()
                            .join(", "))
                } else {
                    format!("(call ...)")
                }
            }
            ExprKind::Identifier(name) => name.clone(),
            ExprKind::Literal(lit) => format!("{:?}", lit),
            ExprKind::Block(exprs) => {
                if exprs.len() == 1 {
                    self.expr_to_source_string(&exprs[0], indent)
                } else {
                    exprs.iter()
                        .map(|e| self.expr_to_source_string(e, indent))
                        .collect::<Vec<_>>()
                        .join("; ")
                }
            }
            _ => format!("{:?}", expr.kind).chars().take(50).collect()
        }
    }
    
    /// Format help for builtin functions
    fn format_builtin_help(&self, name: &str) -> String {
        match name {
            "println" => "println(value)\n  Prints a value to stdout with newline\n  Parameters: value - Any value to print".to_string(),
            "print" => "print(value)\n  Prints a value to stdout without newline\n  Parameters: value - Any value to print".to_string(),
            "len" => "len(collection)\n  Returns the length of a collection\n  Parameters: collection - List, String, or other collection".to_string(),
            "type" => "type(value)\n  Returns the type of a value\n  Parameters: value - Any value".to_string(),
            "str" => "str(value)\n  Converts a value to string\n  Parameters: value - Any value to convert".to_string(),
            _ => format!("{}\n  Builtin function\n  (documentation not available)", name),
        }
    }
    
    /// Evaluate type() function
    fn evaluate_type_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("type() expects 1 argument, got {}", args.len());
        }
        
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let type_name = self.get_value_type_name(&value);
        Ok(Value::String(type_name.to_string()))
    }
    
    /// Evaluate summary() function
    fn evaluate_summary_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("summary() expects 1 argument, got {}", args.len());
        }
        
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let summary = match &value {
            Value::List(items) => format!("List with {} items", items.len()),
            Value::Object(fields) => format!("Object with {} fields", fields.len()),
            Value::String(s) => format!("String of length {}", s.len()),
            Value::DataFrame { columns } => format!("DataFrame with {} columns", columns.len()),
            _ => format!("{} value", self.get_value_type_name(&value)),
        };
        Ok(Value::String(summary))
    }
    
    /// Evaluate dir() function
    fn evaluate_dir_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("dir() expects 1 argument, got {}", args.len());
        }
        
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let members = match value {
            Value::Object(fields) => {
                fields.keys().cloned().collect::<Vec<_>>()
            }
            _ => vec![],
        };
        
        Ok(Value::String(members.join(", ")))
    }
    
    /// Evaluate help() function
    fn evaluate_help_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("help() expects 1 argument, got {}", args.len());
        }
        
        // Check if it's a builtin function first
        if let ExprKind::Identifier(name) = &args[0].kind {
            if self.is_builtin_function(name) {
                return Ok(Value::String(self.format_builtin_help(name)));
            }
        }
        
        // Try to evaluate the argument and get its type
        match self.evaluate_expr(&args[0], deadline, depth + 1) {
            Ok(value) => {
                let help_text = match value {
                    Value::Function { name, params, .. } => {
                        format!("Function: {}\nParameters: {}", name, params.join(", "))
                    }
                    Value::Lambda { params, .. } => {
                        format!("Lambda function\nParameters: {}", params.join(", "))
                    }
                    _ => {
                        format!("Type: {}", self.get_value_type_name(&value))
                    }
                };
                Ok(Value::String(help_text))
            }
            Err(_) => {
                // If evaluation fails, just return a generic message
                Ok(Value::String("No help available for this value".to_string()))
            }
        }
    }
    
    /// Evaluate whos() function - lists all variables with types
    fn evaluate_whos_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        let filter = if args.len() == 1 {
            // Get type filter
            let val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
            if let Value::String(s) = val {
                Some(s)
            } else {
                None
            }
        } else {
            None
        };
        
        let mut output = Vec::new();
        for (name, value) in &self.bindings {
            let type_name = self.get_value_type_name(value);
            if let Some(ref filter_type) = filter {
                if type_name != filter_type {
                    continue;
                }
            }
            output.push(format!("{}: {}", name, type_name));
        }
        
        Ok(Value::String(output.join("\n")))
    }
    
    /// Evaluate who() function - simple list of variable names
    fn evaluate_who_function(&mut self, _args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        let names: Vec<_> = self.bindings.keys().cloned().collect();
        Ok(Value::String(names.join(", ")))
    }
    
    /// Evaluate clear!() function - clears workspace
    fn evaluate_clear_bang_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.is_empty() {
            // Clear all bindings
            let count = self.bindings.len();
            self.bindings.clear();
            Ok(Value::String(format!("Cleared {} variables", count)))
        } else {
            // Clear matching pattern
            let pattern = self.evaluate_expr(&args[0], deadline, depth + 1)?;
            if let Value::String(pat) = pattern {
                let mut cleared = 0;
                let pattern_prefix = pat.trim_end_matches('*');
                let keys_to_remove: Vec<_> = self.bindings.keys()
                    .filter(|k| k.starts_with(&pattern_prefix))
                    .cloned()
                    .collect();
                for key in keys_to_remove {
                    self.bindings.remove(&key);
                    cleared += 1;
                }
                Ok(Value::String(format!("Cleared {} variables", cleared)))
            } else {
                bail!("clear! pattern must be a string")
            }
        }
    }
    
    /// Evaluate save_image() function
    fn evaluate_save_image_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("save_image() expects 1 argument (filename), got {}", args.len());
        }
        
        let filename = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        if let Value::String(path) = filename {
            // Generate Ruchy code to recreate workspace
            let mut content = String::new();
            content.push_str("// Workspace image\n");
            content.push_str("// Generated by save_image()\n\n");
            
            // Save all bindings
            for (name, value) in &self.bindings {
                match value {
                    Value::Int(n) => content.push_str(&format!("let {} = {}\n", name, n)),
                    Value::Float(f) => content.push_str(&format!("let {} = {}\n", name, f)),
                    Value::String(s) => content.push_str(&format!("let {} = \"{}\"\n", name, s.replace('"', "\\\""))),
                    Value::Bool(b) => content.push_str(&format!("let {} = {}\n", name, b)),
                    Value::List(items) => {
                        content.push_str(&format!("let {} = [", name));
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 { content.push_str(", "); }
                            content.push_str(&format!("{}", item));
                        }
                        content.push_str("]\n");
                    }
                    Value::Function { name: fn_name, params, body } => {
                        content.push_str(&format!("fn {}({}) {{ {} }}\n", 
                            fn_name, params.join(", "), 
                            self.format_expr_source(body)));
                    }
                    _ => {} // Skip complex types for now
                }
            }
            
            // Write to file
            fs::write(&path, content)?;
            Ok(Value::String(format!("Workspace saved to {}", path)))
        } else {
            bail!("save_image() requires a string filename")
        }
    }
    
    /// Evaluate workspace() function
    fn evaluate_workspace_function(&mut self, _args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        let var_count = self.bindings.len();
        let func_count = self.bindings.values()
            .filter(|v| matches!(v, Value::Function { .. } | Value::Lambda { .. }))
            .count();
        
        Ok(Value::String(format!("{} variables, {} functions", var_count, func_count)))
    }
    
    /// Evaluate locals() function
    fn evaluate_locals_function(&mut self, _args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        // For now, same as globals since we don't have proper scoping
        self.evaluate_globals_function(&[], Instant::now(), 0)
    }
    
    /// Evaluate globals() function
    fn evaluate_globals_function(&mut self, _args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        let mut output = Vec::new();
        for (name, value) in &self.bindings {
            output.push(format!("{}: {}", name, self.get_value_type_name(value)));
        }
        Ok(Value::String(output.join("\n")))
    }
    
    /// Evaluate reset() function
    fn evaluate_reset_function(&mut self, _args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        self.bindings.clear();
        self.history.clear();
        self.result_history.clear();
        self.definitions.clear();
        self.memory.reset();
        Ok(Value::String("Workspace reset".to_string()))
    }
    
    /// Evaluate del() function
    fn evaluate_del_function(&mut self, args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("del() expects 1 argument, got {}", args.len());
        }
        
        // Get the name to delete
        if let ExprKind::Identifier(name) = &args[0].kind {
            if self.bindings.remove(name).is_some() {
                Ok(Value::Unit)
            } else {
                bail!("Variable '{}' not found", name)
            }
        } else {
            bail!("del() requires a variable name")
        }
    }
    
    /// Evaluate exists() function
    fn evaluate_exists_function(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("exists() expects 1 argument, got {}", args.len());
        }
        
        let name_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        if let Value::String(name) = name_val {
            Ok(Value::Bool(self.bindings.contains_key(&name)))
        } else {
            bail!("exists() requires a string variable name")
        }
    }
    
    /// Evaluate memory_info() function
    fn evaluate_memory_info_function(&mut self, _args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        let current = self.memory.current;
        let max = self.memory.max_size;
        let kb = current / 1024;
        Ok(Value::String(format!("Memory: {} bytes ({} KB) / {} max", current, kb, max)))
    }
    
    /// Evaluate time_info() function
    fn evaluate_time_info_function(&mut self, _args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        // For simplicity, just return a placeholder
        Ok(Value::String("Session time: active".to_string()))
    }

    /// Show the type of an expression
    fn show_type(expr: &str) {
        match Parser::new(expr).parse() {
            Ok(ast) => {
                // Create an inference context for type checking
                let mut ctx = crate::middleend::InferenceContext::new();

                // Infer the type
                match ctx.infer(&ast) {
                    Ok(ty) => {
                        println!("Type: {ty}");
                    }
                    Err(e) => {
                        eprintln!("Type inference error: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Parse error: {e}");
            }
        }
    }

    /// Show the AST of an expression
    fn show_ast(expr: &str) {
        match Parser::new(expr).parse() {
            Ok(ast) => {
                println!("{ast:#?}");
            }
            Err(e) => {
                eprintln!("Parse error: {e}");
            }
        }
    }

    /// Check if input needs continuation (incomplete expression)
    pub fn needs_continuation(input: &str) -> bool {
        let trimmed = input.trim();

        // Empty input doesn't need continuation
        if trimmed.is_empty() {
            return false;
        }

        // Count braces, brackets, and parentheses
        let mut brace_depth = 0;
        let mut bracket_depth = 0;
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in trimmed.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string => brace_depth -= 1,
                '[' if !in_string => bracket_depth += 1,
                ']' if !in_string => bracket_depth -= 1,
                '(' if !in_string => paren_depth += 1,
                ')' if !in_string => paren_depth -= 1,
                _ => {}
            }
        }

        // Need continuation if any delimiters are unmatched
        brace_depth > 0 || bracket_depth > 0 || paren_depth > 0 || in_string ||
        // Or if line ends with certain tokens that expect continuation
        trimmed.ends_with('=') ||
        trimmed.ends_with("->") ||
        trimmed.ends_with("=>") ||
        trimmed.ends_with(',') ||
        trimmed.ends_with('+') ||
        trimmed.ends_with('-') ||
        trimmed.ends_with('*') ||
        trimmed.ends_with('/') ||
        trimmed.ends_with("&&") ||
        trimmed.ends_with("||") ||
        trimmed.ends_with(">>")
    }

    /// Compile and run the current session
    fn compile_session(&mut self) -> Result<()> {
        use std::fmt::Write;

        if self.history.is_empty() {
            println!("No expressions to compile");
            return Ok(());
        }

        println!("Compiling session...");

        // Generate Rust code for all expressions
        let mut rust_code = String::new();
        rust_code.push_str("#![allow(unused)]\n");
        rust_code.push_str("fn main() {\n");

        for expr in &self.history {
            match Parser::new(expr).parse() {
                Ok(ast) => {
                    let transpiled = self.transpiler.transpile(&ast)?;
                    let transpiled_str = transpiled.to_string();
                    // Check if this is already a print statement that should be executed directly
                    let trimmed = transpiled_str.trim();
                    if trimmed.starts_with("println !")
                        || trimmed.starts_with("print !")
                        || trimmed.starts_with("println!")
                        || trimmed.starts_with("print!")
                    {
                        let _ = writeln!(&mut rust_code, "    {transpiled};");
                    } else {
                        let _ = writeln!(
                            &mut rust_code,
                            "    println!(\"{{:?}}\", {{{transpiled}}});"
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse '{expr}': {e}");
                }
            }
        }

        rust_code.push_str("}\n");

        // Write to temporary file
        self.session_counter += 1;
        let file_name = format!("session_{}.rs", self.session_counter);
        let file_path = self.temp_dir.join(&file_name);
        fs::write(&file_path, rust_code)?;

        // Compile with rustc
        let output = Command::new("rustc")
            .arg(&file_path)
            .arg("-o")
            .arg(
                self.temp_dir
                    .join(format!("session_{}", self.session_counter)),
            )
            .current_dir(&self.temp_dir)
            .output()
            .context("Failed to run rustc")?;

        if !output.status.success() {
            eprintln!(
                "Compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Ok(());
        }

        // Run the compiled program
        let exe_path = self
            .temp_dir
            .join(format!("session_{}", self.session_counter));
        let output = Command::new(&exe_path)
            .output()
            .context("Failed to run compiled program")?;

        println!("{}", "Output:".bright_green());
        print!("{}", String::from_utf8_lossy(&output.stdout));

        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// Search through command history with fuzzy matching
    fn search_history(&self, query: &str) {
        let query_lower = query.to_lowercase();
        let mut matches = Vec::new();

        // Simple fuzzy matching: contains all characters in order
        for (i, item) in self.history.iter().enumerate() {
            let item_lower = item.to_lowercase();

            // Check if query characters appear in order in the history item
            let mut query_chars = query_lower.chars();
            let mut current_char = query_chars.next();
            let mut score = 0;

            for item_char in item_lower.chars() {
                if let Some(q_char) = current_char {
                    if item_char == q_char {
                        score += 1;
                        current_char = query_chars.next();
                    }
                }
            }

            // If all query characters were found, it's a match
            if current_char.is_none() {
                matches.push((i, item, score));
            } else if item_lower.contains(&query_lower) {
                // Also include exact substring matches
                matches.push((i, item, query.len()));
            }
        }

        if matches.is_empty() {
            println!("No matches found for '{query}'");
            return;
        }

        // Sort by score (descending) then by recency (descending)
        matches.sort_by(|a, b| b.2.cmp(&a.2).then(b.0.cmp(&a.0)));

        println!(
            "{} History search results for '{}':",
            "Found".bright_green(),
            query
        );
        for (i, (hist_idx, item, _score)) in matches.iter().enumerate().take(10) {
            // Highlight the query in the result
            let highlighted = Self::highlight_match(item, &query_lower);
            println!(
                "  {}: {}",
                format!("{}", hist_idx + 1).bright_black(),
                highlighted
            );

            if i >= 9 {
                break;
            }
        }

        if matches.len() > 10 {
            println!("  ... and {} more matches", matches.len() - 10);
        }

        println!(
            "\n{}: Use :history to see all commands or Ctrl+R for interactive search",
            "Tip".bright_cyan()
        );
    }

    /// Highlight query matches in text
    fn highlight_match(text: &str, query: &str) -> String {
        let mut result = String::new();
        let mut query_chars = query.chars().peekable();
        let mut current_char = query_chars.next();

        for ch in text.chars() {
            let ch_lower = ch.to_lowercase().next().unwrap_or(ch);

            if let Some(q_char) = current_char {
                if ch_lower == q_char {
                    // Highlight matching character
                    result.push_str(&ch.to_string().bright_yellow().bold().to_string());
                    current_char = query_chars.next();
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Save current session to a file
    ///
    /// # Errors
    ///
    /// Returns an error if file writing fails
    fn save_session(&self, filename: &str) -> Result<()> {
        use chrono::Utc;
        use std::io::Write;

        let mut content = String::new();

        // Add header with timestamp
        writeln!(&mut content, "// Ruchy REPL Session")?;
        writeln!(
            &mut content,
            "// Generated: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;
        writeln!(&mut content, "// Commands: {}", self.history.len())?;
        writeln!(&mut content, "// Variables: {}", self.bindings.len())?;
        writeln!(&mut content)?;

        // Add variable bindings as comments
        if !self.bindings.is_empty() {
            writeln!(&mut content, "// Current variable bindings:")?;
            for (name, value) in &self.bindings {
                writeln!(&mut content, "// {name}: {value}")?;
            }
            writeln!(&mut content)?;
        }

        // Add all commands from history
        writeln!(
            &mut content,
            "// Session history (paste into REPL or run as script):"
        )?;
        writeln!(&mut content)?;

        for (i, command) in self.history.iter().enumerate() {
            // Skip commands that start with : (REPL commands)
            if command.starts_with(':') {
                writeln!(
                    &mut content,
                    "// Command {}: {} (REPL command, skipped)",
                    i + 1,
                    command
                )?;
                continue;
            }

            writeln!(&mut content, "// Command {}:", i + 1)?;
            writeln!(&mut content, "{command}")?;
            writeln!(&mut content)?;
        }

        // Add a section for recreating the session
        writeln!(&mut content, "// To recreate this session, you can:")?;
        writeln!(
            &mut content,
            "// 1. Copy and paste commands individually into the REPL"
        )?;
        writeln!(
            &mut content,
            "// 2. Use :load {filename} to execute all commands"
        )?;
        writeln!(
            &mut content,
            "// 3. Remove comments and run as a script: ruchy {filename}"
        )?;

        // Write to file
        let mut file = std::fs::File::create(filename)
            .with_context(|| format!("Failed to create file: {filename}"))?;
        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to file: {filename}"))?;

        Ok(())
    }

    /// Load and evaluate a file
    fn load_file(&mut self, path: &str) -> Result<()> {
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file: {path}"))?;

        println!("Loading {path}...");

        for line in content.lines() {
            if line.trim().is_empty() || line.trim().starts_with("//") {
                continue;
            }

            match self.eval(line) {
                Ok(result) => {
                    println!("{}: {}", line.bright_black(), result);
                }
                Err(e) => {
                    eprintln!("{}: {} - {}", "Error".bright_red(), line, e);
                }
            }
        }

        Ok(())
    }

    /// Evaluate literal expressions
    fn evaluate_literal(&mut self, lit: &Literal) -> Result<Value> {
        match lit {
            Literal::Integer(n) => Ok(Value::Int(*n)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::String(s) => {
                self.memory.try_alloc(s.len())?;
                Ok(Value::String(s.clone()))
            }
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Char(c) => Ok(Value::Char(*c)),
            Literal::Unit => Ok(Value::Unit),
        }
    }

    /// Evaluate if expressions
    fn evaluate_if(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let cond_val = self.evaluate_expr(condition, deadline, depth + 1)?;
        match cond_val {
            Value::Bool(true) => self.evaluate_expr(then_branch, deadline, depth + 1),
            Value::Bool(false) => {
                if let Some(else_expr) = else_branch {
                    self.evaluate_expr(else_expr, deadline, depth + 1)
                } else {
                    Ok(Value::Unit)
                }
            }
            _ => bail!("If condition must be boolean, got: {:?}", cond_val),
        }
    }

    /// Evaluate if-let expression (complexity: 6)
    fn evaluate_if_let(
        &mut self,
        pattern: &Pattern,
        expr: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let value = self.evaluate_expr(expr, deadline, depth + 1)?;
        
        // Try pattern matching
        if let Ok(Some(bindings)) = Self::pattern_matches(&value, pattern) {
            // Save current bindings
            let saved_bindings: Vec<(String, Value)> = bindings
                .iter()
                .filter_map(|(name, _val)| {
                    self.bindings.get(name).map(|old_val| (name.clone(), old_val.clone()))
                })
                .collect();
            
            // Apply pattern bindings
            for (name, val) in bindings {
                self.bindings.insert(name, val);
            }
            
            // Evaluate then branch
            let result = self.evaluate_expr(then_branch, deadline, depth + 1);
            
            // Restore bindings
            for (name, old_val) in saved_bindings {
                self.bindings.insert(name, old_val);
            }
            
            result
        } else {
            // Pattern didn't match, evaluate else branch
            if let Some(else_expr) = else_branch {
                self.evaluate_expr(else_expr, deadline, depth + 1)
            } else {
                Ok(Value::Unit)
            }
        }
    }

    /// Evaluate while-let expression (complexity: 7)
    fn evaluate_while_let(
        &mut self,
        pattern: &Pattern,
        expr: &Expr,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut last_value = Value::Unit;
        
        loop {
            if Instant::now() > deadline {
                bail!("Loop timed out");
            }
            
            let value = self.evaluate_expr(expr, deadline, depth + 1)?;
            
            // Try pattern matching
            if let Ok(Some(bindings)) = Self::pattern_matches(&value, pattern) {
                // Save current bindings
                let saved_bindings: Vec<(String, Value)> = bindings
                    .iter()
                    .filter_map(|(name, _val)| {
                        self.bindings.get(name).map(|old_val| (name.clone(), old_val.clone()))
                    })
                    .collect();
                
                // Apply pattern bindings
                for (name, val) in bindings {
                    self.bindings.insert(name, val);
                }
                
                // Evaluate body
                match self.evaluate_expr(body, deadline, depth + 1) {
                    Ok(val) => {
                        last_value = val;
                        
                        // Restore bindings
                        for (name, old_val) in saved_bindings {
                            self.bindings.insert(name, old_val);
                        }
                    }
                    Err(e) => {
                        // Restore bindings before propagating error
                        for (name, old_val) in saved_bindings {
                            self.bindings.insert(name, old_val);
                        }
                        return Err(e);
                    }
                }
            } else {
                // Pattern didn't match, exit loop
                break;
            }
        }
        
        Ok(last_value)
    }

    /// Evaluate function calls
    fn evaluate_call(
        &mut self,
        func: &Expr,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let ExprKind::Identifier(func_name) = &func.kind {
            match func_name.as_str() {
                "println" => self.evaluate_println(args, deadline, depth),
                "print" => self.evaluate_print(args, deadline, depth),
                "input" => self.evaluate_input(args, deadline, depth),
                "readline" => self.evaluate_readline(args, deadline, depth),
                "assert" => self.evaluate_assert(args, deadline, depth),
                "assert_eq" => self.evaluate_assert_eq(args, deadline, depth), 
                "assert_ne" => self.evaluate_assert_ne(args, deadline, depth),
                "curry" => self.evaluate_curry(args, deadline, depth),
                "uncurry" => self.evaluate_uncurry(args, deadline, depth),
                "read_file" => self.evaluate_read_file(args, deadline, depth),
                "write_file" => self.evaluate_write_file(args, deadline, depth),
                "append_file" => self.evaluate_append_file(args, deadline, depth),
                "file_exists" => self.evaluate_file_exists(args, deadline, depth),
                "delete_file" => self.evaluate_delete_file(args, deadline, depth),
                "current_dir" => self.evaluate_current_dir(args, deadline, depth),
                "env" => self.evaluate_env(args, deadline, depth),
                "set_env" => self.evaluate_set_env(args, deadline, depth),
                "args" => self.evaluate_args(args, deadline, depth),
                "Some" => self.evaluate_some(args, deadline, depth),
                "None" => self.evaluate_none(args, deadline, depth),
                "Ok" => self.evaluate_ok(args, deadline, depth),
                "Err" => self.evaluate_err(args, deadline, depth),
                // Type conversion functions
                "str" => self.evaluate_str_conversion(args, deadline, depth),
                "int" => self.evaluate_int_conversion(args, deadline, depth),
                "float" => self.evaluate_float_conversion(args, deadline, depth),
                "bool" => self.evaluate_bool_conversion(args, deadline, depth),
                // Introspection functions
                "type" => self.evaluate_type_function(args, deadline, depth),
                "summary" => self.evaluate_summary_function(args, deadline, depth),
                "dir" => self.evaluate_dir_function(args, deadline, depth),
                "help" => self.evaluate_help_function(args, deadline, depth),
                // Workspace management functions
                "whos" => self.evaluate_whos_function(args, deadline, depth),
                "who" => self.evaluate_who_function(args, deadline, depth),
                "clear_all" => self.evaluate_clear_bang_function(args, deadline, depth),
                "save_image" => self.evaluate_save_image_function(args, deadline, depth),
                "workspace" => self.evaluate_workspace_function(args, deadline, depth),
                "locals" => self.evaluate_locals_function(args, deadline, depth),
                "globals" => self.evaluate_globals_function(args, deadline, depth),
                "reset" => self.evaluate_reset_function(args, deadline, depth),
                "del" => self.evaluate_del_function(args, deadline, depth),
                "exists" => self.evaluate_exists_function(args, deadline, depth),
                "memory_info" => self.evaluate_memory_info_function(args, deadline, depth),
                "time_info" => self.evaluate_time_info_function(args, deadline, depth),
                // Advanced math functions
                "sin" => self.evaluate_sin(args, deadline, depth),
                "cos" => self.evaluate_cos(args, deadline, depth),
                "tan" => self.evaluate_tan(args, deadline, depth),
                "log" => self.evaluate_log(args, deadline, depth),
                "log10" => self.evaluate_log10(args, deadline, depth),
                "random" => self.evaluate_random(args, deadline, depth),
                "HashMap" => {
                    if !args.is_empty() {
                        bail!("HashMap() constructor expects no arguments, got {}", args.len());
                    }
                    Ok(Value::HashMap(HashMap::new()))
                }
                "HashSet" => {
                    if !args.is_empty() {
                        bail!("HashSet() constructor expects no arguments, got {}", args.len());
                    }
                    Ok(Value::HashSet(HashSet::new()))
                }
                _ => self.evaluate_user_function(func_name, args, deadline, depth),
            }
        } else if let ExprKind::QualifiedName { module, name } = &func.kind {
            // Handle built-in static constructors
            match (module.as_str(), name.as_str()) {
                ("HashMap", "new") => {
                    if !args.is_empty() {
                        bail!("HashMap::new() expects no arguments, got {}", args.len());
                    }
                    return Ok(Value::HashMap(HashMap::new()));
                }
                ("HashSet", "new") => {
                    if !args.is_empty() {
                        bail!("HashSet::new() expects no arguments, got {}", args.len());
                    }
                    return Ok(Value::HashSet(HashSet::new()));
                }
                _ => {}
            }
            
            // Handle static method calls (Type::method)
            let qualified_name = format!("{module}::{name}");
            if let Some((param_names, body)) = self.impl_methods.get(&qualified_name).cloned() {
                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expr(arg, deadline, depth + 1)?);
                }

                // Check argument count
                if arg_values.len() != param_names.len() {
                    bail!(
                        "Function {} expects {} arguments, got {}",
                        qualified_name,
                        param_names.len(),
                        arg_values.len()
                    );
                }

                // Save current bindings
                let saved_bindings = self.bindings.clone();

                // Bind arguments
                for (param, value) in param_names.iter().zip(arg_values.iter()) {
                    self.bindings.insert(param.clone(), value.clone());
                }

                // Evaluate body with return handling
                let result = self.evaluate_function_body(&body, deadline, depth)?;

                // Restore bindings
                self.bindings = saved_bindings;

                Ok(result)
            } else {
                bail!("Unknown static method: {}", qualified_name);
            }
        } else {
            bail!("Complex function calls not yet supported");
        }
    }

    /// Evaluate curry function - converts a function that takes multiple arguments into a series of functions that each take a single argument
    fn evaluate_curry(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() != 1 {
            bail!("curry expects exactly 1 argument (a function)");
        }

        // Evaluate the function argument
        let func_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;

        // For now, return a string representation of currying
        match func_val {
            Value::Function { name, params, .. } => {
                if params.is_empty() {
                    bail!("Cannot curry a function with no parameters");
                }
                // Return a descriptive representation for REPL demo
                let curry_repr = format!(
                    "curry({}) -> {}",
                    name,
                    params
                        .iter()
                        .map(|p| format!("({p} -> ...)"))
                        .collect::<Vec<_>>()
                        .join(" -> ")
                );
                Ok(Value::String(curry_repr))
            }
            _ => bail!("curry expects a function as argument"),
        }
    }

    /// Evaluate uncurry function - converts a curried function back into a function that takes multiple arguments
    fn evaluate_uncurry(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("uncurry expects exactly 1 argument (a curried function)");
        }

        // Evaluate the function argument
        let func_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;

        // For now, return a string representation of uncurrying
        match func_val {
            Value::Function { name, params, .. } => {
                let uncurry_repr = format!("uncurry({}) -> ({}) -> ...", name, params.join(", "));
                Ok(Value::String(uncurry_repr))
            }
            Value::String(s) if s.contains("curry") => {
                // Handle curried functions
                Ok(Value::String(format!("uncurry({s}) -> original function")))
            }
            _ => bail!("uncurry expects a curried function as argument"),
        }
    }

    /// Evaluate println function
    fn evaluate_println(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        // Handle format strings like println("Result: {}", x)
        if !args.is_empty() {
            let first_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
            if let Value::String(format_str) = first_val {
                // Check if it contains format placeholders
                if format_str.contains("{}") && args.len() > 1 {
                    // This is a format string - process it
                    let mut output = format_str;
                    let mut arg_values = Vec::new();
                    
                    // Evaluate all format arguments
                    for arg in &args[1..] {
                        let val = self.evaluate_expr(arg, deadline, depth + 1)?;
                        arg_values.push(val.to_string());
                    }
                    
                    // Replace {} placeholders with values
                    for value in arg_values {
                        if let Some(pos) = output.find("{}") {
                            output.replace_range(pos..pos+2, &value);
                        }
                    }
                    
                    println!("{output}");
                    return Ok(Value::Unit);
                }
                // No format placeholders - treat all args equally, space-separated
                if args.len() == 1 {
                    // Single argument - just print it
                    println!("{format_str}");
                } else {
                    // Multiple arguments - print all space-separated on same line
                    print!("{format_str}");
                    for arg in &args[1..] {
                        let val = self.evaluate_expr(arg, deadline, depth + 1)?;
                        // Print strings without quotes, other types with their normal formatting
                        match val {
                            Value::String(s) => print!(" {s}"),
                            other => print!(" {other:?}"),
                        }
                    }
                    println!(); // Only one newline at the end
                }
                return Ok(Value::Unit);
            }
        }
        
        // Fallback: concatenate all arguments with spaces (original behavior)
        let mut output = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            let val = self.evaluate_expr(arg, deadline, depth + 1)?;
            match val {
                Value::String(s) => output.push_str(&s),
                other => output.push_str(&other.to_string()),
            }
        }
        println!("{output}");
        Ok(Value::Unit)
    }

    /// Evaluate print function
    fn evaluate_print(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        let mut output = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            let val = self.evaluate_expr(arg, deadline, depth + 1)?;
            match val {
                Value::String(s) => output.push_str(&s),
                other => output.push_str(&other.to_string()),
            }
        }
        print!("{output}");
        Ok(Value::Unit)
    }

    /// Evaluate `input` function - prompt user for input
    fn evaluate_input(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        use std::io::{self, Write};
        
        // Handle optional prompt argument
        if args.len() > 1 {
            bail!("input expects 0 or 1 arguments (optional prompt)");
        }
        
        // Show prompt if provided
        if let Some(prompt_expr) = args.first() {
            let prompt_val = self.evaluate_expr(prompt_expr, deadline, depth + 1)?;
            match prompt_val {
                Value::String(prompt) => print!("{prompt}"),
                other => print!("{other}"),
            }
            io::stdout().flush().unwrap_or(());
        }
        
        // Read line from stdin
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Remove trailing newline
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                self.memory.try_alloc(input.len())?;
                Ok(Value::String(input))
            }
            Err(e) => bail!("Failed to read input: {e}"),
        }
    }

    /// Evaluate `readline` function - read a line from stdin 
    fn evaluate_readline(&mut self, args: &[Expr], _deadline: Instant, _depth: usize) -> Result<Value> {
        use std::io;
        
        if !args.is_empty() {
            bail!("readline expects no arguments");
        }
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Remove trailing newline
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                self.memory.try_alloc(input.len())?;
                Ok(Value::String(input))
            }
            Err(e) => bail!("Failed to read line: {e}"),
        }
    }

    /// Evaluate `assert` function - panic if condition is false
    fn evaluate_assert(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.is_empty() || args.len() > 2 {
            bail!("assert expects 1 or 2 arguments (condition, optional message)");
        }
        
        // Evaluate condition
        let condition = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::Bool(is_true) = condition else {
            bail!("assert expects a boolean condition, got {}", std::any::type_name_of_val(&condition))
        };
        
        if !is_true {
            // Get optional message
            let message = if args.len() > 1 {
                let msg_val = self.evaluate_expr(&args[1], deadline, depth + 1)?;
                match msg_val {
                    Value::String(s) => s,
                    other => other.to_string(),
                }
            } else {
                "Assertion failed".to_string()
            };
            
            bail!("Assertion failed: {}", message);
        }
        
        Ok(Value::Unit)
    }

    /// Evaluate `assert_eq` function - panic if values are not equal
    fn evaluate_assert_eq(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            bail!("assert_eq expects 2 or 3 arguments (left, right, optional message)");
        }
        
        // Evaluate both values
        let left = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let right = self.evaluate_expr(&args[1], deadline, depth + 1)?;
        
        // Compare values
        let are_equal = self.values_equal(&left, &right);
        
        if !are_equal {
            // Get optional message
            let message = if args.len() > 2 {
                let msg_val = self.evaluate_expr(&args[2], deadline, depth + 1)?;
                match msg_val {
                    Value::String(s) => s,
                    other => other.to_string(),
                }
            } else {
                format!("assertion failed: `(left == right)`\n  left: `{left}`\n right: `{right}`")
            };
            
            bail!("Assertion failed: {}", message);
        }
        
        Ok(Value::Unit)
    }

    /// Evaluate `assert_ne` function - panic if values are equal
    fn evaluate_assert_ne(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            bail!("assert_ne expects 2 or 3 arguments (left, right, optional message)");
        }
        
        // Evaluate both values
        let left = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let right = self.evaluate_expr(&args[1], deadline, depth + 1)?;
        
        // Compare values
        let are_equal = self.values_equal(&left, &right);
        
        if are_equal {
            // Get optional message
            let message = if args.len() > 2 {
                let msg_val = self.evaluate_expr(&args[2], deadline, depth + 1)?;
                match msg_val {
                    Value::String(s) => s,
                    other => other.to_string(),
                }
            } else {
                format!("assertion failed: `(left != right)`\n  left: `{left}`\n right: `{right}`")
            };
            
            bail!("Assertion failed: {}", message);
        }
        
        Ok(Value::Unit)
    }

    /// Compare two values for equality (helper for assertions)
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Int(a), Value::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
            (Value::Float(a), Value::Int(b)) => (a - *b as f64).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::List(a), Value::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y))
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y))
            }
            _ => false,
        }
    }

    /// Evaluate `read_file` function
    fn evaluate_read_file(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("read_file expects exactly 1 argument (filename)");
        }

        let filename_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::String(filename) = filename_val else {
            bail!("read_file expects a string filename")
        };

        match std::fs::read_to_string(&filename) {
            Ok(content) => Ok(Value::String(content)),
            Err(e) => bail!("Failed to read file '{}': {}", filename, e),
        }
    }

    /// Evaluate `write_file` function  
    fn evaluate_write_file(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 2 {
            bail!("write_file expects exactly 2 arguments (filename, content)");
        }

        let filename_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::String(filename) = filename_val else {
            bail!("write_file expects a string filename")
        };

        let content_val = self.evaluate_expr(&args[1], deadline, depth + 1)?;
        let content = if let Value::String(s) = content_val {
            s
        } else {
            content_val.to_string()
        };

        match std::fs::write(&filename, content) {
            Ok(()) => {
                println!("File '{filename}' written successfully");
                Ok(Value::Unit)
            }
            Err(e) => bail!("Failed to write file '{}': {}", filename, e),
        }
    }

    /// Evaluate `append_file` function
    fn evaluate_append_file(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 2 {
            bail!("append_file expects exactly 2 arguments (filename, content)");
        }

        let filename_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::String(filename) = filename_val else {
            bail!("append_file expects a string filename")
        };

        let content_val = self.evaluate_expr(&args[1], deadline, depth + 1)?;
        let content = if let Value::String(s) = content_val {
            s
        } else {
            content_val.to_string()
        };

        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
        {
            Ok(mut file) => {
                use std::io::Write;
                match file.write_all(content.as_bytes()) {
                    Ok(()) => Ok(Value::Unit),
                    Err(e) => bail!("Failed to append to file '{}': {}", filename, e),
                }
            }
            Err(e) => bail!("Failed to open file '{}' for append: {}", filename, e),
        }
    }

    /// Evaluate `file_exists` function
    fn evaluate_file_exists(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("file_exists expects exactly 1 argument (filename)");
        }

        let filename_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::String(filename) = filename_val else {
            bail!("file_exists expects a string filename")
        };

        let exists = std::path::Path::new(&filename).exists();
        Ok(Value::Bool(exists))
    }

    /// Evaluate `delete_file` function
    fn evaluate_delete_file(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("delete_file expects exactly 1 argument (filename)");
        }

        let filename_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::String(filename) = filename_val else {
            bail!("delete_file expects a string filename")
        };

        match std::fs::remove_file(&filename) {
            Ok(()) => Ok(Value::Unit),
            Err(e) => bail!("Failed to delete file '{}': {}", filename, e),
        }
    }

    /// Evaluate `current_dir` function
    fn evaluate_current_dir(
        &mut self,
        args: &[Expr],
        _deadline: Instant,
        _depth: usize,
    ) -> Result<Value> {
        if !args.is_empty() {
            bail!("current_dir expects no arguments");
        }

        match std::env::current_dir() {
            Ok(path) => Ok(Value::String(path.to_string_lossy().to_string())),
            Err(e) => bail!("Failed to get current directory: {}", e),
        }
    }

    /// Evaluate `env` function
    fn evaluate_env(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("env expects exactly 1 argument (variable name)");
        }

        let var_name_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::String(var_name) = var_name_val else {
            bail!("env expects a string variable name")
        };

        match std::env::var(&var_name) {
            Ok(value) => Ok(Value::String(value)),
            Err(_) => Ok(Value::String(String::new())), // Return empty string for non-existent vars
        }
    }

    /// Evaluate `set_env` function
    fn evaluate_set_env(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 2 {
            bail!("set_env expects exactly 2 arguments (variable name, value)");
        }

        let var_name_val = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        let Value::String(var_name) = var_name_val else {
            bail!("set_env expects a string variable name")
        };

        let value_val = self.evaluate_expr(&args[1], deadline, depth + 1)?;
        let value = if let Value::String(s) = value_val {
            s
        } else {
            value_val.to_string()
        };

        std::env::set_var(var_name, value);
        Ok(Value::Unit)
    }

    /// Evaluate `args` function
    fn evaluate_args(
        &mut self,
        args: &[Expr],
        _deadline: Instant,
        _depth: usize,
    ) -> Result<Value> {
        if !args.is_empty() {
            bail!("args expects no arguments");
        }

        let args_vec = std::env::args().collect::<Vec<String>>();
        let values: Vec<Value> = args_vec.into_iter().map(Value::String).collect();
        Ok(Value::List(values))
    }

    /// Evaluate `Some` constructor
    fn evaluate_some(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("Some expects exactly 1 argument");
        }

        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        Ok(Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![value]),
        })
    }

    /// Evaluate `None` constructor
    fn evaluate_none(
        &mut self,
        args: &[Expr],
        _deadline: Instant,
        _depth: usize,
    ) -> Result<Value> {
        if !args.is_empty() {
            bail!("None expects no arguments");
        }

        Ok(Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        })
    }

    /// Evaluate `Ok` constructor
    fn evaluate_ok(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("Ok expects exactly 1 argument");
        }

        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        Ok(Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Ok".to_string(),
            data: Some(vec![value]),
        })
    }

    /// Evaluate `Err` constructor
    fn evaluate_err(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("Err expects exactly 1 argument");
        }

        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        Ok(Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Err".to_string(),
            data: Some(vec![value]),
        })
    }

    /// Update history variables (_ and _n)
    fn update_history_variables(&mut self) {
        let len = self.result_history.len();
        if len == 0 {
            return;
        }

        // Set _ to the most recent result
        let last_result = self.result_history[len - 1].clone();
        self.bindings.insert("_".to_string(), last_result);

        // Set _n variables for indexed access
        for (i, result) in self.result_history.iter().enumerate() {
            let var_name = format!("_{}", i + 1);
            self.bindings.insert(var_name, result.clone());
        }
    }

    /// Handle REPL magic commands
    fn handle_magic_command(&mut self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        let magic_cmd = parts[0];
        let args = if parts.len() > 1 { parts[1] } else { "" };

        match magic_cmd {
            "%time" => {
                if args.is_empty() {
                    return Ok("Usage: %time <expression>".to_string());
                }
                
                let start = std::time::Instant::now();
                let result = self.eval(args)?;
                let elapsed = start.elapsed();
                
                Ok(format!("{}\nExecuted in: {:?}", result, elapsed))
            }
            
            "%timeit" => {
                if args.is_empty() {
                    return Ok("Usage: %timeit <expression>".to_string());
                }
                
                const ITERATIONS: usize = 1000;
                let mut total_time = std::time::Duration::new(0, 0);
                let mut last_result = String::new();
                
                for _ in 0..ITERATIONS {
                    let start = std::time::Instant::now();
                    last_result = self.eval(args)?;
                    total_time += start.elapsed();
                }
                
                let avg_time = total_time / ITERATIONS as u32;
                Ok(format!(
                    "{}\n{} loops, average: {:?} per loop", 
                    last_result, ITERATIONS, avg_time
                ))
            }
            
            "%run" => {
                if args.is_empty() {
                    return Ok("Usage: %run <script.ruchy>".to_string());
                }
                
                match std::fs::read_to_string(args) {
                    Ok(content) => {
                        let lines: Vec<&str> = content.lines().collect();
                        let mut results = Vec::new();
                        
                        for line in lines {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                                match self.eval(trimmed) {
                                    Ok(result) => results.push(result),
                                    Err(e) => return Err(e.context(format!("Error executing: {}", trimmed))),
                                }
                            }
                        }
                        
                        Ok(results.join("\n"))
                    }
                    Err(e) => Ok(format!("Failed to read file '{}': {}", args, e))
                }
            }
            
            "%debug" => {
                if let Some(ref debug_info) = self.last_error_debug {
                    let mut output = String::new();
                    output.push_str(&format!("=== Debug Information ===\n"));
                    output.push_str(&format!("Expression: {}\n", debug_info.expression));
                    output.push_str(&format!("Error: {}\n", debug_info.error_message));
                    output.push_str(&format!("Time: {:?}\n", debug_info.timestamp));
                    output.push_str(&format!("\n--- Variable Bindings at Error ---\n"));
                    
                    for (name, value) in &debug_info.bindings_snapshot {
                        output.push_str(&format!("{}: {}\n", name, value));
                    }
                    
                    if !debug_info.stack_trace.is_empty() {
                        output.push_str(&format!("\n--- Stack Trace ---\n"));
                        for frame in &debug_info.stack_trace {
                            output.push_str(&format!("  {}\n", frame));
                        }
                    }
                    
                    Ok(output)
                } else {
                    Ok("No debug information available. Run an expression that fails first.".to_string())
                }
            }
            
            "%profile" => {
                Ok("Profiling not yet implemented".to_string())
            }
            
            "%help" => {
                Ok(r#"Available magic commands:
%time <expr>     - Time a single execution
%timeit <expr>   - Time multiple executions (benchmark)
%run <file>      - Execute a .ruchy script file
%debug           - Show debug info from last error
%profile <expr>  - Generate execution profile (TODO)
%help            - Show this help message"#.to_string())
            }
            
            _ => {
                Ok(format!("Unknown magic command: {}. Type %help for available commands.", magic_cmd))
            }
        }
    }

    /// Evaluate `str` type conversion function
    fn evaluate_str_conversion(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("str() expects exactly 1 argument");
        }

        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        Ok(Value::String(value.to_string()))
    }

    /// Evaluate `int` type conversion function
    fn evaluate_int_conversion(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("int() expects exactly 1 argument");
        }

        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Int(n) => Ok(Value::Int(n)),
            Value::Float(f) => Ok(Value::Int(f as i64)),
            Value::Bool(b) => Ok(Value::Int(if b { 1 } else { 0 })),
            Value::String(s) => {
                match s.trim().parse::<i64>() {
                    Ok(n) => Ok(Value::Int(n)),
                    Err(_) => bail!("Cannot convert '{}' to integer", s),
                }
            }
            _ => bail!("Cannot convert value to integer"),
        }
    }

    /// Evaluate `float` type conversion function
    fn evaluate_float_conversion(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("float() expects exactly 1 argument");
        }

        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Float(f) => Ok(Value::Float(f)),
            Value::Int(n) => Ok(Value::Float(n as f64)),
            Value::Bool(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
            Value::String(s) => {
                match s.trim().parse::<f64>() {
                    Ok(f) => Ok(Value::Float(f)),
                    Err(_) => bail!("Cannot convert '{}' to float", s),
                }
            }
            _ => bail!("Cannot convert value to float"),
        }
    }

    /// Evaluate `bool` type conversion function
    fn evaluate_bool_conversion(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("bool() expects exactly 1 argument");
        }

        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Bool(b) => Ok(Value::Bool(b)),
            Value::Int(n) => Ok(Value::Bool(n != 0)),
            Value::Float(f) => Ok(Value::Bool(f != 0.0 && !f.is_nan())),
            Value::String(s) => Ok(Value::Bool(!s.is_empty())),
            Value::Unit => Ok(Value::Bool(false)),
            Value::List(l) => Ok(Value::Bool(!l.is_empty())),
            Value::Object(o) => Ok(Value::Bool(!o.is_empty())),
            _ => Ok(Value::Bool(true)), // Most other values are truthy
        }
    }

    /// Evaluate sin() function
    fn evaluate_sin(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("sin() expects exactly 1 argument");
        }
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Float(f) => Ok(Value::Float(f.sin())),
            Value::Int(n) => Ok(Value::Float((n as f64).sin())),
            _ => bail!("sin() expects a numeric argument"),
        }
    }

    /// Evaluate cos() function
    fn evaluate_cos(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("cos() expects exactly 1 argument");
        }
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Float(f) => Ok(Value::Float(f.cos())),
            Value::Int(n) => Ok(Value::Float((n as f64).cos())),
            _ => bail!("cos() expects a numeric argument"),
        }
    }

    /// Evaluate tan() function
    fn evaluate_tan(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("tan() expects exactly 1 argument");
        }
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Float(f) => Ok(Value::Float(f.tan())),
            Value::Int(n) => Ok(Value::Float((n as f64).tan())),
            _ => bail!("tan() expects a numeric argument"),
        }
    }

    /// Evaluate log() function (natural logarithm)
    fn evaluate_log(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("log() expects exactly 1 argument");
        }
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Float(f) => {
                if f <= 0.0 {
                    bail!("log() requires a positive argument");
                }
                Ok(Value::Float(f.ln()))
            }
            Value::Int(n) => {
                if n <= 0 {
                    bail!("log() requires a positive argument");
                }
                Ok(Value::Float((n as f64).ln()))
            }
            _ => bail!("log() expects a numeric argument"),
        }
    }

    /// Evaluate log10() function (base-10 logarithm)
    fn evaluate_log10(
        &mut self,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if args.len() != 1 {
            bail!("log10() expects exactly 1 argument");
        }
        let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
        match value {
            Value::Float(f) => {
                if f <= 0.0 {
                    bail!("log10() requires a positive argument");
                }
                Ok(Value::Float(f.log10()))
            }
            Value::Int(n) => {
                if n <= 0 {
                    bail!("log10() requires a positive argument");
                }
                Ok(Value::Float((n as f64).log10()))
            }
            _ => bail!("log10() expects a numeric argument"),
        }
    }

    /// Evaluate random() function - returns float between 0.0 and 1.0
    fn evaluate_random(
        &mut self,
        args: &[Expr],
        _deadline: Instant,
        _depth: usize,
    ) -> Result<Value> {
        if !args.is_empty() {
            bail!("random() expects no arguments");
        }
        // Use a simple linear congruential generator for deterministic behavior in tests
        // In production, you'd want to use rand crate
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        // Use a safe LCG that won't overflow
        let a = 1664525u64;
        let c = 1013904223u64;
        let m = 1u64 << 32;
        let random_value = ((seed.wrapping_mul(a).wrapping_add(c)) % m) as f64 / m as f64;
        Ok(Value::Float(random_value))
    }

    /// Execute a user-defined function or lambda by name.
    /// 
    /// Looks up the function in bindings and executes it with parameter binding.
    /// Handles both regular functions and lambdas with identical execution logic.
    /// 
    /// # Arguments
    /// 
    /// * `func_name` - Name of the function to execute
    /// * `args` - Arguments to pass to the function
    /// * `deadline` - Execution deadline for timeout handling
    /// * `depth` - Current recursion depth
    /// 
    /// Example Usage:
    /// 
    /// Executes a user-defined function stored in bindings:
    /// - Looks up function by name
    /// - Validates argument count
    /// - Binds parameters to arguments
    /// - Evaluates function body in new scope
    fn execute_user_defined_function(
        &mut self,
        func_name: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let Some(func_value) = self.bindings.get(func_name).cloned() {
            match func_value {
                Value::Function { params, body, .. } => {
                    self.execute_function_with_params(func_name, &params, &body, args, deadline, depth, "Function")
                }
                Value::Lambda { params, body } => {
                    self.execute_function_with_params(func_name, &params, &body, args, deadline, depth, "Lambda")
                }
                _ => {
                    bail!("'{}' is not a function", func_name);
                }
            }
        } else {
            bail!("Unknown function: {}", func_name);
        }
    }

    /// Execute a function or lambda with parameter binding and scope management.
    /// 
    /// This helper consolidates the common logic between functions and lambdas:
    /// - Validates argument count matches parameter count
    /// - Saves current bindings scope
    /// - Binds arguments to parameters
    /// - Executes function body
    /// - Restores previous scope
    /// 
    /// # Arguments
    /// 
    /// * `func_name` - Name of the function (for error messages)
    /// * `params` - Function parameter names
    /// * `body` - Function body expression
    /// * `args` - Arguments to bind to parameters
    /// * `deadline` - Execution deadline
    /// * `depth` - Recursion depth
    /// * `func_type` - Either "Function" or "Lambda" for error messages
    fn execute_function_with_params(
        &mut self,
        func_name: &str,
        params: &[String],
        body: &Expr,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
        func_type: &str,
    ) -> Result<Value> {
        if args.len() != params.len() {
            bail!(
                "{} {} expects {} arguments, got {}",
                func_type,
                func_name,
                params.len(),
                args.len()
            );
        }

        let saved_bindings = self.bindings.clone();

        for (param, arg) in params.iter().zip(args.iter()) {
            let arg_value = self.evaluate_expr(arg, deadline, depth + 1)?;
            self.bindings.insert(param.clone(), arg_value);
        }

        let result = self.evaluate_function_body(body, deadline, depth)?;
        self.bindings = saved_bindings;
        Ok(result)
    }

    /// Validate argument count for math functions.
    /// 
    /// Example Usage:
    /// 
    /// Validates that a function receives the expected number of arguments:
    /// - sqrt(x) expects exactly 1 argument
    /// - pow(x, y) expects exactly 2 arguments
    /// - Returns an error if count doesn't match
    fn validate_arg_count(&self, func_name: &str, args: &[Expr], expected: usize) -> Result<()> {
        if args.len() != expected {
            bail!("{} takes exactly {} argument{}", func_name, expected, if expected == 1 { "" } else { "s" });
        }
        Ok(())
    }

    /// Apply unary math operation to a numeric value.
    /// 
    /// # Example Usage
    /// Validates that the correct number of arguments is provided to a function.
    /// 
    /// # use `ruchy::runtime::repl::Repl`;
    /// # use `ruchy::runtime::value::Value`;
    /// let repl = `Repl::new()`;
    /// let result = `repl.apply_unary_math_op(&Value::Int(4)`, "`sqrt").unwrap()`;
    /// assert!(matches!(result, `Value::Float`(_)));
    /// ```
    fn apply_unary_math_op(&self, value: &Value, op: &str) -> Result<Value> {
        match (value, op) {
            (Value::Int(n), "sqrt") => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::Float((*n as f64).sqrt()))
            }
            (Value::Float(f), "sqrt") => Ok(Value::Float(f.sqrt())),
            (Value::Int(n), "abs") => Ok(Value::Int(n.abs())),
            (Value::Float(f), "abs") => Ok(Value::Float(f.abs())),
            (Value::Int(n), "floor") => Ok(Value::Int(*n)), // Already floored
            (Value::Float(f), "floor") => Ok(Value::Float(f.floor())),
            (Value::Int(n), "ceil") => Ok(Value::Int(*n)), // Already ceiled
            (Value::Float(f), "ceil") => Ok(Value::Float(f.ceil())),
            (Value::Int(n), "round") => Ok(Value::Int(*n)), // Already rounded
            (Value::Float(f), "round") => Ok(Value::Float(f.round())),
            _ => bail!("{} expects a numeric argument", op),
        }
    }

    /// Apply binary math operation to two numeric values.
    /// 
    /// # Example Usage
    /// Applies unary math operations like sqrt, abs, floor, ceil, round to numeric values.
    /// 
    /// # use `ruchy::runtime::repl::Repl`;
    /// # use `ruchy::runtime::value::Value`;
    /// let repl = `Repl::new()`;
    /// let result = `repl.apply_binary_math_op(&Value::Int(2)`, &`Value::Int(3)`, "`pow").unwrap()`;
    /// assert!(matches!(result, `Value::Int(8)`));
    /// ```
    fn apply_binary_math_op(&self, a: &Value, b: &Value, op: &str) -> Result<Value> {
        match (a, b, op) {
            (Value::Int(base), Value::Int(exp), "pow") => {
                if *exp < 0 {
                    #[allow(clippy::cast_precision_loss)]
                    Ok(Value::Float((*base as f64).powi(*exp as i32)))
                } else {
                    let exp_u32 = u32::try_from(*exp).map_err(|_| anyhow::anyhow!("Exponent too large"))?;
                    match base.checked_pow(exp_u32) {
                        Some(result) => Ok(Value::Int(result)),
                        None => bail!("Integer overflow in pow({}, {})", base, exp),
                    }
                }
            }
            (Value::Float(base), Value::Float(exp), "pow") => Ok(Value::Float(base.powf(*exp))),
            (Value::Int(base), Value::Float(exp), "pow") => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::Float((*base as f64).powf(*exp)))
            }
            (Value::Float(base), Value::Int(exp), "pow") => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::Float(base.powi(*exp as i32)))
            }
            (Value::Int(x), Value::Int(y), "min") => Ok(Value::Int((*x).min(*y))),
            (Value::Float(x), Value::Float(y), "min") => Ok(Value::Float(x.min(*y))),
            (Value::Int(x), Value::Float(y), "min") => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::Float((*x as f64).min(*y)))
            }
            (Value::Float(x), Value::Int(y), "min") => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::Float(x.min(*y as f64)))
            }
            (Value::Int(x), Value::Int(y), "max") => Ok(Value::Int((*x).max(*y))),
            (Value::Float(x), Value::Float(y), "max") => Ok(Value::Float(x.max(*y))),
            (Value::Int(x), Value::Float(y), "max") => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::Float((*x as f64).max(*y)))
            }
            (Value::Float(x), Value::Int(y), "max") => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::Float(x.max(*y as f64)))
            }
            _ => bail!("{} expects numeric arguments", op),
        }
    }

    /// Handle built-in math functions (sqrt, pow, abs, min, max, floor, ceil, round).
    /// 
    /// Returns `Ok(Some(value))` if the function name matches a math function,
    /// `Ok(None)` if it doesn't match any math function, or `Err` if there's an error.
    /// 
    /// # Example Usage
    /// Tries to call math functions like sqrt, pow, abs, min, max, floor, ceil, round.
    /// Dispatches to appropriate unary or binary math operation handler.
    fn try_math_function(
        &mut self,
        func_name: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Option<Value>> {
        match func_name {
            // Unary math functions
            "sqrt" | "abs" | "floor" | "ceil" | "round" => {
                self.validate_arg_count(func_name, args, 1)?;
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                Ok(Some(self.apply_unary_math_op(&value, func_name)?))
            }
            // Binary math functions
            "pow" | "min" | "max" => {
                self.validate_arg_count(func_name, args, 2)?;
                let a = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                let b = self.evaluate_expr(&args[1], deadline, depth + 1)?;
                Ok(Some(self.apply_binary_math_op(&a, &b, func_name)?))
            }
            _ => Ok(None), // Not a math function
        }
    }

    /// Handle built-in enum variant constructors (None, Some, Ok, Err).
    /// 
    /// Returns `Ok(Some(value))` if the function name matches an enum constructor,
    /// `Ok(None)` if it doesn't match any constructor, or `Err` if there's an error.
    /// 
    /// # Example Usage
    /// Applies binary math operations like pow, min, max to two numeric values.
    /// 
    /// # use `ruchy::runtime::repl::Repl`;
    /// # use `ruchy::frontend::ast::Expr`;
    /// # use `std::time::Instant`;
    /// let mut repl = `Repl::new()`;
    /// let args = vec![];
    /// let deadline = `Instant::now()` + `std::time::Duration::from_secs(1)`;
    /// let result = `repl.try_enum_constructor("None`", &args, deadline, `0).unwrap()`;
    /// `assert!(result.is_some())`;
    /// ```
    fn try_enum_constructor(
        &mut self,
        func_name: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Option<Value>> {
        match func_name {
            "None" => {
                if !args.is_empty() {
                    bail!("None takes no arguments");
                }
                Ok(Some(Value::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    data: None,
                }))
            }
            "Some" => {
                if args.len() != 1 {
                    bail!("Some takes exactly 1 argument");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                Ok(Some(Value::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    data: Some(vec![value]),
                }))
            }
            "Ok" => {
                if args.len() != 1 {
                    bail!("Ok takes exactly 1 argument");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                Ok(Some(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Ok".to_string(),
                    data: Some(vec![value]),
                }))
            }
            "Err" => {
                if args.len() != 1 {
                    bail!("Err takes exactly 1 argument");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                Ok(Some(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Err".to_string(),
                    data: Some(vec![value]),
                }))
            }
            _ => Ok(None), // Not an enum constructor
        }
    }

    /// Evaluate user-defined functions
    fn evaluate_user_function(
        &mut self,
        func_name: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        // Try enum constructor first
        if let Some(result) = self.try_enum_constructor(func_name, args, deadline, depth)? {
            return Ok(result);
        }

        // Try built-in math function
        if let Some(result) = self.try_math_function(func_name, args, deadline, depth)? {
            return Ok(result);
        }

        // Try user-defined function lookup and execution
        self.execute_user_defined_function(func_name, args, deadline, depth)
    }

    /// Helper to evaluate a function body and handle return statements
    fn evaluate_function_body(
        &mut self,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match self.evaluate_expr(body, deadline, depth + 1) {
            Ok(val) => Ok(val),
            Err(e) => {
                // Check if this is a return statement
                let err_str = e.to_string();
                if let Some(return_val) = err_str.strip_prefix("return:") {
                    // Parse the return value - it's already a formatted Value string
                    // For now, just extract the string representation
                    // The value was already evaluated, just passed through error
                    if return_val == "()" {
                        Ok(Value::Unit)
                    } else if return_val.starts_with('"') && return_val.ends_with('"') {
                        // String value - remove quotes
                        let s = return_val[1..return_val.len()-1].to_string();
                        Ok(Value::String(s))
                    } else if let Ok(i) = return_val.parse::<i64>() {
                        Ok(Value::Int(i))
                    } else if let Ok(f) = return_val.parse::<f64>() {
                        Ok(Value::Float(f))
                    } else if return_val == "true" {
                        Ok(Value::Bool(true))
                    } else if return_val == "false" {
                        Ok(Value::Bool(false))
                    } else {
                        // Return as string for complex values
                        Ok(Value::String(return_val.to_string()))
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Evaluate match expressions
    fn evaluate_match(
        &mut self,
        match_expr: &Expr,
        arms: &[MatchArm],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let match_value = self.evaluate_expr(match_expr, deadline, depth + 1)?;

        for arm in arms {
            if let Some(bindings) = Self::pattern_matches(&match_value, &arm.pattern)? {
                let saved_bindings = self.bindings.clone();

                // Apply pattern bindings temporarily
                for (name, value) in bindings {
                    self.bindings.insert(name, value);
                }

                // Check pattern guard if present
                let guard_passes = if let Some(guard_expr) = &arm.guard {
                    if let Value::Bool(b) = self.evaluate_expr(guard_expr, deadline, depth + 1)? { 
                        b 
                    } else {
                        self.bindings = saved_bindings;
                        continue; // Guard didn't evaluate to boolean, try next arm
                    }
                } else {
                    true // No guard, so it passes
                };

                if guard_passes {
                    let result = self.evaluate_expr(&arm.body, deadline, depth + 1)?;
                    self.bindings = saved_bindings;
                    return Ok(result);
                }
                
                // Guard failed, restore bindings and try next arm
                self.bindings = saved_bindings;
            }
        }

        bail!("No matching pattern found in match expression");
    }

    /// Evaluate pipeline expressions
    fn evaluate_pipeline(
        &mut self,
        expr: &Expr,
        stages: &[PipelineStage],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut current_value = self.evaluate_expr(expr, deadline, depth + 1)?;

        for stage in stages {
            current_value = self.evaluate_pipeline_stage(&current_value, stage, deadline, depth)?;
        }

        Ok(current_value)
    }

    /// Evaluate a single pipeline stage
    fn evaluate_pipeline_stage(
        &mut self,
        current_value: &Value,
        stage: &PipelineStage,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match &stage.op.kind {
            ExprKind::Call { func, args } => {
                let mut new_args = vec![Self::value_to_literal_expr(current_value, stage.span)?];
                new_args.extend(args.iter().cloned());

                let new_call = Expr::new(
                    ExprKind::Call {
                        func: func.clone(),
                        args: new_args,
                    },
                    stage.span,
                );

                self.evaluate_expr(&new_call, deadline, depth + 1)
            }
            ExprKind::Identifier(_func_name) => {
                let call = Expr::new(
                    ExprKind::Call {
                        func: stage.op.clone(),
                        args: vec![Self::value_to_literal_expr(current_value, stage.span)?],
                    },
                    stage.span,
                );

                self.evaluate_expr(&call, deadline, depth + 1)
            }
            ExprKind::MethodCall { receiver: _, method, args } => {
                // For method calls in pipeline, current_value becomes the receiver
                match current_value {
                    Value::List(items) => {
                        self.evaluate_list_methods(items.clone(), method, args, deadline, depth)
                    }
                    Value::String(s) => {
                        Self::evaluate_string_methods(s, method, args, deadline, depth)
                    }
                    Value::Int(n) => Self::evaluate_int_methods(*n, method),
                    Value::Float(f) => Self::evaluate_float_methods(*f, method),
                    Value::Object(obj) => {
                        Self::evaluate_object_methods(obj.clone(), method, args, deadline, depth)
                    }
                    Value::HashMap(map) => {
                        self.evaluate_hashmap_methods(map.clone(), method, args, deadline, depth)
                    }
                    Value::HashSet(set) => {
                        self.evaluate_hashset_methods(set.clone(), method, args, deadline, depth)
                    }
                    Value::EnumVariant { .. } => {
                        self.evaluate_enum_methods(current_value.clone(), method, args, deadline, depth)
                    }
                    _ => bail!("Cannot call method {} on value of this type", method),
                }
            }
            _ => bail!("Pipeline stages must be function calls, method calls, or identifiers"),
        }
    }

    /// Convert value to literal expression for pipeline
    fn value_to_literal_expr(value: &Value, span: Span) -> Result<Expr> {
        let expr_kind = match value {
            Value::Int(n) => ExprKind::Literal(Literal::Integer(*n)),
            Value::Float(f) => ExprKind::Literal(Literal::Float(*f)),
            Value::String(s) => ExprKind::Literal(Literal::String(s.clone())),
            Value::Bool(b) => ExprKind::Literal(Literal::Bool(*b)),
            Value::Unit => ExprKind::Literal(Literal::Unit),
            Value::List(items) => {
                let elements: Result<Vec<Expr>> = items
                    .iter()
                    .map(|item| Self::value_to_literal_expr(item, span))
                    .collect();
                ExprKind::List(elements?)
            }
            _ => bail!("Cannot pipeline complex value types yet"),
        };
        Ok(Expr::new(expr_kind, span))
    }

    /// Evaluate command execution
    fn evaluate_command(
        program: &str,
        args: &[String],
        _deadline: Instant,
        _depth: usize,
    ) -> Result<Value> {
        use std::process::Command;
        
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", program, e))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(Value::String(stdout.trim().to_string()))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow::anyhow!(
                "Command '{}' failed with exit code {:?}: {}", 
                program, 
                output.status.code(), 
                stderr
            ))
        }
    }

    /// Evaluate macro expansion
    fn evaluate_macro(
        &mut self,
        name: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match name {
            "println" => {
                // Evaluate all arguments and print them
                let mut output = String::new();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        output.push(' ');
                    }
                    let value = self.evaluate_expr(arg, deadline, depth + 1)?;
                    output.push_str(&value.to_string());
                }
                println!("{output}");
                Ok(Value::Unit)
            }
            "vec" => {
                // Evaluate all arguments and create a vector
                let mut elements = Vec::new();
                for arg in args {
                    elements.push(self.evaluate_expr(arg, deadline, depth + 1)?);
                }
                Ok(Value::List(elements))
            }
            _ => {
                anyhow::bail!("Unknown macro: {}", name)
            }
        }
    }

    /// Evaluate import statements (complexity < 10)
    fn evaluate_import(&mut self, path: &str, items: &[ImportItem]) -> Result<Value> {
        // For now, just track the import in bindings
        // Real implementation would:
        // 1. Resolve the module path
        // 2. Load the module
        // 3. Import specified items into current scope
        
        // Import handling
        
        // Add basic standard library support
        match path {
            "std::fs" | "std::fs::read_file" => {
                // Register file system functions
                for item in items {
                    match item {
                        ImportItem::Named(name) if name == "read_file" => {
                            // This function is already built-in
                            // Successfully imported
                        }
                        ImportItem::Named(name) if name == "write_file" => {
                            // This function is already built-in
                            // Successfully imported
                        }
                        ImportItem::Named(name) if name == "fs" => {
                            // Import entire fs module
                            println!("  ✓ Imported fs module");
                        }
                        _ => {}
                    }
                }
            }
            "std::collections" => {
                // Handle collections imports
                for item in items {
                    if let ImportItem::Named(_name) = item {
                        // Successfully imported
                    }
                }
            }
            _ => {
                // O(1) cache lookup first - NO filesystem access if cached
                if let Some(cached_functions) = self.module_cache.get(path) {
                    // CACHE HIT: O(1) performance - import functions from cache
                    for (func_name, func_value) in cached_functions {
                        let should_import = items.is_empty() || 
                            items.iter().any(|item| match item {
                                ImportItem::Wildcard => true,
                                ImportItem::Named(item_name) => item_name == func_name,
                                ImportItem::Aliased { name: item_name, .. } => item_name == func_name,
                            });
                        
                        if should_import {
                            self.bindings.insert(func_name.clone(), func_value.clone());
                        }
                    }
                } else {
                    // CACHE MISS: Load once and cache forever
                    let module_path = format!("{path}.ruchy");
                    
                    if std::path::Path::new(&module_path).exists() {
                        // Read and parse the module file (only once!)
                        let module_content = std::fs::read_to_string(&module_path)
                            .with_context(|| format!("Failed to read module file: {module_path}"))?;
                        
                        // Parse the module (only once!)
                        let mut parser = crate::frontend::Parser::new(&module_content);
                        let module_ast = parser.parse()
                            .with_context(|| format!("Failed to parse module: {module_path}"))?;
                        
                        // Extract and cache all functions from the module
                        let mut module_functions = HashMap::new();
                        self.extract_module_functions(&module_ast, &mut module_functions)?;
                        
                        // Store in O(1) cache for future imports
                        self.module_cache.insert(path.to_string(), module_functions.clone());
                        
                        // Import requested functions into current scope
                        for (func_name, func_value) in &module_functions {
                            let should_import = items.is_empty() ||
                                items.iter().any(|item| match item {
                                    ImportItem::Wildcard => true,
                                    ImportItem::Named(item_name) => item_name == func_name,
                                    ImportItem::Aliased { name: item_name, .. } => item_name == func_name,
                                });
                            
                            if should_import {
                                self.bindings.insert(func_name.clone(), func_value.clone());
                            }
                        }
                    } else {
                        bail!("Module not found: {}", path);
                    }
                }
            }
        }
        
        Ok(Value::Unit)
    }
    
    /// Extract functions from a module AST into a `HashMap` for caching
    fn extract_module_functions(&mut self, module_ast: &Expr, functions_map: &mut HashMap<String, Value>) -> Result<()> {
        // Extract all functions from the module for caching
        if let ExprKind::Block(exprs) = &module_ast.kind {
            for expr in exprs {
                if let ExprKind::Function { name, params, body, .. } = &expr.kind {
                    // Extract function and store in cache map
                    let param_names: Vec<String> = params.iter()
                        .map(|p| match &p.pattern {
                            Pattern::Identifier(name) => name.clone(),
                            _ => "unknown".to_string(), // Simplified for now
                        })
                        .collect();
                    
                    let function_value = Value::Function {
                        name: name.clone(),
                        params: param_names,
                        body: body.clone(),
                    };
                    functions_map.insert(name.clone(), function_value);
                }
            }
        } else if let ExprKind::Function { name, params, body, .. } = &module_ast.kind {
            // Single function module
            let param_names: Vec<String> = params.iter()
                .map(|p| match &p.pattern {
                    Pattern::Identifier(name) => name.clone(),
                    _ => "unknown".to_string(), // Simplified for now
                })
                .collect();
            
            let function_value = Value::Function {
                name: name.clone(),
                params: param_names,
                body: body.clone(),
            };
            functions_map.insert(name.clone(), function_value);
        }
        
        Ok(())
    }
    
    /// Evaluate export statements (complexity < 10)
    fn evaluate_export(&mut self, _items: &[String]) -> Result<Value> {
        // For now, just track the export
        // Real implementation would:
        // 1. Mark items for export
        // 2. Make them available to importing modules
        
        // Export handling
        
        Ok(Value::Unit)
    }
    
    /// Handle package mode commands
    fn handle_pkg_command(&mut self, input: &str) -> Result<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.first().copied() {
            Some("search") if parts.len() > 1 => {
                Ok(format!("Searching for packages matching '{}'...", parts[1]))
            }
            Some("install") if parts.len() > 1 => {
                Ok(format!("Installing package '{}'...", parts[1]))
            }
            Some("list") => {
                Ok("Installed packages:\n(Package management not yet implemented)".to_string())
            }
            _ => {
                Ok("Package commands: search <query>, install <package>, list".to_string())
            }
        }
    }
    
    /// Handle help mode commands
    fn handle_help_command(&mut self, keyword: &str) -> Result<String> {
        let help_text = match keyword {
            "fn" => "fn - Define a function\nSyntax: fn name(params) { body }\nExample: fn add(a, b) { a + b }",
            "let" => "let - Bind a value to a variable\nSyntax: let name = value\nExample: let x = 42",
            "if" => "if - Conditional execution\nSyntax: if condition { then } else { otherwise }\nExample: if x > 0 { \"positive\" } else { \"negative\" }",
            "for" => "for - Loop over a collection\nSyntax: for item in collection { body }\nExample: for x in [1,2,3] { println(x) }",
            "match" => "match - Pattern matching\nSyntax: match value { pattern => result, ... }\nExample: match x { 0 => \"zero\", _ => \"nonzero\" }",
            _ => &format!("No help available for '{}'\nTry: fn, let, if, for, match, while", keyword),
        };
        Ok(help_text.to_string())
    }
    
    /// Handle math mode commands
    fn handle_math_command(&mut self, expr: &str) -> Result<String> {
        // For now, just evaluate normally but could add special math functions
        let deadline = Instant::now() + self.config.timeout;
        let mut parser = Parser::new(expr);
        let ast = parser.parse().context("Failed to parse math expression")?;
        let value = self.evaluate_expr(&ast, deadline, 0)?;
        Ok(format!("= {}", value))
    }
    
    /// Handle debug mode evaluation
    fn handle_debug_evaluation(&mut self, input: &str) -> Result<String> {
        let start = Instant::now();
        
        // Parse and show AST
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;
        let ast_str = format!("AST: {:?}\n", ast);
        
        // Evaluate
        let deadline = Instant::now() + self.config.timeout;
        let value = self.evaluate_expr(&ast, deadline, 0)?;
        
        let elapsed = start.elapsed();
        Ok(format!("{}Result: {}\nTime: {:?}", ast_str, value, elapsed))
    }
    
    /// Handle timed evaluation
    fn handle_timed_evaluation(&mut self, input: &str) -> Result<String> {
        let start = Instant::now();
        
        // Parse
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;
        
        // Evaluate
        let deadline = Instant::now() + self.config.timeout;
        let value = self.evaluate_expr(&ast, deadline, 0)?;
        
        let elapsed = start.elapsed();
        Ok(format!("{}\n⏱ Time: {:?}", value, elapsed))
    }
    
    /// Generate a stack trace from an error
    fn generate_stack_trace(&self, error: &anyhow::Error) -> Vec<String> {
        let mut stack_trace = Vec::new();
        
        // Add the main error
        stack_trace.push(format!("Error: {}", error));
        
        // Add error chain
        let mut current = error.source();
        while let Some(err) = current {
            stack_trace.push(format!("Caused by: {}", err));
            current = err.source();
        }
        
        // Add current evaluation context if available
        if let Some(last_expr) = self.history.last() {
            stack_trace.push(format!("Last successful expression: {}", last_expr));
        }
        
        stack_trace
    }
}
