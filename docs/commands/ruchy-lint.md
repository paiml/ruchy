# ruchy lint - Code Quality Analysis

The `ruchy lint` command provides comprehensive static analysis for Ruchy code, detecting issues, enforcing style guidelines, and suggesting improvements with automatic fixes.

## Overview

`ruchy lint` analyzes Ruchy source code to identify potential issues, style violations, security vulnerabilities, and performance problems. It supports automatic fixing, custom rules, and multiple output formats for seamless integration with development workflows.

## Basic Usage

```bash
# Lint a single file
ruchy lint main.ruchy

# Lint all files in project
ruchy lint --all

# Auto-fix issues where possible
ruchy lint --fix main.ruchy

# Strict mode with all rules enabled
ruchy lint --strict main.ruchy

# JSON output for automation
ruchy lint --format=json main.ruchy
```

## Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `file` | The file to lint | Required if not `--all` |
| `--all` | Lint all files in project | `false` |
| `--fix` | Auto-fix issues where possible | `false` |
| `--strict` | Enable all rules (strictest mode) | `false` |
| `--verbose` | Show additional context | `false` |
| `--format <FORMAT>` | Output format (text, json) | `text` |
| `--rules <CATEGORIES>` | Specific rule categories | All enabled |
| `--deny-warnings` | Treat warnings as errors | `false` |
| `--max-complexity <N>` | Maximum function complexity | `10` |
| `--config <PATH>` | Path to config file | `.ruchy-lint.toml` |
| `--init-config` | Generate default config | `false` |

## Rule Categories

`ruchy lint` organizes checks into logical categories:

### Unused Code Detection
```ruchy
// ❌ Unused variable
let unused_var = 42;
let x = 5;

// ❌ Unused import
use std::collections::HashMap;

// ❌ Dead code
fun never_called() {
    println("This function is never used");
}
```

### Style Violations
```ruchy
// ❌ Poor naming convention
let a = 5;
fun DoSomething() { }

// ✅ Good naming convention  
let user_count = 5;
fun do_something() { }

// ❌ Inconsistent spacing
let x=5+3;

// ✅ Consistent spacing
let x = 5 + 3;
```

### Complexity Issues
```ruchy
// ❌ Too complex (cyclomatic complexity > 10)
fun complex_function(x: i32) -> i32 {
    if x > 10 {
        if x > 20 {
            if x > 30 {
                if x > 40 {
                    if x > 50 {
                        return x * 2;
                    } else {
                        return x * 3;
                    }
                } else {
                    return x * 4;
                }
            } else {
                return x * 5;
            }
        } else {
            return x * 6;
        }
    } else {
        return x * 7;
    }
}
```

### Security Issues
```ruchy
// ❌ Potential SQL injection
let query = "SELECT * FROM users WHERE id = " + user_input;

// ❌ Hardcoded secrets
let api_key = "sk-1234567890abcdef";

// ❌ Unsafe operations
let raw_data = read_unsafe_memory(ptr);
```

### Performance Problems
```ruchy
// ❌ Inefficient string concatenation in loop
let result = "";
for item in items {
    result = result + item.to_string();
}

// ✅ Efficient string building
let mut result = String::new();
for item in items {
    result.push_str(&item.to_string());
}
```

## Output Formats

### Text Format (Default)

```bash
$ ruchy lint main.ruchy
⚠ Found 3 issues in main.ruchy
  main.ruchy:5: warning - Variable 'unused_var' is defined but never used
    Suggestion: Remove unused variable 'unused_var'
  main.ruchy:12: error - Function complexity exceeds limit of 10
    Suggestion: Consider breaking this into smaller functions  
  main.ruchy:18: warning - Inconsistent spacing around operator
    Suggestion: Use 'x = 5 + 3' instead of 'x=5+3'

Summary:
  Errors: 1
  Warnings: 2  
  Notes: 0
  Grade: B- (0.72/1.0)
```

### JSON Format

```bash
$ ruchy lint main.ruchy --format=json
[
  {
    "line": 5,
    "column": 5,
    "severity": "warning",
    "rule": "unused-variable",
    "message": "Variable 'unused_var' is defined but never used",
    "suggestion": "Remove unused variable 'unused_var'"
  },
  {
    "line": 12,
    "column": 1,
    "severity": "error", 
    "rule": "complexity",
    "message": "Function complexity exceeds limit of 10",
    "suggestion": "Consider breaking this into smaller functions"
  }
]
```

### Verbose Output

