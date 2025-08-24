//! REPL implementation for interactive Ruchy development
//!
//! Production-grade REPL with resource bounds, error recovery, and grammar coverage
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
    BinaryOp, Expr, ExprKind, Literal, MatchArm, Pattern, PipelineStage, Span, UnaryOp,
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
            "curry".to_string(),
            "uncurry".to_string(),
            "read_file".to_string(),
            "write_file".to_string(),
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
            let method_partial = &text_before_cursor[dot_pos + 1..];
            // Use HashSet to avoid O(n²) duplicates check
            let mut seen = HashSet::new();
            let mut completions = Vec::new();

            for method in &self.list_methods {
                if method.starts_with(method_partial) && seen.insert(method.clone()) {
                    completions.push(method.clone());
                }
            }
            for method in &self.string_methods {
                if method.starts_with(method_partial) && seen.insert(method.clone()) {
                    completions.push(method.clone());
                }
            }
            return completions;
        }

        // Complete keywords, functions, and variables
        let mut completions = Vec::new();

        completions.extend(
            self.keywords
                .iter()
                .filter(|kw| kw.starts_with(partial_word))
                .cloned(),
        );

        completions.extend(
            self.builtin_functions
                .iter()
                .filter(|func| func.starts_with(partial_word))
                .cloned(),
        );

        completions.extend(
            bindings
                .keys()
                .filter(|var| var.starts_with(partial_word))
                .cloned(),
        );

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
    /// Accumulated definitions for the session
    definitions: Vec<String>,
    /// Bindings and their types/values
    bindings: HashMap<String, Value>,
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
            definitions: Vec::new(),
            bindings: HashMap::new(),
            impl_methods: HashMap::new(),
            enum_definitions: HashMap::new(),
            transpiler: Transpiler::new(),
            temp_dir,
            session_counter: 0,
            config,
            memory,
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

        // Handle let bindings specially
        if let ExprKind::Let { name, type_annotation: _, .. } = &ast.kind {
            self.bindings.insert(name.clone(), value.clone());
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

        // Set evaluation deadline
        let deadline = Instant::now() + self.config.timeout;

        // Parse the input
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;

        // Check memory for AST
        self.memory.try_alloc(std::mem::size_of_val(&ast))?;

        // Evaluate the expression
        let value = self.evaluate_expr(&ast, deadline, 0)?;

        // Store successful evaluation
        self.history.push(input.to_string());

        // Let bindings are handled in evaluate_expr, no need to duplicate here

        Ok(value.to_string())
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
            ExprKind::Let { name, type_annotation: _, value, body, .. } => {
                self.evaluate_let_binding(name, value, body, deadline, depth)
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
                let parts: Vec<Value> = s
                    .split_whitespace()
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::List(parts))
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

    /// Evaluate while loop (complexity: 8)
    fn evaluate_while_loop(
        &mut self,
        condition: &Expr,
        body: &Expr,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut result = Value::Unit;
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
                    result = self.evaluate_expr(body, deadline, depth + 1)?;
                    iterations += 1;
                }
                Value::Bool(false) => break,
                _ => bail!("While condition must be boolean, got: {:?}", cond_val),
            }
        }

        Ok(result)
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
            self.bindings.insert(name.clone(), val.clone());
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
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let val = self.evaluate_expr(value, deadline, depth + 1)?;
        self.bindings.insert(name.to_string(), val.clone());

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
            }
        }
        Ok(Value::String(result))
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
    fn evaluate_enum_methods(
        &mut self,
        receiver: Value,
        method: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let Value::EnumVariant { enum_name, variant_name, data } = receiver {
            match (enum_name.as_str(), variant_name.as_str(), method) {
                // Result methods
                ("Result", "Ok", "unwrap") if args.is_empty() => {
                    if let Some(values) = data {
                        if !values.is_empty() {
                            return Ok(values[0].clone());
                        }
                    }
                    Ok(Value::Unit)
                }
                ("Result", "Err", "unwrap") if args.is_empty() => {
                    let error_msg = if let Some(values) = &data {
                        if values.is_empty() {
                            "called `Result::unwrap()` on an `Err` value".to_string()
                        } else {
                            format!("called `Result::unwrap()` on an `Err` value: {}", values[0])
                        }
                    } else {
                        "called `Result::unwrap()` on an `Err` value".to_string()
                    };
                    bail!(error_msg)
                }
                ("Result", "Ok", "expect") if args.len() == 1 => {
                    // Ignore the custom message for Ok variant
                    if let Some(values) = data {
                        if !values.is_empty() {
                            return Ok(values[0].clone());
                        }
                    }
                    Ok(Value::Unit)
                }
                ("Result", "Err", "expect") if args.len() == 1 => {
                    let custom_msg = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                    let msg = match custom_msg {
                        Value::String(s) => s,
                        _ => format!("{custom_msg}"),
                    };
                    bail!(msg)
                }
                // Option methods
                ("Option", "Some", "unwrap") if args.is_empty() => {
                    if let Some(values) = data {
                        if !values.is_empty() {
                            return Ok(values[0].clone());
                        }
                    }
                    Ok(Value::Unit)
                }
                ("Option", "None", "unwrap") if args.is_empty() => {
                    bail!("called `Option::unwrap()` on a `None` value")
                }
                ("Option", "Some", "expect") if args.len() == 1 => {
                    // Ignore the custom message for Some variant
                    if let Some(values) = data {
                        if !values.is_empty() {
                            return Ok(values[0].clone());
                        }
                    }
                    Ok(Value::Unit)
                }
                ("Option", "None", "expect") if args.len() == 1 => {
                    let custom_msg = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                    let msg = match custom_msg {
                        Value::String(s) => s,
                        _ => format!("{custom_msg}"),
                    };
                    bail!(msg)
                }
                // Result combinators
                ("Result", "Ok", "map") if args.len() == 1 => {
                    // Apply function to Ok value
                    if let Some(values) = data {
                        if !values.is_empty() {
                            let func_arg = &args[0];
                            // Create a synthetic call expression
                            let call_expr = Expr::new(
                                ExprKind::Call {
                                    func: Box::new(func_arg.clone()),
                                    args: vec![Expr::new(
                                        ExprKind::Literal(crate::frontend::ast::Literal::from_value(&values[0])),
                                        Span { start: 0, end: 0 },
                                    )],
                                },
                                Span { start: 0, end: 0 },
                            );
                            let mapped_value = self.evaluate_expr(&call_expr, deadline, depth + 1)?;
                            return Ok(Value::EnumVariant {
                                enum_name: "Result".to_string(),
                                variant_name: "Ok".to_string(),
                                data: Some(vec![mapped_value]),
                            });
                        }
                    }
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Ok".to_string(),
                        data: Some(vec![Value::Unit]),
                    })
                }
                ("Result", "Err", "map") if args.len() == 1 => {
                    // map does nothing on Err, return as-is
                    Ok(Value::EnumVariant {
                        enum_name,
                        variant_name,
                        data,
                    })
                }
                ("Result", "Ok", "and_then") if args.len() == 1 => {
                    // Apply function that returns Result to Ok value
                    if let Some(values) = data {
                        if !values.is_empty() {
                            let func_arg = &args[0];
                            let call_expr = Expr::new(
                                ExprKind::Call {
                                    func: Box::new(func_arg.clone()),
                                    args: vec![Expr::new(
                                        ExprKind::Literal(crate::frontend::ast::Literal::from_value(&values[0])),
                                        Span { start: 0, end: 0 },
                                    )],
                                },
                                Span { start: 0, end: 0 },
                            );
                            // and_then flattens the Result
                            return self.evaluate_expr(&call_expr, deadline, depth + 1);
                        }
                    }
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Ok".to_string(),
                        data: Some(vec![Value::Unit]),
                    })
                }
                ("Result", "Err", "and_then") if args.len() == 1 => {
                    // and_then does nothing on Err, return as-is
                    Ok(Value::EnumVariant {
                        enum_name,
                        variant_name,
                        data,
                    })
                }
                // Option combinators
                ("Option", "Some", "map") if args.len() == 1 => {
                    // Apply function to Some value
                    if let Some(values) = data {
                        if !values.is_empty() {
                            let func_arg = &args[0];
                            let call_expr = Expr::new(
                                ExprKind::Call {
                                    func: Box::new(func_arg.clone()),
                                    args: vec![Expr::new(
                                        ExprKind::Literal(crate::frontend::ast::Literal::from_value(&values[0])),
                                        Span { start: 0, end: 0 },
                                    )],
                                },
                                Span { start: 0, end: 0 },
                            );
                            let mapped_value = self.evaluate_expr(&call_expr, deadline, depth + 1)?;
                            return Ok(Value::EnumVariant {
                                enum_name: "Option".to_string(),
                                variant_name: "Some".to_string(),
                                data: Some(vec![mapped_value]),
                            });
                        }
                    }
                    Ok(Value::EnumVariant {
                        enum_name: "Option".to_string(),
                        variant_name: "Some".to_string(),
                        data: Some(vec![Value::Unit]),
                    })
                }
                ("Option", "None", "map") if args.len() == 1 => {
                    // map does nothing on None, return as-is
                    Ok(Value::EnumVariant {
                        enum_name,
                        variant_name,
                        data,
                    })
                }
                ("Option", "Some", "and_then") if args.len() == 1 => {
                    // Apply function that returns Option to Some value
                    if let Some(values) = data {
                        if !values.is_empty() {
                            let func_arg = &args[0];
                            let call_expr = Expr::new(
                                ExprKind::Call {
                                    func: Box::new(func_arg.clone()),
                                    args: vec![Expr::new(
                                        ExprKind::Literal(crate::frontend::ast::Literal::from_value(&values[0])),
                                        Span { start: 0, end: 0 },
                                    )],
                                },
                                Span { start: 0, end: 0 },
                            );
                            // and_then flattens the Option
                            return self.evaluate_expr(&call_expr, deadline, depth + 1);
                        }
                    }
                    Ok(Value::EnumVariant {
                        enum_name: "Option".to_string(),
                        variant_name: "Some".to_string(),
                        data: Some(vec![Value::Unit]),
                    })
                }
                ("Option", "None", "and_then") if args.len() == 1 => {
                    // and_then does nothing on None, return as-is
                    Ok(Value::EnumVariant {
                        enum_name,
                        variant_name,
                        data,
                    })
                }
                _ => bail!("Method {} not supported on {}", method, enum_name),
            }
        } else {
            bail!("evaluate_enum_methods called on non-enum variant")
        }
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
            match obj_val {
                Value::List(list) => {
                    let start_idx = usize::try_from(start)
                        .map_err(|_| anyhow::anyhow!("Invalid start index: {}", start))?;
                    let end_idx = if inclusive {
                        usize::try_from(end + 1)
                            .map_err(|_| anyhow::anyhow!("Invalid end index: {}", end + 1))?
                    } else {
                        usize::try_from(end)
                            .map_err(|_| anyhow::anyhow!("Invalid end index: {}", end))?
                    };
                    
                    if start_idx > list.len() || end_idx > list.len() {
                        return Err(anyhow::anyhow!("Slice indices out of bounds"));
                    }
                    if start_idx > end_idx {
                        return Err(anyhow::anyhow!("Invalid slice range: start > end"));
                    }
                    
                    return Ok(Value::List(list[start_idx..end_idx].to_vec()));
                }
                Value::String(s) => {
                    let chars: Vec<char> = s.chars().collect();
                    let start_idx = usize::try_from(start)
                        .map_err(|_| anyhow::anyhow!("Invalid start index: {}", start))?;
                    let end_idx = if inclusive {
                        usize::try_from(end + 1)
                            .map_err(|_| anyhow::anyhow!("Invalid end index: {}", end + 1))?
                    } else {
                        usize::try_from(end)
                            .map_err(|_| anyhow::anyhow!("Invalid end index: {}", end))?
                    };
                    
                    if start_idx > chars.len() || end_idx > chars.len() {
                        return Err(anyhow::anyhow!("Slice indices out of bounds"));
                    }
                    if start_idx > end_idx {
                        return Err(anyhow::anyhow!("Invalid slice range: start > end"));
                    }
                    
                    return Ok(Value::String(chars[start_idx..end_idx].iter().collect()));
                }
                _ => {
                    return Err(anyhow::anyhow!("Cannot slice into {:?}", obj_val));
                }
            }
        }

        match (obj_val, index_val) {
            (Value::List(list), Value::Int(idx)) => {
                let idx = usize::try_from(idx)
                    .map_err(|_| anyhow::anyhow!("Invalid index: {}", idx))?;
                if idx >= list.len() {
                    return Err(anyhow::anyhow!("Index {} out of bounds for list of length {}", idx, list.len()));
                }
                Ok(list[idx].clone())
            }
            (Value::String(s), Value::Int(idx)) => {
                let idx = usize::try_from(idx)
                    .map_err(|_| anyhow::anyhow!("Invalid index: {}", idx))?;
                let chars: Vec<char> = s.chars().collect();
                if idx >= chars.len() {
                    return Err(anyhow::anyhow!("Index {} out of bounds for string of length {}", idx, chars.len()));
                }
                Ok(Value::String(chars[idx].to_string()))
            }
            (Value::Object(obj), Value::String(key)) => {
                // Object indexing with string keys
                match obj.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Err(anyhow::anyhow!("Key '{}' not found in object", key)),
                }
            }
            (obj_val, index_val) => Err(anyhow::anyhow!("Cannot index into {:?} with index {:?}", obj_val, index_val)),
        }
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
        self.bindings
            .get(name)
            .cloned()
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
            (Value::Object(obj_fields), Pattern::Struct { name: _struct_name, fields: pattern_fields }) => {
                // For now, we don't check struct name since objects are generic
                // Check all pattern fields match
                for pattern_field in pattern_fields {
                    let field_name = &pattern_field.name;
                    
                    // Find the corresponding field in the object
                    if let Some(field_value) = obj_fields.get(field_name) {
                        // Check if pattern matches (if specified)
                        if let Some(pattern) = &pattern_field.pattern {
                            let mut temp_bindings = HashMap::new();
                            if !Self::pattern_matches_recursive(field_value, pattern, &mut temp_bindings)? {
                                return Ok(false);
                            }
                        }
                        // For shorthand patterns ({ x } instead of { x: x }), 
                        // we just check the field exists, which it does
                    } else {
                        // Required field not found in struct
                        return Ok(false);
                    }
                }
                Ok(true)
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
                        if self.handle_command(&line)? {
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

    /// Print help message
    fn print_help() {
        println!("{}", "Available commands:".bright_cyan());
        println!("  :help, :h       - Show this help message");
        println!("  :quit, :q       - Exit the REPL");
        println!("  :history        - Show evaluation history");
        println!("  :search <query> - Search history with fuzzy matching");
        println!("  :clear          - Clear definitions and history");
        println!("  :reset          - Full reset to initial state");
        println!("  :bindings, :env - Show current variable bindings");
        println!("  :type <expr>    - Show type of expression");
        println!("  :ast <expr>     - Show AST of expression");
        println!("  :compile        - Compile and run the session");
        println!("  :load <file>    - Load and evaluate a file");
        println!("  :save <file>    - Save session to file");
        println!();
        println!("{}", "Examples:".bright_cyan());
        println!("  2 + 2           - Evaluate expression");
        println!("  let x = 10      - Define variable");
        println!("  :type x * 2     - Show type of expression");
        println!("  :ast if true {{ 1 }} else {{ 2 }}");
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
                "curry" => self.evaluate_curry(args, deadline, depth),
                "uncurry" => self.evaluate_uncurry(args, deadline, depth),
                "read_file" => self.evaluate_read_file(args, deadline, depth),
                "write_file" => self.evaluate_write_file(args, deadline, depth),
                _ => self.evaluate_user_function(func_name, args, deadline, depth),
            }
        } else if let ExprKind::QualifiedName { module, name } = &func.kind {
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

    /// Evaluate user-defined functions
    fn evaluate_user_function(
        &mut self,
        func_name: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        // Check if this is a built-in enum variant constructor
        match func_name {
            "None" => {
                if !args.is_empty() {
                    bail!("None takes no arguments");
                }
                return Ok(Value::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    data: None,
                });
            }
            "Some" => {
                if args.len() != 1 {
                    bail!("Some takes exactly 1 argument");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                return Ok(Value::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    data: Some(vec![value]),
                });
            }
            "Ok" => {
                if args.len() != 1 {
                    bail!("Ok takes exactly 1 argument");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                return Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Ok".to_string(),
                    data: Some(vec![value]),
                });
            }
            "Err" => {
                if args.len() != 1 {
                    bail!("Err takes exactly 1 argument");
                }
                let value = self.evaluate_expr(&args[0], deadline, depth + 1)?;
                return Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Err".to_string(),
                    data: Some(vec![value]),
                });
            }
            _ => {}
        }

        if let Some(func_value) = self.bindings.get(func_name).cloned() {
            match func_value {
                Value::Function { params, body, .. } => {
                    if args.len() != params.len() {
                        bail!(
                            "Function {} expects {} arguments, got {}",
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

                    let result = self.evaluate_function_body(&body, deadline, depth)?;
                    self.bindings = saved_bindings;
                    Ok(result)
                }
                Value::Lambda { params, body } => {
                    if args.len() != params.len() {
                        bail!(
                            "Lambda expects {} arguments, got {}",
                            params.len(),
                            args.len()
                        );
                    }

                    let saved_bindings = self.bindings.clone();

                    for (param, arg) in params.iter().zip(args.iter()) {
                        let arg_value = self.evaluate_expr(arg, deadline, depth + 1)?;
                        self.bindings.insert(param.clone(), arg_value);
                    }

                    let result = self.evaluate_function_body(&body, deadline, depth)?;
                    self.bindings = saved_bindings;
                    Ok(result)
                }
                _ => {
                    bail!("'{}' is not a function", func_name);
                }
            }
        } else {
            bail!("Unknown function: {}", func_name);
        }
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
            _ => bail!("Pipeline stages must be function calls or identifiers"),
        }
    }

    /// Convert value to literal expression for pipeline
    fn value_to_literal_expr(value: &Value, span: Span) -> Result<Expr> {
        let literal = match value {
            Value::Int(n) => Literal::Integer(*n),
            Value::Float(f) => Literal::Float(*f),
            Value::String(s) => Literal::String(s.clone()),
            Value::Bool(b) => Literal::Bool(*b),
            Value::Unit => Literal::Unit,
            _ => bail!("Cannot pipeline complex value types yet"),
        };
        Ok(Expr::new(ExprKind::Literal(literal), span))
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
}
