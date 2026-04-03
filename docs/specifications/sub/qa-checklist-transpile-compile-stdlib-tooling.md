# Sub-spec: QA Beta Checklist — Categories 4-7 (TRANSPILE, COMPILE, STDLIB, TOOLING)

**Parent:** [100-point-qa-beta-checklist-4.0-beta.md](../100-point-qa-beta-checklist-4.0-beta.md)

---

## Category 4: TRANSPILE (10 Checkpoints)

### [QA-041] Basic Transpilation
- **Description**: Verify Ruchy to Rust conversion
- **Steps**:
  1. Run `ruchy transpile file.ruchy`
  2. Inspect generated .rs file
  3. Verify it's valid Rust syntax
- **Expected**: Valid, readable Rust output
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-042] Function Transpilation
- **Description**: Verify function conversion
- **Steps**:
  1. Transpile function with type annotations
  2. Transpile function without annotations
  3. Verify correct Rust signatures
- **Expected**: Type inference produces correct Rust types
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-043] Control Flow Transpilation
- **Description**: Verify if/match/loop conversion
- **Steps**:
  1. Transpile if-else chains
  2. Transpile match expressions
  3. Transpile for/while loops
- **Expected**: Equivalent Rust control flow
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-044] Type Annotation Preservation
- **Description**: Verify explicit types preserved
- **Steps**:
  1. Transpile: `let x: i64 = 42`
  2. Verify Rust uses i64, not i32
- **Expected**: Explicit types respected
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-045] String Handling in Transpilation
- **Description**: Verify string type mapping
- **Steps**:
  1. Transpile string operations
  2. Verify correct String vs &str usage
  3. Check f-string conversion
- **Expected**: Correct Rust string handling
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-046] Array/Vec Transpilation
- **Description**: Verify collection conversion
- **Steps**:
  1. Transpile array literals to Vec
  2. Verify indexing generates correct code
  3. Check iterator methods
- **Expected**: Correct Vec operations
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-047] Closure Transpilation
- **Description**: Verify lambda conversion
- **Steps**:
  1. Transpile closures with captures
  2. Verify Rust closure syntax
  3. Check move semantics
- **Expected**: Correct Rust closures
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-048] Struct Transpilation
- **Description**: Verify struct conversion
- **Steps**:
  1. Transpile struct definitions
  2. Transpile impl blocks
  3. Verify derive macros added appropriately
- **Expected**: Idiomatic Rust structs
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-049] No Unsafe Code Generated
- **Description**: Verify zero unsafe blocks
- **Steps**:
  1. Transpile various programs
  2. Search generated code for `unsafe`
  3. Verify none present
- **Expected**: Zero unsafe blocks (per Issue #132)
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-050] Transpilation Determinism
- **Description**: Verify identical output on re-runs
- **Steps**:
  1. Transpile same file twice
  2. Compare outputs byte-for-byte
- **Expected**: Identical output each time
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## Category 5: COMPILE (10 Checkpoints)

### [QA-051] rustc Compilation
- **Description**: Verify transpiled code compiles
- **Steps**:
  1. Run `ruchy compile file.ruchy`
  2. Verify rustc succeeds
  3. Run resulting binary
- **Expected**: Successful compilation and execution
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-052] Release Mode Compilation
- **Description**: Verify optimized builds
- **Steps**:
  1. Run `ruchy compile --release file.ruchy`
  2. Compare binary size to debug
  3. Run and verify correctness
- **Expected**: Smaller binary, same correctness
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-053] Compilation Error Messages
- **Description**: Verify helpful rustc errors
- **Steps**:
  1. Introduce type error in Ruchy
  2. Compile and observe error
  3. Verify error points to Ruchy source
- **Expected**: Errors traceable to Ruchy code
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-054] External Crate Dependencies
- **Description**: Verify Cargo.toml generation
- **Steps**:
  1. Compile program using stdlib features (HTTP, etc.)
  2. Check generated Cargo.toml includes dependencies
- **Expected**: Correct dependency management
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-055] Binary Execution
- **Description**: Verify compiled binary works
- **Steps**:
  1. Compile a program with I/O
  2. Run the binary directly
  3. Verify I/O works correctly
- **Expected**: Binary behaves same as interpreter
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-056] Cross-Platform Compilation
- **Description**: Verify works on multiple OS
- **Steps**:
  1. Test on Linux
  2. Test on macOS (if available)
  3. Test on Windows (if available)
- **Expected**: Works on supported platforms
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-057] Compilation Performance
- **Description**: Verify reasonable compile times
- **Steps**:
  1. Compile a 1000-line program
  2. Measure total time
- **Expected**: Complete in < 60 seconds
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-058] Incremental Compilation
- **Description**: Verify incremental builds work
- **Steps**:
  1. Compile, modify one function, recompile
  2. Verify faster than full rebuild
- **Expected**: Incremental improvement
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-059] Debug Symbols
- **Description**: Verify debugging support
- **Steps**:
  1. Compile with debug info
  2. Run under debugger (gdb/lldb)
  3. Verify can set breakpoints
