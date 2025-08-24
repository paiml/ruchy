# CLAUDE.md - Ruchy Compiler Implementation Protocol

## IMPORTANT: Auto-Generated Files
**NEVER EDIT `deep_context.md`** - This file is auto-generated and will be overwritten. Any changes should be made to the source files instead.

## Prime Directive

**Generate correct code that compiles on first attempt. Quality is built-in, not bolted-on.**

## Scientific Method Protocol

**WE DON'T GUESS, WE PROVE VIA QUANTITATIVE METHODS AND TESTING.**

### Evidence-Based Development Rules:
1. **NO ASSUMPTIONS**: Every claim must be backed by concrete evidence
2. **MEASURE EVERYTHING**: Use tests, benchmarks, and metrics to validate behavior  
3. **REPRODUCE ISSUES**: Create minimal test cases that demonstrate problems
4. **QUANTIFY IMPROVEMENTS**: Before/after metrics prove effectiveness
5. **DOCUMENT EVIDENCE**: All findings must be recorded with reproducible steps

### Investigation Protocol:
1. **Hypothesis**: State what you believe is happening
2. **Test**: Create specific tests that prove/disprove the hypothesis  
3. **Measure**: Collect concrete data (test results, timings, coverage)
4. **Analyze**: Draw conclusions only from the evidence
5. **Document**: Record findings and next steps

**Example**: Don't say "REPL might have different parser" - instead create identical test cases in both unit tests and REPL, measure the difference, identify the exact divergence point.

## Toyota Way Implementation

**STOP THE LINE FOR ANY DEFECT. NO DEFECT IS TOO SMALL. NO SHORTCUT IS ACCEPTABLE.**

### Core Toyota Principles:
1. **Jidoka (Autonomation)**: Build quality into the process, detect problems immediately
2. **Genchi Genbutsu**: Go to the source, understand the root cause through direct observation
3. **Kaizen**: Continuous improvement through systematic problem-solving
4. **Respect for People**: Create systems that prevent human error, not blame people
5. **Long-term Philosophy**: Short-term fixes create long-term problems

### Defect Response Protocol (MANDATORY):
```
1. HALT DEVELOPMENT: Stop all forward progress when defect detected
2. ROOT CAUSE ANALYSIS: Use 5 Whys to find true cause, not symptoms  
3. POKA-YOKE: Design error-prevention into the system
4. SYSTEMATIC TESTING: Add comprehensive test coverage to prevent recurrence
5. PROCESS IMPROVEMENT: Update development process to catch similar issues earlier
6. VERIFICATION: Prove the fix works and cannot regress
```

### Testing Hierarchy (Idiot-Proof Prevention):
```
Level 1: Unit Tests         - Function-level correctness
Level 2: Integration Tests  - Component interaction  
Level 3: E2E Tests          - Full system behavior
Level 4: Property Tests     - Mathematical invariants
Level 5: Fuzz Tests         - Random input robustness
Level 6: Regression Tests   - Historical defect prevention
Level 7: Performance Tests  - Non-functional requirements
```

**NEVER AGAIN RULE**: Every defect must be made impossible to repeat through systematic prevention.

## Toyota Way Success Story: Property Testing Victory

**CASE STUDY**: Suspected "REPL vs Unit Test Parser Inconsistency" (2024-12)

**Manual Testing Claims**: 
- ❌ "REPL fails to parse `std::fs::read_file` while unit tests pass"
- ❌ "3+ segment qualified names broken in REPL"  
- ❌ "Parser inconsistency between binary and library"

**Property Testing Evidence**:
- ✅ **410 systematic test cases**: 0 inconsistencies found
- ✅ **135 fuzz-generated cases**: 0 inconsistencies found  
- ✅ **Perfect parser consistency** proved across all contexts

**ROOT CAUSE**: Manual testing methodology error, NOT code defect.

**LESSONS LEARNED**:
1. **Human perception is unreliable** - manual tests can create false patterns
2. **Property testing is objective** - mathematically proves system behavior
3. **Systematic > Anecdotal** - 545 automated tests > individual manual observations  
4. **Toyota Way Works** - stopping the line led to better understanding, not wasted time

**PREVENTION**: All suspected "parsing inconsistencies" must be validated with property tests before investigation.

