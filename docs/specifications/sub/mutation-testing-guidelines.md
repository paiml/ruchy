# Sub-spec: Mutation Testing — Test Writing Guidelines to Kill Mutants

**Parent:** [MUTATION_TESTING.md](../MUTATION_TESTING.md) Section 8

---
## Test Writing Guidelines to Kill Mutants

### 1. Use Exact Assertions

❌ **Bad** (doesn't kill arithmetic mutants):
```rust
#[test]
fn test_addition() {
    let result = eval("2 + 3");
    assert!(result.is_ok());  // Passes even if + changed to *
}
```

✅ **Good** (kills arithmetic mutants):
```rust
#[test]
fn test_addition() {
    assert_eq!(eval("2 + 3"), Value::Integer(5));  // Fails if + → -
    assert_eq!(eval("10 + 7"), Value::Integer(17)); // Fails if + → *
}
```

### 2. Test All Match Arms

❌ **Bad** (doesn't kill match arm deletions):
```rust
#[test]
fn test_literals() {
    assert!(eval("42").is_ok());  // Only tests one arm
}
```

✅ **Good** (kills match arm deletions):
```rust
#[test]
fn test_all_literal_types() {
    assert_eq!(eval("42"), Value::Integer(42));
    assert_eq!(eval("3.14"), Value::Float(3.14));
    assert_eq!(eval("true"), Value::Bool(true));
    assert_eq!(eval("\"hi\""), Value::String("hi".into()));
    assert_eq!(eval("nil"), Value::Nil);
}
```

### 3. Test Both Branches

❌ **Bad** (doesn't kill boolean operator mutants):
```rust
#[test]
fn test_and() {
    assert_eq!(eval("true && true"), Value::Bool(true));  // Doesn't distinguish && from ||
}
```

✅ **Good** (kills boolean operator mutants):
```rust
#[test]
fn test_and_operator() {
    assert_eq!(eval("true && true"), Value::Bool(true));
    assert_eq!(eval("true && false"), Value::Bool(false));  // Kills &&→||
    assert_eq!(eval("false && true"), Value::Bool(false));
    assert_eq!(eval("false && false"), Value::Bool(false));
}
```

### 4. Test Boundary Conditions

❌ **Bad** (doesn't kill comparison mutants):
```rust
#[test]
fn test_less_than() {
    assert_eq!(eval("5 < 10"), Value::Bool(true));  // Doesn't test boundary
}
```

✅ **Good** (kills comparison mutants):
```rust
#[test]
fn test_less_than_exact() {
    assert_eq!(eval("5 < 10"), Value::Bool(true));   // Normal case
    assert_eq!(eval("10 < 10"), Value::Bool(false)); // Kills <→<=
    assert_eq!(eval("15 < 10"), Value::Bool(false)); // Kills <→>
}
```

### 5. Verify Non-Default Values

❌ **Bad** (doesn't kill Default::default() mutants):
```rust
#[test]
fn test_parser() {
    let ast = parse("let x = 42");
    assert!(ast.is_ok());  // Passes even if AST is Default
}
```

✅ **Good** (kills Default::default() mutants):
```rust
#[test]
fn test_parser_structure() {
    let ast = parse("let x = 42").unwrap();

    match &ast.kind {
        ExprKind::Let { name, value, .. } => {
            assert_eq!(name, "x");
            assert!(matches!(value.kind, ExprKind::Literal(Literal::Integer(42))));
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}
```

### 6. Test Side Effects

❌ **Bad** (doesn't kill empty function mutants):
```rust
#[test]
fn test_print() {
    eval("println(\"hello\")").unwrap();  // Doesn't verify output
}
```

✅ **Good** (kills empty function mutants):
```rust
#[test]
fn test_print_output() {
    let output = capture_stdout(|| {
        eval("println(\"hello\")").unwrap();
    });
    assert_eq!(output, "hello\n");  // Verifies println actually prints
}
```

## Integration with Quality Gates

### Pre-commit Hook

Add mutation testing to quality gates:

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Quick mutation test on changed files
changed_files=$(git diff --cached --name-only --diff-filter=ACM | grep "\.rs$")

if [ -n "$changed_files" ]; then
    for file in $changed_files; do
        echo "🧬 Running mutation tests on $file..."
        cargo mutants --file "$file" --check || {
            echo "❌ BLOCKED: Mutation tests failed for $file"
            echo "Run: cargo mutants --file $file --missed"
            exit 1
        }
    done
fi
```

### CI/CD Pipeline

Add to `.github/workflows/quality.yml`:

```yaml
mutation-testing:
  name: Mutation Testing
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Install cargo-mutants
      run: cargo install cargo-mutants

    - name: Run mutation tests
      run: |
        cargo mutants --workspace --json --output mutants.json

    - name: Check mutation kill rate
      run: |
        KILL_RATE=$(jq '.summary.kill_rate' mutants.json)
        if (( $(echo "$KILL_RATE < 0.90" | bc -l) )); then
          echo "❌ Mutation kill rate $KILL_RATE below 90% threshold"
          exit 1
        fi
        echo "✅ Mutation kill rate: $KILL_RATE"

    - name: Upload mutation report
      uses: actions/upload-artifact@v4
      with:
        name: mutation-report
        path: mutants.json
```

