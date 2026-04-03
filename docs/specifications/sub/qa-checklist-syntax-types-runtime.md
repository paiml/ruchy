# Sub-spec: QA Beta Checklist — Categories 1-3 (SYNTAX, TYPES, RUNTIME)

**Parent:** [100-point-qa-beta-checklist-4.0-beta.md](../100-point-qa-beta-checklist-4.0-beta.md)

---

## Category 1: SYNTAX (15 Checkpoints)

### [QA-001] Basic Variable Declaration
- **Description**: Verify `let` and `const` declarations work correctly
- **Steps**:
  1. Create file with: `let x = 42` and `const PI = 3.14159`
  2. Run with `ruchy run file.ruchy`
  3. Verify no errors
- **Expected**: Silent success, no output unless printed
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-002] Function Definition Syntax
- **Description**: Verify function definitions with various signatures
- **Steps**:
  1. Test: `fun greet(name: String) -> String { "Hello, " + name }`
  2. Test: `fun add(a, b) { a + b }` (no type annotations)
  3. Test: `fun side_effect() { print("hi") }` (no return)
- **Expected**: All three syntaxes accepted
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-003] Control Flow: If-Else
- **Description**: Verify if/else/elif chains
- **Steps**:
  1. Test nested if-else with 3+ levels
  2. Test if as expression: `let x = if cond { 1 } else { 2 }`
  3. Test elif chains
- **Expected**: Correct branch execution
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-004] Control Flow: Match Expression
- **Description**: Verify pattern matching
- **Steps**:
  1. Test match on integers: `match x { 1 => "one", _ => "other" }`
  2. Test match on strings
  3. Test match with guards (if supported)
- **Expected**: Correct pattern selection
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-005] Loops: For and While
- **Description**: Verify loop constructs
- **Steps**:
  1. Test: `for i in 0..10 { print(i) }`
  2. Test: `for item in array { ... }`
  3. Test: `while condition { ... }`
  4. Test: `break` and `continue`
- **Expected**: Correct iteration, break/continue work
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-006] Array Literals
- **Description**: Verify array syntax and operations
- **Steps**:
  1. Test: `let arr = [1, 2, 3]`
  2. Test: `arr[0]` (indexing)
  3. Test: `arr.push(4)` (mutation)
  4. Test: `len(arr)`
- **Expected**: Array operations work correctly
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-007] Map/Object Literals
- **Description**: Verify map/object syntax
- **Steps**:
  1. Test: `let obj = { "key": "value" }`
  2. Test: `obj["key"]` and `obj.key`
  3. Test nested objects
- **Expected**: Map operations work correctly
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-008] String Literals and Interpolation
- **Description**: Verify string handling
- **Steps**:
  1. Test: `"hello world"` (basic)
  2. Test: `f"Hello {name}"` (f-strings)
  3. Test: escape sequences `\n`, `\t`, `\\`
  4. Test: multiline strings (if supported)
- **Expected**: All string formats work
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-009] Lambda/Closure Syntax
- **Description**: Verify anonymous functions
- **Steps**:
  1. Test: `let double = |x| x * 2`
  2. Test: `let add = |a, b| a + b`
  3. Test: closure capturing outer variables
- **Expected**: Lambdas execute correctly
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-010] Pipeline Operator
- **Description**: Verify `|>` pipeline syntax
- **Steps**:
  1. Test: `5 |> double |> add_one`
  2. Test: chaining 5+ operations
  3. Test: pipeline with lambdas
- **Expected**: Values flow through pipeline
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-011] Struct/Class Definition
- **Description**: Verify custom type definitions
- **Steps**:
  1. Test: `struct Point { x: i32, y: i32 }`
  2. Test: instantiation `Point { x: 1, y: 2 }`
  3. Test: field access `point.x`
- **Expected**: Structs work as expected
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-012] Enum Definition
- **Description**: Verify enum types
- **Steps**:
  1. Test: `enum Color { Red, Green, Blue }`
  2. Test: `enum Option<T> { Some(T), None }`
  3. Test: pattern matching on enums
