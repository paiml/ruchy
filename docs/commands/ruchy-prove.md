# ruchy prove - Mathematical Proof Verification

The `ruchy prove` command provides formal verification capabilities for Ruchy applications, enabling mathematical proofs, property verification, and counterexample generation using advanced SMT solvers.

## Overview

`ruchy prove` is a theorem prover that analyzes Ruchy code to verify mathematical properties, check assertions, and generate counterexamples for false statements. It combines static analysis with SMT solver integration to provide formal guarantees about program correctness.

## Basic Usage

```bash
# Verify proofs in a file
ruchy prove math_proofs.ruchy --check

# Interactive proof session
ruchy prove

# Verify with counterexample generation
ruchy prove assertions.ruchy --check --counterexample

# JSON output for automation
ruchy prove proofs.ruchy --check --format=json

# Verbose proof details
ruchy prove --check --verbose proofs.ruchy
```

## Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `file` | File to verify (optional for REPL) | None (starts REPL) |
| `--backend <BACKEND>` | SMT backend (z3, cvc5, yices2) | `z3` |
| `--ml-suggestions` | Enable ML-powered tactics | `false` |
| `--timeout <MS>` | SMT query timeout in milliseconds | `5000` |
| `--script <PATH>` | Load proof script | None |
| `--export <PATH>` | Export proof to file | None |
| `--check` | Non-interactive verification mode | `false` |
| `--counterexample` | Generate counterexamples | `false` |
| `--verbose` | Show detailed proof output | `false` |
| `--format <FORMAT>` | Output format (text, json, coq, lean) | `text` |

## Supported Proof Types

### Basic Assertions

Verify simple mathematical statements:

```ruchy
// Basic tautologies
assert true
assert 1 + 1 == 2
assert 2 + 2 == 4
assert 3 > 2

// Arithmetic properties
assert 5 * 6 == 30
assert 10 / 2 == 5
assert 7 % 3 == 1
```

### Conditional Properties

Verify properties that hold under specific conditions:

```ruchy
// Conditional assertions
assert if x > 0 then x + 1 > x
assert if a >= b then a - b >= 0
assert if n % 2 == 0 then n + 2 % 2 == 0
```

### Universal Quantification

Prove properties that hold for all values:

```ruchy
// Mathematical identities
assert forall x: i32. x + 0 == x
assert forall x: i32. x * 1 == x
assert forall x: i32, y: i32. x + y == y + x

// Ordering properties
assert forall x: i32. x <= x
assert forall x: i32, y: i32, z: i32. 
    if x <= y && y <= z then x <= z
```

### Existential Quantification

Prove existence of values with specific properties:

```ruphy
// Existence proofs
assert exists x: i32. x > 10
assert exists x: i32, y: i32. x * y == 12
assert exists n: i32. n > 0 && n * n == 16
```

### Function Properties

Verify properties of user-defined functions:

```ruchy
// Function definition
fun factorial(n: i32) -> i32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

// Function properties
assert factorial(0) == 1
assert factorial(1) == 1
assert factorial(4) == 24

// Recursive property
assert forall n: i32. n > 0 => factorial(n) == n * factorial(n - 1)
```

## SMT Backend Configuration

`ruchy prove` supports multiple SMT solvers for different proof strategies:

### Z3 (Default)
```bash
ruchy prove --backend=z3 proofs.ruchy --check
```

**Strengths:**
- Excellent arithmetic reasoning
- Strong quantifier handling
- Good performance on most problems

### CVC5
```bash
ruchy prove --backend=cvc5 proofs.ruchy --check
```

**Strengths:**
- Advanced string reasoning
- Bit-vector operations
- Inductive data types

### Yices2
```bash
ruchy prove --backend=yices2 proofs.ruchy --check
```

**Strengths:**
- Fast linear arithmetic
- Boolean satisfiability
- Optimization queries

## Output Formats

### Text Format (Default)