## Scripting Policy

**CRITICAL**: Use ONLY Ruchy scripts for adhoc scripting and testing. No Python, Bash scripts, or other languages for testing Ruchy functionality. This ensures we dogfood our own language and discover usability issues early.

✅ **Allowed**: `*.ruchy` files loaded via `:load` command in REPL
❌ **Forbidden**: Python scripts, shell scripts, or any non-Ruchy testing code

## Implementation Hierarchy

```yaml
Navigation:
1. SPECIFICATION.md     # What to build (reference)
2. docs/execution/roadmap.md  # Strategic priorities and current tasks
3. docs/execution/      # Tactical work breakdown
   ├── roadmap.yaml    # PDMT task generation template
   └── velocity.json   # Empirical performance data
4. ../ruchy-book/INTEGRATION.md  # Book compatibility tracking
```

## Book Compatibility Monitoring

**CRITICAL**: Check `../ruchy-book/INTEGRATION.md` FREQUENTLY for:
- Current compatibility: 22% (57/259 examples) + 100% one-liners (20/20)
- v0.7.22 Quality Update: Interpreter complexity reduced 209 → 138
- Regression detection from previous versions

Priority fixes from book testing:
1. **Fat Arrow Syntax** - 23 failures
2. **String Interpolation** - 18 failures (f"Hello, {name}!")
3. **Async/Await** - 12 failures
4. **Array Operations** - .map(), .filter(), .reduce() ✅ PARTIALLY FIXED
5. **String Methods** - .len(), .to_upper(), .trim() ✅ FIXED

## Quality Status (v0.7.22)

**INTERPRETER COMPLEXITY**: 
- evaluate_expr: 138 (was 209, target <50)
- Value::fmt: 66 (target <30)
- Value::format_dataframe: 69 (target <30)
- **Next Sprint**: Complete reduction to <50 for all functions

## Known Runtime Bugs (from ../ruchy-book/docs/bugs/)

**✅ FIXED BUG #001**: File Operations (RESOLVED in v0.7.10)
- File operations now work correctly
- 3.7x improvement in book compatibility

## Critical Quality Gate Defect (Toyota Way Investigation)

**DEFECT**: Pre-commit hook hangs at dogfooding test (discovered 2024-12)

**Five Whys Root Cause Analysis**:
1. Why does quality gate hang? → Dogfooding test never completes
2. Why doesn't it complete? → The debug binary is used instead of release
3. Why does debug binary fail? → It generates invalid Rust code (wraps in {} incorrectly)
4. Why are debug/release different? → Different optimization levels expose transpiler bugs
5. Why wasn't this caught? → Pre-commit hook inconsistently uses debug vs release binaries

**ROOT CAUSE**: Transpiler generates different code in debug vs release mode, violating determinism

**IMMEDIATE FIX**: Always use release binary in quality gates
**LONG-TERM FIX**: Ensure transpiler is deterministic across all build modes

**PREVENTION**: 
- Add property test: `assert!(transpile_debug(x) == transpile_release(x))`
- Quality gates must use consistent binary paths
- Never allow behavioral differences between debug/release

## Task Execution Protocol

### Pre-Implementation Verification

```rust
// HALT. Before implementing ANY feature:
□ Check ../ruchy-book/INTEGRATION.md for latest compatibility report
□ Check ../ruchy-book/docs/bugs/ruchy-runtime-bugs.md for known issues
□ Locate specification section in SPECIFICATION.md
□ Find task ID in docs/execution/roadmap.md
□ Verify dependencies completed via DAG
□ Check existing patterns in codebase
□ Confirm complexity budget (<10 cognitive)
```

### Task Code Format

```bash
# Every commit references execution task:
git commit -m "DF-P-001: Implement DataFrame literal parsing

Validates: SPECIFICATION.md Section 3.1
Performance: 65MB/s parsing throughput
Coverage: 94% on parser/expr.rs"
```

## Compiler Architecture Patterns

### Parser Pattern - Pratt with Error Recovery

```rust
impl Parser {
    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;
        
        while let Some(&op) = self.peek() {
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp { break; }
            
            self.advance();
            let rhs = self.parse_expr(r_bp)?;
            lhs = Expr::binary(op, lhs, rhs, self.span());
        }
        
        Ok(lhs)
    }
}
```