```bash
$ ruchy lint main.ruchy --verbose
🔍 Analyzing main.ruchy with strict rules...

📊 Analysis Results:
  File: main.ruchy
  Lines: 45
  Functions: 3
  Complexity: Average 8.2, Max 15

⚠ Issues Found:

  Line 5: unused-variable (warning)
    Variable 'unused_var' is defined but never used
    
    4 | let x = 5;
    5 | let unused_var = 42;  // ← Issue here
    6 | let y = x + 1;
    
    Help: Remove unused variable declaration
    
  Line 12: complexity (error)
    Function complexity exceeds limit of 10 (actual: 15)
    
    12 | fun complex_calculation(data: Vec<i32>) -> i32 {
    
    Help: Consider extracting helper functions to reduce complexity

📈 Quality Score: B- (0.72/1.0)
  - Code Style: A (0.95/1.0)
  - Complexity: C (0.60/1.0)  
  - Security: A+ (1.00/1.0)
  - Performance: B+ (0.85/1.0)
```

## Auto-Fix Functionality

`ruchy lint` can automatically fix many common issues:

```bash
# Preview fixes without applying
ruchy lint --fix --dry-run main.ruchy

# Apply fixes
ruchy lint --fix main.ruchy
```

### Fixable Issues

Auto-fix supports:
- Unused imports removal
- Spacing and formatting corrections
- Variable name suggestions
- Simple refactoring patterns

```bash
$ ruchy lint --fix main.ruchy
⚠ Found 3 issues in main.ruchy
  main.ruchy:1: warning - Unused import 'HashMap'
  main.ruchy:5: warning - Inconsistent spacing
  main.ruchy:8: warning - Variable could be renamed

→ Attempting auto-fix...
✓ Fixed 2 issues
✗ 1 issue requires manual attention

🔧 Applied fixes:
  - Removed unused import 'HashMap'
  - Fixed spacing: 'x = 5 + 3'

⚠ Manual fixes needed:
  - Line 12: Function complexity too high (requires refactoring)
```

## Configuration

### Default Configuration

Generate a configuration file:

```bash
ruchy lint --init-config
```

This creates `.ruchy-lint.toml`:

```toml
[rules]
# Enable/disable rule categories
unused = true
style = true
complexity = true
security = true
performance = true

[complexity]
# Maximum cyclomatic complexity per function
max_function_complexity = 10
# Maximum cognitive complexity per function  
max_cognitive_complexity = 15
# Maximum lines per function
max_function_lines = 50

[style]
# Naming conventions
function_case = "snake_case"  # snake_case, camelCase
variable_case = "snake_case"
constant_case = "UPPER_CASE"
type_case = "PascalCase"

# Formatting preferences
max_line_length = 100
indent_size = 4
use_tabs = false

[security]
# Security rule severity levels
hardcoded_secrets = "error"
sql_injection = "error"  
unsafe_operations = "warning"

[performance]
# Performance rule thresholds
warn_string_concat_loops = true
warn_inefficient_collections = true
```

### Custom Rules

Define project-specific rules:

```toml
[custom_rules]
# Require documentation for public functions
require_pub_docs = true

# Enforce specific import order
import_order = "std,external,internal"

# Disallow specific patterns
disallow_patterns = [
    "unwrap()",           # Prefer expect() with messages
    "TODO",              # No TODO comments in main branch
    "println!.*debug"    # No debug prints in production
]
```

## Rule-Specific Configuration

### Targeting Specific Rules

```bash
# Run only unused code detection
ruchy lint --rules=unused main.ruchy

# Run style and complexity checks
ruchy lint --rules=style,complexity main.ruchy

# Exclude performance checks
ruchy lint --rules=unused,style,complexity,security main.ruchy
```

### Severity Levels

Configure how different issues are treated:

```bash
# Treat all warnings as errors
ruchy lint --deny-warnings main.ruchy

# Custom complexity limit
ruchy lint --max-complexity=5 main.ruchy
```

## Project-Wide Linting

Lint entire projects efficiently:

```bash
# Lint all Ruchy files in project
ruchy lint --all

# Lint specific directories
ruchy lint src/ tests/ examples/

# Exclude directories
ruchy lint --all --ignore="target/,build/,dist/"
```

### Parallel Processing

Large projects benefit from parallel analysis:

```bash
# Enable parallel linting
ruchy lint --all --parallel

# Control number of threads
ruchy lint --all --parallel --jobs=4
```

## Integration with Development Workflow

### Pre-commit Hooks

`.git/hooks/pre-commit`:
```bash
#!/bin/bash
# Run linting before each commit
ruchy lint --all --format=json > lint-results.json

if [ $? -ne 0 ]; then
    echo "❌ Linting failed. Please fix issues before committing."
    cat lint-results.json | jq '.[] | select(.severity == "error")'
    exit 1
fi

echo "✅ Linting passed"
```

### CI/CD Integration

#### GitHub Actions

```yaml
name: Lint
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Ruchy
        run: cargo install ruchy
      - name: Run Linter
        run: |
          ruchy lint --all --format=json --deny-warnings > lint-results.json
          ruchy lint --all --format=text
      - name: Upload Results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: lint-results
          path: lint-results.json
```