```bash
$ ruchy prove math_facts.ruchy --check --verbose
🔍 Starting interactive prover with backend: z3
⚙️  Configuration:
  SMT Backend: Z3
  Timeout: 5000ms
  ML Suggestions: false
  Counterexamples: false

✓ Checking proofs in math_facts.ruchy...
Found 4 assertions to verify
  1: true
  2: 2 + 2 == 4  
  3: 1 + 1 == 2
  4: 3 > 2

✅ All 4 proofs verified successfully
  ✅ Proof 1: true (0ms)
  ✅ Proof 2: 2 + 2 == 4 (0ms) 
  ✅ Proof 3: 1 + 1 == 2 (0ms)
  ✅ Proof 4: 3 > 2 (0ms)
```

### JSON Format

```bash
$ ruchy prove math_facts.ruchy --check --format=json
{
  "file": "math_facts.ruchy",
  "status": "verified", 
  "total": 4,
  "passed": 4,
  "failed": 0,
  "proofs": [
    {
      "assertion": "true",
      "is_verified": true,
      "counterexample": null,
      "error": null,
      "verification_time_ms": 0
    },
    {
      "assertion": "2 + 2 == 4",
      "is_verified": true,
      "counterexample": null,
      "error": null, 
      "verification_time_ms": 0
    }
  ]
}
```

### Counterexample Generation

When assertions fail, `ruchy prove` can generate concrete counterexamples:

```bash
$ ruchy prove false_claims.ruchy --check --counterexample
✓ Checking proofs in false_claims.ruchy...
Found 2 assertions to verify
  1: 2 + 2 == 5
  2: exists x: i32. x > x

❌ 2 of 2 proofs failed verification
  ❌ Proof 1: 2 + 2 == 5
     Counterexample: 2 + 2 = 4, not 5
  ❌ Proof 2: exists x: i32. x > x  
     Counterexample: No integer x satisfies x > x
```

## Interactive Proof Mode

Start an interactive theorem proving session:

```bash
$ ruchy prove
🚀 Starting Ruchy Interactive Prover
Type 'help' for available commands

prove> prove 2 + 2 == 4
✅ Proof successful: 2 + 2 == 4

prove> prove 2 + 2 == 5  
❌ Proof failed: 2 + 2 == 5
Counterexample: 2 + 2 evaluates to 4, not 5

prove> help
Available commands:
  prove <statement>     - Verify a mathematical statement
  assume <statement>    - Add assumption to context
  show                  - Display current proof context
  tactics               - List available proof tactics
  goals                 - Show active proof goals
  quit                  - Exit prover

prove> quit
Goodbye!
```

### Interactive Commands

#### Basic Proof Commands
- `prove <statement>` - Attempt to verify a statement
- `assume <statement>` - Add assumption to proof context
- `show` - Display current assumptions and goals

#### Advanced Tactics
- `simplify` - Simplify current goal using algebraic rules
- `split` - Split conjunctive goals into subgoals  
- `induction <var>` - Apply mathematical induction
- `contradiction` - Derive contradiction from false assumption

#### Context Management
- `goals` - Show all active proof goals
- `context` - Show current assumptions
- `clear` - Clear proof context
- `save <name>` - Save current proof state
- `load <name>` - Load saved proof state

## Advanced Proof Patterns

### Mathematical Induction

```ruchy
// Prove sum formula: 1 + 2 + ... + n = n*(n+1)/2
theorem sum_formula(n: i32): i32 
  requires n >= 0
  ensures result == n * (n + 1) / 2
{
    if n == 0 {
        0  // Base case
    } else {
        n + sum_formula(n - 1)  // Inductive step
    }
}

// Inductive proof
assert forall n: i32. n >= 0 => 
    sum_formula(n) == n * (n + 1) / 2
```

### Contract-Based Verification

```ruchy
// Function with preconditions and postconditions
fun safe_divide(a: i32, b: i32) -> i32
  requires b != 0
  ensures result * b == a  // (for exact division)
{
    a / b
}

// Contract verification
assert safe_divide(10, 2) == 5
assert safe_divide(15, 3) == 5
// assert safe_divide(10, 0)  // Would fail precondition
```