### Type Inference - Bidirectional Checking

```rust
impl TypeChecker {
    fn check(&mut self, expr: &Expr, expected: Type) -> Result<(), TypeError> {
        match (&expr.kind, expected) {
            (ExprKind::Lambda(params, body), Type::Function(arg_tys, ret_ty)) => {
                self.check_params(params, arg_tys)?;
                self.check(body, *ret_ty)
            }
            _ => {
                let inferred = self.infer(expr)?;
                self.unify(inferred, expected)
            }
        }
    }
    
    fn infer(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match &expr.kind {
            ExprKind::Variable(name) => self.env.lookup(name),
            ExprKind::Apply(func, arg) => {
                let func_ty = self.infer(func)?;
                let (param_ty, ret_ty) = self.expect_function(func_ty)?;
                self.check(arg, param_ty)?;
                Ok(ret_ty)
            }
            _ => self.infer_primitive(expr)
        }
    }
}
```

### Zero-Cost Transpilation

```rust
impl Transpiler {
    fn transpile_dataframe(&self, df: &DataFrameExpr) -> TokenStream {
        // Direct Polars LogicalPlan generation - no intermediate allocation
        match &df.kind {
            DfKind::Literal(cols) => {
                let columns = cols.iter().map(|c| self.transpile_column(c));
                quote! { ::polars::prelude::df![#(#columns),*] }
            }
            DfKind::Pipeline(source, ops) => {
                let base = self.transpile(source);
                ops.iter().fold(base, |acc, op| {
                    self.apply_operation(acc, op)
                })
            }
        }
    }
    
    fn apply_operation(&self, frame: TokenStream, op: &DfOp) -> TokenStream {
        // Zero-copy operation chaining
        match op {
            DfOp::Filter(predicate) => {
                let pred = self.transpile(predicate);
                quote! { #frame.lazy().filter(#pred) }
            }
            DfOp::GroupBy(keys) => {
                let key_exprs = self.transpile_keys(keys);
                quote! { #frame.groupby([#(#key_exprs),*]) }
            }
        }
    }
}
```

## Memory Management Patterns

### Arena Allocation for AST

```rust
pub struct AstArena {
    nodes: TypedArena<Expr>,
    strings: StringInterner,
}

impl AstArena {
    pub fn alloc(&self, expr: Expr) -> &Expr {
        // O(1) allocation, bulk deallocation
        self.nodes.alloc(expr)
    }
    
    pub fn intern(&mut self, s: &str) -> InternedString {
        // String deduplication
        self.strings.get_or_intern(s)
    }
}
```

### Session-Scoped Resources

```rust
struct CompilerSession {
    arena: AstArena,
    type_cache: FxHashMap<TypeId, Type>,
    symbol_table: SymbolTable,
}

impl Drop for CompilerSession {
    fn drop(&mut self) {
        // Bulk deallocation - no per-node overhead
    }
}
```

## Performance Invariants

### Parsing Throughput

```rust
#[bench]
fn bench_parse_throughput(b: &mut Bencher) {
    let input = include_str!("../corpus/large.ruchy"); // 10K LOC
    b.iter(|| {
        let mut parser = Parser::new(input);
        parser.parse_module()
    });
    
    // Invariant: >50MB/s
    assert!(b.bytes_per_second() > 50_000_000);
}
```

### Type Inference Latency

```rust
#[bench]
fn bench_type_inference(b: &mut Bencher) {
    let ast = test_ast();
    b.iter(|| {
        let mut checker = TypeChecker::new();
        checker.infer_module(&ast)
    });
    
    // Invariant: <15ms for typical program
    assert!(b.ns_per_iter() < 15_000_000);
}
```

## Error Diagnostics

### Elm-Level Error Quality

```rust
impl Diagnostic {
    fn render(&self) -> String {
        let mut output = String::new();
        
        // Source context with highlighting
        writeln!(output, "{}", self.source_snippet());
        writeln!(output, "{}", "^".repeat(self.span.len()).red());
        
        // Primary message
        writeln!(output, "\n{}: {}", "Error".red().bold(), self.message);
        
        // Actionable suggestion
        if let Some(suggestion) = &self.suggestion {
            writeln!(output, "\n{}: {}", "Hint".green(), suggestion);
        }
        
        output
    }
}
```

