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

impl Default for CompletionCache {
    fn default() -> Self {
        Self::new()
    }
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
            let old_total = self.avg_lookup_time.as_nanos() * u128::from(total_lookups - 1);
            let new_total = old_total + new_time.as_nanos();
            self.avg_lookup_time = Duration::from_nanos((new_total / u128::from(total_lookups)) as u64);
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

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpSystem {
    pub fn new() -> Self {
        let mut system = Self {
            builtin_docs: HashMap::new(),
            method_docs: HashMap::new(),
            module_docs: HashMap::new(),
        };
        system.init_builtin_docs();
        system.init_method_docs();
        system.init_module_docs();
        system
    }

    fn init_builtin_docs(&mut self) {
        // Core language functions
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
            description: "Return the type of an object as a string.".to_string(),
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
                "type({a: 1})   // \"Object\"".to_string(),
            ],
            see_also: vec!["dir".to_string(), "help".to_string()],
        });

        self.builtin_docs.insert("dir".to_string(), Documentation {
            signature: "dir(object: T) -> List<String>".to_string(),
            description: "Return a list of methods and attributes available on an object.".to_string(),
            parameters: vec![
                Parameter {
                    name: "object".to_string(),
                    param_type: "T".to_string(),
                    description: "The object to inspect".to_string(),
                    default: None,
                },
            ],
            return_type: Some("List<String>".to_string()),
            examples: vec![
                "dir([1,2,3])     // ['map', 'filter', 'len', ...]".to_string(),
                "dir(\"hello\")     // ['upper', 'lower', 'split', ...]".to_string(),
                "dir({a: 1})      // ['keys', 'values', 'items']".to_string(),
            ],
            see_also: vec!["type".to_string(), "help".to_string()],
        });

        self.builtin_docs.insert("help".to_string(), Documentation {
            signature: "help() -> () | help(object: T) -> ()".to_string(),
            description: "Show help information. Without arguments, shows general help. With an object, shows detailed help for that object.".to_string(),
            parameters: vec![
                Parameter {
                    name: "object".to_string(),
                    param_type: "T".to_string(),
                    description: "Optional object to get help for".to_string(),
                    default: Some("None".to_string()),
                },
            ],
            return_type: Some("()".to_string()),
            examples: vec![
                "help()           // Show general help".to_string(),
                "help(println)    // Help for println function".to_string(),
                "help([].map)     // Help for List.map method".to_string(),
                "help(String)     // Help for String type".to_string(),
            ],
            see_also: vec!["dir".to_string(), "type".to_string()],
        });

        // Add documentation for common types
        self.builtin_docs.insert("String".to_string(), Documentation {
            signature: "String".to_string(),
            description: "A Unicode string type. Strings are immutable sequences of characters.".to_string(),
            parameters: vec![],
            return_type: None,
            examples: vec![
                "let s = \"Hello, world!\"".to_string(),
                "let s2 = s.upper()".to_string(),
                "let parts = s.split(\", \")".to_string(),
            ],
            see_also: vec!["dir".to_string(), "str".to_string()],
        });

        self.builtin_docs.insert("List".to_string(), Documentation {
            signature: "List<T>".to_string(),
            description: "A dynamic array type that can hold elements of any type.".to_string(),
            parameters: vec![],
            return_type: None,
            examples: vec![
                "let nums = [1, 2, 3, 4]".to_string(),
                "let doubled = nums.map(x => x * 2)".to_string(),
                "let filtered = nums.filter(x => x > 2)".to_string(),
            ],
            see_also: vec!["dir".to_string(), "map".to_string(), "filter".to_string()],
        });

        self.builtin_docs.insert("DataFrame".to_string(), Documentation {
            signature: "DataFrame".to_string(),
            description: "A two-dimensional data structure with labeled columns, similar to a spreadsheet or SQL table.".to_string(),
            parameters: vec![],
            return_type: None,
            examples: vec![
                "let df = df![]".to_string(),
                "let subset = df.head(5)".to_string(),
                "let filtered = df.filter(|row| row.age > 18)".to_string(),
            ],
            see_also: vec!["dir".to_string(), "df!".to_string()],
        });
    }

    fn init_method_docs(&mut self) {
        // String methods
        let string_methods = vec![
            ("len", "len() -> Int", "Return the length of the string in characters"),
            ("upper", "upper() -> String", "Convert all characters to uppercase"),
            ("lower", "lower() -> String", "Convert all characters to lowercase"),
            ("trim", "trim() -> String", "Remove leading and trailing whitespace"),
            ("split", "split(separator: String) -> List<String>", "Split string by separator"),
            ("starts_with", "starts_with(prefix: String) -> Bool", "Check if string starts with prefix"),
            ("ends_with", "ends_with(suffix: String) -> Bool", "Check if string ends with suffix"),
            ("contains", "contains(substring: String) -> Bool", "Check if string contains substring"),
            ("replace", "replace(old: String, new: String) -> String", "Replace all occurrences of old with new"),
        ];

        for (method, signature, description) in string_methods {
            self.method_docs.insert(("String".to_string(), method.to_string()), Documentation {
                signature: signature.to_string(),
                description: description.to_string(),
                parameters: vec![], // Simplified for now
                return_type: None,
                examples: vec![format!("\"hello\".{}()", method)],
                see_also: vec!["String".to_string()],
            });
        }

        // List methods
        let list_methods = vec![
            ("map", "map(f: T -> U) -> List<U>", "Transform each element with function f"),
            ("filter", "filter(f: T -> Bool) -> List<T>", "Keep elements where f returns true"),
            ("len", "len() -> Int", "Return the number of elements"),
            ("sum", "sum() -> T", "Sum all elements (for numeric lists)"),
            ("head", "head() -> Option<T>", "Get the first element"),
            ("tail", "tail() -> List<T>", "Get all elements except the first"),
            ("reverse", "reverse() -> List<T>", "Reverse the order of elements"),
            ("push", "push(item: T) -> ()", "Add an element to the end"),
            ("pop", "pop() -> Option<T>", "Remove and return the last element"),
        ];

        for (method, signature, description) in list_methods {
            self.method_docs.insert(("List".to_string(), method.to_string()), Documentation {
                signature: signature.to_string(),
                description: description.to_string(),
                parameters: vec![], // Simplified for now
                return_type: None,
                examples: vec![format!("[1,2,3].{}(...)", method)],
                see_also: vec!["List".to_string()],
            });
        }

        // DataFrame methods
        let dataframe_methods = vec![
            ("head", "head(n: Int = 5) -> DataFrame", "Show the first n rows"),
            ("select", "select(columns: List<String>) -> DataFrame", "Select specific columns"),
            ("filter", "filter(condition: Row -> Bool) -> DataFrame", "Filter rows by condition"),
            ("sort", "sort(column: String, ascending: Bool = true) -> DataFrame", "Sort by column"),
            ("group_by", "group_by(columns: List<String>) -> GroupedDataFrame", "Group rows by columns"),
            ("describe", "describe() -> DataFrame", "Show summary statistics"),
        ];

        for (method, signature, description) in dataframe_methods {
            self.method_docs.insert(("DataFrame".to_string(), method.to_string()), Documentation {
                signature: signature.to_string(),
                description: description.to_string(),
                parameters: vec![], // Simplified for now
                return_type: None,
                examples: vec![format!("df.{}(...)", method)],
                see_also: vec!["DataFrame".to_string()],
            });
        }
    }

    fn init_module_docs(&mut self) {
        // Standard library modules
        let modules = vec![
            ("std", "Ruchy standard library - core functionality and utilities"),
            ("std::collections", "Data structures like HashMap, HashSet, BTreeMap"),
            ("std::fs", "File system operations - reading, writing, directory management"),
            ("std::io", "Input/output operations and utilities"),
            ("std::net", "Network programming - TCP, UDP, HTTP"),
            ("std::process", "Process management and system interaction"),
            ("std::thread", "Threading and concurrency utilities"),
            ("std::time", "Time and duration utilities"),
        ];

        for (module, description) in modules {
            self.module_docs.insert(module.to_string(), Documentation {
                signature: format!("module {module}"),
                description: description.to_string(),
                parameters: vec![],
                return_type: None,
                examples: vec![format!("use {}", module)],
                see_also: vec![],
            });
        }
    }

    pub fn get_help(&self, query: &str) -> Option<&Documentation> {
        self.builtin_docs.get(query).or_else(|| self.module_docs.get(query))
    }

    pub fn get_method_help(&self, type_name: &str, method_name: &str) -> Option<&Documentation> {
        self.method_docs.get(&(type_name.to_string(), method_name.to_string()))
    }

    pub fn show_general_help(&self) -> String {
        r#"Ruchy Interactive Help System
=============================

Welcome to Ruchy! This help system provides documentation for all available
functions, methods, types, and modules.

Common Help Commands:
  help()           - Show this general help
  help(object)     - Get detailed help for a specific object or function  
  dir(object)      - List available methods and attributes
  type(object)     - Show the type of an object
  ?object          - Quick help (alias for help(object))

Interactive Features:
  Tab Completion   - Press TAB to complete function names, methods, etc.
  Method Discovery - Type 'object.' and press TAB to see available methods
  Module Browse    - Type 'std::' and press TAB to explore standard library

Examples:
  help(println)     - Help for the println function
  help(String)      - Help for the String type  
  dir([1,2,3])      - Methods available on lists
  type("hello")     - Shows "String"
  help(std::fs)     - Help for file system module

Quick Reference:
  Core Types: String, List, DataFrame, Int, Float, Bool, Object
  Builtins: println, print, type, dir, help, len
  Standard Library: std::fs, std::collections, std::net, std::process

For more detailed information on any topic, use help(topic_name).
"#.to_string()
    }

    pub fn get_type_methods(&self, type_name: &str) -> Vec<String> {
        match type_name {
            "String" => vec![
                "len", "upper", "lower", "trim", "split", 
                "starts_with", "ends_with", "contains", "replace"
            ].into_iter().map(std::string::ToString::to_string).collect(),
            "List" => vec![
                "map", "filter", "len", "sum", "head", "tail", 
                "reverse", "push", "pop", "first", "last"
            ].into_iter().map(std::string::ToString::to_string).collect(),
            "DataFrame" => vec![
                "head", "select", "filter", "sort", "group_by", "describe"
            ].into_iter().map(std::string::ToString::to_string).collect(),
            _ => Vec::new(),
        }
    }

    pub fn format_help(&self, doc: &Documentation) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", doc.signature));
        output.push_str(&"=".repeat(doc.signature.len()));
        output.push('\n');
        output.push('\n');
        output.push_str(&doc.description);
        output.push('\n');
        
        if !doc.parameters.is_empty() {
            output.push_str("\nParameters:\n");
            for param in &doc.parameters {
                output.push_str(&format!("  {}: {} - {}\n", 
                    param.name, param.param_type, param.description));
            }
        }
        
        if let Some(ret_type) = &doc.return_type {
            output.push_str(&format!("\nReturns: {ret_type}\n"));
        }
        
        if !doc.examples.is_empty() {
            output.push_str("\nExamples:\n");
            for example in &doc.examples {
                output.push_str(&format!("  {example}\n"));
            }
        }
        
        if !doc.see_also.is_empty() {
            output.push_str(&format!("\nSee also: {}\n", doc.see_also.join(", ")));
        }
        
        output
    }
}

