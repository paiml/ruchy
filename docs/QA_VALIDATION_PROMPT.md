# QA Validation Prompt: Ruchy Beta Graduation

## Overview

This document provides complete instructions for the QA team to validate the Ruchy compiler for beta graduation. The goal is to verify that all 100 items in the [100-Point QA Validation Checklist](specifications/unified-specifications-2025-next-features-language-stabilization.md#appendix-e-100-point-qa-validation-checklist) are satisfied.

## Prerequisites

1.  **Environment**: Linux or macOS (Windows support is experimental).
2.  **Rust Toolchain**: Stable channel (1.70+).
3.  **Ruchy Repository**: Cloned and up-to-date.
4.  **Dependencies**: `cargo`, `grep`, `timeout` (standard on Linux).

## Setup Steps

```bash
# 1. Clone the repository
git clone https://github.com/ruchy-lang/ruchy.git
cd ruchy

# 2. Build the release binary
cargo build --release

# 3. Add to PATH (temporary)
export PATH="$PWD/target/release:$PATH"

# 4. Verify installation
ruchy --version
```

## Validation Tasks

Perform these 5 validation tasks. For a quick automated check, use the script in Task 1.

### Task 1: Automated 100-Point Check

Run the provided validation script. This covers approximately 80% of the checklist automatically.

```bash
./scripts/qa-validate.sh --full
```

**Record the output score.**

### Task 2: Manual REPL Validation (Items 51-60)

Verify the interactive experience.

```bash
# Start REPL
ruchy repl

# Type the following lines:
let x = 10
let y = 20
fun add(a, b) { a + b }
add(x, y)
# Expected output: 30

# Press Ctrl+D to exit
```

### Task 3: Cross-Compilation / Transpilation (Items 36-50)

Verify transpilation to Rust.

```bash
# Create a test file
echo 'fun main() { println("Hello Beta") }' > test_beta.ruchy

# Transpile
ruchy transpile test_beta.ruchy

# Verify output contains valid Rust code
grep "fn main" test_beta.rs
grep "println!" test_beta.rs
```

### Task 4: Error Handling Check (Items 76-82)

Verify error messages are readable.

```bash
# 1. Syntax Error
ruchy -e "let x = "
# Expected: "Syntax Error" with line number

# 2. Type Error
ruchy -e "let x: i64 = \"string\""
# Expected: "Type mismatch" or similar
```

### Task 5: Performance Smoke Test (Items 91-95)

```bash
# Compile performance check
time ruchy compile examples/01_hello.ruchy
# Expected: < 2 seconds (debug) or < 0.5s (release)
```

## Reporting Template

Copy the template below to `docs/qa/reports/BETA_VALIDATION_YYYY-MM-DD.md` and fill it out.

```markdown
# QA Validation Report: Beta Candidate

**Validator**: [Your Name]
**Date**: [YYYY-MM-DD]
**Version**: [Output of `ruchy --version`]
**OS**: [Output of `uname -a`]

### Automated Validation Score
- **Script Score**: [XX]/100
- **Status**: [PASS/FAIL]

### Manual Validation Notes

| Task | Status | Observations |
|------|--------|--------------|
| 1. Script | [PASS/FAIL] | |
| 2. REPL | [PASS/FAIL] | |
| 3. Transpile | [PASS/FAIL] | |
| 4. Errors | [PASS/FAIL] | |
| 5. Perf | [PASS/FAIL] | |

### Critical Issues Found
1. [Issue ID/Description]
2. ...

### Recommendation
[ ] APPROVE for Beta
[ ] REJECT (Blocking issues identified)
```

## Quick Validation Script

A snippet of `scripts/qa-validate.sh` is provided below for reference. The full script in the repository contains all checks.

```bash
#!/bin/bash
# ... (See scripts/qa-validate.sh)
```

## Contact

- **Development Lead**: [Contact Info]
- **Bug Tracker**: GitHub Issues
- **Specification**: `docs/specifications/unified-specifications-2025-next-features-language-stabilization.md`