# Ruchy One-Liner and Script Execution Specification v1.0

## Abstract

This specification defines Ruchy's multi-modal execution system, enabling seamless transitions from shell one-liners to REPL expressions to full scripts. The design synthesizes best practices from Ruby, Perl, Python, AWK, and modern functional languages while maintaining Rust's performance guarantees.

## 1. Execution Modes

### 1.1 Mode Hierarchy

```rust
enum ExecutionMode {
    // Direct command-line execution
    OneLiner {
        code: String,
        auto_mode: AutoMode,
        implicit_imports: Vec<Prelude>,
    },
    
    // Interactive REPL
    Interactive {
        session: ReplSession,
        context: PersistentContext,
    },
    
    // Script file execution
    Script {
        path: PathBuf,
        shebang: Option<ShebangConfig>,
    },
    
    // Compiled binary
    Compiled {
        source: PathBuf,
        target: Target,
    },
}

enum AutoMode {
    None,           // -e: Execute as-is
    Print,          // -p: Auto-print after processing
    Process,        // -n: Process line-by-line, no print
    Filter,         // -f: Process and print if truthy
    Accumulate,     // -a: Collect results into array
}
```

## 2. Command-Line Interface

### 2.1 Invocation Patterns

```bash
# One-liner execution modes
ruchy -e 'println("hello")'                    # Direct execution
ruchy -p 'it.to_uppercase()'                   # Process & print each line
ruchy -n 'sum += it.parse::<i32>()?'           # Process without printing
ruchy -f 'it.contains("error")'                # Filter lines
ruchy -a 'it.split(",")[0]'                    # Accumulate results

# Pipeline composition
cat data.csv | ruchy -p 'it.split(",")[2]'    # Extract third column
ls -la | ruchy -f 'it.contains(".rs")'         # Filter Rust files
seq 1 100 | ruchy -a 'it.parse::<i32>()? * 2' # Double and collect

# Script execution
ruchy script.ruchy                             # Run script file
./script.ruchy                                 # Shebang execution
ruchy -c script.ruchy                          # Compile to binary
```

### 2.2 Implicit Variables

```rust
pub struct ImplicitContext {
    // Current line/item being processed
    it: String,           // Current line from stdin
    
    // Accumulator variables
    sum: Dynamic,         // Running sum (auto-typed)
    count: usize,         // Line counter
    acc: Vec<Dynamic>,    // Accumulator array
    
    // Regex results
    captures: Vec<String>, // From last regex match
    
    // Field access
    fields: Vec<String>,   // it.split_whitespace()
    f1, f2, f3: String,    // First three fields
}
```

## 3. One-Liner Semantics

### 3.1 Automatic Imports

```rust
// Always available in one-liner mode
implicit_prelude! {
    use std::io::{self, BufRead, Write};
    use std::fs;
    use std::collections::*;
    use regex::Regex;
    use itertools::Itertools;
    
    // Ruchy extensions
    use ruchy::prelude::*;
}
```

### 3.2 Dynamic Type Resolution

```rust
pub enum Dynamic {
    // Concrete variants for common types
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<Dynamic>),
    
    // Opaque for complex types
    Opaque(Box<dyn Any>),
}

impl DynamicResolution {
    fn resolve_at_jit(&self, usage: &[Operation]) -> ConcreteType {
        // First execution: track operations on Dynamic
        let observed_ops = usage.iter().collect::<HashSet<_>>();
        
        // JIT compilation: specialize to concrete type
        match observed_ops {
            ops if ops.all_numeric() => ConcreteType::I64,
            ops if ops.has_float_division() => ConcreteType::F64,
            ops if ops.all_string() => ConcreteType::String,
            _ => ConcreteType::Dynamic, // Keep dynamic if mixed
        }
    }
}

// Example transformation during JIT
// First run: sum is Dynamic, tracks += operations with i32
// JIT compile: sum becomes i32, eliminating enum overhead
// Result: Zero-cost abstraction after 3rd execution
```

### 3.3 Expression Transformation Rules

