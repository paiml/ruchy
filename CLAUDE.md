# CLAUDE.md - Ruchy Compiler Implementation Protocol

## Prime Directive

**Generate correct code that compiles on first attempt. Quality is built-in, not bolted-on.**

## Scripting Policy

**CRITICAL**: Use ONLY Ruchy scripts for adhoc scripting and testing. No Python, Bash scripts, or other languages for testing Ruchy functionality. This ensures we dogfood our own language and discover usability issues early.

✅ **Allowed**: `*.ruchy` files loaded via `:load` command in REPL
❌ **Forbidden**: Python scripts, shell scripts, or any non-Ruchy testing code

## Implementation Hierarchy

```yaml
Navigation:
1. SPECIFICATION.md     # What to build (reference)
2. ROADMAP.md          # Strategic priorities
3. docs/execution/      # Tactical work breakdown
   ├── roadmap.yaml    # PDMT task generation template
   ├── roadmap.md      # Generated execution DAG
   └── velocity.json   # Empirical performance data
4. ../ruchy-book/INTEGRATION.md  # Book compatibility tracking
```

## Book Compatibility Monitoring

**CRITICAL**: Check `../ruchy-book/INTEGRATION.md` FREQUENTLY for:
- Current compatibility: 6% (15/259 examples) + 100% one-liners (20/20)
- Top broken features that book examples expect
- Regression detection from previous versions

Priority fixes from book testing:
1. **Fat Arrow Syntax** - 23 failures
2. **String Interpolation** - 18 failures (f"Hello, {name}!")
3. **Async/Await** - 12 failures
4. **Array Operations** - .map(), .filter(), .reduce()
5. **String Methods** - .len(), .to_upper(), .trim()

## Task Execution Protocol

### Pre-Implementation Verification

```rust
// HALT. Before implementing ANY feature:
□ Check ../ruchy-book/INTEGRATION.md for latest compatibility report
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

## PMAT MCP Quality Proxy (Real-Time Enforcement)

### MCP Server Configuration
```json
// ~/.config/claude/mcps/pmat.json
{
  "mcpServers": {
    "pmat": {
      "command": "~/.local/bin/pmat",
      "args": ["serve", "--mode", "mcp", "--config", "~/ruchy/pmat.toml"],
      "env": {
        "PMAT_PROJECT_ROOT": "~/ruchy",
        "PMAT_QUALITY_MODE": "strict"
      }
    }
  }
}
```

### PMAT Thresholds (Enforced in Real-Time)
```toml
# pmat.toml - MCP proxy blocks violations instantly
[thresholds]
cyclomatic_complexity = 10      # Blocks at write-time
cognitive_complexity = 15        # No mental overload
halstead_effort = 5000          # Computational limits
maintainability_index = 70      # Minimum maintainability
test_coverage = 80              # Coverage gate
satd_comments = 0               # Zero technical debt
mutation_score = 75             # Mutation testing gate
```

### MCP Quality Proxy Tools
```bash
# PMAT exposes these MCP tools to Claude:

pmat_analyze_code       # Real-time complexity analysis
pmat_check_coverage     # Test coverage verification  
pmat_detect_smells      # Code smell detection
pmat_suggest_refactor   # Automated refactoring hints
pmat_mutation_test      # Mutation testing on-demand
pmat_quality_gate       # Full quality check

# These run automatically as you code via MCP
```

### Live Quality Feedback Pattern
```rust
// As you type, PMAT MCP provides instant feedback:

fn process_data(data: &Data) -> Result<(), Error> {
    // PMAT: Complexity 3/10 ✅
    validate(data)?;
    
    if data.complex {  // PMAT: +1 complexity (4/10)
        for item in &data.items {  // PMAT: +2 (6/10)
            if item.check() {  // PMAT: +3 nested (9/10) ⚠️
                // PMAT WARNING: Approaching complexity limit
                process_item(item)?;
            }
        }
    }
    Ok(())
}

// PMAT automatically suggests:
// "Extract loop to process_items() function"
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
- `git commit --no-verify` - NEVER use this
- Skipping tests "temporarily" - NO exceptions
- Ignoring failing quality checks - Must fix EVERY defect
- Dismissing warnings as "unrelated" - All defects matter

**Toyota Way Principle**: Stop the line for ANY defect. No defect is too small. No shortcut is acceptable.

If quality gates hang or fail:
1. Debug and fix the quality gate itself
2. Fix the underlying issue causing the failure
3. NEVER proceed without passing gates

### MANDATORY RELEASE PUBLISHING PROTOCOL

**CRITICAL**: After EVERY version update, you MUST publish to crates.io immediately.

```bash
# MANDATORY after version bump and git push:
cargo publish                    # Publish main package
cd ruchy-cli && cargo publish   # Publish CLI package
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

# GATE 2: Complexity enforcement
pmat check --max-complexity 10 --fail-fast || {
    echo "❌ BLOCKED: Complexity exceeds 10"
    echo "Refactor before committing"
    exit 1
}

# GATE 3: Zero SATD policy
! grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" || {
    echo "❌ BLOCKED: SATD comments found"
    echo "Fix or file GitHub issues, don't commit debt"
    exit 1
}

# GATE 4: Lint zero tolerance
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "❌ BLOCKED: Lint warnings found" 
    exit 1
}

# GATE 5: Coverage threshold
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

---

**Remember**: Compiler engineering is about systematic transformation, not clever hacks. Every abstraction must have zero runtime cost. Every error must be actionable. Every line of code must justify its complexity budget.
- no "cruft" at root of repo.  always clean up temp files/documents before committing.  Zero tolerance for waste.  we follow toyota way.