## PMAT Agent Mode Integration (Toyota Way Real-Time Quality Engineering)

**BREAKTHROUGH**: PMAT v2.10.0 Agent Mode provides autonomous quality engineering following Toyota Way principles with continuous monitoring, proactive refactoring, and intelligent quality gate enforcement.

### Claude Code MCP Agent Configuration
```json
// ~/.config/claude/mcps/pmat-agent.json - Full Agent Mode Integration
{
  "mcpServers": {
    "pmat-agent": {
      "command": "pmat",
      "args": ["mcp", "serve", "--mode", "agent", "--config", "/home/noah/src/ruchy/pmat-agent.toml"],
      "transport": "stdio",
      "settings": {
        "quality_monitoring": {
          "enabled": true,
          "complexity_threshold": 10,    // Toyota Way: stricter than standard
          "watch_patterns": ["**/*.rs", "Cargo.toml", "src/**/*.rs"],
          "notify_on_violations": true,
          "continuous_monitoring": true
        },
        "toyota_way": {
          "enforce_complexity": true,     // ≤10 complexity mandatory
          "zero_satd_tolerance": true,    // No technical debt allowed
          "require_tests": true,          // Test coverage required
          "kaizen_mode": true,            // Continuous improvement
          "genchi_genbutsu": true         // Go and see real problems
        },
        "agent_behavior": {
          "proactive_suggestions": true,
          "auto_refactor_threshold": 15,  // Suggest refactoring at 15 complexity
          "background_daemon": true,      // Continuous monitoring
          "jidoka_mode": true            // Stop the line for quality issues
        }
      }
    }
  }
}
```

### Ruchy Project PMAT Agent Configuration
```toml
# pmat-agent.toml - Toyota Way Quality Engineering Configuration
[agent]
version = "2.10.0"
name = "ruchy-toyota-way-agent"
environment = "development"

# Toyota Way Standards (BLOCKING)
complexity_threshold = 10        # Strict Toyota Way standard (vs industry 20)
satd_enabled = true             # Zero SATD tolerance
check_interval_seconds = 30     # Frequent Kaizen monitoring
watch_patterns = ["*.rs", "Cargo.toml", "tests/*.rs", "benches/*.rs"]

[quality_monitor]
enabled = true
debounce_ms = 1000              # Fast feedback loop
full_analysis_interval_minutes = 15
incremental_analysis = true

# Real-time file system watching
watch_depth = 8
ignore_patterns = ["target/", ".git/", "*.tmp", "*.log", "fuzz/corpus/"]
max_file_size_mb = 10

[toyota_way]
# Toyota Production System integration
jidoka_enabled = true           # Autonomation with human touch
kaizen_interval_minutes = 30    # Continuous improvement cycles
genchi_genbutsu_analysis = true # Go and see the actual code
poka_yoke_prevention = true     # Error prevention at source

[mcp]
protocol_version = "2024-11-05"
server_name = "ruchy-toyota-way-agent" 
debug_mode = false

# Enable all Toyota Way tools
[mcp.tools]
enable_all = true
complexity_analysis = true
quality_gates = true
continuous_monitoring = true
kaizen_refactoring = true
jidoka_enforcement = true
```

### PMAT Agent MCP Tools (Toyota Way Integration)
```typescript
interface PmatAgentTools {
  // Continuous Quality Monitoring (Jidoka)
  start_quality_monitoring: {
    project_path: "/home/noah/src/ruchy"
    complexity_threshold: 10        // Toyota Way strict standard
    toyota_way_mode: true
    jidoka_enabled: true           // Stop line for defects
  }
  
  // Kaizen-Based Refactoring
  suggest_kaizen_refactoring: {
    file_path: string
    target_complexity: 10
    refactor_strategy: "toyota-way"   // Small, incremental improvements
    genchi_genbutsu: true            // Analyze root cause
  }
  
  // Real-Time Quality Gates (Poka-Yoke)
  run_toyota_quality_gates: {
    scope: "repository" | "file" | "function"
    target: string
    prevent_defects: true           // Error prevention
    output_format: "claude-friendly"
  }
  
  // Genchi Genbutsu Analysis
  analyze_complexity_trends: {
    time_window: "24h"
    genchi_genbutsu: true          // Go see actual complexity trends
    predict_quality_issues: true
    kaizen_opportunities: true
  }
  
  // Jidoka Health Check
  toyota_way_health_check: {
    include_satd: true             // Zero SATD tolerance
    include_dead_code: true
    include_complexity_violations: true
    stop_line_on_defects: true     // Toyota Way principle
    generate_kaizen_plan: true
  }
}
```