```rust
impl OneLineTransformer {
    fn transform(&self, expr: &str, mode: AutoMode) -> String {
        // Parse for END block (AWK-style)
        let (loop_expr, end_block) = self.split_end_block(expr);
        
        match mode {
            AutoMode::Print => {
                // Wrap in stdin iterator with auto-print
                format!(
                    "for it in io::stdin().lines() {{
                        let it = it?;
                        let result = {{ {} }};
                        println!(\"{{}}\", result);
                    }}
                    {}",
                    loop_expr,
                    end_block.map(|e| format!("{{ {} }}", e)).unwrap_or_default()
                )
            },
            
            AutoMode::Process => {
                // Process without printing, execute END block after
                format!(
                    "let mut sum = 0;
                     let mut count = 0;
                     for it in io::stdin().lines() {{
                        let it = it?;
                        count += 1;
                        {}
                     }}
                     {}",
                    loop_expr,
                    end_block.map(|e| format!("{{ {} }}", e)).unwrap_or_default()
                )
            },
            
            AutoMode::Filter => {
                // Print only if expression is truthy
                format!(
                    "for it in io::stdin().lines() {{
                        let it = it?;
                        if {} {{ println!(\"{{}}\", it); }}
                    }}",
                    expr
                )
            },
            
            AutoMode::Accumulate => {
                // Collect results into array
                format!(
                    "let acc: Vec<_> = io::stdin().lines()
                        .filter_map(|it| {{
                            let it = it.ok()?;
                            Some({{ {} }})
                        }})
                        .collect();
                    println!(\"{{:?}}\", acc);",
                    expr
                )
            },
            
            _ => expr.to_string(),
        }
    }
}
```

## 4. Smart Operators

### 4.1 Pipeline Enhancement

```rust
// Extended pipeline operators for one-liners
operators! {
    // Basic pipeline (inherited from core Ruchy)
    |>   // Pipe forward
    
    // Broadcast operators (Julia-inspired)
    .|>  // Map over collection
    .?>  // Try-map with Option
    .!>  // Try-map with Result
    
    // Accumulation operators
    +>   // Sum-reduce
    *>   // Product-reduce
    &>   // All (logical AND)
    ||>  // Any (logical OR)
    
    // Field extraction (AWK-inspired)
    $1, $2, $3  // Field access by position
    $NF         // Last field
    $$          // All fields as array
}
```

### 4.2 Regex Integration

```rust
// Perl-style regex operators with Rust safety
impl RegexOps for str {
    // Match operator
    fn m(&self, pattern: &str) -> Option<Captures> {
        Regex::new(pattern).ok()?.captures(self)
    }
    
    // Substitution operator
    fn s(&self, pattern: &str, replacement: &str) -> String {
        Regex::new(pattern)
            .map(|re| re.replace_all(self, replacement).into())
            .unwrap_or_else(|_| self.to_string())
    }
}

// In one-liners:
// ruchy -p 'it.m(r"\d+")?.get(0)?'  // Extract first number
// ruchy -p 'it.s(r"\s+", "_")'      // Replace whitespace with underscore
```

## 5. REPL Integration

### 5.1 Seamless Mode Switching

```rust
impl ReplSession {
    fn eval_line(&mut self, input: &str) -> Result<Value> {
        match self.detect_mode(input) {
            Mode::OneLiner if input.starts_with('!') => {
                // Execute as shell command
                self.shell_exec(&input[1..])
            },
            
            Mode::Pipeline if input.contains("|>") => {
                // Process as pipeline expression
                self.eval_pipeline(input)
            },
            
            Mode::MultiLine if input.ends_with('{') => {
                // Start multi-line input
                self.begin_multiline(input)
            },
            
            _ => self.eval_expression(input),
        }
    }
}
```

### 5.2 Contextual Assistance

```rust
// REPL provides context-aware completions for one-liner patterns
impl Completer {
    fn complete(&self, partial: &str) -> Vec<Completion> {
        match self.context {
            Context::Pipeline => self.suggest_pipeline_stages(partial),
            Context::Regex => self.suggest_regex_patterns(partial),
            Context::Iterator => self.suggest_iterator_methods(partial),
            Context::Field => self.suggest_field_accessors(partial),
        }
    }
}
```

## 6. Script Header Processing

### 6.1 Shebang Extensions

```rust
#!/usr/bin/env ruchy
//! FLAGS: -p                     # Default to print mode
//! DEPS: regex="1.5", serde="1.0" # Inline dependencies
//! IMPLICIT: it, sum, fields     # Available without declaration

// Script begins here
it.parse::<i32>()? * 2
```

### 6.2 Mode Detection