### Editor Integration

#### VS Code Settings

`.vscode/settings.json`:
```json
{
  "ruchy.lint.enable": true,
  "ruchy.lint.onSave": true,
  "ruchy.lint.rules": "all",
  "ruchy.lint.severity": "warning"
}
```

#### Vim/Neovim

```vim
" Auto-lint on save
autocmd BufWritePost *.ruchy !ruchy lint --format=json %
```

## Quality Grading System

`ruchy lint` provides an overall quality grade:

### Grade Scale
- **A+** (0.97-1.00): Exceptional code quality
- **A**  (0.90-0.96): High quality, minor issues
- **B+** (0.83-0.89): Good quality, some improvements needed
- **B**  (0.75-0.82): Acceptable quality, several issues  
- **C+** (0.65-0.74): Below average, many issues
- **C**  (0.50-0.64): Poor quality, significant problems
- **D**  (0.25-0.49): Very poor quality
- **F**  (0.00-0.24): Unacceptable quality

### Grade Components

```bash
📈 Quality Breakdown:
  Overall Grade: B+ (0.85/1.0)
  
  Components:
  • Code Style:    A  (0.92/1.0) - Clean, consistent formatting
  • Complexity:    B  (0.78/1.0) - Some complex functions  
  • Security:      A+ (1.00/1.0) - No security issues detected
  • Performance:   B+ (0.88/1.0) - Minor performance concerns
  • Documentation: C+ (0.68/1.0) - Missing some function docs
```

## Advanced Usage

### Custom Linting Pipelines

```bash
#!/bin/bash
# comprehensive-lint.sh

echo "🔍 Running comprehensive code quality analysis..."

# Stage 1: Basic linting
echo "Stage 1: Basic analysis..."
ruchy lint --all --format=json > basic-lint.json

# Stage 2: Strict analysis
echo "Stage 2: Strict analysis..."  
ruchy lint --all --strict --format=json > strict-lint.json

# Stage 3: Security focus
echo "Stage 3: Security analysis..."
ruchy lint --all --rules=security --deny-warnings

# Stage 4: Performance focus
echo "Stage 4: Performance analysis..."
ruchy lint --all --rules=performance --verbose

# Generate combined report
echo "📊 Generating quality report..."
ruchy score . --include-lint
```

### Baseline Comparisons

Track quality improvements over time:

```bash
# Establish baseline
ruchy lint --all --format=json > baseline-lint.json

# Compare after changes
ruchy lint --all --format=json > current-lint.json
diff-lint baseline-lint.json current-lint.json
```

## Best Practices

### 1. Start with Warnings, Graduate to Errors

```toml
# Development phase - allow warnings
[rules.severity]
unused = "warning"
style = "warning"
complexity = "warning"

# Production phase - strict enforcement  
[rules.severity]
unused = "error"
style = "error"
complexity = "error"
```

### 2. Incremental Adoption

```bash
# Week 1: Fix unused code
ruchy lint --rules=unused --fix --all

# Week 2: Address style issues
ruchy lint --rules=style --fix --all  

# Week 3: Tackle complexity
ruchy lint --rules=complexity --all

# Week 4: Full strict mode
ruchy lint --strict --all
```

### 3. Team Consistency

```bash
# Shared configuration in version control
git add .ruchy-lint.toml
git commit -m "Add shared linting configuration"

# Enforce in CI
ruchy lint --all --config=.ruchy-lint.toml --deny-warnings
```

### 4. Quality Gates

```bash
# Require A- grade or better
ruchy lint --all --min-grade=0.83

# Require zero errors
ruchy lint --all --deny-warnings --max-warnings=0

# Complexity budget
ruchy lint --all --max-complexity=8
```

## Troubleshooting

### Common Issues

1. **Configuration not found**
   ```
   Warning: .ruchy-lint.toml not found, using defaults
   ```
   Solution: Run `ruchy lint --init-config`

2. **Too many false positives**
   ```
   main.ruchy:45: Variable '_result' is unused
   ```
   Solution: Use underscore prefix for intentionally unused variables

3. **Performance issues on large codebases**
   ```
   Linting taking too long...
   ```
   Solution: Use `--parallel` flag or exclude large generated files

### Ignoring Specific Issues

```ruchy
// Ignore next line
#[allow(unused_variable)]
let temporary_debug = 42;

// Ignore entire function
#[allow(complexity)]
fun legacy_complex_function() {
    // Complex legacy code here
}

// Ignore file-wide
#![allow(style)]
```

## Examples

See the [examples directory](../../examples/) for comprehensive examples demonstrating various Ruchy features.

## See Also

- [`ruchy test`](ruchy-test.md) - Test execution and coverage
- [`ruchy prove`](ruchy-prove.md) - Mathematical proof verification
- [`ruchy score`](ruchy-score.md) - Unified quality scoring