### Live Toyota Way Quality Feedback
```rust
// PMAT Agent provides Toyota Way guidance as you code:

pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, EvalError> {
    // 🏭 PMAT TOYOTA WAY AGENT: Function complexity 138/10 ❌ JIDOKA STOP!
    // 🔧 KAIZEN SUGGESTION: Split into 5 dispatcher functions  
    // 📍 GENCHI GENBUTSU: Root cause is massive match expression
    // 🚫 POKA-YOKE: Preventing further complexity increases
    
    match &expr.kind {
        ExprKind::Literal(lit) => {
            // ✅ PMAT AGENT: Extracted evaluate_literal_expr() - complexity reduced
            self.evaluate_literal_expr(lit)
        }
        ExprKind::Variable(name) => {
            // ✅ PMAT AGENT: Dispatcher pattern applied successfully  
            self.evaluate_variable_expr(name)
        }
        // PMAT AGENT: 47 more cases -> Extract to evaluate_complex_expr()
        _ => self.evaluate_complex_expr(expr)
    }
}

// 🎯 PMAT Agent Toyota Way Recommendations:
// 1. KAIZEN: "Reduce complexity from 138 → 10 through dispatcher pattern"
// 2. JIDOKA: "Stop development until complexity is fixed"
// 3. GENCHI GENBUTSU: "Analyze AST structure to understand root cause"
// 4. POKA-YOKE: "Prevent new complexity through real-time monitoring"
```

### Toyota Way Quality Gate Integration
```bash
#!/bin/bash
# Enhanced Toyota Way pre-commit hook with PMAT Agent

set -e

echo "🏭 PMAT Toyota Way Agent: Jidoka Quality Gates"

# Kaizen: Continuous improvement check
pmat mcp call toyota_way_health_check '{
  "stop_line_on_defects": true,
  "generate_kaizen_plan": true,
  "zero_satd_tolerance": true
}'

# Genchi Genbutsu: Go and see the actual problems  
pmat mcp call analyze_complexity_trends '{
  "genchi_genbutsu": true,
  "kaizen_opportunities": true,
  "predict_quality_issues": true
}'

# Jidoka: Autonomation with human touch
pmat mcp call run_toyota_quality_gates '{
  "scope": "repository",
  "prevent_defects": true,
  "toyota_way_strict": true
}'

# Poka-Yoke: Error prevention verification
if [ $? -ne 0 ]; then
    echo "❌ JIDOKA STOP: Quality defects detected"
    echo "Following Toyota Way - must fix ALL defects before proceeding"
    echo "No compromise on quality - every defect matters"
    exit 1
fi

echo "✅ Toyota Way Agent: All quality gates passed - continue with confidence"
```

### Zero-SATD Enforcement
```rust
// PMAT MCP blocks these in real-time:

fn temporary_hack() {
    // TODO: Fix this later  // ❌ PMAT: BLOCKED - SATD detected
    // FIXME: Memory leak     // ❌ PMAT: BLOCKED - SATD detected
    // HACK: Works for now    // ❌ PMAT: BLOCKED - SATD detected
    
    // Instead, PMAT forces:
    // Track in GitHub Issues, not code comments
}
```

## MANDATORY Quality Gates (BLOCKING - Not Advisory)

### SACRED RULE: NEVER BYPASS QUALITY GATES

**ABSOLUTELY FORBIDDEN**:
- `git commit --no-verify` - NEVER use this - NO EXCEPTIONS EVER
- Skipping tests "temporarily" - NO exceptions
- Ignoring failing quality checks - Must fix EVERY defect
- Dismissing warnings as "unrelated" - All defects matter
- Using `--no-verify` flag - This violates Toyota Way principles

**Toyota Way Principle**: Stop the line for ANY defect. No defect is too small. No shortcut is acceptable.