```rust
impl ScriptAnalyzer {
    fn detect_script_mode(&self, content: &str) -> ScriptMode {
        let first_lines = content.lines().take(10).collect::<Vec<_>>();
        
        if first_lines.iter().any(|l| l.contains("io::stdin()")) {
            ScriptMode::StreamProcessor
        } else if first_lines.iter().any(|l| l.contains("fn main()")) {
            ScriptMode::Traditional
        } else if first_lines.len() == 1 {
            ScriptMode::OneLiner
        } else {
            ScriptMode::Expression
        }
    }
}
```

## 7. Performance Optimization

### 7.1 JIT Compilation for Hot One-Liners

```rust
pub struct OneLineCache {
    // LRU cache of compiled one-liners
    compiled: LruCache<Blake3Hash, CompiledExpr>,
    
    // Execution count for JIT threshold
    exec_count: HashMap<Blake3Hash, usize>,
    
    // JIT threshold (default: 3 executions)
    jit_threshold: usize,
}

impl OneLineCache {
    fn execute(&mut self, expr: &str, mode: AutoMode) -> Result<()> {
        let hash = blake3::hash(expr.as_bytes());
        
        *self.exec_count.entry(hash).or_insert(0) += 1;
        
        if self.exec_count[&hash] >= self.jit_threshold {
            // JIT compile for repeated use
            let compiled = self.compile_to_native(expr, mode)?;
            self.compiled.put(hash, compiled);
        }
        
        match self.compiled.get(&hash) {
            Some(compiled) => compiled.run(),
            None => self.interpret(expr, mode),
        }
    }
}
```

### 7.2 Stream Processing Optimization

```rust
// Zero-copy processing for large streams
impl StreamProcessor {
    fn process_optimal(&self, expr: &CompiledExpr) -> Result<()> {
        use memmap2::Mmap;
        
        // Memory-map stdin for large files
        let input = unsafe { Mmap::map(&io::stdin())? };
        
        // Parallel processing for CPU-bound operations
        input.par_lines()
            .map(|line| expr.apply(line))
            .try_for_each(|result| {
                writeln!(io::stdout(), "{}", result?)
            })
    }
}
```

## 8. Error Handling

### 8.1 Graceful Degradation

```rust
pub struct OneLineError {
    kind: ErrorKind,
    line_number: Option<usize>,
    suggestion: Option<String>,
}

impl OneLineExecutor {
    fn handle_error(&self, err: OneLineError) -> Result<()> {
        match err.kind {
            ErrorKind::ParseError => {
                // Suggest parentheses or quotes
                eprintln!("Parse error. Try wrapping in quotes or parentheses");
            },
            ErrorKind::TypeError => {
                // Suggest type annotation
                eprintln!("Type error. Try: it.parse::<i32>()?");
            },
            ErrorKind::IoError => {
                // Continue processing remaining lines
                self.continue_after_error()
            },
        }
    }
}
```

## 9. MCP Integration in One-Liners

### 9.1 MCP Tool Invocation

```bash
# Direct MCP tool calls from command line
ruchy -m 'analyze_sentiment(it)' < tweets.txt
ruchy --mcp 'GPT.complete(it, max_tokens: 50)' < prompts.txt

# Pipeline with MCP stages
cat article.md | ruchy -p 'it' | ruchy -m 'summarize(it, style: "technical")'

# Parallel MCP processing with rate limiting
find . -name "*.rs" | ruchy --mcp-parallel 'explain_code(fs::read(it)?)'
```

### 9.2 MCP Protocol Mechanics

```rust
impl McpOneLiner {
    fn transform(&self, expr: &str) -> Result<String> {
        // Parse MCP tool invocation
        let (tool, args) = self.parse_mcp_call(expr)?;
        
        // Generate protocol-compliant wrapper
        format!(
            r#"
            use mcp::{{Client, Tool}};
            
            let client = Client::from_env()?;  // MCP_ENDPOINT env var
            let tool = client.tool("{}")?;
            
            for it in io::stdin().lines() {{
                let it = it?;
                let result = tool.call({{
                    "input": it,
                    {}
                }}).await?;
                println!("{{}}", result);
            }}
            "#,
            tool, args
        )
    }
}

// MCP tool discovery in one-liner mode
impl McpCompleter {
    fn complete_tools(&self) -> Vec<String> {
        // List available MCP tools from connected servers
        self.client.list_tools()
            .map(|tool| format!("{}()", tool.name))
            .collect()
    }
}
```

### 9.3 Streaming MCP Responses

