# Enforced YAML Grammar Specification and Property Validation

**Version**: 1.0.0
**Status**: SPEC-001 Active
**Created**: 2025-11-11

## 1. Executive Summary

This specification defines a **canonical YAML-based grammar** for the Ruchy language with **automated property-based validation** enforced via pre-commit hooks (<30s execution time).

### Goals
1. **Single Source of Truth**: YAML grammar file is the canonical specification
2. **Automated Validation**: Property tests verify parser implements 100% of grammar
3. **Pre-Commit Enforcement**: Fast (<30s) validation prevents grammar drift
4. **Completion Reporting**: Show exactly which grammar components are implemented
5. **Developer Experience**: `cargo run --example grammar_validator` with `--full` flag

## 2. System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Canonical Grammar                        │
│              grammar/ruchy-grammar.yaml                      │
│  (Validated by serde, schema-checked, version-controlled)   │
└──────────────────────┬──────────────────────────────────────┘
                       │
          ┌────────────┴────────────┐
          │                         │
          ▼                         ▼
┌──────────────────┐      ┌──────────────────┐
│  Parser           │      │  Property Tests  │
│  Implementation   │◄─────┤  (Validation)    │
│  src/frontend/    │      │  examples/       │
└──────────────────┘      └──────────────────┘
          │                         │
          │                         ▼
          │              ┌──────────────────────┐
          │              │  Completion Report   │
          │              │  ✅ 85% Implemented  │
          │              │  ❌ Missing: X, Y, Z │
          └──────────────┴──────────────────────┘
```

## 3. YAML Grammar Schema

### 3.1 File Structure

```yaml
version: "1.0.0"
language: "ruchy"
updated: "2025-11-11"

meta:
  description: "Canonical Ruchy language grammar"
  ll_k: 2  # Maximum lookahead
  production_count: 41

lexical:
  keywords: [...]
  operators: [...]
  literals: {...}

grammar:
  program:
    rule: "item*"
    description: "Top-level program structure"
    productions:
      - name: "import_stmt"
        implemented: true
        test_coverage: 95
      - name: "function_decl"
        implemented: true
        test_coverage: 100

  expressions:
    - name: "if_expr"
      rule: "'if' expr block ('else' (if_expr | block))?"
      precedence: null
      associativity: null
      implemented: true
      test_cases: ["basic_if", "if_else", "if_else_if"]

  types:
    - name: "simple_type"
      rule: "identifier ('::' identifier)*"
      implemented: true

  patterns:
    - name: "literal_pattern"
      rule: "literal"
      implemented: true

validation:
  property_tests:
    - name: "all_keywords_parsed"
      generator: "arbitrary_keyword"
      property: "parser accepts all keywords"
    - name: "all_operators_parsed"
      generator: "arbitrary_operator"
      property: "parser accepts all operators"
```

### 3.2 Schema Validation

The YAML file must pass:
- **Serde deserialization** (type safety)
- **JSON Schema validation** (structure)
- **Completeness check** (all productions have implementation status)

### 3.3 Three-Mode Validation (CRITICAL)

**SACRED RULE**: `implemented: true` means feature works in ALL THREE MODES:

1. **Interpreter Mode** (`ruchy run`, `ruchy -e`) - Runtime execution
2. **Transpile Mode** (`ruchy transpile`) - Ruchy → Rust code generation
3. **Compile Mode** (`ruchy compile`) - Full compilation to native binary

**Validation Schema**:
```yaml
expressions:
  if_expr:
    rule: "'if' expr block ('else' (if_expr | block))?"
    implemented: true
    modes:
      interpreter: true    # Works in runtime eval
      transpile: true      # Generates valid Rust
      compile: true        # Compiles to binary
    test_coverage:
      interpreter: 100     # % of interpreter tests passing
      transpile: 100       # % of transpile tests passing
      compile: 100         # % of compile tests passing