**CRITICAL RULE**: NEVER use `--no-verify` in git commits. This bypasses critical quality gates that protect code integrity. If quality gates are blocking:
1. Fix ALL clippy warnings immediately - add `# Errors` sections, doctests, etc.
2. If gates are genuinely broken, fix the gates themselves
3. NEVER bypass with --no-verify - this is ABSOLUTELY FORBIDDEN

**WHEN CLIPPY BLOCKS**: Always fix the root cause:
- Missing `# Errors` sections → Add proper documentation with examples
- Using `unwrap()` → Replace with `expect()` with meaningful messages  
- Dead code warnings → Remove or prefix with underscore
- Missing doctests → Add runnable examples to documentation

If quality gates hang or fail:
1. Debug and fix the quality gate itself
2. Fix the underlying issue causing the failure
3. NEVER proceed without passing gates

### MANDATORY RELEASE PUBLISHING PROTOCOL

**CRITICAL**: After EVERY version update, you MUST publish to crates.io immediately.

```bash
# MANDATORY after version bump and git push:
cargo publish                    # Publish main package only
# NOTE: ruchy-cli is DEPRECATED - do NOT publish (MUDA/waste)
```

**NO EXCEPTIONS**: 
- Every release pushed to GitHub MUST be published to crates.io
- Users depend on published releases being available
- Never leave a version unpublished after pushing to git
- Publishing is NOT optional - it's part of the release process

## MANDATORY Quality Gates (BLOCKING - Not Advisory)

**CRITICAL**: After the shameful failures of v0.4.6 (false claims, broken transpiler), quality gates are now BLOCKING and ENFORCED.

### Pre-commit Hooks (MANDATORY)
```bash
#!/bin/bash
# .git/hooks/pre-commit - BLOCKS commits that violate quality
set -e

echo "🔒 MANDATORY Quality Gates..."

# GATE 1: Basic functionality (FATAL if fails)
echo 'println("Hello")' | timeout 5s ruchy repl | grep -q "Hello" || {
    echo "❌ FATAL: Can't even print 'Hello' in REPL"
    echo "Fix basic transpiler before ANY commits"
    exit 1
}

# GATE 2: Language Feature Compatibility (CRITICAL - NO REGRESSIONS ALLOWED)
echo "🔍 Testing language feature compatibility..."
cargo test test_one_liners --test compatibility_suite --quiet || {
    echo "❌ FATAL: One-liner compatibility regression detected"
    echo "One-liners are our baseline - fix immediately"
    exit 1
}

cargo test test_basic_language_features --test compatibility_suite --quiet || {
    echo "❌ FATAL: Basic language features broken"
    echo "Core language regression detected - fix immediately"  
    exit 1
}

# GATE 3: Complexity enforcement (PMAT Agent Mode Enhanced)
pmat agent analyze --max-complexity 10 --auto-fix || {
    echo "❌ BLOCKED: Complexity exceeds 10"
    echo "PMAT Agent Mode can auto-refactor - run: pmat agent refactor"
    exit 1
}

# GATE 4: Zero SATD policy
! grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" || {
    echo "❌ BLOCKED: SATD comments found"
    echo "Fix or file GitHub issues, don't commit debt"
    exit 1
}

# GATE 5: Lint zero tolerance
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "❌ BLOCKED: Lint warnings found" 
    exit 1
}

# GATE 6: Coverage threshold
cargo tarpaulin --min 80 --fail-under || {
    echo "❌ BLOCKED: Coverage below 80%"
    exit 1
}

echo "✅ All quality gates passed"
```

### CI/CD Pipeline Enforcement
```yaml
# .github/workflows/quality-gates.yml
name: MANDATORY Quality Gates
on: [push, pull_request]

jobs:
  quality-gates:
    runs-on: ubuntu-latest
    steps:
      - name: Install Quality Gates
        run: |
          curl -L https://install.pmat.dev | sh
          
      - name: GATE 1 - Basic REPL Function
        run: |
          echo 'println("CI Test")' | timeout 10s ruchy repl | grep -q "CI Test"
          
      - name: GATE 2 - Complexity Check  
        run: |
          pmat check --max-complexity 10 --fail-fast
          
      - name: GATE 3 - Zero SATD
        run: |
          ! grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs"
          
      - name: GATE 4 - Lint Zero Tolerance
        run: |
          cargo clippy --all-targets --all-features -- -D warnings
          
      - name: GATE 5 - Coverage Gate
        run: |
          cargo tarpaulin --min 80 --fail-under
          
      - name: GATE 6 - Dogfooding Test
        run: |
          # Must be able to run scripts written in Ruchy
          find scripts -name "*.ruchy" -exec ruchy run {} \;
```