```bash
# Stream processing with MCP
tail -f logs.txt | ruchy --mcp-stream 'detect_anomaly(it, window: 100)'

# Ruchy handles backpressure and buffering
ruchy -e 'stdin.lines() |> buffer(100) |> GPT.batch_complete() |> flatten()'
```

## 10. Disassembly and Introspection

### 10.1 One-Liner Disassembly

```bash
# Show Rust transpilation
ruchy --emit=rust -e 'it.parse::<i32>()? * 2'
// Output: for it in io::stdin().lines() { ... }

# Show LLVM IR
ruchy --emit=llvm -p 'it.trim()'

# Show assembly with performance hints
ruchy --emit=asm -e '[1,2,3] .|> (* 2)'
// Output: SIMD vectorized multiplication detected

# Show MIR for optimization analysis
ruchy --emit=mir -n 'sum += it.parse::<i32>()?'
```

### 10.2 Performance Profiling Integration

```rust
impl DisassemblyMode {
    fn analyze_oneliner(&self, expr: &str, mode: EmitMode) -> String {
        let rust_code = self.transpile(expr);
        
        match mode {
            EmitMode::Rust => {
                // Pretty-print with syntax highlighting
                self.format_rust(rust_code)
            },
            EmitMode::Asm => {
                // Compile and annotate assembly
                let asm = self.compile_to_asm(rust_code)?;
                self.annotate_performance(asm, vec![
                    "SIMD opportunity at line 3",
                    "Branch prediction hint: likely taken",
                    "Cache miss probable on first iteration"
                ])
            },
            EmitMode::Cost => {
                // Show cost model
                format!(
                    "Allocations: {}\n\
                     Copies: {}\n\
                     Branches: {}\n\
                     Estimated cycles: {}",
                    self.count_allocations(&rust_code),
                    self.count_copies(&rust_code),
                    self.count_branches(&rust_code),
                    self.estimate_cycles(&rust_code)
                )
            }
        }
    }
}
```

### 10.3 Interactive Disassembly in REPL

```rust
// REPL commands for introspection
impl ReplDisassembly {
    fn handle_introspection(&mut self, cmd: &str) -> Result<()> {
        match cmd {
            ":rust" => {
                // Show Rust for last expression
                println!("{}", self.last_expr_rust());
            },
            ":asm" => {
                // Show assembly for last expression
                self.show_assembly(self.last_expr());
            },
            ":cost" => {
                // Show performance characteristics
                self.show_cost_model(self.last_expr());
            },
            ":mcp-trace" => {
                // Show MCP protocol messages
                self.show_mcp_trace(self.last_mcp_call());
            }
        }
    }
}
```

## 11. Examples

### 11.1 Common Patterns

```bash
# Sum numbers from stdin
seq 1 100 | ruchy -n 'sum += it.parse::<i32>()?; END { println(sum) }'

# CSV processing
cat data.csv | ruchy -p 'it.split(",")[2].trim()'

# JSON extraction (with automatic serde)
curl api.example.com | ruchy -e 'json!().["results"][0]["name"]'

# Log analysis
tail -f app.log | ruchy -f 'it.contains("ERROR") && !it.contains("IGNORE")'

# Parallel processing
find . -name "*.rs" | ruchy -p --parallel 'fs::metadata(it)?.len()'

# Accumulate and analyze
ps aux | ruchy -a 'fields[2].parse::<f32>()?' | ruchy -e 'acc.sum() / acc.len()'
```

### 11.1 Common Patterns

```bash
# Sum numbers from stdin
seq 1 100 | ruchy -n 'sum += it.parse::<i32>()?; END { println(sum) }'

# CSV processing
cat data.csv | ruchy -p 'it.split(",")[2].trim()'

# JSON extraction (with automatic serde)
curl api.example.com | ruchy -e 'json!().["results"][0]["name"]'

# Log analysis
tail -f app.log | ruchy -f 'it.contains("ERROR") && !it.contains("IGNORE")'

# Parallel processing
find . -name "*.rs" | ruchy -p --parallel 'fs::metadata(it)?.len()'

# Accumulate and analyze
ps aux | ruchy -a 'fields[2].parse::<f32>()?' | ruchy -e 'acc.sum() / acc.len()'

# MCP-powered code analysis
git diff | ruchy -m 'review_changes(it, style: "security")'

# Real-time anomaly detection
tail -f metrics.json | ruchy --mcp-stream 'detect_anomaly(it)' | ruchy -f 'it.severity > 0.8'
```

### 11.2 Advanced Composition