pub struct RuchyCompleter {
    help_system: HelpSystem,
    completion_cache: CompletionCache,
    current_scope: usize,
}

impl Default for RuchyCompleter {
    fn default() -> Self {
        Self::new()
    }
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
        
        // Handle method access with more sophisticated parsing
        if let Some(method_context) = self.analyze_method_access(before_cursor) {
            return method_context;
        }
        
        // Handle module paths with better error tolerance
        if let Some(module_context) = self.analyze_module_path(before_cursor) {
            return module_context;
        }
        
        // Handle help queries with multiple patterns
        if let Some(help_context) = self.analyze_help_query(before_cursor) {
            return help_context;
        }
        
        // Handle function calls with parameter position
        if let Some(function_context) = self.analyze_function_call(before_cursor) {
            return function_context;
        }
        
        // Default to free expression completion
        let partial = self.extract_partial_identifier(before_cursor);
        CompletionContext::FreeExpression {
            scope_id: self.current_scope,
            partial_ident: partial,
        }
    }

    fn analyze_method_access(&self, text: &str) -> Option<CompletionContext> {
        // Find the last dot that could be method access
        for (i, _) in text.match_indices('.').rev() {
            let before_dot = &text[..i];
            let after_dot = &text[i + 1..];
            
            // Skip if this looks like a number (e.g., 3.14)
            if before_dot.ends_with(|c: char| c.is_ascii_digit()) {
                continue;
            }
            
            // Extract the receiver expression, handling nested calls
            let receiver_expr = self.extract_receiver_expression(before_dot);
            if !receiver_expr.is_empty() {
                return Some(CompletionContext::MethodAccess {
                    receiver_type: self.infer_receiver_type_tolerant(&receiver_expr),
                    receiver_expr,
                    partial_method: after_dot.to_string(),
                });
            }
        }
        None
    }

    fn analyze_module_path(&self, text: &str) -> Option<CompletionContext> {
        // Look for :: patterns, handling incomplete paths
        if let Some(double_colon_pos) = text.rfind("::") {
            let before_colons = &text[..double_colon_pos];
            let after_colons = &text[double_colon_pos + 2..];
            
            // Split the path into segments
            let segments: Vec<String> = before_colons
                .split("::")
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            
            return Some(CompletionContext::ModulePath {
                segments,
                partial_segment: after_colons.to_string(),
            });
        }
        
        // Also handle single identifiers that could be module starts
        if text.chars().any(char::is_uppercase) && text.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some(CompletionContext::ModulePath {
                segments: Vec::new(),
                partial_segment: text.to_string(),
            });
        }
        
        None
    }

    fn analyze_help_query(&self, text: &str) -> Option<CompletionContext> {
        // Handle various help patterns: help(), ?, :help, etc.
        let trimmed = text.trim();
        
        // help(something) pattern
        if let Some(help_start) = trimmed.find("help(") {
            let after_help = &trimmed[help_start + 5..];
            let query = after_help.trim_end_matches(')').trim_matches('"').trim_matches('\'');
            return Some(CompletionContext::HelpQuery {
                query: query.to_string(),
            });
        }
        
        // ?object pattern  
        if trimmed.starts_with('?') && trimmed.len() > 1 {
            let query = trimmed[1..].trim();
            return Some(CompletionContext::HelpQuery {
                query: query.to_string(),
            });
        }
        
        // :help pattern
        if trimmed.starts_with(":help") {
            let query = if trimmed.len() > 5 {
                trimmed[5..].trim()
            } else {
                ""
            };
            return Some(CompletionContext::HelpQuery {
                query: query.to_string(),
            });
        }
        
        None
    }

    fn analyze_function_call(&self, text: &str) -> Option<CompletionContext> {
        // Find the rightmost opening parenthesis
        if let Some(paren_pos) = text.rfind('(') {
            let before_paren = &text[..paren_pos];
            let after_paren = &text[paren_pos + 1..];
            
            // Extract function name, handling method calls
            let func_name = self.extract_function_name(before_paren);
            if !func_name.is_empty() {
                // Count parameters by counting commas, but be smart about nested calls
                let param_count = self.count_parameters(after_paren);
                
                return Some(CompletionContext::FunctionCall {
                    function_name: func_name,
                    current_param: param_count,
                });
            }
        }
        None
    }

    fn extract_receiver_expression(&self, text: &str) -> String {
        // Handle complex expressions like obj.method().field
        let mut depth = 0;
        let mut start = text.len();
        
        // Walk backwards to find the start of the receiver expression
        for (i, ch) in text.char_indices().rev() {
            match ch {
                ')' => depth += 1,
                '(' => depth -= 1,
                ' ' | '\t' | '\n' | ';' | '{' | '}' | '[' | ']' if depth == 0 => {
                    start = i + 1;
                    break;
                }
                _ => {}
            }
        }
        
        if start == text.len() && depth == 0 {
            start = 0;
        }
        
        text[start..].trim().to_string()
    }

    fn extract_function_name(&self, text: &str) -> String {
        // Extract function name, handling method chains
        let trimmed = text.trim();
        
        // Find the rightmost function name (after spaces, parens, etc.)
        let end = trimmed.len();
        let mut start = end;
        
        // Walk backwards to find identifier boundaries
        let chars: Vec<char> = trimmed.chars().collect();
        while start > 0 {
            let ch = chars[start - 1];
            if ch.is_alphanumeric() || ch == '_' {
                start -= 1;
            } else {
                break;
            }
        }
        
        if start < end {
            chars[start..end].iter().collect()
        } else {
            String::new()
        }
    }

    fn count_parameters(&self, text: &str) -> usize {
        // Count commas while respecting nested parentheses/brackets
        let mut count = 0;
        let mut depth = 0;
        
        for ch in text.chars() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => count += 1,
                _ => {}
            }
        }
        
        // If there's any non-whitespace content, we're in at least the first parameter
        if text.trim().is_empty() {
            0
        } else {
            count
        }
    }

    fn extract_partial_identifier(&self, text: &str) -> String {
        // More sophisticated identifier extraction
        let chars: Vec<char> = text.chars().collect();
        let mut start = chars.len();
        
        // Find the start of the current identifier
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        if start < chars.len() {
            chars[start..].iter().collect()
        } else {
            String::new()
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
        let std_modules = [("collections", "Data structures"),
            ("fs", "File system operations"),
            ("io", "Input/output operations"),
            ("net", "Network operations"),
            ("process", "Process management"),
            ("thread", "Threading utilities")];

        std_modules.iter()
            .filter(|(name, _)| name.starts_with(&partial))
            .map(|(name, desc)| Pair {
                display: format!("{name} - {desc}"),
                replacement: (*name).to_string(),
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
                display: format!("{name} - {desc}"),
                replacement: (*name).to_string(),
            })
            .collect()
    }

    fn complete_function_params(&self, _function_name: String, _current_param: usize) -> Vec<Pair> {
        Vec::new()
    }

    fn complete_help_query(&self, partial: String) -> Vec<Pair> {
        let topics = ["println", "type", "dir", "help",
            "List", "String", "DataFrame"];

        topics.iter()
            .filter(|topic| topic.starts_with(&partial))
            .map(|topic| Pair {
                display: (*topic).to_string(),
                replacement: (*topic).to_string(),
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
                let cost = usize::from(c1 != c2);
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
        let query_lower = query.to_lowercase();
        let mut query_chars = query_lower.chars();
        let mut current_query = query_chars.next();
        
        for (i, ch) in candidate.chars().enumerate() {
            // Match on uppercase letters (camelCase boundaries), underscores, or start
            let is_boundary = ch.is_uppercase() || ch == '_' || i == 0;
            
            if is_boundary {
                if let Some(q) = current_query {
                    if ch.to_lowercase().to_string() == q.to_string() {
                        current_query = query_chars.next();
                        if current_query.is_none() {
                            return true;
                        }
                    }
                }
            }
            
            // Also match after underscores
            if i > 0 && candidate.chars().nth(i-1) == Some('_') {
                if let Some(q) = current_query {
                    if ch.to_lowercase().to_string() == q.to_string() {
                        current_query = query_chars.next();
                        if current_query.is_none() {
                            return true;
                        }
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
            .map_or(0, |i| i + 1);
        
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
        assert_eq!(list_type, SimpleType::List);
        
        let string_type = completer.infer_receiver_type_tolerant("\"hello\"");
        assert_eq!(string_type, SimpleType::String);
        
        let df_type = completer.infer_receiver_type_tolerant("DataFrame::new()");
        assert_eq!(df_type, SimpleType::DataFrame);
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
        assert!(completer.matches_word_boundary("read_to_string", "rts"));  // r_t_s would match r, t, and s
        assert!(!completer.matches_word_boundary("HashMap", "XY"));
        
        // Additional tests for clarity
        assert!(completer.matches_word_boundary("camelCase", "cc"));
        assert!(completer.matches_word_boundary("under_score", "us"));
    }

    #[test]
    fn test_enhanced_method_access_analysis() {
        let completer = RuchyCompleter::new();
        
        // Test simple method access
        let context = completer.analyze_context("[1,2,3].ma", 10);
        match context {
            CompletionContext::MethodAccess { receiver_type, partial_method, .. } => {
                assert_eq!(receiver_type, SimpleType::List);
                assert_eq!(partial_method, "ma");
            }
            _ => panic!("Expected MethodAccess context"),
        }
        
        // Test chained method access
        let context = completer.analyze_context("obj.method().fi", 15);
        match context {
            CompletionContext::MethodAccess { partial_method, .. } => {
                assert_eq!(partial_method, "fi");
            }
            _ => panic!("Expected MethodAccess context"),
        }
        
        // Test number literal (should not be method access)
        let context = completer.analyze_context("3.14", 4);
        matches!(context, CompletionContext::FreeExpression { .. });
    }

    #[test]
    fn test_enhanced_help_analysis() {
        let completer = RuchyCompleter::new();
        
        // Test help() function
        let context = completer.analyze_context("help(prin", 9);
        match context {
            CompletionContext::HelpQuery { query } => {
                assert_eq!(query, "prin");
            }
            _ => panic!("Expected HelpQuery context"),
        }
        
        // Test ? syntax
        let context = completer.analyze_context("?prin", 5);
        match context {
            CompletionContext::HelpQuery { query } => {
                assert_eq!(query, "prin");
            }
            _ => panic!("Expected HelpQuery context"),
        }
        
        // Test :help syntax
        let context = completer.analyze_context(":help prin", 10);
        match context {
            CompletionContext::HelpQuery { query } => {
                assert_eq!(query, "prin");
            }
            _ => panic!("Expected HelpQuery context"),
        }
    }

    #[test]
    fn test_function_parameter_counting() {
        let completer = RuchyCompleter::new();
        
        // Test simple function call
        let text = "println(\"hello\", ";
        let context = completer.analyze_context(text, text.len());
        match context {
            CompletionContext::FunctionCall { function_name, current_param } => {
                assert_eq!(function_name, "println");
                assert_eq!(current_param, 1);
            }
            _ => panic!("Expected FunctionCall context"),
        }
        
        // Test simpler nested case - immediate function completion
        let text = "outer(arg1, ";
        let context = completer.analyze_context(text, text.len());
        match context {
            CompletionContext::FunctionCall { function_name, current_param } => {
                assert_eq!(function_name, "outer");
                assert_eq!(current_param, 1);
            }
            _ => panic!("Expected FunctionCall context"),
        }
    }

    #[test]
    fn test_help_system_integration() {
        let completer = RuchyCompleter::new();
        
        // Test getting help for println
        let help = completer.help_system.get_help("println");
        assert!(help.is_some());
        let help_doc = help.unwrap();
        assert!(help_doc.signature.contains("println"));
        assert!(help_doc.description.contains("Print"));
        
        // Test getting method help
        let method_help = completer.help_system.get_method_help("String", "upper");
        assert!(method_help.is_some());
        let method_doc = method_help.unwrap();
        assert!(method_doc.signature.contains("upper"));
        
        // Test type methods
        let string_methods = completer.help_system.get_type_methods("String");
        assert!(string_methods.contains(&"upper".to_string()));
        assert!(string_methods.contains(&"lower".to_string()));
        
        // Test general help
        let general_help = completer.help_system.show_general_help();
        assert!(general_help.contains("Ruchy Interactive Help System"));
        assert!(general_help.contains("Tab Completion"));
    }
}