## The Make Lint Contract (Zero Warnings Allowed)
```bash
# make lint command from Makefile:
cargo clippy --all-targets --all-features -- -D warnings
```

**Critical**: The `-D warnings` flag treats EVERY clippy warning as a hard error. This is more rigid than standard clippy and creates a quality backlog.

### What This Means for Your Code

```rust
// Standard clippy: These would be warnings
x.to_string();           // Warning: could use .into()
&vec![1, 2, 3];         // Warning: could use slice
if x == true { }        // Warning: could omit == true

// With make lint: These FAIL the build
x.to_string();          // ERROR - build fails
&vec![1, 2, 3];        // ERROR - build fails  
if x == true { }       // ERROR - build fails
```

### Surviving -D warnings

```rust
// Write defensive code from the start:
x.into();               // Prefer into() over to_string()
&[1, 2, 3];            // Use slice literals
if x { }               // Omit redundant comparisons

// For unavoidable warnings, be explicit:
#[allow(clippy::specific_lint)]  // Document why
fn special_case() { }
```

## REVISED Implementation Order (Post-Failure Analysis)

**STOP**: The original order was wrong and led to v0.4.6 failures. New order based on user experience:

### Phase 0: FOUNDATION (MUST WORK FIRST)
1. **Fix Transpiler Basics**
    - println generates println! correctly
    - Basic string handling without compilation errors
    - Simple arithmetic transpiles correctly

2. **One-liner Support** (Week 1 CRITICAL PATH)
    - CLI `-e` flag implementation
    - Stdin pipe support
    - Exit codes for scripting
    - JSON output mode

3. **REPL Core Functions**
    - Function calling after definition
    - Variable persistence across lines
    - Block expressions return correct value

### Phase 1: CONTROL FLOW
4. **Pattern Matching** (Users expect this)
    - Match expression evaluation
    - Pattern guards working
    - Exhaustiveness checking

5. **Loops** (Basic language feature)
    - For loops in REPL
    - While loops in REPL
    - Break/continue support

### Phase 2: FUNCTIONAL FEATURES
6. **Pipeline Operators** (Core selling point)
    - |> operator evaluation
    - Function composition
    - Method chaining

7. **String Interpolation** (User convenience)
    - f"Hello {name}" syntax
    - Expression interpolation
    - Format specifiers

### Phase 3: ADVANCED (Only after basics work)
8. **DataFrame Support** (Only if foundation solid)
9. **Result Type** (Error handling)
10. **Actor System** (Concurrency model)

**NEW RULE**: No Phase N+1 until Phase N is 100% working in REPL and dogfooded with .ruchy scripts.

## Architectural Decisions

### Why Pratt Parsing?
Operator precedence without grammar ambiguity. O(n) time, O(1) space per precedence level.

### Why Bidirectional Type Checking?
Combines inference power with predictable checking. Lambda parameters inferred from context.

### Why Direct Polars Transpilation?
Zero intermediate representation. DataFrame operations compile to optimal LogicalPlan.

### Why Arena Allocation?
Bulk deallocation. No per-node overhead. Cache-friendly traversal.

## The Development Flow

```
1. LOCATE specification section
2. IDENTIFY task in execution roadmap
3. VERIFY dependencies complete
4. IMPLEMENT with <10 complexity
5. VALIDATE performance invariants
6. COMMIT with task reference
```

## Sprint Hygiene Protocol

### Pre-Sprint Cleanup (MANDATORY)
```bash
# Remove all debug binaries before starting sprint
rm -f test_* debug_* 
find . -type f -executable -not -path "./target/*" -not -path "./.git/*" -delete

# Verify no large files
find . -type f -size +100M -not -path "./target/*" -not -path "./.git/*"

# Clean build artifacts
cargo clean
```