```bash
# Multi-stage pipeline with different modes
cat access.log |
  ruchy -f 'it.m(r"GET .* 200")' |          # Filter successful GETs
  ruchy -p 'it.m(r"/api/(\w+)")?.get(1)?' | # Extract API endpoint
  ruchy -a 'it' |                            # Accumulate
  ruchy -e 'acc.iter().counts()'            # Frequency count

# Performance analysis pipeline
ruchy --emit=rust -e 'data |> map(process) |> filter(valid)' |
  ruchy --emit=asm |
  ruchy -m 'explain_optimization(it)'

# MCP-assisted debugging
cargo test 2>&1 | ruchy -f 'it.contains("FAILED")' | ruchy -m 'explain_test_failure(it)'
```

### 11.3 Disassembly Examples

```bash
# Verify zero-cost abstractions
ruchy --emit=asm -e '[1,2,3].iter().map(|x| x * 2).sum::<i32>()'
# Output: Shows SIMD vectorization, no heap allocations

# Compare pipeline vs nested calls
ruchy --emit=cost -e 'data |> filter(f) |> map(g) |> reduce(h)'
ruchy --emit=cost -e 'reduce(map(filter(data, f), g), h)'
# Output: Identical cost model - true zero-cost abstraction

# Profile MCP overhead
ruchy --emit=mcp-trace -m 'GPT.complete("test")' 
# Output: Shows serialization cost, network latency, deserialization
```

## 12. Configuration

### 12.1 User Preferences

```toml
# ~/.ruchy/config.toml
[oneliner]
default_mode = "print"        # Default to -p
implicit_imports = ["regex", "itertools", "chrono"]
jit_threshold = 3             # Compile after 3 executions
parallel_threshold = 10000    # Use parallel iteration for >10k lines
mcp_timeout = 5000           # MCP call timeout in ms
mcp_retry = 3                # Retry failed MCP calls

[disassembly]
default_emit = "rust"        # Default disassembly format
show_cost = true            # Always show cost model
annotate_optimizations = true # Highlight optimization opportunities
inline_threshold = 20        # Inline functions under 20 instructions

[repl]
oneliner_history = true       # Save one-liners to history
syntax_highlight = true       # Highlight one-liner syntax
mcp_async = true             # Non-blocking MCP calls
auto_disassemble = false    # Show assembly for expensive ops
```

## 13. Technical Implementation

### 13.1 MCP Protocol State Machine

```rust
pub struct McpStateMachine {
    state: McpState,
    buffer: CircularBuffer<Message>,
    pending: VecDeque<RequestId>,
}

enum McpState {
    Ready,
    Connecting { endpoint: Url },
    Authenticated { session: SessionId },
    Streaming { request: RequestId },
    RateLimited { retry_after: Duration },
}

impl McpStateMachine {
    fn process_oneliner(&mut self, expr: &str) -> Result<()> {
        match self.state {
            McpState::Ready => {
                let request = self.build_request(expr)?;
                self.submit(request)
            },
            McpState::RateLimited { retry_after } => {
                // Queue for later execution
                self.pending.push_back(request);
                Ok(())
            },
            _ => self.await_ready(expr),
        }
    }
}
```

### 13.2 Disassembly Cache Architecture

```rust
pub struct DisassemblyCache {
    // Blake3 hash -> compiled artifacts
    artifacts: DashMap<Blake3Hash, Artifacts>,
    
    // Expression -> optimization opportunities
    optimizations: DashMap<String, Vec<Optimization>>,
    
    // Cost model cache
    costs: LruCache<Blake3Hash, CostModel>,
}

struct Artifacts {
    rust: String,
    mir: Option<String>,
    llvm_ir: Option<String>,
    assembly: Option<String>,
    cost: CostModel,
}

impl DisassemblyCache {
    fn get_or_compute(&self, expr: &str, mode: EmitMode) -> Result<String> {
        let hash = blake3::hash(expr.as_bytes());
        
        self.artifacts
            .entry(hash)
            .or_insert_with(|| self.compile_all(expr))
            .get_format(mode)
    }
}
```

## 14. Security and Cost Control

### 14.1 MCP Safety Boundaries