```

**Failure Modes** (NOT ACCEPTABLE):
- ❌ "Works in interpreter but not transpile" → `implemented: false` until ALL modes work
- ❌ "Transpiles but doesn't compile" → `implemented: false` until rustc succeeds
- ❌ "Parses but runtime fails" → `implemented: false` until execution works

**Quality Gate**: A feature is ONLY `implemented: true` when:
- ✅ Parser accepts syntax
- ✅ Interpreter executes correctly
- ✅ Transpiler generates valid Rust
- ✅ rustc compiles without errors
- ✅ Compiled binary executes correctly

## 4. Property Test Implementation

### 4.1 Test Categories

```rust
#[proptest]
fn test_all_keywords_in_grammar(#[strategy(arbitrary_keyword())] kw: String) {
    // For each keyword in YAML grammar
    // Property: Parser must accept it in valid contexts
    let code = format!("let {} = 42", kw);
    assert!(parser.accepts_keyword(&kw));
}

#[proptest]
fn test_all_operators_in_grammar(#[strategy(arbitrary_operator())] op: String) {
    // For each operator in YAML grammar
    // Property: Parser must correctly parse it
    let code = format!("1 {} 2", op);
    assert!(parser.parses_operator(&op));
}

#[proptest]
fn test_all_expressions_parseable(#[strategy(arbitrary_expr())] expr_type: String) {
    // For each expression type in YAML grammar
    // Property: Parser generates correct AST
    let examples = grammar.get_test_cases(&expr_type);
    for example in examples {
        assert!(parser.parse(&example).is_ok());
    }
}
```

### 4.2 Validation Flow

```
1. Load grammar/ruchy-grammar.yaml
2. For each grammar component:
   a. Check implementation status
   b. Run property tests
   c. Measure test coverage
3. Generate completion report
4. Exit code 0 if ≥target%, 1 otherwise
```

## 5. Completion Reporting

### 5.1 Summary Output (Default)

```
🔍 Ruchy Grammar Validation Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Implementation: 85.4% (35/41 components)
✅ Test Coverage:  92.7% (38/41 tested)
⚠️  Property Tests: 73.2% (30/41 validated)

Overall Score: B+ (85.4%)

⏱️  Execution Time: 12.3s
```

### 5.2 Full Report (`--full` flag)

```
📊 Detailed Grammar Validation Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

LEXICAL (100%)
✅ keywords (23/23)     [test: ✅ | property: ✅]
✅ operators (28/28)    [test: ✅ | property: ✅]
✅ literals  (5/5)      [test: ✅ | property: ✅]

GRAMMAR (82.9%)
✅ program              [test: ✅ | property: ✅]
✅ import_stmt          [test: ✅ | property: ✅]
✅ function_decl        [test: ✅ | property: ✅]
❌ effect_decl          [test: ❌ | property: ❌]
   → Missing: Effect system not implemented
   → Action: Implement src/frontend/parser/effects.rs
   → Est. Effort: 3-5 hours

EXPRESSIONS (90.5%)
✅ if_expr              [test: ✅ | property: ✅]
✅ match_expr           [test: ✅ | property: ✅]
⚠️  pipeline_expr       [test: ✅ | property: ⚠️]
   → Partial: Basic pipelines work, chaining has edge cases
   → Action: Add property tests for complex chains
   → Coverage: src/frontend/parser/expressions.rs:234-267

...

❌ MISSING COMPONENTS (6/41):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. effect_decl - Effect system parser
2. handler_expr - Effect handler expressions
3. refined_type - Refinement types
4. lazy_expr - Lazy evaluation
5. async_block - Async block expressions
6. macro_call - Macro system

📁 Files to Create:
- src/frontend/parser/effects.rs
- src/frontend/parser/macros.rs
- tests/grammar_effects.rs

⏱️  Total Time: 12.3s
```

## 6. Pre-Commit Hook Integration

### 6.1 Hook Script (`.git/hooks/pre-commit`)

```bash
#!/bin/bash
# Grammar validation hook - must complete in <30s

echo "🔍 Validating grammar implementation..."

# Run validator with fast mode (summary only)
if ! timeout 30 cargo run --example grammar_validator --release 2>&1 | tee /tmp/grammar_validation.log; then
    echo "❌ Grammar validation failed"
    echo "Run 'cargo run --example grammar_validator --full' for details"
    exit 1