### Post-Sprint Checklist
```bash
# 1. Remove debug artifacts
rm -f test_* debug_* *.o *.a

# 2. Update tracking
git add docs/execution/velocity.json docs/execution/roadmap.md

# 3. Verify no cruft
git status --ignored

# 4. Push with clean history
git push origin main
```

### .gitignore Requirements
Always maintain in .gitignore:
- `test_*` (debug test binaries)
- `debug_*` (debug executables) 
- `!*.rs` (except rust source files)
- `!*.toml` (except config files)

## Language Feature Testing Protocol

### CRITICAL REQUIREMENT: Language Compatibility First

**NO CODE CHANGES can be committed without passing language feature compatibility tests.**

Following research from Rust, Python, Elixir, Ruby, SQLite, Haskell, and JavaScript/Deno ecosystems, we implement multi-tier testing:

#### 1. **Compatibility Test Suite** (MANDATORY)
```bash
# Run before EVERY commit - no exceptions
make compatibility  # Or: cargo test compatibility_report --test compatibility_suite -- --nocapture --ignored
```

**Current Standards (v1.0.0 - PERFECT COMPATIBILITY ACHIEVED!)**:
- ✅ **One-liners**: 100% (15/15) - BASELINE - never regress
- ✅ **Basic Language Features**: 100% (5/5) - Core syntax complete  
- ✅ **Control Flow**: 100% (5/5) - if/match/for/while/pattern-guards
- ✅ **Data Structures**: 100% (7/7) - Objects fully functional + .items()/.keys()/.values()
- ✅ **String Operations**: 100% (5/5) - All string methods working
- ✅ **Numeric Operations**: 100% (4/4) - Integer.to_string() + all math ops
- ✅ **Advanced Features**: 100% (4/4) - Pattern guards COMPLETED! 🎉

**🏆 ULTIMATE ACHIEVEMENT: 100% PERFECT LANGUAGE COMPATIBILITY! 🏆
🎯 TOTAL: 41/41 FEATURES WORKING - NO DEFECTS LEFT BEHIND**

#### 2. **Property-Based Testing** (Best Practice from Haskell/Elixir)
```bash
# Test mathematical invariants and edge cases  
cargo test properties/ --release
```

Tests universal properties like:
- Arithmetic commutativity: `a + b == b + a`
- String associativity: `(a + b) + c == a + (b + c)`
- Parser robustness with malformed input
- Function call determinism

#### 3. **Performance Regression Detection** (SQLite Standard)
```bash
# Detect performance regressions automatically
cargo test --test compatibility_suite -- --nocapture
```

Thresholds (fail if exceeded):
- Simple arithmetic: <10ms
- Function calls: <20ms  
- Loop iteration: <50ms
- String operations: <30ms

#### 4. **Regression Prevention** (Toyota Way)
Every GitHub issue and bug MUST have a corresponding test to prevent recurrence.

### Test Organization (Industry Standard)
```
tests/
├── compatibility_suite.rs      # Main feature compatibility (100% required)
├── properties/                 # Property-based testing (Haskell style)
├── regression/                 # Bug prevention (every GitHub issue)
└── benchmarks/                # Performance baselines (SQLite style)
```

### Implementation Status

✅ **COMPLETED**:
- Comprehensive compatibility test suite
- Performance regression detection  
- Statistical reporting (Python pytest style)
- Multi-tier test categorization

🎯 **NEXT PHASE**:
- Property-based testing with QuickCheck patterns
- Fuzzing integration for parser robustness
- Cross-platform compatibility verification

### Quality Gate Integration

Language compatibility testing is now **GATE 2** in our mandatory pre-commit hooks - more critical than complexity or linting because **language regressions break user code**.

---

**Remember**: Compiler engineering is about systematic transformation, not clever hacks. Every abstraction must have zero runtime cost. Every error must be actionable. Every line of code must justify its complexity budget.
- no "cruft" at root of repo.  always clean up temp files/documents before committing.  Zero tolerance for waste.  we follow toyota way.
- if fixing documentation, always ensure a doctests exists, if not create.
- Language compatibility tests are now MANDATORY quality gates - never bypass
- ruchy-cli is deprecated, stop publishing it.