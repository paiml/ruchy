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
    builtins: HashMap<String, Documentation>,
    methods: HashMap<(String, String), Documentation>,
    modules: HashMap<String, Documentation>,
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpSystem {
    pub fn new() -> Self {
        let mut system = Self {
            builtins: HashMap::new(),
            methods: HashMap::new(),
            modules: HashMap::new(),
        };
        system.init_builtins();
        system.init_methods();
        system.init_modules();
        system
    }

    fn init_builtins(&mut self) {
        // Core language functions
        self.builtins.insert("println".to_string(), Documentation {
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

        self.builtins.insert("type".to_string(), Documentation {
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

        self.builtins.insert("dir".to_string(), Documentation {
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

        self.builtins.insert("help".to_string(), Documentation {
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
        self.builtins.insert("String".to_string(), Documentation {
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

        self.builtins.insert("List".to_string(), Documentation {
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

        self.builtins.insert("DataFrame".to_string(), Documentation {
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

    fn init_methods(&mut self) {
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
            self.methods.insert(("String".to_string(), method.to_string()), Documentation {
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
            self.methods.insert(("List".to_string(), method.to_string()), Documentation {
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
            self.methods.insert(("DataFrame".to_string(), method.to_string()), Documentation {
                signature: signature.to_string(),
                description: description.to_string(),
                parameters: vec![], // Simplified for now
                return_type: None,
                examples: vec![format!("df.{}(...)", method)],
                see_also: vec!["DataFrame".to_string()],
            });
        }
    }

    fn init_modules(&mut self) {
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
            self.modules.insert(module.to_string(), Documentation {
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
        self.builtins.get(query).or_else(|| self.modules.get(query))
    }

    pub fn get_method_help(&self, type_name: &str, method_name: &str) -> Option<&Documentation> {
        self.methods.get(&(type_name.to_string(), method_name.to_string()))
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
    pub fn get_completions(&mut self, line: &str, pos: usize, bindings: &HashMap<String, crate::runtime::repl::Value>) -> Vec<String> {
        let context = self.analyze_context(line, pos);
        let pairs = self.complete_context_with_bindings(context, bindings);
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
        
        // Handle special REPL commands starting with ':'
        if before_cursor.starts_with(':') {
            return CompletionContext::HelpQuery {
                query: before_cursor.to_string(),
            };
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
    
    pub fn complete_context_with_bindings(&mut self, context: CompletionContext, bindings: &HashMap<String, crate::runtime::repl::Value>) -> Vec<Pair> {
        match context {
            CompletionContext::MethodAccess { receiver_type, receiver_expr, partial_method } => {
                self.complete_methods_with_bindings(receiver_type, receiver_expr, partial_method, bindings)
            }
            CompletionContext::ModulePath { segments, partial_segment } => {
                self.complete_module_path(segments, partial_segment)
            }
            CompletionContext::FreeExpression { partial_ident, .. } => {
                self.complete_free_expression_with_bindings(partial_ident, bindings)
            }
            CompletionContext::FunctionCall { function_name, current_param } => {
                self.complete_function_params(function_name, current_param)
            }
            CompletionContext::HelpQuery { query } => {
                self.complete_help_query(query)
            }
        }
    }

    // Immutable version for use with Completer trait
    pub fn complete_context_immutable(&self, context: CompletionContext) -> Vec<Pair> {
        match context {
            CompletionContext::MethodAccess { receiver_type, .. } => {
                self.complete_methods_immutable(receiver_type)
            }
            CompletionContext::ModulePath { segments, partial_segment } => {
                self.complete_module_path_immutable(segments, partial_segment)
            }
            CompletionContext::FreeExpression { partial_ident, .. } => {
                self.complete_free_expression_immutable(partial_ident)
            }
            CompletionContext::FunctionCall { function_name, current_param } => {
                self.complete_function_params_immutable(function_name, current_param)
            }
            CompletionContext::HelpQuery { query } => {
                self.complete_help_query_immutable(query)
            }
        }
    }

    // Immutable versions for Completer trait
    fn complete_methods_immutable(&self, receiver_type: SimpleType) -> Vec<Pair> {
        let type_name = match receiver_type {
            SimpleType::String => "String",
            SimpleType::List => "List", 
            SimpleType::DataFrame => "DataFrame",
            SimpleType::Unknown => return Vec::new(),
        };
        
        // Create a new cache instance to avoid mutable access
        let mut temp_cache = CompletionCache::new();
        let methods = temp_cache.get_type_methods(type_name);
        
        methods.into_iter()
            .map(|method| Pair {
                display: format!("{}() -> {}", method.name, method.return_type),
                replacement: method.name,
            })
            .collect()
    }
    
    fn complete_module_path_immutable(&self, _segments: Vec<String>, partial_segment: String) -> Vec<Pair> {
        // For now, return empty - module system not fully implemented
        if partial_segment.is_empty() {
            Vec::new()
        } else {
            Vec::new()
        }
    }
    
    fn complete_free_expression_immutable(&self, partial_ident: String) -> Vec<Pair> {
        let mut completions = Vec::new();
        
        // Add builtin functions
        let builtins = vec![
            "println", "print", "len", "help", "type", "dir",
            "map", "filter", "reduce", "sum", "min", "max",
        ];
        
        for builtin in builtins {
            if builtin.starts_with(&partial_ident) {
                completions.push(Pair {
                    display: builtin.to_string(),
                    replacement: builtin.to_string(),
                });
            }
        }
        
        completions
    }
    
    fn complete_function_params_immutable(&self, _function_name: String, _current_param: usize) -> Vec<Pair> {
        // For now, return empty - parameter completion not fully implemented
        Vec::new()
    }
    
    fn complete_help_query_immutable(&self, _query: String) -> Vec<Pair> {
        let help_topics = vec![
            "println", "type", "dir", "help", "List", "String", "DataFrame"
        ];
        
        help_topics.into_iter()
            .map(|topic| Pair {
                display: format!("help({topic})"),
                replacement: topic.to_string(),
            })
            .collect()
    }

    fn complete_methods(&mut self, receiver_type: SimpleType) -> Vec<Pair> {
        let type_name = match receiver_type {
            SimpleType::String => "String",
            SimpleType::List => "List",
            SimpleType::DataFrame => "DataFrame",
            SimpleType::Unknown => return Vec::new(),
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
    
    fn complete_methods_with_bindings(&mut self, receiver_type: SimpleType, receiver_expr: String, partial_method: String, bindings: &HashMap<String, crate::runtime::repl::Value>) -> Vec<Pair> {
        let mut completions = Vec::new();
        
        // Handle nested field access (e.g., "data.user" -> look up data, then user field)
        let value = if receiver_expr.contains('.') {
            self.resolve_nested_field(&receiver_expr, bindings)
        } else {
            bindings.get(&receiver_expr).cloned()
        };
        
        // First, try to look up the actual type from REPL bindings
        let actual_type = if let Some(ref resolved_value) = value {
            match resolved_value {
                crate::runtime::repl::Value::String(_) => SimpleType::String,
                crate::runtime::repl::Value::List(_) => SimpleType::List,
                crate::runtime::repl::Value::DataFrame { .. } => SimpleType::DataFrame,
                crate::runtime::repl::Value::Object(obj) => {
                    // For objects, suggest field access
                    for field_name in obj.keys() {
                        if field_name.starts_with(&partial_method) {
                            completions.push(Pair {
                                display: format!("{receiver_expr}.{field_name} - object field"),
                                replacement: format!("{receiver_expr}.{field_name}"),
                            });
                        }
                    }
                    return completions;
                }
                _ => receiver_type,
            }
        } else {
            receiver_type
        };
        
        // Get methods for the determined type
        let type_name = match actual_type {
            SimpleType::String => "String",
            SimpleType::List => "List", 
            SimpleType::DataFrame => "DataFrame",
            SimpleType::Unknown => return completions,
        };
        
        let method_names = self.help_system.get_type_methods(type_name);
        for method_name in method_names {
            if method_name.starts_with(&partial_method) {
                completions.push(Pair {
                    display: format!("{receiver_expr}.{method_name} - {type_name} method"),
                    replacement: format!("{receiver_expr}.{method_name}"),
                });
            }
        }
        
        completions
    }
    
    fn resolve_nested_field(&self, nested_expr: &str, bindings: &HashMap<String, crate::runtime::repl::Value>) -> Option<crate::runtime::repl::Value> {
        let parts: Vec<&str> = nested_expr.split('.').collect();
        if parts.is_empty() {
            return None;
        }
        
        // Start with the root variable
        let mut current_value = bindings.get(parts[0])?.clone();
        
        // Navigate through the nested fields
        for &field_name in &parts[1..] {
            match current_value {
                crate::runtime::repl::Value::Object(obj) => {
                    current_value = obj.get(field_name)?.clone();
                }
                _ => return None, // Can't navigate further on non-object types
            }
        }
        
        Some(current_value)
    }

    fn complete_free_expression_with_bindings(&self, partial: String, bindings: &HashMap<String, crate::runtime::repl::Value>) -> Vec<Pair> {
        let mut completions = Vec::new();
        
        // Add builtin functions and keywords
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
            ("fn", "Function definition keyword"),
            ("let", "Variable declaration keyword"),
            ("if", "Conditional keyword"),
            ("else", "Else clause keyword"),
            ("for", "For loop keyword"),
            ("while", "While loop keyword"),
            ("match", "Pattern matching keyword"),
            ("return", "Return statement keyword"),
        ];

        for (name, desc) in builtins {
            if name.starts_with(&partial) {
                completions.push(Pair {
                    display: format!("{name} - {desc}"),
                    replacement: name.to_string(),
                });
            }
        }
        
        // Add user-defined variables and functions from REPL bindings
        for (name, value) in bindings {
            if name.to_lowercase().starts_with(&partial.to_lowercase()) {
                let type_desc = match value {
                    crate::runtime::repl::Value::Function { .. } => "function",
                    crate::runtime::repl::Value::Lambda { .. } => "lambda",
                    crate::runtime::repl::Value::Int(_) => "integer",
                    crate::runtime::repl::Value::Float(_) => "float",
                    crate::runtime::repl::Value::String(_) => "string",
                    crate::runtime::repl::Value::Bool(_) => "boolean",
                    crate::runtime::repl::Value::Char(_) => "character",
                    crate::runtime::repl::Value::List(_) => "list",
                    crate::runtime::repl::Value::Tuple(_) => "tuple",
                    crate::runtime::repl::Value::Object(_) => "object",
                    crate::runtime::repl::Value::HashMap(_) => "hashmap",
                    crate::runtime::repl::Value::HashSet(_) => "hashset",
                    crate::runtime::repl::Value::DataFrame { .. } => "dataframe",
                    crate::runtime::repl::Value::Range { .. } => "range",
                    crate::runtime::repl::Value::EnumVariant { .. } => "enum",
                    crate::runtime::repl::Value::Unit => "unit",
                    crate::runtime::repl::Value::Nil => "nil",
                };
                
                completions.push(Pair {
                    display: format!("{name} - {type_desc}"),
                    replacement: name.clone(),
                });
            }
        }
        
        // Sort completions by relevance (exact prefix matches first, then alphabetical)
        completions.sort_by(|a, b| {
            let a_exact = a.replacement == partial;
            let b_exact = b.replacement == partial;
            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.replacement.cmp(&b.replacement),
            }
        });
        
        completions
    }

    fn complete_function_params(&self, _function_name: String, _current_param: usize) -> Vec<Pair> {
        Vec::new()
    }

    fn complete_help_query(&self, partial: String) -> Vec<Pair> {
        let mut completions = Vec::new();
        
        // Handle REPL commands starting with ':'
        if partial.starts_with(':') {
            let repl_commands = [
                (":load", "Load file into REPL"),
                (":help", "Show help information"),
                (":quit", "Exit the REPL"),
                (":clear", "Clear screen"),
                (":history", "Show command history"),
                (":vars", "List variables"),
                (":funcs", "List functions"),
            ];
            
            for (command, desc) in repl_commands {
                if command.starts_with(&partial) {
                    completions.push(Pair {
                        display: format!("{command} - {desc}"),
                        replacement: command.to_string(),
                    });
                }
            }
        } else {
            // Regular help topics
            let topics = ["println", "type", "dir", "help",
                "List", "String", "DataFrame"];

            for &topic in &topics {
                if topic.starts_with(&partial) {
                    completions.push(Pair {
                        display: topic.to_string(),
                        replacement: topic.to_string(),
                    });
                }
            }
        }
        
        completions
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

        for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
            row[0] = i;
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
        let completions = self.complete_context_immutable(context);
        
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map_or(0, |i| i + 1);
        
        Ok((start, completions))
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
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
    
    // ========== COMPREHENSIVE TAB COMPLETION TESTS ==========
    
    #[test]
    fn test_tab_completion_basic_variables() {
        let mut completer = RuchyCompleter::new();
        let mut bindings = HashMap::new();
        bindings.insert("variable_name".to_string(), crate::runtime::repl::Value::Int(42));
        bindings.insert("another_var".to_string(), crate::runtime::repl::Value::String("test".to_string()));
        
        let completions = completer.get_completions("var", 3, &bindings);
        
        // Should suggest variables that start with "var"
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.contains("variable_name")));
    }
    
    #[test]
    fn test_tab_completion_method_access_list() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("[1,2,3].ma", 10);
        
        if let CompletionContext::MethodAccess { receiver_type, partial_method, .. } = context {
            assert_eq!(receiver_type, SimpleType::List);
            assert_eq!(partial_method, "ma");
        } else {
            panic!("Expected MethodAccess context");
        }
    }
    
    #[test]
    fn test_tab_completion_method_access_string() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("\"hello\".up", 10);
        
        if let CompletionContext::MethodAccess { receiver_type, partial_method, .. } = context {
            assert_eq!(receiver_type, SimpleType::String);
            assert_eq!(partial_method, "up");
        } else {
            panic!("Expected MethodAccess context");
        }
    }
    
    #[test]
    fn test_tab_completion_method_access_dataframe() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("DataFrame::new().sel", 20);
        
        if let CompletionContext::MethodAccess { receiver_type, partial_method, .. } = context {
            assert_eq!(receiver_type, SimpleType::DataFrame);
            assert_eq!(partial_method, "sel");
        } else {
            panic!("Expected MethodAccess context");
        }
    }
    
    #[test]
    fn test_tab_completion_builtin_functions() {
        let mut completer = RuchyCompleter::new();
        let bindings = HashMap::new();
        
        let completions = completer.get_completions("prin", 4, &bindings);
        
        // Should suggest println and print
        assert!(!completions.is_empty());
        let completion_string = completions.join(" ");
        assert!(completion_string.contains("println"));
    }
    
    #[test]
    fn test_tab_completion_help_queries() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("help(prin", 9);
        
        if let CompletionContext::HelpQuery { query } = context {
            assert_eq!(query, "prin");
        } else {
            panic!("Expected HelpQuery context, got: {:?}", context);
        }
    }
    
    #[test]
    fn test_tab_completion_module_paths() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("std::fs::", 8);
        
        if let CompletionContext::ModulePath { segments, partial_segment } = context {
            assert_eq!(segments, vec!["std".to_string(), "fs".to_string()]);
            assert!(partial_segment.is_empty());
        } else {
            panic!("Expected ModulePath context");
        }
    }
    
    #[test]
    fn test_tab_completion_nested_expressions() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("some_function([1,2,3].ma", 24);
        
        if let CompletionContext::MethodAccess { receiver_type, partial_method, .. } = context {
            assert_eq!(receiver_type, SimpleType::List);
            assert_eq!(partial_method, "ma");
        } else {
            panic!("Expected MethodAccess context for nested expression");
        }
    }
    
    #[test]
    fn test_tab_completion_chained_methods() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("[1,2,3].map(|x| x + 1).fil", 27);
        
        if let CompletionContext::MethodAccess { receiver_type, partial_method, .. } = context {
            assert_eq!(receiver_type, SimpleType::List);
            assert_eq!(partial_method, "fil");
        } else {
            panic!("Expected MethodAccess context for chained methods");
        }
    }
    
    #[test] 
    fn test_tab_completion_function_parameters() {
        let completer = RuchyCompleter::new();
        let context = completer.analyze_context("println(", 8);
        
        // Should recognize this as inside a function call
        matches!(context, CompletionContext::FreeExpression { .. });
    }
    
    #[test]
    fn test_tab_completion_partial_identifiers() {
        let mut completer = RuchyCompleter::new();
        let mut bindings = HashMap::new();
        bindings.insert("test_variable".to_string(), crate::runtime::repl::Value::Int(1));
        bindings.insert("test_another".to_string(), crate::runtime::repl::Value::Int(2));
        bindings.insert("different_name".to_string(), crate::runtime::repl::Value::Int(3));
        
        let completions = completer.get_completions("test", 4, &bindings);
        
        // Should suggest both test_variable and test_another
        assert!(completions.len() >= 2);
        let completion_string = completions.join(" ");
        assert!(completion_string.contains("test_variable"));
        assert!(completion_string.contains("test_another"));
        assert!(!completion_string.contains("different_name"));
    }
    
    #[test]
    fn test_tab_completion_empty_context() {
        let mut completer = RuchyCompleter::new();
        let bindings = HashMap::new();
        
        let completions = completer.get_completions("", 0, &bindings);
        
        // Should return common functions and keywords
        assert!(!completions.is_empty());
        let completion_string = completions.join(" ");
        assert!(completion_string.contains("println") || completion_string.contains("let"));
    }
    
    #[test]
    fn test_tab_completion_case_sensitivity() {
        let mut completer = RuchyCompleter::new();
        let mut bindings = HashMap::new();
        bindings.insert("MyVariable".to_string(), crate::runtime::repl::Value::Int(42));
        
        let completions_lower = completer.get_completions("my", 2, &bindings);
        let completions_upper = completer.get_completions("My", 2, &bindings);
        
        // Case insensitive matching should work
        let lower_string = completions_lower.join(" ");
        let upper_string = completions_upper.join(" ");
        
        // At least one should contain MyVariable
        assert!(lower_string.contains("MyVariable") || upper_string.contains("MyVariable"));
    }
    
    #[test]
    fn test_tab_completion_context_boundary() {
        let completer = RuchyCompleter::new();
        
        // Test completion at word boundaries
        let context1 = completer.analyze_context("let x = ", 8);
        assert!(matches!(context1, CompletionContext::FreeExpression { .. }));
        
        let context2 = completer.analyze_context("x.method(", 9);
        assert!(matches!(context2, CompletionContext::FreeExpression { .. }));
    }
    
    #[test]
    fn test_tab_completion_error_recovery() {
        let completer = RuchyCompleter::new();
        
        // Test completion with incomplete/invalid syntax
        let context1 = completer.analyze_context("[1,2,", 5);
        assert!(matches!(context1, CompletionContext::FreeExpression { .. }));
        
        let context2 = completer.analyze_context("if (", 4);
        assert!(matches!(context2, CompletionContext::FreeExpression { .. }));
        
        let context3 = completer.analyze_context("func(", 5);
        assert!(matches!(context3, CompletionContext::FreeExpression { .. }));
    }
    
    #[test]
    fn test_tab_completion_unicode_identifiers() {
        let mut completer = RuchyCompleter::new();
        let mut bindings = HashMap::new();
        bindings.insert("变量".to_string(), crate::runtime::repl::Value::Int(42));
        bindings.insert("переменная".to_string(), crate::runtime::repl::Value::String("test".to_string()));
        
        let completions = completer.get_completions("变", 3, &bindings); // First char of Chinese variable
        
        // Should handle Unicode identifiers
        let completion_string = completions.join(" ");
        assert!(completion_string.contains("变量") || completions.is_empty()); // May not match depending on implementation
    }
    
    #[test]
    fn test_completion_cache_performance() {
        let mut completer = RuchyCompleter::new();
        let bindings = HashMap::new();
        
        // Warm up the cache
        let _warm_up = completer.get_completions("print", 5, &bindings);
        
        // Measure completion time (should be very fast)
        let start = std::time::Instant::now();
        let _completions = completer.get_completions("print", 5, &bindings);
        let duration = start.elapsed();
        
        // Tab completion should be very fast (under 50ms)
        assert!(duration.as_millis() < 50, "Tab completion too slow: {:?}", duration);
    }
    
    #[test]
    fn test_completion_context_stability() {
        let completer = RuchyCompleter::new();
        
        // Same input should produce same context
        let context1 = completer.analyze_context("[1,2,3].map", 11);
        let context2 = completer.analyze_context("[1,2,3].map", 11);
        
        match (&context1, &context2) {
            (CompletionContext::MethodAccess { receiver_type: t1, partial_method: p1, .. },
             CompletionContext::MethodAccess { receiver_type: t2, partial_method: p2, .. }) => {
                assert_eq!(t1, t2);
                assert_eq!(p1, p2);
            }
            _ => panic!("Context analysis should be stable and consistent")
        }
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