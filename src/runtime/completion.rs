use std::collections::HashMap;
use std::time::{Duration, Instant};
use rustyline::completion::{Completer, Pair};
use rustyline::Context;

// Simplified type representation for completion
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleType {
    String,
    List,
    DataFrame,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum CompletionContext {
    MethodAccess {
        receiver_type: SimpleType,
        receiver_expr: String,
        partial_method: String,
    },
    ModulePath {
        segments: Vec<String>,
        partial_segment: String,
    },
    FreeExpression {
        scope_id: usize,
        partial_ident: String,
    },
    FunctionCall {
        function_name: String,
        current_param: usize,
    },
    HelpQuery {
        query: String,
    },
}

#[derive(Debug, Clone)]
pub struct MethodCompletion {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub return_type: String,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct SymbolCompletion {
    pub name: String,
    pub symbol_type: SimpleType,
    pub kind: String,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct ModuleCompletion {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct Documentation {
    pub signature: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub examples: Vec<String>,
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub default: Option<String>,
}

pub struct CompletionCache {
    type_methods: HashMap<String, Vec<MethodCompletion>>,
    scope_symbols: HashMap<usize, Vec<SymbolCompletion>>,
    module_contents: HashMap<String, Vec<ModuleCompletion>>,
    
    hit_count: u64,
    miss_count: u64,
    avg_lookup_time: Duration,
    last_access: HashMap<String, Instant>,
}

impl CompletionCache {
    pub fn new() -> Self {
        let mut cache = Self {
            type_methods: HashMap::new(),
            scope_symbols: HashMap::new(),
            module_contents: HashMap::new(),
            hit_count: 0,
            miss_count: 0,
            avg_lookup_time: Duration::from_micros(0),
            last_access: HashMap::new(),
        };
        cache.warmup_common_types();
        cache
    }

    fn warmup_common_types(&mut self) {
        self.type_methods.insert("List".to_string(), vec![
            MethodCompletion {
                name: "map".to_string(),
                signature: "map(f: T -> U) -> List<U>".to_string(),
                description: "Transform each element with function f".to_string(),
                return_type: "List<U>".to_string(),
                priority: 10,
            },
            MethodCompletion {
                name: "filter".to_string(),
                signature: "filter(f: T -> Bool) -> List<T>".to_string(),
                description: "Keep elements where f returns true".to_string(),
                return_type: "List<T>".to_string(),
                priority: 10,
            },
            MethodCompletion {
                name: "sum".to_string(),
                signature: "sum() -> T".to_string(),
                description: "Sum all elements".to_string(),
                return_type: "T".to_string(),
                priority: 9,
            },
            MethodCompletion {
                name: "len".to_string(),
                signature: "len() -> Int".to_string(),
                description: "Number of elements".to_string(),
                return_type: "Int".to_string(),
                priority: 10,
            },
            MethodCompletion {
                name: "head".to_string(),
                signature: "head() -> Option<T>".to_string(),
                description: "First element".to_string(),
                return_type: "Option<T>".to_string(),
                priority: 8,
            },
            MethodCompletion {
                name: "tail".to_string(),
                signature: "tail() -> List<T>".to_string(),
                description: "All elements except first".to_string(),
                return_type: "List<T>".to_string(),
                priority: 8,
            },
            MethodCompletion {
                name: "reverse".to_string(),
                signature: "reverse() -> List<T>".to_string(),
                description: "Reverse order of elements".to_string(),
                return_type: "List<T>".to_string(),
                priority: 7,
            },
        ]);

        self.type_methods.insert("String".to_string(), vec![
            MethodCompletion {
                name: "len".to_string(),
                signature: "len() -> Int".to_string(),
                description: "Length of string".to_string(),
                return_type: "Int".to_string(),
                priority: 10,
            },
            MethodCompletion {
                name: "upper".to_string(),
                signature: "upper() -> String".to_string(),
                description: "Convert to uppercase".to_string(),
                return_type: "String".to_string(),
                priority: 9,
            },
            MethodCompletion {
                name: "lower".to_string(),
                signature: "lower() -> String".to_string(),
                description: "Convert to lowercase".to_string(),
                return_type: "String".to_string(),
                priority: 9,
            },
            MethodCompletion {
                name: "trim".to_string(),
                signature: "trim() -> String".to_string(),
                description: "Remove leading/trailing whitespace".to_string(),
                return_type: "String".to_string(),
                priority: 9,
            },
            MethodCompletion {
                name: "split".to_string(),
                signature: "split(separator: String) -> List<String>".to_string(),
                description: "Split string by separator".to_string(),
                return_type: "List<String>".to_string(),
                priority: 8,
            },
            MethodCompletion {
                name: "starts_with".to_string(),
                signature: "starts_with(prefix: String) -> Bool".to_string(),
                description: "Check if string starts with prefix".to_string(),
                return_type: "Bool".to_string(),
                priority: 7,
            },
            MethodCompletion {
                name: "ends_with".to_string(),
                signature: "ends_with(suffix: String) -> Bool".to_string(),
                description: "Check if string ends with suffix".to_string(),
                return_type: "Bool".to_string(),
                priority: 7,
            },
        ]);

        self.type_methods.insert("DataFrame".to_string(), vec![
            MethodCompletion {
                name: "head".to_string(),
                signature: "head(n: Int = 5) -> DataFrame".to_string(),
                description: "Show first n rows".to_string(),
                return_type: "DataFrame".to_string(),
                priority: 10,
            },
            MethodCompletion {
                name: "select".to_string(),
                signature: "select(columns: List<String>) -> DataFrame".to_string(),
                description: "Select specific columns".to_string(),
                return_type: "DataFrame".to_string(),
                priority: 9,
            },
            MethodCompletion {
                name: "filter".to_string(),
                signature: "filter(condition: Row -> Bool) -> DataFrame".to_string(),
                description: "Filter rows by condition".to_string(),
                return_type: "DataFrame".to_string(),
                priority: 9,
            },
            MethodCompletion {
                name: "sort".to_string(),
                signature: "sort(column: String, ascending: Bool = true) -> DataFrame".to_string(),
                description: "Sort by column".to_string(),
                return_type: "DataFrame".to_string(),
                priority: 8,
            },
            MethodCompletion {
                name: "group_by".to_string(),
                signature: "group_by(columns: List<String>) -> GroupedDataFrame".to_string(),
                description: "Group by columns".to_string(),
                return_type: "GroupedDataFrame".to_string(),
                priority: 8,
            },
            MethodCompletion {
                name: "describe".to_string(),
                signature: "describe() -> DataFrame".to_string(),
                description: "Show summary statistics".to_string(),
                return_type: "DataFrame".to_string(),
                priority: 7,
            },
        ]);
    }

    pub fn get_type_methods(&mut self, type_name: &str) -> Vec<MethodCompletion> {
        let start = Instant::now();
        let result = self.type_methods.get(type_name).cloned().unwrap_or_default();
        
        if self.type_methods.contains_key(type_name) {
            self.hit_count += 1;
        } else {
            self.miss_count += 1;
        }
        
        let elapsed = start.elapsed();
        self.update_avg_lookup_time(elapsed);
        self.last_access.insert(type_name.to_string(), Instant::now());
        
        result
    }

    fn update_avg_lookup_time(&mut self, new_time: Duration) {
        let total_lookups = self.hit_count + self.miss_count;
        if total_lookups == 0 {
            self.avg_lookup_time = new_time;
        } else {
            let old_total = self.avg_lookup_time.as_nanos() * (total_lookups - 1) as u128;
            let new_total = old_total + new_time.as_nanos();
            self.avg_lookup_time = Duration::from_nanos((new_total / total_lookups as u128) as u64);
        }
    }

    pub fn check_performance(&self) {
        let total_lookups = self.hit_count + self.miss_count;
        if total_lookups > 0 {
            let hit_rate = self.hit_count as f64 / total_lookups as f64;
            if hit_rate < 0.7 {
                eprintln!("⚠️  Low completion cache hit rate: {:.1}%", hit_rate * 100.0);
                eprintln!("   Consider increasing cache size or warmup coverage");
            }
        }
        
        if self.avg_lookup_time > Duration::from_millis(50) {
            eprintln!("⚠️  High completion lookup time: {:?}", self.avg_lookup_time);
            eprintln!("   Consider optimizing cache structure");
        }
    }
}

pub struct HelpSystem {
    builtin_docs: HashMap<String, Documentation>,
    method_docs: HashMap<(String, String), Documentation>,
    module_docs: HashMap<String, Documentation>,
}

impl HelpSystem {
    pub fn new() -> Self {
        let mut system = Self {
            builtin_docs: HashMap::new(),
            method_docs: HashMap::new(),
            module_docs: HashMap::new(),
        };
        system.init_builtin_docs();
        system
    }

    fn init_builtin_docs(&mut self) {
        self.builtin_docs.insert("println".to_string(), Documentation {
            signature: "println(value: T) -> ()".to_string(),
            description: "Print a value to stdout followed by a newline.".to_string(),
            parameters: vec![
                Parameter {
                    name: "value".to_string(),
                    param_type: "T".to_string(),
                    description: "The value to print".to_string(),
                    default: None,
                },
            ],
            return_type: Some("()".to_string()),
            examples: vec![
                "println(\"Hello, world!\")".to_string(),
                "println(42)".to_string(),
                "println([1, 2, 3])".to_string(),
            ],
            see_also: vec!["print".to_string()],
        });

        self.builtin_docs.insert("type".to_string(), Documentation {
            signature: "type(object: T) -> String".to_string(),
            description: "Return the type of an object.".to_string(),
            parameters: vec![
                Parameter {
                    name: "object".to_string(),
                    param_type: "T".to_string(),
                    description: "The object to inspect".to_string(),
                    default: None,
                },
            ],
            return_type: Some("String".to_string()),
            examples: vec![
                "type(\"hello\")  // \"String\"".to_string(),
                "type(42)       // \"Int\"".to_string(),
                "type([1,2,3])  // \"List\"".to_string(),
            ],
            see_also: vec!["dir".to_string(), "help".to_string()],
        });
    }

    pub fn get_help(&self, query: &str) -> Option<&Documentation> {
        self.builtin_docs.get(query)
    }
}

pub struct RuchyCompleter {
    help_system: HelpSystem,
    completion_cache: CompletionCache,
    current_scope: usize,
}

impl RuchyCompleter {
    pub fn new() -> Self {
        Self {
            help_system: HelpSystem::new(),
            completion_cache: CompletionCache::new(),
            current_scope: 0,
        }
    }

    // Backward compatibility method for REPL
    pub fn get_completions(&mut self, line: &str, pos: usize, _bindings: &HashMap<String, crate::runtime::repl::Value>) -> Vec<String> {
        let context = self.analyze_context(line, pos);
        let pairs = self.complete_context(context);
        pairs.into_iter().map(|p| p.replacement).collect()
    }

    pub fn analyze_context(&self, line: &str, pos: usize) -> CompletionContext {
        let before_cursor = &line[..pos];
        
        if before_cursor.ends_with('.') {
            if let Some(dot_pos) = before_cursor.rfind('.') {
                let receiver_expr = &before_cursor[..dot_pos];
                return CompletionContext::MethodAccess {
                    receiver_type: self.infer_receiver_type_tolerant(receiver_expr),
                    receiver_expr: receiver_expr.to_string(),
                    partial_method: String::new(),
                };
            }
        }
        
        if before_cursor.contains("::") {
            let segments: Vec<&str> = before_cursor.split("::").collect();
            let partial = segments.last().unwrap_or(&"").to_string();
            let complete_segments: Vec<String> = segments[..segments.len()-1]
                .iter()
                .map(|s| s.to_string())
                .collect();
            return CompletionContext::ModulePath {
                segments: complete_segments,
                partial_segment: partial,
            };
        }
        
        if before_cursor.starts_with("help(") || before_cursor.starts_with("?") {
            let query = before_cursor
                .trim_start_matches("help(")
                .trim_start_matches("?")
                .trim_end_matches(")");
            return CompletionContext::HelpQuery {
                query: query.to_string(),
            };
        }
        
        if let Some(paren_pos) = before_cursor.rfind('(') {
            let before_paren = &before_cursor[..paren_pos];
            if let Some(func_name) = before_paren.split_whitespace().last() {
                let param_count = before_cursor[paren_pos+1..].matches(',').count();
                return CompletionContext::FunctionCall {
                    function_name: func_name.to_string(),
                    current_param: param_count,
                };
            }
        }
        
        let partial = before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_string();
        
        CompletionContext::FreeExpression {
            scope_id: self.current_scope,
            partial_ident: partial,
        }
    }

    fn infer_receiver_type_tolerant(&self, expr: &str) -> SimpleType {
        if expr.starts_with('[') && expr.contains(']') {
            return SimpleType::List;
        }
        if expr.starts_with('"') || expr.starts_with('\'') {
            return SimpleType::String;
        }
        if expr.contains("DataFrame") || expr.contains("df") {
            return SimpleType::DataFrame;
        }
        
        SimpleType::Unknown
    }

    pub fn complete_context(&mut self, context: CompletionContext) -> Vec<Pair> {
        match context {
            CompletionContext::MethodAccess { receiver_type, .. } => {
                self.complete_methods(receiver_type)
            }
            CompletionContext::ModulePath { segments, partial_segment } => {
                self.complete_module_path(segments, partial_segment)
            }
            CompletionContext::FreeExpression { partial_ident, .. } => {
                self.complete_free_expression(partial_ident)
            }
            CompletionContext::FunctionCall { function_name, current_param } => {
                self.complete_function_params(function_name, current_param)
            }
            CompletionContext::HelpQuery { query } => {
                self.complete_help_query(query)
            }
        }
    }

    fn complete_methods(&mut self, receiver_type: SimpleType) -> Vec<Pair> {
        let type_name = match receiver_type {
            SimpleType::String => "String",
            SimpleType::List => "List",
            SimpleType::DataFrame => "DataFrame",
            _ => return Vec::new(),
        };

        let methods = self.completion_cache.get_type_methods(type_name);
        methods.iter()
            .map(|m| Pair {
                display: format!("{} - {}", m.name, m.description),
                replacement: m.name.clone(),
            })
            .collect()
    }

    fn complete_module_path(&self, _segments: Vec<String>, partial: String) -> Vec<Pair> {
        let std_modules = vec![
            ("collections", "Data structures"),
            ("fs", "File system operations"),
            ("io", "Input/output operations"),
            ("net", "Network operations"),
            ("process", "Process management"),
            ("thread", "Threading utilities"),
        ];

        std_modules.iter()
            .filter(|(name, _)| name.starts_with(&partial))
            .map(|(name, desc)| Pair {
                display: format!("{} - {}", name, desc),
                replacement: name.to_string(),
            })
            .collect()
    }

    fn complete_free_expression(&self, partial: String) -> Vec<Pair> {
        let builtins = vec![
            ("println", "Print with newline"),
            ("print", "Print without newline"),
            ("type", "Get type of object"),
            ("dir", "List object attributes"),
            ("help", "Get help on object"),
            ("len", "Get length"),
            ("true", "Boolean true"),
            ("false", "Boolean false"),
            ("None", "None value"),
        ];

        builtins.iter()
            .filter(|(name, _)| name.starts_with(&partial))
            .map(|(name, desc)| Pair {
                display: format!("{} - {}", name, desc),
                replacement: name.to_string(),
            })
            .collect()
    }

    fn complete_function_params(&self, _function_name: String, _current_param: usize) -> Vec<Pair> {
        Vec::new()
    }

    fn complete_help_query(&self, partial: String) -> Vec<Pair> {
        let topics = vec![
            "println", "type", "dir", "help",
            "List", "String", "DataFrame",
        ];

        topics.iter()
            .filter(|topic| topic.starts_with(&partial))
            .map(|topic| Pair {
                display: topic.to_string(),
                replacement: topic.to_string(),
            })
            .collect()
    }

    fn calculate_completion_score(&self, candidate: &str, query: &str) -> f64 {
        let mut score = 0.0;
        
        if candidate.starts_with(query) {
            score += 100.0;
            if candidate.len() == query.len() { 
                score += 50.0;
            }
            score -= (candidate.len() - query.len()) as f64 * 0.1;
        } else {
            let edit_distance = self.levenshtein_distance(candidate, query);
            let max_len = candidate.len().max(query.len());
            if max_len > 0 && edit_distance <= max_len / 2 {
                score += 50.0 * (1.0 - edit_distance as f64 / max_len as f64);
            }
        }
        
        if self.matches_word_boundary(candidate, query) { 
            score += 20.0;
        }
        
        score
    }

    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,
                        matrix[i + 1][j] + 1,
                    ),
                    matrix[i][j] + cost,
                );
            }
        }

        matrix[len1][len2]
    }

    fn matches_word_boundary(&self, candidate: &str, query: &str) -> bool {
        let mut query_chars = query.chars();
        let mut current_query = query_chars.next();
        
        for ch in candidate.chars() {
            if ch.is_uppercase() || (candidate.starts_with(ch) && ch.is_lowercase()) {
                if let Some(q) = current_query {
                    if ch.to_lowercase().to_string() == q.to_lowercase().to_string() {
                        current_query = query_chars.next();
                    }
                }
            }
        }
        
        current_query.is_none()
    }
}