fi

# Check minimum threshold (85%)
if grep -q "Overall Score:.*[0-7][0-9]" /tmp/grammar_validation.log; then
    echo "❌ Grammar completion below 85% threshold"
    exit 1
fi

echo "✅ Grammar validation passed"
```

### 6.2 Performance Requirements

- **Fast Path** (default): <15s (summary only)
- **Full Report** (`--full`): <30s (detailed analysis)
- **CI Mode** (`--ci`): <30s (exit code only, no colors)

### 6.3 Bypass Mechanism

```bash
# Emergency bypass (requires explicit justification)
SKIP_GRAMMAR_VALIDATION=1 git commit -m "..."
```

## 7. Implementation Plan (EXTREME TDD)

### Phase 1: RED - Create Failing Tests

```rust
#[test]
fn test_yaml_grammar_loads() {
    let grammar = load_grammar("grammar/ruchy-grammar.yaml");
    assert!(grammar.is_ok(), "Grammar must be valid YAML");
}

#[test]
fn test_all_keywords_documented() {
    let grammar = load_grammar("grammar/ruchy-grammar.yaml").unwrap();
    let lexer_keywords = get_lexer_keywords();

    for kw in lexer_keywords {
        assert!(grammar.has_keyword(&kw), "Keyword '{}' missing from grammar", kw);
    }
}

#[test]
fn test_validator_reports_completion() {
    let report = run_validator();
    assert!(report.contains("Implementation:"));
    assert!(report.contains("%"));
}
```

### Phase 2: GREEN - Minimal Implementation

1. Create `grammar/ruchy-grammar.yaml` (extract from `grammer.md`)
2. Create `examples/grammar_validator.rs` (load + validate)
3. Implement completion calculation
4. Add summary output

### Phase 3: REFACTOR - Quality & Performance

1. Optimize for <15s execution
2. Add caching for grammar file
3. Parallel property test execution
4. Add color output with `termcolor`

### Phase 4: VALIDATE - Integration Testing

1. Run against full codebase
2. Verify pre-commit hook works
3. Test with intentionally broken grammar
4. Measure actual execution time

## 8. Success Criteria

✅ **Correctness**:
- YAML grammar matches `grammer.md` 100%
- All implemented features marked `implemented: true`
- All property tests pass

✅ **Performance**:
- Default mode: <15s
- Full report: <30s
- CI mode: <20s

✅ **Usability**:
- `cargo run --example grammar_validator` shows summary
- `--full` flag shows detailed breakdown
- `--missing` flag shows only unimplemented features
- `--json` flag outputs machine-readable report

✅ **Automation**:
- Pre-commit hook enforces validation
- CI/CD pipeline includes grammar check
- Violations block merge

## 9. Future Enhancements

### 9.1 Grammar Evolution Tracking

```yaml
history:
  - version: "1.0.0"
    date: "2025-11-11"
    changes: "Initial grammar specification"
  - version: "1.1.0"
    date: "2025-12-01"
    changes: "Added effect system"
    backwards_compatible: false
```

### 9.2 Cross-Language Grammar Validation

```bash
# Validate TypeScript grammar matches Ruchy
cargo run --example grammar_validator --compare typescript_grammar.yaml
```

### 9.3 Auto-Generated Parser Tests

```rust
// From grammar YAML, generate parser tests automatically
#[grammar_test(expr = "if_expr")]
fn test_if_expr() {
    // Generated from grammar.yaml test_cases
}
```

## 10. References

- **Grammar Source**: `docs/architecture/grammer.md`
- **Specification**: `docs/SPECIFICATION.md`
- **Parser Impl**: `src/frontend/parser/`
- **Property Tests**: `tests/properties/`

## 11. Maintenance

- **Owner**: Parser Team
- **Review Frequency**: Every sprint
- **Update Trigger**: Any parser change
- **Validation**: Automated via pre-commit + CI

---

**Document Status**: ACTIVE
**Last Updated**: 2025-11-11
**Next Review**: 2025-11-18
