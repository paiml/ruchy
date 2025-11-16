# ruchy score - Unified Quality Scoring

The `ruchy score` command provides comprehensive quality assessment for Ruchy projects, combining multiple analysis dimensions into a single, actionable quality score with detailed breakdowns and improvement recommendations.

## Overview

`ruchy score` analyzes Ruchy codebases across multiple quality dimensions including code style, complexity, security, performance, documentation, and test coverage. It produces both an overall quality score (0.0-1.0) and detailed component scores with specific improvement suggestions.

## Basic Usage

```bash
# Score a single file
ruchy score main.ruchy

# Score entire project
ruchy score .

# Fast scoring (AST-only analysis)
ruchy score --fast .

# Deep analysis with all metrics
ruchy score --deep .

# Compare against baseline
ruchy score --baseline=main .

# Minimum score threshold
ruchy score --min=0.8 .
```

## Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `path` | File or directory to score | Required |
| `--depth <DEPTH>` | Analysis depth (shallow/standard/deep) | `standard` |
| `--fast` | Fast feedback mode (AST-only, <100ms) | `false` |
| `--deep` | Deep analysis for CI (complete, <30s) | `false` |
| `--watch` | Watch mode with progressive refinement | `false` |
| `--explain` | Explain score changes from baseline | `false` |
| `--baseline <REF>` | Baseline branch/commit for comparison | None |
| `--min <SCORE>` | Minimum score threshold (0.0-1.0) | None |
| `--config <PATH>` | Configuration file | `.ruchy-score.toml` |
| `--format <FORMAT>` | Output format (text/json/html) | `text` |
| `--verbose` | Verbose output with detailed metrics | `false` |
| `--output <PATH>` | Output file for score report | None |

## Scoring System

### Overall Quality Scale

| Grade | Score Range | Description |
|-------|-------------|-------------|
| A+ | 0.97-1.00 | Exceptional - Production ready, exemplary code |
| A  | 0.90-0.96 | Excellent - High quality with minor issues |
| B+ | 0.83-0.89 | Good - Solid quality, some improvements needed |
| B  | 0.75-0.82 | Acceptable - Decent quality, several issues |
| C+ | 0.65-0.74 | Below Average - Needs significant improvement |
| C  | 0.50-0.64 | Poor - Many quality issues present |
| D  | 0.25-0.49 | Very Poor - Major quality problems |
| F  | 0.00-0.24 | Failing - Unacceptable quality |

## Quality Dimensions

### 1. Code Style (25% weight)
Measures consistency, formatting, and naming conventions:

```bash
Code Style Assessment:
✅ Consistent formatting          (100%)
✅ Proper naming conventions      (95%)
⚠️  Line length violations        (80%)
✅ Import organization           (100%)
✅ Whitespace consistency        (98%)

Style Score: A- (0.89/1.0)
```

**Measured factors:**
- Naming convention adherence
- Formatting consistency
- Line length and structure
- Comment quality and placement
- Import organization

### 2. Complexity (20% weight)
Analyzes code complexity and maintainability:

```bash
Complexity Assessment:
✅ Function complexity (avg: 4.2)  (85%)
⚠️  Cyclomatic complexity         (75%)
✅ Cognitive load                 (90%)
⚠️  Nesting depth (max: 5)        (70%)
✅ Function length                (88%)

Complexity Score: B+ (0.82/1.0)
```

**Measured factors:**
- Cyclomatic complexity per function
- Cognitive complexity
- Nesting depth
- Function length
- Class/module complexity

### 3. Security (20% weight)
Identifies security vulnerabilities and risks:

```bash
Security Assessment:
✅ No hardcoded secrets           (100%)
✅ Input validation               (95%)
✅ SQL injection prevention       (100%)
✅ Memory safety                  (98%)
✅ Dependency vulnerabilities     (92%)

Security Score: A+ (0.97/1.0)
```

**Measured factors:**
- Hardcoded credentials detection
- Input validation patterns
- SQL injection vulnerabilities
- Buffer overflow risks
- Dependency security status

### 4. Performance (15% weight)
Evaluates performance characteristics:

