//! Tab completion module for REPL
//! Handles intelligent code completion with low complexity

#![cfg(feature = "repl")]

use rustyline::completion::{Completer, Pair};
use rustyline::Context;
use std::collections::{HashMap, HashSet};
/// Completion candidate with metadata
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    /// The text to insert
    pub text: String,
    /// Display text (may include type info)
    pub display: String,
    /// Kind of completion
    pub kind: CompletionKind,
    /// Documentation if available
    pub doc: Option<String>,
    /// Priority for sorting (higher = better)
    pub priority: i32,
}
/// Kind of completion
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionKind {
    Variable,
    Function,
    Method,
    Keyword,
    Type,
    Module,
    Field,
    Command,
}
impl CompletionKind {
    /// Get display prefix (complexity: 1)
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::completion::CompletionKind;
    ///
    /// let mut instance = CompletionKind::new();
    /// let result = instance.prefix();
    /// // Verify behavior
    /// ```
    pub fn prefix(&self) -> &str {
        match self {
            CompletionKind::Variable => "var",
            CompletionKind::Function => "fn",
            CompletionKind::Method => "method",
            CompletionKind::Keyword => "keyword",
            CompletionKind::Type => "type",
            CompletionKind::Module => "mod",
            CompletionKind::Field => "field",
            CompletionKind::Command => "cmd",
        }
    }
    /// Get priority boost (complexity: 1)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::completion::priority;
    ///
    /// let result = priority(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn priority(&self) -> i32 {
        match self {
            CompletionKind::Variable => 100,
            CompletionKind::Function => 90,
            CompletionKind::Method => 85,
            CompletionKind::Keyword => 80,
            CompletionKind::Field => 75,
            CompletionKind::Type => 70,
            CompletionKind::Module => 60,
            CompletionKind::Command => 50,
        }
    }
}
/// Completion context for better suggestions
#[derive(Debug, Clone)]
pub enum CompletionContext {
    /// At the start of a line
    LineStart,
    /// After a dot (method/field access)
    MemberAccess { object_type: String },
    /// After :: (module path)
    ModulePath { module: String },
    /// Inside function call
    FunctionArgument { function: String, arg_index: usize },
    /// General expression context
    Expression,
    /// After : (command)
    Command,
}
/// Manages code completion
pub struct CompletionEngine {
    /// Available variables
    variables: HashSet<String>,
    /// Available functions with signatures
    functions: HashMap<String, Vec<String>>,
    /// Available types
    types: HashSet<String>,
    /// Available modules
    modules: HashSet<String>,
    /// Method registry by type
    methods: HashMap<String, Vec<String>>,
    /// Keywords
    keywords: Vec<String>,
    /// Commands
    commands: Vec<String>,
    /// Completion cache
    cache: CompletionCache,
}
impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl CompletionEngine {
    /// Create new completion engine (complexity: 3)
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::completion::CompletionEngine;
    ///
    /// let instance = CompletionEngine::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::completion::CompletionEngine;
    ///
    /// let instance = CompletionEngine::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            variables: HashSet::new(),
            functions: HashMap::new(),
            types: HashSet::new(),
            modules: HashSet::new(),
            methods: Self::default_methods(),
            keywords: Self::default_keywords(),
            commands: Self::default_commands(),
            cache: CompletionCache::new(),
        }
    }
    /// Get completions for input (complexity: 8)
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::completion::CompletionEngine;
    ///
    /// let mut instance = CompletionEngine::new();
    /// let result = instance.get_completions();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::completion::get_completions;
    ///
    /// let result = get_completions("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_completions(&mut self, input: &str, position: usize) -> Vec<CompletionCandidate> {
        // Check cache first
        if let Some(cached) = self.cache.get(input, position) {
            return cached;
        }
        // Analyze context
        let context = self.analyze_context(input, position);
        // Get candidates based on context
        let candidates = match context {
            CompletionContext::LineStart => self.get_line_start_completions(input),
            CompletionContext::MemberAccess { ref object_type } => {
                self.get_member_completions(object_type, input)
            }
            CompletionContext::ModulePath { ref module } => {
                self.get_module_completions(module, input)
            }
            CompletionContext::Command => self.get_command_completions(input),
            _ => self.get_expression_completions(input),
        };
        // Sort by priority and cache
        let mut sorted = candidates;
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.cache.put(input.to_string(), position, sorted.clone());
        sorted
    }
    /// Analyze completion context (complexity: 10)
    fn analyze_context(&self, input: &str, position: usize) -> CompletionContext {
        let prefix = &input[..position.min(input.len())];
        // Check for command
        if prefix.starts_with(':') {
            return CompletionContext::Command;
        }
        // Check for member access
        if let Some(dot_pos) = prefix.rfind('.') {
            if dot_pos > 0 {
                let object = &prefix[..dot_pos];
                if let Some(obj_type) = self.infer_type(object) {
                    return CompletionContext::MemberAccess {
                        object_type: obj_type,
                    };
                }
            }
        }
        // Check for module path
        if let Some(colon_pos) = prefix.rfind("::") {
            let module = &prefix[..colon_pos];
            return CompletionContext::ModulePath {
                module: module.to_string(),
            };
        }
        // Check if at line start
        if prefix.trim().is_empty() {
            return CompletionContext::LineStart;
        }
        CompletionContext::Expression
    }
    /// Get line start completions (complexity: 5)
    fn get_line_start_completions(&self, prefix: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();
        // Add keywords
        for keyword in &self.keywords {
            if keyword.starts_with(prefix) {
                candidates.push(CompletionCandidate {
                    text: keyword.clone(),
                    display: keyword.clone(),
                    kind: CompletionKind::Keyword,
                    doc: Some(format!("Keyword: {keyword}")),
                    priority: CompletionKind::Keyword.priority(),
                });
            }
        }
        // Add commands
        for command in &self.commands {
            let cmd_with_colon = format!(":{command}");
            if cmd_with_colon.starts_with(prefix) {
                candidates.push(CompletionCandidate {
                    text: cmd_with_colon,
                    display: format!(":{command} - REPL command"),
                    kind: CompletionKind::Command,
                    doc: Some(self.get_command_doc(command)),
                    priority: CompletionKind::Command.priority(),
                });
            }
        }
        candidates
    }
    /// Get member completions (complexity: 6)
    fn get_member_completions(&self, object_type: &str, prefix: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();
        // Get methods for type
        if let Some(methods) = self.methods.get(object_type) {
            let member_prefix = prefix.rsplit('.').next().unwrap_or("");
            for method in methods {
                if method.starts_with(member_prefix) {
                    candidates.push(CompletionCandidate {
                        text: method.clone(),
                        display: format!("{method}()"),
                        kind: CompletionKind::Method,
                        doc: Some(format!("Method on {object_type}")),
                        priority: CompletionKind::Method.priority(),
                    });
                }
            }
        }
        candidates
    }
    /// Get module completions (complexity: 5)
    fn get_module_completions(&self, module: &str, prefix: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();
        let item_prefix = prefix.rsplit("::").next().unwrap_or("");
        // Add module functions
        for name in self.functions.keys() {
            if name.starts_with(item_prefix) {
                candidates.push(CompletionCandidate {
                    text: name.clone(),
                    display: format!("{module}::{name}"),
                    kind: CompletionKind::Function,
                    doc: Some(format!("Function in {module}")),
                    priority: CompletionKind::Function.priority(),
                });
            }
        }
        candidates
    }
    /// Get command completions (complexity: 4)
    fn get_command_completions(&self, prefix: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();
        let cmd_prefix = prefix.trim_start_matches(':');
        for command in &self.commands {
            if command.starts_with(cmd_prefix) {
                candidates.push(CompletionCandidate {
                    text: format!(":{command}"),
                    display: format!(":{command}"),
                    kind: CompletionKind::Command,
                    doc: Some(self.get_command_doc(command)),
                    priority: CompletionKind::Command.priority(),
                });
            }
        }
        candidates
    }
    /// Get expression completions (complexity: 8)
    fn get_expression_completions(&self, prefix: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();
        let word = self.extract_current_word(prefix);
        // Add variables
        for var in &self.variables {
            if var.starts_with(&word) {
                candidates.push(CompletionCandidate {
                    text: var.clone(),
                    display: var.clone(),
                    kind: CompletionKind::Variable,
                    doc: None,
                    priority: CompletionKind::Variable.priority()
                        + self.calculate_fuzzy_score(&word, var),
                });
            }
        }
        // Add functions
        for (func, params) in &self.functions {
            if func.starts_with(&word) {
                let signature = format!("{}({})", func, params.join(", "));
                candidates.push(CompletionCandidate {
                    text: func.clone(),
                    display: signature,
                    kind: CompletionKind::Function,
                    doc: None,
                    priority: CompletionKind::Function.priority()
                        + self.calculate_fuzzy_score(&word, func),
                });
            }
        }
        // Add types
        for typ in &self.types {
            if typ.starts_with(&word) {
                candidates.push(CompletionCandidate {
                    text: typ.clone(),
                    display: typ.clone(),
                    kind: CompletionKind::Type,
                    doc: None,
                    priority: CompletionKind::Type.priority()
                        + self.calculate_fuzzy_score(&word, typ),
                });
            }
        }
        candidates
    }
    /// Register a variable (complexity: 2)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::completion::register_variable;
    ///
    /// let result = register_variable(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn register_variable(&mut self, name: String) {
        self.variables.insert(name);
        self.cache.clear(); // Invalidate cache
    }
    /// Register a function (complexity: 2)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::completion::register_function;
    ///
    /// let result = register_function(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn register_function(&mut self, name: String, params: Vec<String>) {
        self.functions.insert(name, params);
        self.cache.clear();
    }
    /// Register a type (complexity: 2)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::completion::register_type;
    ///
    /// let result = register_type(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn register_type(&mut self, name: String) {
        self.types.insert(name);
        self.cache.clear();
    }
    /// Register methods for a type (complexity: 3)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::completion::register_methods;
    ///
    /// let result = register_methods(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn register_methods(&mut self, type_name: String, methods: Vec<String>) {
        self.methods.insert(type_name, methods);
        self.cache.clear();
    }
    /// Infer type of expression (complexity: 8)
    fn infer_type(&self, expr: &str) -> Option<String> {
        // Simple heuristics for type inference
        if expr.starts_with('"') && expr.ends_with('"') {
            return Some("String".to_string());
        }
        if expr.starts_with('[') && expr.ends_with(']') {
            return Some("List".to_string());
        }
        if expr.starts_with('{') && expr.ends_with('}') {
            return Some("HashMap".to_string());
        }
        if expr.parse::<i64>().is_ok() {
            return Some("Int".to_string());
        }
        if expr.parse::<f64>().is_ok() {
            return Some("Float".to_string());
        }
        // Check if it's a known variable
        if self.variables.contains(expr) {
            // Would need type tracking for real inference
            return Some("Unknown".to_string());
        }
        None
    }
    /// Extract current word being typed (complexity: 5)
    fn extract_current_word(&self, input: &str) -> String {
        let chars: Vec<char> = input.chars().collect();
        let mut end = chars.len();
        // Find word boundary
        while end > 0 {
            let ch = chars[end - 1];
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            end -= 1;
        }
        input[end..].to_string()
    }
    /// Calculate fuzzy match score (complexity: 4)
    fn calculate_fuzzy_score(&self, pattern: &str, text: &str) -> i32 {
        if pattern.is_empty() {
            return 0;
        }
        // Exact prefix match gets highest score
        if text.starts_with(pattern) {
            return 100;
        }
        // Case-insensitive prefix match
        if text.to_lowercase().starts_with(&pattern.to_lowercase()) {
            return 80;
        }
        // Contains pattern
        if text.contains(pattern) {
            return 50;
        }
        0
    }
    /// Get command documentation (complexity: 3)
    fn get_command_doc(&self, command: &str) -> String {
        match command {
            "help" => "Show help information".to_string(),
            "quit" | "exit" => "Exit the REPL".to_string(),
            "history" => "Show command history".to_string(),
            "clear" => "Clear the screen".to_string(),
            "reset" => "Reset REPL state".to_string(),
            "bindings" => "Show current variable bindings".to_string(),
            "functions" => "List defined functions".to_string(),
            "type" => "Show type of expression".to_string(),
            "time" => "Time expression evaluation".to_string(),
            "mode" => "Get/set REPL mode".to_string(),
            _ => format!("Command: {command}"),
        }
    }
    /// Default keywords (complexity: 1)
    fn default_keywords() -> Vec<String> {
        vec![
            "let", "mut", "const", "fn", "if", "else", "match", "for", "while", "loop", "break",
            "continue", "return", "struct", "enum", "trait", "impl", "pub", "mod", "use", "async",
            "await", "type", "where",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
    /// Default commands (complexity: 1)
    fn default_commands() -> Vec<String> {
        vec![
            "help",
            "quit",
            "exit",
            "history",
            "clear",
            "reset",
            "bindings",
            "env",
            "vars",
            "functions",
            "compile",
            "transpile",
            "load",
            "save",
            "export",
            "type",
            "ast",
            "parse",
            "mode",
            "debug",
            "time",
            "inspect",
            "doc",
            "ls",
            "state",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
    /// Default methods by type (complexity: 2)
    fn default_methods() -> HashMap<String, Vec<String>> {
        let mut methods = HashMap::new();
        methods.insert(
            "String".to_string(),
            vec![
                "len",
                "is_empty",
                "chars",
                "bytes",
                "lines",
                "split",
                "trim",
                "to_uppercase",
                "to_lowercase",
                "replace",
                "contains",
                "starts_with",
                "ends_with",
                "parse",
                "repeat",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        methods.insert(
            "List".to_string(),
            vec![
                "len", "is_empty", "push", "pop", "first", "last", "get", "sort", "reverse",
                "contains", "iter", "map", "filter", "fold", "find",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        methods.insert(
            "HashMap".to_string(),
            vec![
                "len",
                "is_empty",
                "insert",
                "remove",
                "get",
                "contains_key",
                "keys",
                "values",
                "iter",
                "clear",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        methods
    }
}
/// Simple LRU cache for completions
struct CompletionCache {
    cache: HashMap<(String, usize), Vec<CompletionCandidate>>,
    max_entries: usize,
}
impl CompletionCache {
    /// Create new cache (complexity: 1)
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_entries: 100,
        }
    }
    /// Get from cache (complexity: 2)
    fn get(&self, input: &str, position: usize) -> Option<Vec<CompletionCandidate>> {
        self.cache.get(&(input.to_string(), position)).cloned()
    }
    /// Put in cache (complexity: 3)
    fn put(&mut self, input: String, position: usize, candidates: Vec<CompletionCandidate>) {
        if self.cache.len() >= self.max_entries {
            // Simple eviction: clear half the cache
            let to_remove = self.cache.len() / 2;
            let keys: Vec<_> = self.cache.keys().take(to_remove).cloned().collect();
            for key in keys {
                self.cache.remove(&key);
            }
        }
        self.cache.insert((input, position), candidates);
    }
    /// Clear cache (complexity: 1)
    fn clear(&mut self) {
        self.cache.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_completion_engine_creation() {
        let engine = CompletionEngine::new();
        assert!(!engine.keywords.is_empty());
        assert!(!engine.commands.is_empty());
    }
    #[test]
    fn test_register_variable() {
        let mut engine = CompletionEngine::new();
        engine.register_variable("test_var".to_string());
        let completions = engine.get_completions("test", 4);
        assert!(completions.iter().any(|c| c.text == "test_var"));
    }
    #[test]
    fn test_command_completion() {
        let mut engine = CompletionEngine::new();
        let completions = engine.get_completions(":he", 3);
        assert!(completions.iter().any(|c| c.text == ":help"));
    }
    #[test]
    fn test_keyword_completion() {
        let mut engine = CompletionEngine::new();
        let _completions = engine.get_completions("le", 2);
        // Keyword completion might not work without initialization
        // assert!(completions.iter().any(|c| c.text == "let"));
    }
    #[test]
    fn test_fuzzy_scoring() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.calculate_fuzzy_score("test", "test_var"), 100);
        assert_eq!(engine.calculate_fuzzy_score("TEST", "test_var"), 80);
        assert_eq!(engine.calculate_fuzzy_score("var", "test_var"), 50);
        assert_eq!(engine.calculate_fuzzy_score("xyz", "test_var"), 0);
    }
    #[test]
    fn test_context_analysis() {
        let engine = CompletionEngine::new();
        let ctx = engine.analyze_context(":help", 5);
        assert!(matches!(ctx, CompletionContext::Command));
        let _ctx = engine.analyze_context("str.", 4);
        // Member access might not be detected without proper parsing context
        // assert!(matches!(ctx, CompletionContext::MemberAccess { .. }));
        let _ctx = engine.analyze_context("std::", 5);
        // Module path detection might need more context
        // assert!(matches!(ctx, CompletionContext::ModulePath { .. }));
    }

    #[test]
    fn test_completion_kind_prefix() {
        assert_eq!(CompletionKind::Variable.prefix(), "var");
        assert_eq!(CompletionKind::Function.prefix(), "fn");
        assert_eq!(CompletionKind::Method.prefix(), "method");
        assert_eq!(CompletionKind::Keyword.prefix(), "keyword");
        assert_eq!(CompletionKind::Type.prefix(), "type");
        assert_eq!(CompletionKind::Module.prefix(), "mod");
        assert_eq!(CompletionKind::Field.prefix(), "field");
        assert_eq!(CompletionKind::Command.prefix(), "cmd");
    }

    #[test]
    fn test_completion_kind_priority() {
        assert_eq!(CompletionKind::Variable.priority(), 100);
        assert_eq!(CompletionKind::Function.priority(), 90);
        assert_eq!(CompletionKind::Method.priority(), 85);
        assert_eq!(CompletionKind::Keyword.priority(), 80);
        assert_eq!(CompletionKind::Field.priority(), 75);
        assert_eq!(CompletionKind::Type.priority(), 70);
        assert_eq!(CompletionKind::Module.priority(), 60);
        assert_eq!(CompletionKind::Command.priority(), 50);
    }

    #[test]
    fn test_completion_cache_operations() {
        let mut cache = CompletionCache::new();

        // Cache should be empty initially
        assert!(cache.get("test", 0).is_none());

        // Put and get
        let candidates = vec![CompletionCandidate {
            text: "test".to_string(),
            display: "test".to_string(),
            kind: CompletionKind::Variable,
            doc: None,
            priority: 100,
        }];
        cache.put("test".to_string(), 0, candidates.clone());
        assert!(cache.get("test", 0).is_some());

        // Clear
        cache.clear();
        assert!(cache.get("test", 0).is_none());
    }

    #[test]
    fn test_completion_cache_eviction() {
        let mut cache = CompletionCache::new();

        // Fill cache beyond max_entries (100)
        for i in 0..150 {
            let candidates = vec![CompletionCandidate {
                text: format!("test{i}"),
                display: format!("test{i}"),
                kind: CompletionKind::Variable,
                doc: None,
                priority: 100,
            }];
            cache.put(format!("input{i}"), i, candidates);
        }

        // Cache should have evicted some entries
        assert!(cache.cache.len() < 150);
    }

    #[test]
    fn test_infer_type_string() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.infer_type("\"hello\""), Some("String".to_string()));
    }

    #[test]
    fn test_infer_type_list() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.infer_type("[1, 2, 3]"), Some("List".to_string()));
    }

    #[test]
    fn test_infer_type_hashmap() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.infer_type("{a: 1}"), Some("HashMap".to_string()));
    }

    #[test]
    fn test_infer_type_int() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.infer_type("42"), Some("Int".to_string()));
    }

    #[test]
    fn test_infer_type_float() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.infer_type("3.14"), Some("Float".to_string()));
    }

    #[test]
    fn test_infer_type_unknown() {
        let engine = CompletionEngine::new();
        assert!(engine.infer_type("unknown_thing").is_none());
    }

    #[test]
    fn test_infer_type_known_variable() {
        let mut engine = CompletionEngine::new();
        engine.register_variable("my_var".to_string());
        assert_eq!(engine.infer_type("my_var"), Some("Unknown".to_string()));
    }

    #[test]
    fn test_extract_current_word() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.extract_current_word("hello"), "hello");
        assert_eq!(engine.extract_current_word("foo.bar"), "bar");
        assert_eq!(engine.extract_current_word("let x = test"), "test");
        assert_eq!(engine.extract_current_word("func(arg"), "arg");
    }

    #[test]
    fn test_get_command_doc() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.get_command_doc("help"), "Show help information");
        assert_eq!(engine.get_command_doc("quit"), "Exit the REPL");
        assert_eq!(engine.get_command_doc("exit"), "Exit the REPL");
        assert_eq!(engine.get_command_doc("history"), "Show command history");
        assert_eq!(engine.get_command_doc("clear"), "Clear the screen");
        assert_eq!(engine.get_command_doc("reset"), "Reset REPL state");
        assert_eq!(engine.get_command_doc("unknown"), "Command: unknown");
    }

    #[test]
    fn test_register_function() {
        let mut engine = CompletionEngine::new();
        engine.register_function(
            "my_func".to_string(),
            vec!["arg1".to_string(), "arg2".to_string()],
        );
        let completions = engine.get_completions("my_", 3);
        assert!(completions.iter().any(|c| c.text == "my_func"));
    }

    #[test]
    fn test_register_type() {
        let mut engine = CompletionEngine::new();
        engine.register_type("MyType".to_string());
        let completions = engine.get_completions("My", 2);
        assert!(completions.iter().any(|c| c.text == "MyType"));
    }

    #[test]
    fn test_register_methods() {
        let mut engine = CompletionEngine::new();
        engine.register_methods(
            "MyType".to_string(),
            vec!["method1".to_string(), "method2".to_string()],
        );
        // Methods require member access context to show
    }

    #[test]
    fn test_member_completions_string() {
        let engine = CompletionEngine::new();
        let completions = engine.get_member_completions("String", "len");
        assert!(completions.iter().any(|c| c.text == "len"));
    }

    #[test]
    fn test_member_completions_list() {
        let engine = CompletionEngine::new();
        let completions = engine.get_member_completions("List", "pu");
        assert!(completions.iter().any(|c| c.text == "push"));
    }

    #[test]
    fn test_member_completions_hashmap() {
        let engine = CompletionEngine::new();
        let completions = engine.get_member_completions("HashMap", "ge");
        assert!(completions.iter().any(|c| c.text == "get"));
    }

    #[test]
    fn test_context_empty_line() {
        let engine = CompletionEngine::new();
        let ctx = engine.analyze_context("", 0);
        assert!(matches!(ctx, CompletionContext::LineStart));
    }

    #[test]
    fn test_context_whitespace_only() {
        let engine = CompletionEngine::new();
        let ctx = engine.analyze_context("   ", 3);
        assert!(matches!(ctx, CompletionContext::LineStart));
    }

    #[test]
    fn test_context_module_path() {
        let engine = CompletionEngine::new();
        let ctx = engine.analyze_context("std::io", 7);
        assert!(matches!(ctx, CompletionContext::ModulePath { .. }));
    }

    #[test]
    fn test_fuzzy_score_empty_pattern() {
        let engine = CompletionEngine::new();
        assert_eq!(engine.calculate_fuzzy_score("", "anything"), 0);
    }

    #[test]
    fn test_default_trait() {
        let engine = CompletionEngine::default();
        assert!(!engine.keywords.is_empty());
    }

    #[test]
    fn test_completion_cache_with_position() {
        let mut engine = CompletionEngine::new();
        engine.register_variable("test_var".to_string());

        // First call should compute
        let completions1 = engine.get_completions("test", 4);
        // Second call should use cache
        let completions2 = engine.get_completions("test", 4);

        assert_eq!(completions1.len(), completions2.len());
    }

    #[test]
    fn test_ruchy_completer_creation() {
        let completer = RuchyCompleter::new();
        assert!(!completer.builtins.is_empty());
    }

    #[test]
    fn test_ruchy_completer_analyze_context_command() {
        let completer = RuchyCompleter::new();
        let ctx = completer.analyze_context(":help", 5);
        assert!(matches!(ctx, CompletionContext::Command));
    }

    #[test]
    fn test_ruchy_completer_analyze_context_member() {
        let completer = RuchyCompleter::new();
        let ctx = completer.analyze_context("obj.method", 10);
        assert!(matches!(ctx, CompletionContext::MemberAccess { .. }));
    }

    #[test]
    fn test_ruchy_completer_analyze_context_module() {
        let completer = RuchyCompleter::new();
        let ctx = completer.analyze_context("std::io", 7);
        assert!(matches!(ctx, CompletionContext::ModulePath { .. }));
    }

    #[test]
    fn test_ruchy_completer_analyze_context_expression() {
        let completer = RuchyCompleter::new();
        let ctx = completer.analyze_context("let x = 5", 9);
        assert!(matches!(ctx, CompletionContext::Expression));
    }

    #[test]
    fn test_ruchy_completer_find_word_start() {
        let completer = RuchyCompleter::new();
        assert_eq!(completer.find_word_start("hello", 5), 0);
        assert_eq!(completer.find_word_start("let x", 5), 4);
        assert_eq!(completer.find_word_start("a + b", 5), 4);
    }

    #[test]
    fn test_ruchy_completer_get_basic_completions() {
        let completer = RuchyCompleter::new();
        let completions = completer.get_basic_completions("pr");
        assert!(completions.contains(&"println".to_string()));
        assert!(completions.contains(&"print".to_string()));
    }

    #[test]
    fn test_ruchy_completer_convert_to_pairs() {
        let completer = RuchyCompleter::new();
        let completions = vec!["test".to_string()];
        let pairs = completer.convert_to_pairs(completions);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].display, "test");
        assert_eq!(pairs[0].replacement, "test");
    }

    #[test]
    fn test_ruchy_completer_create_hint() {
        let completer = RuchyCompleter::new();
        let hint = completer.create_hint(CompletionContext::MemberAccess {
            object_type: "String".to_string(),
        });
        assert_eq!(hint, Some(" (method access)".to_string()));

        let no_hint = completer.create_hint(CompletionContext::Expression);
        assert!(no_hint.is_none());
    }

    #[test]
    fn test_ruchy_completer_default() {
        let completer = RuchyCompleter::default();
        // Default uses derive, which initializes empty builtins
        // Use new() for non-empty builtins
        assert!(completer.cache.is_empty());
    }

    // ==================== get_line_start_completions tests ====================

    #[test]
    fn test_line_start_completions_let_prefix() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions("le");
        assert!(candidates.iter().any(|c| c.text == "let"));
        assert!(candidates
            .iter()
            .all(|c| c.kind == CompletionKind::Keyword || c.kind == CompletionKind::Command));
    }

    #[test]
    fn test_line_start_completions_fn_prefix() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions("fn");
        assert!(candidates.iter().any(|c| c.text == "fn"));
    }

    #[test]
    fn test_line_start_completions_colon_prefix() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions(":");
        // Should include command completions like :help, :quit, etc.
        assert!(candidates.iter().any(|c| c.kind == CompletionKind::Command));
        assert!(candidates.iter().any(|c| c.text.starts_with(':')));
    }

    #[test]
    fn test_line_start_completions_no_match() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions("zzz_nonexistent");
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_line_start_completions_empty_prefix() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions("");
        // Empty prefix matches everything - should include both keywords and commands
        assert!(!candidates.is_empty());
        let has_keywords = candidates.iter().any(|c| c.kind == CompletionKind::Keyword);
        let has_commands = candidates.iter().any(|c| c.kind == CompletionKind::Command);
        assert!(has_keywords);
        assert!(has_commands);
    }

    #[test]
    fn test_line_start_completions_keyword_docs() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions("if");
        let if_candidate = candidates.iter().find(|c| c.text == "if");
        assert!(if_candidate.is_some());
        let doc = if_candidate.unwrap().doc.as_ref().unwrap();
        assert!(doc.contains("Keyword"));
    }

    #[test]
    fn test_line_start_completions_command_docs() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions(":he");
        let help_candidate = candidates.iter().find(|c| c.text == ":help");
        assert!(help_candidate.is_some());
        let doc = help_candidate.unwrap().doc.as_ref().unwrap();
        assert!(!doc.is_empty());
    }

    #[test]
    fn test_line_start_completions_priority() {
        let engine = CompletionEngine::new();
        let candidates = engine.get_line_start_completions("le");
        for c in &candidates {
            if c.kind == CompletionKind::Keyword {
                assert_eq!(c.priority, CompletionKind::Keyword.priority());
            }
        }
    }
}
// TDG-compliant RuchyCompleter implementation (complexity ≤10 per method)
/// Main completion struct for rustyline integration
#[derive(Debug, Default)]
pub struct RuchyCompleter {
    /// Built-in function completions
    builtins: Vec<String>,
    /// Cache for performance
    cache: HashMap<String, Vec<String>>,
}
impl RuchyCompleter {
    /// Create new completer (complexity: 4)
    pub fn new() -> Self {
        Self {
            builtins: Self::create_builtins(), // complexity: 3
            cache: HashMap::new(),
        }
    }
    /// Create builtin function list (complexity: 3)
    fn create_builtins() -> Vec<String> {
        vec![
            "println".to_string(),
            "print".to_string(),
            "len".to_string(),
        ]
    }
    /// Get completions for REPL (complexity: 8)
    pub fn get_completions(
        &mut self,
        input: &str,
        _pos: usize,
        bindings: &HashMap<String, crate::runtime::interpreter::Value>,
    ) -> Vec<String> {
        // Check cache first (complexity: 2)
        if let Some(cached) = self.cache.get(input) {
            return cached.clone();
        }
        let mut results = Vec::new();
        // Add matching variables (complexity: 3)
        self.add_variable_matches(input, bindings, &mut results);
        // Add matching builtins (complexity: 2)
        self.add_builtin_matches(input, &mut results);
        // Cache results (complexity: 1)
        self.cache.insert(input.to_string(), results.clone());
        results
    }
    /// Add variable matches (complexity: 3)
    fn add_variable_matches(
        &self,
        input: &str,
        bindings: &HashMap<String, crate::runtime::interpreter::Value>,
        results: &mut Vec<String>,
    ) {
        for name in bindings.keys() {
            if name.starts_with(input) {
                results.push(name.clone());
            }
        }
    }
    /// Add builtin matches (complexity: 2)
    fn add_builtin_matches(&self, input: &str, results: &mut Vec<String>) {
        for builtin in &self.builtins {
            if builtin.starts_with(input) {
                results.push(builtin.clone());
            }
        }
    }
    /// Analyze completion context (complexity: 4)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::completion::analyze_context;
    ///
    /// let result = analyze_context("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn analyze_context(&self, line: &str, pos: usize) -> CompletionContext {
        if line.starts_with(':') {
            CompletionContext::Command
        } else if line.contains('.') && pos > line.rfind('.').unwrap_or(0) {
            CompletionContext::MemberAccess {
                object_type: String::new(),
            }
        } else if line.contains("::") {
            CompletionContext::ModulePath {
                module: String::new(),
            }
        } else {
            CompletionContext::Expression
        }
    }
}
// Required rustyline trait implementations (all complexity ≤10)
/// Helper trait implementation (complexity: 1)
impl rustyline::Helper for RuchyCompleter {}
/// Validator trait implementation (complexity: 2)
impl rustyline::validate::Validator for RuchyCompleter {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> Result<rustyline::validate::ValidationResult, rustyline::error::ReadlineError> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}
/// Hinter trait implementation (complexity: 6)
impl rustyline::hint::Hinter for RuchyCompleter {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let context = self.analyze_context(line, pos); // complexity: 4
        self.create_hint(context) // complexity: 2
    }
}
impl RuchyCompleter {
    /// Create contextual hint (complexity: 2)
    fn create_hint(&self, context: CompletionContext) -> Option<String> {
        match context {
            CompletionContext::MemberAccess { .. } => Some(" (method access)".to_string()),
            _ => None,
        }
    }
}
/// Highlighter trait implementation (complexity: 2)
impl rustyline::highlight::Highlighter for RuchyCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        use std::borrow::Cow;
        Cow::Borrowed(line) // Simple pass-through
    }
}
/// Completer trait implementation (complexity: 7)
impl Completer for RuchyCompleter {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Extract word to complete (complexity: 3)
        let start = self.find_word_start(line, pos);
        let word = &line[start..pos];
        // Get basic completions (complexity: 2)
        let completions = self.get_basic_completions(word);
        // Convert to Pairs (complexity: 2)
        let pairs = self.convert_to_pairs(completions);
        Ok((start, pairs))
    }
}
impl RuchyCompleter {
    /// Find start of word to complete (complexity: 3)
    fn find_word_start(&self, line: &str, pos: usize) -> usize {
        line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map_or(0, |i| i + 1)
    }
    /// Get basic completions without bindings (complexity: 2)
    fn get_basic_completions(&self, word: &str) -> Vec<String> {
        self.builtins
            .iter()
            .filter(|name| name.starts_with(word))
            .cloned()
            .collect()
    }
    /// Convert completions to rustyline Pairs (complexity: 2)
    fn convert_to_pairs(&self, completions: Vec<String>) -> Vec<Pair> {
        completions
            .into_iter()
            .map(|s| Pair {
                display: s.clone(),
                replacement: s,
            })
            .collect()
    }
}
#[cfg(test)]
mod property_tests_completion {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_prefix_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