impl Completer for RuchyCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let context = self.analyze_context(line, pos);
        let mut completer = RuchyCompleter::new();
        let completions = completer.complete_context(context);
        
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        Ok((start, completions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_analysis_method_access() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("[1,2,3].", 8);
        matches!(context, CompletionContext::MethodAccess { .. });
    }

    #[test]
    fn test_context_analysis_module_path() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("std::fs::", 9);
        matches!(context, CompletionContext::ModulePath { .. });
    }

    #[test]
    fn test_context_analysis_help_query() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("help(print", 10);
        matches!(context, CompletionContext::HelpQuery { .. });
    }

    #[test]
    fn test_error_tolerant_type_inference() {
        let completer = RuchyCompleter::new();
        
        let list_type = completer.infer_receiver_type_tolerant("[1,2,3]");
        matches!(list_type, Type::List(_));
        
        let string_type = completer.infer_receiver_type_tolerant("\"hello\"");
        assert_eq!(string_type, Type::String);
        
        let df_type = completer.infer_receiver_type_tolerant("DataFrame::new()");
        assert_eq!(df_type, Type::DataFrame);
    }

    #[test]
    fn test_completion_scoring() {
        let completer = RuchyCompleter::new();
        
        let exact_score = completer.calculate_completion_score("println", "println");
        assert!(exact_score > 100.0);
        
        let prefix_score = completer.calculate_completion_score("println", "print");
        assert!(prefix_score > 50.0);
        
        let fuzzy_score = completer.calculate_completion_score("println", "prnt");
        assert!(fuzzy_score > 0.0);
    }

    #[test]
    fn test_cache_performance_monitoring() {
        let mut cache = CompletionCache::new();
        
        cache.get_type_methods("String");
        cache.get_type_methods("String");
        cache.get_type_methods("Unknown");
        
        assert_eq!(cache.hit_count, 2);
        assert_eq!(cache.miss_count, 1);
        
        cache.check_performance();
    }

    #[test]
    fn test_word_boundary_matching() {
        let completer = RuchyCompleter::new();
        
        assert!(completer.matches_word_boundary("HashMap", "HM"));
        assert!(completer.matches_word_boundary("HashMap", "hm"));
        assert!(completer.matches_word_boundary("read_to_string", "rts"));
        assert!(!completer.matches_word_boundary("HashMap", "XY"));
    }
}