```bash
Performance Assessment:
✅ Algorithm efficiency           (90%)
⚠️  Memory allocations           (75%)
✅ I/O operations                (85%)
⚠️  String concatenation         (65%)
✅ Collection usage              (88%)

Performance Score: B (0.81/1.0)
```

**Measured factors:**
- Algorithm complexity analysis
- Memory allocation patterns
- I/O efficiency
- String manipulation
- Collection usage patterns

### 5. Documentation (10% weight)
Assesses code documentation quality:

```bash
Documentation Assessment:
⚠️  Public API documentation      (70%)
✅ Inline comments               (85%)
⚠️  Examples and usage           (60%)
✅ Type annotations              (95%)
✅ Error documentation           (88%)

Documentation Score: B (0.80/1.0)
```

**Measured factors:**
- Public API documentation coverage
- Inline comment quality
- Example availability
- Type annotation completeness
- Error handling documentation

### 6. Test Coverage (10% weight)
Evaluates test completeness:

```bash
Test Coverage Assessment:
✅ Line coverage: 85%             (85%)
✅ Branch coverage: 78%           (78%)
⚠️  Function coverage: 92%        (92%)
✅ Integration test coverage      (80%)
✅ Test quality                  (88%)

Test Coverage Score: B+ (0.85/1.0)
```

**Measured factors:**
- Line coverage percentage
- Branch coverage completeness
- Function coverage
- Integration test presence
- Test code quality

## Analysis Depth Levels

### Shallow Analysis (--depth=shallow)
- **Duration:** <10ms
- **Scope:** AST-only analysis
- **Metrics:** Basic style, structure, complexity
- **Use case:** Real-time editor feedback

```bash
$ ruchy score --depth=shallow main.ruchy
=== Quick Quality Assessment ===
File: main.ruchy
Score: B+ (0.84/1.0)
Analysis: Shallow (9ms)

Key Issues:
• Function complexity: 2 functions exceed recommended limits
• Documentation: Missing docs for 3 public functions
```

### Standard Analysis (default)
- **Duration:** 100ms - 2s
- **Scope:** Full static analysis
- **Metrics:** All categories except deep coverage
- **Use case:** Development workflow

```bash
$ ruchy score main.ruchy
=== Quality Score ===
File: main.ruchy
Score: A- (0.87/1.0)
Analysis Depth: standard

📊 Component Scores:
  Code Style:      A  (0.89/1.0)
  Complexity:      B+ (0.82/1.0)  
  Security:        A+ (0.97/1.0)
  Performance:     B  (0.81/1.0)
  Documentation:   B  (0.80/1.0)
  Test Coverage:   B+ (0.85/1.0)

🎯 Improvement Opportunities:
  1. Reduce complexity in parse_expression() (complexity: 12)
  2. Add documentation for public APIs (3 functions)
  3. Optimize string concatenation in format_output()
```

### Deep Analysis (--deep)
- **Duration:** 5s - 30s
- **Scope:** Complete analysis including external tools
- **Metrics:** All categories with maximum precision
- **Use case:** CI/CD pipelines, comprehensive audits