- **Expected**: Enums and variants work
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-013] Impl Blocks
- **Description**: Verify method implementations
- **Steps**:
  1. Test: `impl Point { fun distance(self) { ... } }`
  2. Test: calling methods `point.distance()`
  3. Test: associated functions (no self)
- **Expected**: Methods callable on instances
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-014] Comments
- **Description**: Verify comment syntax
- **Steps**:
  1. Test: `// single line comment`
  2. Test: `/* multi-line comment */`
  3. Test: `/// doc comment`
  4. Verify comments don't affect execution
- **Expected**: Comments ignored in execution
- **Severity**: Low
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-015] Unicode Identifiers
- **Description**: Verify non-ASCII identifiers (if supported)
- **Steps**:
  1. Test: `let café = "coffee"`
  2. Test: `let 数字 = 42`
  3. Test: emoji in strings (not identifiers)
- **Expected**: Unicode handled correctly or clear error
- **Severity**: Low
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## Category 2: TYPES (10 Checkpoints)

### [QA-016] Integer Types
- **Description**: Verify integer arithmetic
- **Steps**:
  1. Test: basic arithmetic `+`, `-`, `*`, `/`, `%`
  2. Test: integer overflow behavior
  3. Test: division by zero handling
- **Expected**: Correct results, graceful error on edge cases
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-017] Float Types
- **Description**: Verify floating-point operations
- **Steps**:
  1. Test: `3.14 * 2.0`
  2. Test: `0.1 + 0.2` (IEEE 754 behavior)
  3. Test: special values (NaN, Infinity)
- **Expected**: IEEE 754 compliant behavior
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-018] Boolean Types
- **Description**: Verify boolean operations
- **Steps**:
  1. Test: `true && false`, `true || false`
  2. Test: `!true`
  3. Test: short-circuit evaluation
- **Expected**: Correct boolean logic
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-019] String Types
- **Description**: Verify string operations
- **Steps**:
  1. Test: concatenation `"a" + "b"`
  2. Test: `len("hello")`
  3. Test: string methods (split, trim, etc.)
- **Expected**: String operations work
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-020] Type Inference
- **Description**: Verify automatic type inference
- **Steps**:
  1. Test: `let x = 42` (infer i32)
  2. Test: `let arr = [1, 2, 3]` (infer Vec<i32>)
  3. Test: function return type inference
- **Expected**: Types correctly inferred
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-021] Generic Types
- **Description**: Verify generics (if supported)
- **Steps**:
  1. Test: `fun identity<T>(x: T) -> T { x }`
  2. Test: generic structs
  3. Test: type constraints
- **Expected**: Generics instantiate correctly
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-022] Option Type
- **Description**: Verify Option/nullable handling
- **Steps**:
  1. Test: `Some(42)` and `None`
  2. Test: unwrapping with match
  3. Test: `?.` operator (if supported)
- **Expected**: Null safety enforced
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-023] Result Type
- **Description**: Verify error handling types
- **Steps**:
  1. Test: `Ok(value)` and `Err(error)`
  2. Test: `?` propagation (if supported)
  3. Test: match on Result
- **Expected**: Errors propagate correctly
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-024] Type Coercion
- **Description**: Verify implicit/explicit conversions
- **Steps**:
  1. Test: integer to float coercion
  2. Test: `as` keyword for casting
  3. Test: string to number parsing
- **Expected**: Clear coercion rules
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-025] Tuple Types
- **Description**: Verify tuple support
- **Steps**:
  1. Test: `let pair = (1, "hello")`
  2. Test: destructuring `let (a, b) = pair`
  3. Test: tuple indexing `pair.0`
- **Expected**: Tuples work correctly
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---

## Category 3: RUNTIME (15 Checkpoints)

### [QA-026] Variable Scoping
- **Description**: Verify lexical scoping rules
- **Steps**:
  1. Test: inner scope shadows outer
  2. Test: variables not accessible outside scope
  3. Test: closure captures outer scope
