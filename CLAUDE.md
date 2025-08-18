# CLAUDE.md - Ruchy Compiler Implementation Protocol

## Prime Directive

**Generate correct code that compiles on first attempt. Quality is built-in, not bolted-on.**

## Implementation Hierarchy

```yaml
Navigation:
1. SPECIFICATION.md     # What to build (reference)
2. ROADMAP.md          # Strategic priorities
3. docs/execution/      # Tactical work breakdown
   ├── roadmap.yaml    # PDMT task generation template
   ├── roadmap.md      # Generated execution DAG
   └── velocity.json   # Empirical performance data
```

## Task Execution Protocol

### Pre-Implementation Verification

```rust
// HALT. Before implementing ANY feature:
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

## The Make Lint Contract (Zero Warnings Allowed)
## Quality Enforcement
See: docs/quality/lint-reduction-process-and-quality.md
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

## Canonical Implementation Order

Based on dependency analysis and critical path:

1. **DataFrame Support** (Blocking all examples)
    - Parser extensions for df literals
    - Type system DataFrame/Series types
    - Transpiler Polars generation

2. **Result Type** (Error handling foundation)
    - Parser ? operator precedence
    - Type inference for Result<T, E>
    - Error propagation in transpiler

3. **Actor System** (Concurrency model)
    - Parser actor/receive syntax
    - Type system message types
    - Runtime mailbox implementation

Each phase validates against SPECIFICATION.md sections and maintains performance invariants.

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