```bash
$ ruchy score --deep .
=== Comprehensive Quality Assessment ===
Project: ruchy-calculator
Score: A- (0.88/1.0)
Analysis: Deep (12.3s)

📈 Detailed Breakdown:

Code Style: A (0.91/1.0)
├─ Naming Conventions:     A+ (0.98/1.0)
├─ Formatting:             A  (0.92/1.0)  
├─ Import Organization:    A+ (0.95/1.0)
├─ Comment Quality:        B+ (0.87/1.0)
└─ Structure:              A- (0.89/1.0)

Complexity: B+ (0.83/1.0)
├─ Cyclomatic:             B  (0.78/1.0)
├─ Cognitive:              A- (0.87/1.0)
├─ Nesting:                B+ (0.85/1.0)
└─ Function Length:        A- (0.89/1.0)

Security: A+ (0.97/1.0)
├─ Vulnerability Scan:     A+ (1.00/1.0)
├─ Input Validation:       A  (0.92/1.0)
├─ Memory Safety:          A+ (0.98/1.0)
└─ Dependency Security:    A  (0.94/1.0)

Performance: B (0.79/1.0)
├─ Algorithm Analysis:     B+ (0.85/1.0)
├─ Memory Patterns:        B- (0.72/1.0)
├─ I/O Efficiency:         B+ (0.83/1.0)
└─ Resource Usage:         B  (0.78/1.0)

Documentation: B (0.78/1.0)
├─ API Coverage:           C+ (0.68/1.0)
├─ Inline Comments:        B+ (0.84/1.0)
├─ Examples:               C  (0.62/1.0)
└─ Type Annotations:       A  (0.94/1.0)

Test Coverage: B+ (0.85/1.0)
├─ Line Coverage:          B+ (0.85/1.0)
├─ Branch Coverage:        B  (0.78/1.0)
├─ Function Coverage:      A- (0.89/1.0)
└─ Integration Tests:      B+ (0.84/1.0)

🎯 Top Improvement Priorities:
  1. Add API documentation (estimated +0.04 score)
  2. Optimize memory allocations (estimated +0.03 score)
  3. Increase branch coverage (estimated +0.02 score)
  4. Reduce function complexity (estimated +0.02 score)
  5. Add usage examples (estimated +0.02 score)

📋 Action Items:
  • Document 8 public functions in src/calculator.ruchy
  • Refactor process_input() to reduce complexity from 15 to <10
  • Add integration tests for error handling paths
  • Review memory allocation patterns in parse_expression()
  • Create examples/ directory with usage demonstrations
```

## Output Formats

### Text Format (Default)
Human-readable output with visual indicators and improvement suggestions.

### JSON Format
Structured data for automation and tooling integration:

```bash
$ ruchy score --format=json main.ruchy
{
  "file": "main.ruchy",
  "overall_score": 0.87,
  "grade": "A-",
  "analysis_depth": "standard",
  "analysis_duration_ms": 145,
  "timestamp": "2025-01-15T10:30:45Z",
  "components": {
    "code_style": {
      "score": 0.89,
      "weight": 0.25,
      "grade": "A",
      "details": {
        "naming_conventions": 0.98,
        "formatting": 0.92,
        "imports": 0.95,
        "comments": 0.87,
        "structure": 0.89
      }
    },
    "complexity": {
      "score": 0.82,
      "weight": 0.20,
      "grade": "B+",
      "details": {
        "cyclomatic": 0.78,
        "cognitive": 0.87,
        "nesting": 0.85,
        "function_length": 0.89
      },
      "issues": [
        {
          "function": "parse_expression",
          "complexity": 12,
          "threshold": 10,
          "severity": "warning"
        }
      ]
    }
  },
  "improvement_suggestions": [
    {
      "priority": 1,
      "category": "documentation",
      "issue": "Missing API documentation",
      "impact": "+0.04 score",
      "effort": "medium",
      "files": ["src/calculator.ruchy"],
      "functions": ["add", "subtract", "multiply"]
    }
  ],
  "metrics": {
    "lines_of_code": 1247,
    "functions": 23,
    "complexity_average": 4.2,
    "test_coverage": 0.85,
    "documentation_coverage": 0.68
  }
}
```

### HTML Format
Rich HTML reports with interactive visualizations:

```bash
$ ruchy score --format=html --output=quality-report.html .
```

Generates interactive HTML with:
- Quality score dashboard
- Trend visualizations  
- Drill-down capabilities
- Exportable metrics
- Printable reports

## Baseline Comparison

Track quality improvements over time by comparing against baselines:

```bash
# Compare against main branch
ruchy score --baseline=main --explain .

# Compare against specific commit
ruchy score --baseline=abc123 --explain .

# Compare against previous release
ruchy score --baseline=v1.0.0 --explain .
```

```bash
=== Quality Score Comparison ===
Current Score: A- (0.87/1.0)
Baseline Score: B+ (0.84/1.0)
Improvement: +0.03 (+3.6%)

📈 Score Changes:
  Code Style:      A  (0.89) ← B+ (0.86) [+0.03]
  Complexity:      B+ (0.82) ← B+ (0.82) [+0.00]
  Security:        A+ (0.97) ← A+ (0.97) [+0.00]  
  Performance:     B  (0.81) ← C+ (0.76) [+0.05]
  Documentation:   B  (0.80) ← C  (0.72) [+0.08]
  Test Coverage:   B+ (0.85) ← B  (0.83) [+0.02]

🎉 Improvements Made:
  ✅ Added documentation for 5 public functions
  ✅ Optimized string concatenation performance
  ✅ Improved test coverage by 2%
  ✅ Fixed formatting inconsistencies

⚠️  Areas Still Needing Work:
  • Function complexity remains high (no change)
  • Memory allocation patterns need optimization
```

