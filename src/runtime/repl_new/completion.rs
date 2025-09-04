//! Tab completion module for REPL
//! Handles intelligent code completion with low complexity

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

impl CompletionEngine {
    /// Create new completion engine (complexity: 3)
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
                    return CompletionContext::MemberAccess { object_type: obj_type };
                }
            }
        }
        
        // Check for module path
        if let Some(colon_pos) = prefix.rfind("::") {
            let module = &prefix[..colon_pos];
            return CompletionContext::ModulePath { 
                module: module.to_string() 
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
                    doc: Some(format!("Keyword: {}", keyword)),
                    priority: CompletionKind::Keyword.priority(),
                });
            }
        }
        
        // Add commands
        for command in &self.commands {
            let cmd_with_colon = format!(":{}", command);
            if cmd_with_colon.starts_with(prefix) {
                candidates.push(CompletionCandidate {
                    text: cmd_with_colon,
                    display: format!(":{} - REPL command", command),
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
                        display: format!("{}()", method),
                        kind: CompletionKind::Method,
                        doc: Some(format!("Method on {}", object_type)),
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
        for (name, _) in &self.functions {
            if name.starts_with(item_prefix) {
                candidates.push(CompletionCandidate {
                    text: name.clone(),
                    display: format!("{}::{}", module, name),
                    kind: CompletionKind::Function,
                    doc: Some(format!("Function in {}", module)),
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
                    text: format!(":{}", command),
                    display: format!(":{}", command),
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
                    priority: CompletionKind::Variable.priority() + 
                             self.calculate_fuzzy_score(&word, var),
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
                    priority: CompletionKind::Function.priority() +
                             self.calculate_fuzzy_score(&word, func),
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
                    priority: CompletionKind::Type.priority() +
                             self.calculate_fuzzy_score(&word, typ),
                });
            }
        }
        
        candidates
    }

    /// Register a variable (complexity: 2)
    pub fn register_variable(&mut self, name: String) {
        self.variables.insert(name);
        self.cache.clear(); // Invalidate cache
    }

    /// Register a function (complexity: 2)
    pub fn register_function(&mut self, name: String, params: Vec<String>) {
        self.functions.insert(name, params);
        self.cache.clear();
    }

    /// Register a type (complexity: 2)
    pub fn register_type(&mut self, name: String) {
        self.types.insert(name);
        self.cache.clear();
    }

    /// Register methods for a type (complexity: 3)
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
            _ => format!("Command: {}", command),
        }
    }

    /// Default keywords (complexity: 1)
    fn default_keywords() -> Vec<String> {
        vec![
            "let", "mut", "const", "fn", "if", "else", "match", "for", "while",
            "loop", "break", "continue", "return", "struct", "enum", "trait",
            "impl", "pub", "mod", "use", "async", "await", "type", "where",
        ].into_iter().map(String::from).collect()
    }

    /// Default commands (complexity: 1)
    fn default_commands() -> Vec<String> {
        vec![
            "help", "quit", "exit", "history", "clear", "reset", "bindings",
            "env", "vars", "functions", "compile", "transpile", "load", "save",
            "export", "type", "ast", "parse", "mode", "debug", "time", "inspect",
            "doc", "ls", "state",
        ].into_iter().map(String::from).collect()
    }

    /// Default methods by type (complexity: 2)
    fn default_methods() -> HashMap<String, Vec<String>> {
        let mut methods = HashMap::new();
        
        methods.insert("String".to_string(), vec![
            "len", "is_empty", "chars", "bytes", "lines", "split", "trim",
            "to_uppercase", "to_lowercase", "replace", "contains", "starts_with",
            "ends_with", "parse", "repeat",
        ].into_iter().map(String::from).collect());
        
        methods.insert("List".to_string(), vec![
            "len", "is_empty", "push", "pop", "first", "last", "get", "sort",
            "reverse", "contains", "iter", "map", "filter", "fold", "find",
        ].into_iter().map(String::from).collect());
        
        methods.insert("HashMap".to_string(), vec![
            "len", "is_empty", "insert", "remove", "get", "contains_key",
            "keys", "values", "iter", "clear",
        ].into_iter().map(String::from).collect());
        
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
        let completions = engine.get_completions("le", 2);
        
        assert!(completions.iter().any(|c| c.text == "let"));
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
        
        let ctx = engine.analyze_context("str.", 4);
        assert!(matches!(ctx, CompletionContext::MemberAccess { .. }));
        
        let ctx = engine.analyze_context("std::", 5);
        assert!(matches!(ctx, CompletionContext::ModulePath { .. }));
    }
}