### Loop Invariants

```ruchy
fun array_sum(arr: Vec<i32>) -> i32 
  ensures result >= 0  // Assuming non-negative elements
{
    let mut sum = 0;
    let mut i = 0;
    
    // Loop invariant: sum equals sum of first i elements
    while i < arr.len() 
      invariant 0 <= i <= arr.len()
      invariant sum == sum_of_slice(arr, 0, i)
    {
        sum = sum + arr[i];
        i = i + 1;
    }
    
    sum
}
```

## Proof Script Automation

Create reusable proof scripts for complex verification:

### Basic Proof Script

```ruchy
// proofs/arithmetic_facts.ruchy
script "Arithmetic Properties" {
    // Commutativity
    prove forall x: i32, y: i32. x + y == y + x;
    
    // Associativity  
    prove forall x: i32, y: i32, z: i32. 
        (x + y) + z == x + (y + z);
    
    // Identity
    prove forall x: i32. x + 0 == x;
    
    // Distributivity
    prove forall x: i32, y: i32, z: i32.
        x * (y + z) == x * y + x * z;
}
```

Load and execute scripts:

```bash
ruchy prove --script=proofs/arithmetic_facts.ruchy
```

### Advanced Script Features

```ruchy
script "Advanced Number Theory" {
    // Define custom tactics
    tactic solve_linear := {
        simplify;
        linear_arithmetic;
    };
    
    // Use custom tactic
    prove forall x: i32. 2 * x + 1 > 2 * x by solve_linear;
    
    // Conditional proofs
    section "Prime Numbers" {
        assume forall p: i32. prime(p) => p > 1;
        
        prove forall p: i32. prime(p) => p >= 2;
        prove exists p: i32. prime(p) && p > 10;
    };
}
```

## Performance and Timeout Handling

### Timeout Configuration

Set appropriate timeouts for different proof complexities:

```bash
# Quick verification (100ms timeout)
ruchy prove --timeout=100 simple_facts.ruchy --check

# Standard verification (5 second timeout)  
ruchy prove --timeout=5000 medium_proofs.ruchy --check

# Deep verification (30 second timeout)
ruchy prove --timeout=30000 complex_theorems.ruchy --check
```

### Performance Optimization

```ruchy
// Efficient: Use specific types and bounds
assert forall x: u8. x + 1 > x  // u8 has finite domain

// Inefficient: Unbounded quantification
assert forall x: i32. x + 1 > x  // i32 has large domain

// Efficient: Bounded quantification
assert forall x: i32. 0 <= x <= 100 => x + 1 > x

// Efficient: Structured reasoning
assert forall x: i32, y: i32. 
    (x > 0 && y > 0) => (x + y > x)
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Formal Verification
on: [push, pull_request]

jobs:
  prove:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Ruchy  
        run: cargo install ruchy
      - name: Install Z3
        run: sudo apt-get install z3
      - name: Verify Proofs
        run: |
          ruchy prove src/ --check --format=json > proof-results.json
          ruchy prove --check --timeout=10000 src/
      - name: Upload Results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: proof-results
          path: proof-results.json
```

### Exit Codes

- `0` - All proofs verified successfully
- `1` - One or more proofs failed verification  
- `2` - Prover error (timeout, solver issue, parse error)

## Best Practices

### 1. Structure Proofs Hierarchically

```ruchy
// Start with simple facts
assert 1 + 1 == 2;
assert 2 + 2 == 4;

// Build to more complex properties
assert forall x: i32. x + 1 > x;

// Culminate in high-level theorems
theorem arithmetic_progression_sum(n: i32, d: i32) -> i32
  requires n >= 0
  ensures result == n * (2 * first + (n - 1) * d) / 2
{
    // Proof implementation
}
```

### 2. Use Meaningful Assertion Messages