## Watch Mode

Monitor quality in real-time during development:

```bash
$ ruchy score --watch .
🔍 Watching for changes in .
Current Score: B+ (0.84/1.0)

[File changed: src/main.ruchy]
Re-analyzing... Score: B+ (0.85/1.0) [+0.01]
↗️ Improvement: Fixed formatting issues

[File changed: src/calculator.ruchy]  
Re-analyzing... Score: A- (0.87/1.0) [+0.02]
↗️ Improvement: Added function documentation

[File changed: tests/integration_test.ruchy]
Re-analyzing... Score: A- (0.88/1.0) [+0.01]  
↗️ Improvement: Increased test coverage
```

## Configuration

### Default Configuration

Create a configuration file:

```bash
ruchy score --init-config
```

Generates `.ruchy-score.toml`:

```toml
[scoring]
# Component weights (must sum to 1.0)
code_style_weight = 0.25
complexity_weight = 0.20
security_weight = 0.20
performance_weight = 0.15
documentation_weight = 0.10
test_coverage_weight = 0.10

[thresholds]
# Minimum scores for each grade
a_plus = 0.97
a = 0.90
b_plus = 0.83
b = 0.75
c_plus = 0.65
c = 0.50

[complexity]
max_cyclomatic = 10
max_cognitive = 15
max_nesting = 4
max_function_lines = 50

[style]
max_line_length = 100
indent_size = 4
enforce_naming = true

[coverage]
min_line_coverage = 0.80
min_branch_coverage = 0.70
min_function_coverage = 0.90

[analysis]
# Analysis timeouts
shallow_timeout_ms = 50
standard_timeout_ms = 2000
deep_timeout_ms = 30000

# Include/exclude patterns
include_patterns = ["*.ruchy"]
exclude_patterns = ["target/", "build/", "*.generated.ruchy"]
```

### Custom Scoring Weights

Adjust component importance for your project:

```toml
[scoring]
# Security-critical project
security_weight = 0.35
complexity_weight = 0.25  
code_style_weight = 0.20
performance_weight = 0.10
documentation_weight = 0.05
test_coverage_weight = 0.05

# Documentation-heavy project  
documentation_weight = 0.30
code_style_weight = 0.25
complexity_weight = 0.20
test_coverage_weight = 0.15
security_weight = 0.05
performance_weight = 0.05
```

## Integration Scenarios

### Development Workflow

```bash
# Quick feedback during coding
ruchy score --fast main.ruchy

# Standard check before commit
ruchy score .

# Pre-push quality gate
ruchy score --min=0.8 .
```

### CI/CD Pipeline

```yaml
# .github/workflows/quality.yml
name: Quality Assessment
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Need history for baseline comparison
      - name: Install Ruchy
        run: cargo install ruchy
      - name: Quality Assessment
        run: |
          # Deep analysis with baseline comparison
          ruchy score --deep --baseline=origin/main --format=json . > quality.json
          
          # Enforce minimum quality
          ruchy score --min=0.75 .
      - name: Upload Quality Report
        uses: actions/upload-artifact@v3
        with:
          name: quality-report
          path: quality.json
```

### Quality Gates

Set up automated quality enforcement:

```bash
#!/bin/bash
# quality-gate.sh

echo "🔍 Running quality assessment..."

# Get current score
SCORE=$(ruchy score --format=json . | jq -r '.overall_score')
MIN_SCORE=0.80

if (( $(echo "$SCORE < $MIN_SCORE" | bc -l) )); then
    echo "❌ Quality gate failed: $SCORE < $MIN_SCORE"
    echo "Please improve code quality before merging."
    exit 1
else
    echo "✅ Quality gate passed: $SCORE >= $MIN_SCORE"
fi
```