```rust
pub struct McpSafetyGuard {
    // Automatic truncation for large inputs
    max_input_size: usize,      // Default: 16KB
    max_tokens: usize,           // Default: 1000
    cost_threshold: f64,         // Default: $0.10
    
    // Rate limiting
    requests_per_minute: u32,    // Default: 60
    concurrent_requests: u32,    // Default: 10
    
    // Content filtering
    sanitize_pii: bool,         // Default: true
    redact_patterns: Vec<Regex>, // API keys, passwords, etc.
}

impl McpSafetyGuard {
    fn process_input(&self, input: &str) -> Result<String> {
        // Size check
        if input.len() > self.max_input_size {
            if self.should_prompt() {
                eprintln!("Input exceeds {}KB. Truncating...", 
                         self.max_input_size / 1024);
            }
            input = &input[..self.max_input_size];
        }
        
        // Cost estimation
        let estimated_cost = self.estimate_cost(input.len(), self.max_tokens);
        if estimated_cost > self.cost_threshold {
            if !self.confirm_cost(estimated_cost)? {
                return Err(CostLimitExceeded);
            }
        }
        
        // Sanitization
        let sanitized = if self.sanitize_pii {
            self.redact_sensitive(input)?
        } else {
            input.to_string()
        };
        
        Ok(sanitized)
    }
    
    fn redact_sensitive(&self, input: &str) -> String {
        // Redact common sensitive patterns
        let mut result = input.to_string();
        
        // API keys, tokens
        result = Regex::new(r"(api[_-]?key|token|secret)['\"]?\s*[:=]\s*['\"]?[\w-]+")
            .unwrap()
            .replace_all(&result, "$1=<REDACTED>")
            .to_string();
            
        // Email addresses
        result = Regex::new(r"\b[\w._%+-]+@[\w.-]+\.[A-Z|a-z]{2,}\b")
            .unwrap()
            .replace_all(&result, "<EMAIL>")
            .to_string();
            
        // Custom patterns
        for pattern in &self.redact_patterns {
            result = pattern.replace_all(&result, "<REDACTED>").to_string();
        }
        
        result
    }
}
```

### 14.2 Shell Command Sandboxing

```rust
impl ShellSandbox {
    fn validate_command(&self, cmd: &str) -> Result<()> {
        // Prevent dangerous operations in one-liner mode
        const DANGEROUS: &[&str] = &[
            "rm -rf", "dd", "mkfs", ":(){:|:&};:", // Fork bomb
            "> /dev/sda", "sudo", "doas", "su",
        ];
        
        for danger in DANGEROUS {
            if cmd.contains(danger) {
                return Err(SecurityViolation(format!(
                    "Blocked dangerous command: {}", danger
                )));
            }
        }
        
        // Warn on potentially destructive operations
        if cmd.contains("rm") || cmd.contains("delete") {
            eprintln!("⚠️  Destructive operation detected. Proceed with caution.");
        }
        
        Ok(())
    }
}
```

### 14.3 Resource Limits

```rust
pub struct ResourceLimits {
    // Memory limits
    max_heap: usize,        // Default: 1GB for one-liners
    max_stack: usize,       // Default: 8MB
    
    // Time limits  
    max_runtime: Duration,  // Default: 30s
    max_cpu_time: Duration, // Default: 25s
    
    // I/O limits
    max_output: usize,      // Default: 100MB
    max_open_files: u32,    // Default: 256
}

impl ResourceEnforcer {
    fn setup_limits(&self) -> Result<()> {
        // Use setrlimit on Unix
        #[cfg(unix)]
        {
            use nix::sys::resource::{setrlimit, Resource};
            
            setrlimit(Resource::RLIMIT_AS, self.max_heap)?;
            setrlimit(Resource::RLIMIT_CPU, self.max_cpu_time.as_secs())?;
            setrlimit(Resource::RLIMIT_NOFILE, self.max_open_files)?;
        }
        
        Ok(())
    }
}
```

## 15. Implementation Phases

This specification establishes Ruchy as a premier scripting language that scales from shell one-liners to compiled binaries. The integration of MCP and disassembly as first-class features in one-liner mode provides unprecedented capability:

1. **MCP Integration**: Stream processing with LLM-powered transformations directly from the command line
2. **Disassembly Transparency**: Immediate visibility into performance characteristics of one-liners
3. **Zero-friction entry**: One-liners work immediately without boilerplate
4. **Progressive complexity**: Graduate from one-liners to scripts naturally
5. **Performance preservation**: JIT compilation for hot paths, parallel processing for large data
6. **Mechanical transparency**: Every transformation is observable via disassembly

The system maintains mechanical simplicity: each mode is a predictable transformation to Rust, ensuring zero-cost abstractions even in one-liner mode with full introspection capabilities.