- **Expected**: Proper lexical scoping
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-027] Function Calls
- **Description**: Verify function invocation
- **Steps**:
  1. Test: simple function call
  2. Test: recursive function (fibonacci)
  3. Test: mutual recursion
  4. Test: tail recursion (if optimized)
- **Expected**: Correct execution, no stack overflow on reasonable depth
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-028] Memory Management
- **Description**: Verify no memory leaks in interpreter
- **Steps**:
  1. Run a loop creating 10,000 objects
  2. Monitor memory usage
  3. Verify memory is reclaimed
- **Expected**: Memory stable or grows minimally
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-029] Stack Overflow Handling
- **Description**: Verify deep recursion handling
- **Steps**:
  1. Test: recursive function with no base case
  2. Test: very deep recursion (10,000+ calls)
- **Expected**: Graceful error, not process crash
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-030] Print Function
- **Description**: Verify stdout output
- **Steps**:
  1. Test: `print("hello")`
  2. Test: `println("hello")`
  3. Test: printing various types
- **Expected**: Correct output to stdout
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-031] REPL Interaction
- **Description**: Verify interactive REPL
- **Steps**:
  1. Launch `ruchy repl`
  2. Enter expressions, verify results
  3. Test multi-line input
  4. Test `:help`, `:quit` commands
- **Expected**: Interactive session works
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-032] File Script Execution
- **Description**: Verify running .ruchy files
- **Steps**:
  1. Create `test.ruchy` with valid code
  2. Run `ruchy run test.ruchy`
  3. Verify output matches expectations
- **Expected**: Script executes correctly
- **Severity**: Critical
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-033] Import/Module System
- **Description**: Verify module imports
- **Steps**:
  1. Create two files, one importing from other
  2. Test: `use std::io`
  3. Test: relative imports
- **Expected**: Imports resolve correctly
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-034] Global Variables
- **Description**: Verify global variable behavior
- **Steps**:
  1. Define global at top of file
  2. Access from function
  3. Test mutation rules
- **Expected**: Globals accessible, mutation rules enforced
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-035] Expression Evaluation Order
- **Description**: Verify left-to-right evaluation
- **Steps**:
  1. Test: `a() + b() + c()` with side effects
  2. Verify order of side effects
- **Expected**: Left-to-right, deterministic
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-036] Operator Precedence
- **Description**: Verify correct operator precedence
- **Steps**:
  1. Test: `2 + 3 * 4` should be 14
  2. Test: `2 * 3 + 4` should be 10
  3. Test: boolean operators precedence
- **Expected**: Standard precedence rules
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-037] Comparison Operators
- **Description**: Verify comparison semantics
- **Steps**:
  1. Test: `<`, `<=`, `>`, `>=`, `==`, `!=`
  2. Test: comparing different types (should error or have clear rules)
  3. Test: chained comparisons (if supported)
- **Expected**: Correct comparison results
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-038] Short-Circuit Evaluation
- **Description**: Verify && and || short-circuit
- **Steps**:
  1. Test: `false && side_effect()` - side effect should NOT run
  2. Test: `true || side_effect()` - side effect should NOT run
- **Expected**: Short-circuit prevents evaluation
- **Severity**: High
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-039] Assignment Operators
- **Description**: Verify compound assignments
- **Steps**:
  1. Test: `+=`, `-=`, `*=`, `/=`
  2. Test: on arrays/maps (if supported)
- **Expected**: Compound assignments work
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

### [QA-040] Range Expressions
- **Description**: Verify range syntax
- **Steps**:
  1. Test: `0..10` (exclusive)
  2. Test: `0..=10` (inclusive, if supported)
  3. Test: ranges in for loops
  4. Test: array slicing with ranges
- **Expected**: Ranges iterate correctly
- **Severity**: Medium
- **Status**: ☐ Pass ☐ Fail ☐ Blocked

---