- **Expected**: Debuggable binaries
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-060] Static Linking Option
- **Description**: Verify static binary creation
- **Steps**:
  1. Compile with static linking flag (if supported)
  2. Verify binary has no dynamic dependencies
- **Expected**: Fully static binary option
- **Severity**: Low
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## Category 6: STDLIB (10 Checkpoints)

### [QA-061] File I/O
- **Description**: Verify file operations
- **Steps**:
  1. Test: `fs::read("file.txt")`
  2. Test: `fs::write("file.txt", content)`
  3. Test: error on missing file
- **Expected**: File operations work, errors graceful
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-062] HTTP Client
- **Description**: Verify HTTP requests
- **Steps**:
  1. Test: `http::get("https://httpbin.org/get")`
  2. Test: `http::post(url, body)`
  3. Test: timeout handling
- **Expected**: HTTP requests succeed
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-063] JSON Handling
- **Description**: Verify JSON operations
- **Steps**:
  1. Test: `json::parse(string)`
  2. Test: `json::stringify(object)`
  3. Test: invalid JSON error handling
- **Expected**: JSON round-trips correctly
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-064] Time/Date Functions
- **Description**: Verify time operations
- **Steps**:
  1. Test: `time::now()`
  2. Test: `time::sleep(duration)`
  3. Test: date formatting
- **Expected**: Time functions work
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-065] Math Functions
- **Description**: Verify math operations
- **Steps**:
  1. Test: `math::sqrt(16.0)`
  2. Test: `math::sin`, `math::cos`
  3. Test: `math::pow`, `math::log`
- **Expected**: Correct mathematical results
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-066] String Functions
- **Description**: Verify string utilities
- **Steps**:
  1. Test: `str.split(delimiter)`
  2. Test: `str.trim()`
  3. Test: `str.replace(from, to)`
  4. Test: `str.contains(substr)`
- **Expected**: String operations work
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-067] Collection Functions
- **Description**: Verify array/map utilities
- **Steps**:
  1. Test: `arr.map(fn)`
  2. Test: `arr.filter(predicate)`
  3. Test: `arr.reduce(fn, initial)`
  4. Test: `arr.sort()`
- **Expected**: Functional operations work
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-068] Environment Variables
- **Description**: Verify env access
- **Steps**:
  1. Test: `env::get("PATH")`
  2. Test: `env::set("MY_VAR", "value")`
  3. Test: missing variable handling
- **Expected**: Env operations work
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-069] Random Number Generation
- **Description**: Verify random functions
- **Steps**:
  1. Test: `random::int(0, 100)`
  2. Test: `random::float()`
  3. Test: distribution is reasonable
- **Expected**: Random numbers generated
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-070] Regular Expressions
- **Description**: Verify regex support
- **Steps**:
  1. Test: `regex::match(pattern, string)`
  2. Test: `regex::replace(pattern, string, replacement)`
  3. Test: invalid regex error
- **Expected**: Regex operations work
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## Category 7: TOOLING (10 Checkpoints)

### [QA-071] CLI Help
- **Description**: Verify help documentation
- **Steps**:
  1. Run `ruchy --help`
  2. Run `ruchy run --help`
  3. Verify all commands documented
- **Expected**: Comprehensive help text
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-072] Version Command
- **Description**: Verify version reporting
- **Steps**:
  1. Run `ruchy --version`
  2. Verify version matches Cargo.toml
- **Expected**: Correct version displayed
- **Severity**: Low
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-073] Lint Tool
- **Description**: Verify code linting
- **Steps**:
  1. Run `ruchy lint file.ruchy`
  2. Introduce style issue
  3. Verify lint catches it
- **Expected**: Lint warnings reported
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-074] Format Tool
- **Description**: Verify code formatting
- **Steps**:
  1. Run `ruchy fmt file.ruchy`
  2. Verify code reformatted
  3. Test `--check` flag
- **Expected**: Consistent formatting
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-075] AST Dump
- **Description**: Verify AST inspection
- **Steps**:
  1. Run `ruchy ast file.ruchy`
  2. Verify readable AST output
- **Expected**: AST structure displayed
- **Severity**: Low
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-076] Test Runner
- **Description**: Verify test execution
- **Steps**:
  1. Write test functions in Ruchy
  2. Run `ruchy test file.ruchy`
  3. Verify pass/fail reporting
- **Expected**: Tests executed, results reported
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-077] Coverage Tool
- **Description**: Verify code coverage
- **Steps**:
  1. Run `ruchy coverage file.ruchy`
  2. Verify coverage percentage reported
- **Expected**: Coverage metrics displayed
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-078] Benchmark Tool
- **Description**: Verify benchmarking
- **Steps**:
  1. Run `ruchy bench file.ruchy`
  2. Verify timing output
- **Expected**: Benchmark results displayed
- **Severity**: Low
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-079] Project Initialization
- **Description**: Verify new project creation
- **Steps**:
  1. Run `ruchy new my_project`
  2. Verify directory structure created
  3. Verify can build immediately
- **Expected**: Working project scaffold
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-080] Watch Mode
- **Description**: Verify file watching
- **Steps**:
  1. Run `ruchy watch file.ruchy`
  2. Modify file
  3. Verify automatic re-run
- **Expected**: Auto-reload on changes
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---
