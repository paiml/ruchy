# Introduction: Empirical Language Proof

## Why This Book Exists

This is not a typical language tutorial. This is an **empirical proof** that Ruchy works.

Every feature documented here has:
- ✅ **Runnable code** you can copy into the notebook
- ✅ **Expected output** you can verify
- ✅ **Automated tests** proving it works
- ✅ **Coverage reports** proving tests are thorough
- ✅ **Mutation tests** proving tests catch real bugs
- ✅ **E2E tests** proving it works in browsers

## The Promise

**If you can run this code in the Ruchy notebook and get the expected output, the language feature works.**

No hand-waving. No "coming soon." No "should work."

Just: Here's the code, here's the output, here's the test that proves it.

## How to Use This Book

### 1. Run the Notebook

```bash
# Start the notebook server
cargo run --features notebook --bin ruchy notebook

# Or open the web version
open http://localhost:8000/notebook.html
```

### 2. Try Each Feature

Copy the code from each chapter into the notebook. Run it. Verify the output matches.

### 3. Check the Proof

Every chapter links to:
- The automated test file
- The coverage report
- The mutation test results
- The E2E test

If you don't trust the docs, **check the tests.**

## Quality Standards

This book and the notebook are held to **wasm-labs EXTREME quality standards**:

### Coverage Requirements
- ✅ **Line Coverage**: ≥85%
- ✅ **Branch Coverage**: ≥90%
- ✅ **Mutation Score**: ≥90%

### Testing Requirements
- ✅ **Unit Tests**: Every function
- ✅ **Property Tests**: 10,000+ random inputs
- ✅ **Mutation Tests**: Empirical bug-catching proof
- ✅ **E2E Tests**: Real browsers (Chrome, Firefox, Safari)

### WASM Requirements
- ✅ **Size**: <500KB
- ✅ **Purity**: 0 WASI imports
- ✅ **Validation**: Deep bytecode inspection

## The 41 Features

This book proves all 41 Ruchy language features work in the notebook:

### Foundation (9 features)
1. Integer/Float/String/Bool/Nil literals
2. Variable binding and assignment
3. Comments (line and block)
4. Arithmetic operators (+, -, *, /, %)
5. Comparison operators (<, >, <=, >=, ==, !=)
6. Logical operators (&&, ||, !)
7. Bitwise operators (&, |, ^, <<, >>)
8. If-else expressions
9. Match expressions

### Functions & Data (11 features)
10. For loops
11. While loops
12. Loop control (break, continue)
13. Function definitions
14. Function parameters and returns
15. Closures and lambdas
16. Higher-order functions
17. Arrays
18. Tuples
19. Objects/Maps
20. Structs

### Advanced (10 features)
21. Enums
22. Pattern destructuring
23. Pattern guards
24. Exhaustiveness checking
25. Try-catch error handling
26. Option type
27. Result type
28. String interpolation (f-strings)
29. String methods
30. String escaping

### Standard Library (10 features)
31. File I/O (fs)
32. HTTP client
33. JSON parsing
34. Path operations
35. Environment variables
36. Process execution
37. Time/Date operations
38. Logging
39. Regular expressions
40. DataFrames

### Meta (1 feature)
41. WASM compilation

---

## Let's Begin

Ready to see the proof? Let's start with the basics: **literals**.

[Continue to Basic Syntax →](./01-basic-syntax/README.md)