## Team Quality Metrics

### Quality Trends Dashboard

```bash
# Generate historical data
ruchy score --format=json . > "quality-$(date +%Y%m%d).json"

# Weekly quality report
ruchy score --baseline=HEAD~7 --explain . > weekly-quality.txt
```

### Team Quality Standards

```toml
# team-standards.toml
[team_quality_standards]
minimum_overall = 0.80
minimum_security = 0.95
minimum_test_coverage = 0.85

[quality_goals]
target_overall = 0.90
target_documentation = 0.85
target_complexity = 0.85

[quality_tracking]
baseline_branch = "main"
report_frequency = "weekly"  
quality_review_threshold = 0.05  # Trigger review if score drops by 5%
```

## Advanced Usage

### Custom Quality Metrics

```bash
# Project-specific scoring
ruchy score --config=custom-weights.toml .

# Security-focused analysis
ruchy score --security-weight=0.5 .

# Performance-critical analysis  
ruchy score --performance-weight=0.4 .
```

### Automated Quality Improvement

```bash
#!/bin/bash
# auto-improve-quality.sh

echo "🔧 Starting automated quality improvements..."

# Fix auto-fixable issues
ruchy lint --fix .

# Update documentation
ruchy doc --generate-missing .

# Run quality assessment
BEFORE=$(ruchy score --format=json . | jq -r '.overall_score')
echo "Score before improvements: $BEFORE"

# After auto-improvements
AFTER=$(ruchy score --format=json . | jq -r '.overall_score')  
echo "Score after improvements: $AFTER"

IMPROVEMENT=$(echo "$AFTER - $BEFORE" | bc -l)
echo "Quality improvement: +$IMPROVEMENT"
```

## Best Practices

### 1. Establish Team Baselines

```bash
# Set project baseline on main branch
git checkout main
ruchy score --format=json . > baseline-quality.json

# All feature branches compare against this
ruchy score --baseline=main --min-improvement=0.00 .
```

### 2. Incremental Quality Improvement

```bash
# Week 1: Focus on style
ruchy score --focus=style .
ruchy lint --fix .

# Week 2: Address complexity  
ruchy score --focus=complexity .
# Manual refactoring based on recommendations

# Week 3: Improve documentation
ruchy score --focus=documentation .
ruchy doc --generate-missing .

# Week 4: Enhance test coverage
ruchy score --focus=coverage .
ruchy test --coverage --min=85 .
```

### 3. Quality-Driven Code Reviews

```bash
# Pre-review quality check
ruchy score --baseline=main feature-branch/

# Review quality impact
ruchy score --baseline=main --explain feature-branch/ > quality-impact.txt
```

### 4. Continuous Quality Monitoring

```bash
# Daily quality tracking (in CI)
ruchy score --format=json . | jq '{date: now, score: .overall_score}' >> quality-history.jsonl

# Weekly trend analysis
cat quality-history.jsonl | jq -s 'sort_by(.date)'
```

## Troubleshooting

### Performance Issues

1. **Slow analysis on large codebases**
   ```bash
   # Use shallow analysis for speed
   ruchy score --fast .
   
   # Exclude large generated files
   ruchy score --exclude="target/,*.generated.ruchy" .
   ```

2. **Timeout during deep analysis**
   ```bash
   # Increase timeout
   ruchy score --deep --timeout=60000 .
   
   # Use standard analysis instead
   ruchy score --depth=standard .
   ```

### Configuration Issues

1. **Custom config not found**
   ```bash
   # Specify config explicitly
   ruchy score --config=.ruchy-score.toml .
   
   # Generate default config
   ruchy score --init-config
   ```

2. **Weights don't sum to 1.0**
   ```
   Error: Component weights must sum to 1.0 (current: 0.95)
   ```
   Solution: Check weight configuration in `.ruchy-score.toml`

## Examples

See the [examples directory](../../examples/) for comprehensive examples demonstrating various Ruchy features.

## See Also

- [`ruchy test`](ruchy-test.md) - Test execution and coverage metrics
- [`ruchy lint`](ruchy-lint.md) - Code quality analysis
- [`ruchy prove`](ruchy-prove.md) - Formal verification