```ruchy
// Good: Descriptive assertions
assert 2 + 2 == 4, "Basic arithmetic should work";
assert factorial(5) == 120, "5! should equal 120";

// Better: Include mathematical context
assert forall n: i32. n >= 0 => factorial(n) >= 1, 
    "Factorial is always positive for non-negative inputs";
```

### 3. Modular Proof Development

```ruchy
// core_arithmetic.ruchy
script "Core Arithmetic" {
    prove forall x: i32. x + 0 == x;
    prove forall x: i32, y: i32. x + y == y + x;
}

// advanced_algebra.ruchy  
use core_arithmetic;

script "Advanced Algebra" {
    // Build on core arithmetic
    prove forall a: i32, b: i32, c: i32.
        (a + b) * c == a * c + b * c;
}
```

### 4. Performance-Conscious Proof Design

```ruchy
// Efficient: Specific bounds
assert forall x: i32. 0 <= x <= 1000 => x * x >= 0;

// Inefficient: Unbounded quantification  
assert forall x: i32. x * x >= 0;  // Takes longer to verify

// Efficient: Structured cases
assert forall x: i32. 
    (x >= 0 => x * x >= 0) && 
    (x < 0 => x * x >= 0);
```

## Troubleshooting

### Common Issues

1. **Proof timeout**
   ```
   ❌ Proof failed: Verification timeout after 5000ms
   ```
   Solutions:
   - Increase timeout: `--timeout=10000`
   - Simplify assertion structure
   - Add more specific bounds/assumptions

2. **Solver not found**
   ```
   Error: Z3 solver not found in PATH
   ```
   Solutions:
   - Install Z3: `sudo apt-get install z3`
   - Try different backend: `--backend=cvc5`
   - Check solver installation

3. **Unsupported proof pattern**
   ```
   ❌ Error: Unknown assertion pattern - verification not implemented
   ```
   Solutions:
   - Use supported assertion patterns
   - File feature request for new patterns
   - Write proof script with custom tactics

### Debug Mode

Enable detailed debugging:

```bash
RUST_LOG=debug ruchy prove --verbose proofs.ruchy --check
```

This shows:
- SMT solver queries
- Intermediate proof steps
- Solver response details
- Timing information

## Integration with Other Tools

Combine `ruchy prove` with other quality tools:

```bash
# Complete verification pipeline
ruchy lint src/          # Static analysis
ruchy test tests/        # Dynamic testing  
ruchy prove src/         # Formal verification
ruchy score .           # Quality assessment
```

## Examples

### Complete Example: Verified Sorting Algorithm

```ruchy
// verified_sort.ruchy

// Predicate: array is sorted
predicate sorted(arr: Vec<i32>) -> bool {
    forall i: usize, j: usize. 
        0 <= i < j < arr.len() => arr[i] <= arr[j]
}

// Verified insertion sort
fun insertion_sort(arr: Vec<i32>) -> Vec<i32>
  ensures sorted(result)
  ensures permutation(arr, result)
{
    let mut result = arr;
    let mut i = 1;
    
    while i < result.len()
      invariant sorted(slice(result, 0, i))
      invariant permutation(arr, result)
    {
        let key = result[i];
        let mut j = i;
        
        while j > 0 && result[j-1] > key
          invariant j <= i
          invariant forall k: usize. j < k <= i => result[k] >= key
        {
            result[j] = result[j-1];
            j = j - 1;
        }
        
        result[j] = key;
        i = i + 1;
    }
    
    result
}

// Verification tests
assert sorted(insertion_sort([1, 2, 3]));
assert sorted(insertion_sort([3, 2, 1]));
assert sorted(insertion_sort([1]));
assert sorted(insertion_sort([]));

// Property-based verification
assert forall arr: Vec<i32>. sorted(insertion_sort(arr));
```

## See Also

- [`ruchy test`](ruchy-test.md) - Dynamic testing and coverage
- [`ruchy lint`](ruchy-lint.md) - Static code analysis
- [`ruchy score`](ruchy-score.md) - Unified quality scoring