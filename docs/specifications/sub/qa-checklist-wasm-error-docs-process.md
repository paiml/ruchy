# Sub-spec: QA Beta Checklist — Categories 8-10 (WASM, ERROR, DOCS) & QA Process

**Parent:** [100-point-qa-beta-checklist-4.0-beta.md](../100-point-qa-beta-checklist-4.0-beta.md)

---

## Category 8: WASM (10 Checkpoints)

### [QA-081] WASM Compilation
- **Description**: Verify WASM output
- **Steps**:
  1. Run `ruchy wasm file.ruchy`
  2. Verify .wasm file generated
  3. Verify file is valid WASM
- **Expected**: Valid WASM binary
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-082] Browser Execution
- **Description**: Verify WASM runs in browser
- **Steps**:
  1. Load WASM in Chrome/Firefox
  2. Execute exported functions
  3. Verify correct results
- **Expected**: WASM executes in browsers
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-083] WASM Size
- **Description**: Verify reasonable WASM size
- **Steps**:
  1. Compile hello world to WASM
  2. Measure file size
- **Expected**: < 1MB for simple programs
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-084] JS Interop
- **Description**: Verify JavaScript interoperability
- **Steps**:
  1. Call WASM function from JS
  2. Pass data to WASM
  3. Receive result in JS
- **Expected**: Seamless JS integration
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-085] WASM Memory
- **Description**: Verify memory handling
- **Steps**:
  1. Allocate large arrays in WASM
  2. Verify no memory leaks
  3. Test memory limits
- **Expected**: Stable memory behavior
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-086] WASM Strings
- **Description**: Verify string handling in WASM
- **Steps**:
  1. Pass string from JS to WASM
  2. Return string from WASM to JS
  3. Verify encoding correct
- **Expected**: UTF-8 strings work correctly
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-087] WASM Performance
- **Description**: Verify WASM performance
- **Steps**:
  1. Run fibonacci(40) in WASM
  2. Compare to native execution
- **Expected**: Within 2x of native speed
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-088] WASM Playground
- **Description**: Verify online playground
- **Steps**:
  1. Visit ruchy playground (if deployed)
  2. Enter code, run
  3. Verify output
- **Expected**: Working web playground
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-089] WASM Async
- **Description**: Verify async in WASM
- **Steps**:
  1. Test async functions in WASM
  2. Verify promises work with JS
- **Expected**: Async/await works in WASM
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-090] WASM Error Handling
- **Description**: Verify errors propagate to JS
- **Steps**:
  1. Trigger error in WASM
  2. Verify JS can catch it
  3. Verify error message readable
- **Expected**: Errors catchable in JS
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## Category 9: ERROR (5 Checkpoints)

### [QA-091] Syntax Error Messages
- **Description**: Verify helpful syntax errors
- **Steps**:
  1. Introduce syntax error (missing brace)
  2. Verify error shows line number
  3. Verify error suggests fix
- **Expected**: Clear, actionable error messages
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-092] Type Error Messages
- **Description**: Verify type mismatch errors
- **Steps**:
  1. Pass string where int expected
  2. Verify error shows types involved
  3. Verify error shows location
- **Expected**: Clear type error messages
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-093] Runtime Error Messages
- **Description**: Verify runtime error handling
- **Steps**:
  1. Cause index out of bounds
  2. Cause null pointer access
  3. Verify stack trace shown
- **Expected**: Stack trace with source locations
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-094] Error Recovery
- **Description**: Verify parser continues after errors
- **Steps**:
  1. File with multiple syntax errors
  2. Verify all errors reported (not just first)
- **Expected**: Multiple errors reported
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-095] Exit Codes
- **Description**: Verify correct exit codes
- **Steps**:
  1. Run successful program, check exit code 0
  2. Run failing program, check exit code non-zero
- **Expected**: Standard exit code conventions
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## Category 10: DOCS (5 Checkpoints)

### [QA-096] README Accuracy
- **Description**: Verify README is current
- **Steps**:
  1. Follow installation instructions
  2. Try quick start example
  3. Verify links work
- **Expected**: README instructions work
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-097] API Documentation
- **Description**: Verify stdlib docs
- **Steps**:
  1. Check docs.rs/ruchy (or similar)
  2. Verify all public APIs documented
  3. Try example code from docs
- **Expected**: Comprehensive API docs
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-098] Tutorial Completeness
- **Description**: Verify tutorial works
- **Steps**:
  1. Follow ruchy-book chapters 1-5
  2. Verify all examples run
  3. Note any outdated information
- **Expected**: Tutorial is accurate
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-099] Changelog Accuracy
- **Description**: Verify CHANGELOG is current
- **Steps**:
  1. Review CHANGELOG.md
  2. Verify features listed exist
  3. Verify breaking changes documented
- **Expected**: Accurate changelog
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-100] Error Message Documentation
- **Description**: Verify error codes documented
- **Steps**:
  1. Look for error code reference
  2. Verify common errors explained
  3. Verify workarounds provided
- **Expected**: Error documentation exists
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## QA Process Guidelines

### Pre-QA Setup

1. **Environment**: Fresh machine or VM with only Rust toolchain
2. **Version**: Install from crates.io (not local build)
3. **Isolation**: No prior Ruchy installations
4. **Documentation**: Only use public documentation

### Execution Protocol

Following Toyota's standardized work principles [1]:

1. **One tester per category** to ensure focus
2. **Document everything** - screenshots, logs, exact commands
3. **Block on Critical failures** - stop category if Critical fails
4. **Daily standups** to share findings

### Defect Classification

| Severity | Definition | Action |
|----------|------------|--------|
| **Critical** | Prevents basic usage, data loss, security issue | Block release |
| **High** | Major feature broken, no workaround | Block release |
| **Medium** | Feature partially broken, workaround exists | Document, fix in beta.2 |
| **Low** | Cosmetic, minor inconvenience | Track for future |

### Sign-Off Requirements

**Beta release requires:**
- [ ] 100% of Critical checkpoints pass
- [ ] 95% of High checkpoints pass
- [ ] 80% of Medium checkpoints pass
- [ ] All failures documented with reproduction steps
- [ ] QA lead sign-off

---
