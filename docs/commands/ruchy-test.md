# ruchy test - Native Ruchy Test Runner

The `ruchy test` command provides comprehensive testing capabilities for Ruchy applications, supporting both individual files and project-wide test discovery.

## Overview

`ruchy test` is a native test runner that executes `.ruchy` test files using the Ruchy interpreter. It supports parallel execution, filtering, coverage reporting, and multiple output formats for seamless integration with CI/CD pipelines.

## Basic Usage

```bash
# Run all tests in current directory
ruchy test

# Run a specific test file
ruchy test my_test.ruchy

# Run tests in a specific directory
ruchy test tests/

# Run tests with verbose output
ruchy test --verbose

# Run tests with JSON output for automation
ruchy test --format=json
```

## Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `path` | File or directory to test | Current directory |
| `--watch` | Watch for changes and re-run tests | `false` |
| `--verbose` | Show detailed test output | `false` |
| `--filter <PATTERN>` | Run only tests matching pattern | None |
| `--coverage` | Generate coverage report | `false` |
| `--coverage-format <FORMAT>` | Coverage format (text, html, json) | `text` |
| `--parallel` | Run tests in parallel | `false` |
| `--threshold <PERCENT>` | Minimum coverage threshold | None |
| `--format <FORMAT>` | Output format (text, json, junit) | `text` |

## Test Discovery

`ruchy test` automatically discovers `.ruchy` files in the specified path:

```bash
project/
├── src/
│   ├── main.ruchy
│   └── utils.ruchy
├── tests/
│   ├── integration_test.ruchy
│   ├── unit_test.ruchy
│   └── performance_test.ruchy
└── examples/
    └── demo_test.ruchy
```

```bash
# Runs all .ruchy files in tests/ directory
ruchy test tests/

# Runs only files matching "integration"
ruchy test tests/ --filter integration
```

## Test File Format

Ruchy test files are standard `.ruchy` files that execute assertions and test logic:

```ruchy
// tests/math_test.ruchy
let result = 2 + 2
if result == 4 {
    println("✅ Addition test passed")
} else {
    println("❌ Addition test failed: expected 4, got", result)
}

// Test string operations
let message = "hello world"
if message.len() > 0 {
    println("✅ String length test passed")
}

// Test function calls
fun add(a: i32, b: i32) -> i32 {
    a + b
}

let sum = add(5, 3)
if sum == 8 {
    println("✅ Function test passed")
}
```

## Output Formats

### Text Format (Default)

```bash
$ ruchy test tests/
🧪 Running 3 .ruchy test files...

...

📊 Test Results:
   Total: 3
   Passed: 3
   Duration: 0.12s

✅ All tests passed!
```

### JSON Format

```bash
$ ruchy test tests/ --format=json
{
  "total": 3,
  "passed": 3,
  "failed": 0,
  "duration_seconds": 0.12,
  "results": [
    {
      "file": "tests/math_test.ruchy",
      "success": true,
      "duration_ms": 45.2,
      "error": null
    }
  ]
}
```

### Verbose Output

```bash
$ ruchy test tests/ --verbose
🔍 Discovering .ruchy test files in tests/
🧪 Running 3 .ruchy test files...

📄 Testing: tests/math_test.ruchy
   📖 Parsing test file...
   🏃 Executing test...
   📤 Test result: Unit
   ✅ math_test.ruchy (45.2ms)

📄 Testing: tests/string_test.ruchy
   📖 Parsing test file...
   🏃 Executing test...
   📤 Test result: Unit
   ✅ string_test.ruchy (23.1ms)
```

## Watch Mode

Watch mode automatically re-runs tests when files change:

```bash
ruchy test --watch
```

```
👁 Watching . for test changes...
Press Ctrl+C to stop watching

🧪 Running 5 .ruchy test files...
✅ All tests passed!

[File changed: src/math.ruchy]
🧪 Re-running tests...
✅ All tests passed!
```

## Coverage Reporting

Generate code coverage reports to understand test completeness:

```bash
# Generate text coverage report
ruchy test --coverage

# Generate HTML coverage report
ruchy test --coverage --coverage-format=html

# Require minimum 80% coverage
ruchy test --coverage --threshold=80
```

Coverage output:
```
📈 Coverage Report:
   Lines Covered: 245/300 (81.7%)
   Functions Covered: 18/20 (90.0%)
   Branches Covered: 42/50 (84.0%)
   
   ✅ Coverage meets threshold: 80%
```

## Filtering Tests

Use filters to run specific subsets of tests:

```bash
# Run only integration tests
ruchy test --filter integration

# Run unit tests only
ruchy test tests/ --filter unit

# Run performance tests
ruchy test --filter performance
```

## Parallel Execution

Speed up test execution with parallel processing:

```bash
# Run tests in parallel
ruchy test --parallel

# Combines well with other options
ruchy test --parallel --verbose --coverage
```

## CI/CD Integration

`ruchy test` provides proper exit codes and structured output for automation:

### GitHub Actions Example

```yaml
name: Test
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Ruchy
        run: cargo install ruchy
      - name: Run Tests
        run: ruchy test --format=json --coverage --threshold=80
```

### Exit Codes

- `0` - All tests passed
- `1` - One or more tests failed
- `2` - Test runner error (file not found, parse error, etc.)

## Error Handling

Detailed error reporting helps debug test failures:

```bash
❌ Failed Tests:
   tests/broken_test.ruchy - Test execution failed for: tests/broken_test.ruchy
   
Caused by:
    0: Failed to parse test file: tests/broken_test.ruchy
    1: Syntax error at line 5: unexpected token 'let'
```

## Best Practices

### 1. Organize Tests by Category

```
tests/
├── unit/           # Unit tests for individual functions
├── integration/    # Integration tests for component interaction
├── e2e/           # End-to-end system tests
└── fixtures/      # Test data and utilities
```

### 2. Use Descriptive File Names

```
tests/
├── math_operations_test.ruchy
├── string_processing_test.ruchy
├── file_io_integration_test.ruchy
└── api_client_e2e_test.ruchy
```

### 3. Structure Test Files Clearly

```ruchy
// tests/user_service_test.ruchy

// Test data setup
let test_user = {
    name: "Alice",
    email: "alice@example.com",
    age: 30
}

// Test 1: User creation
let created_user = create_user(test_user)
if created_user.id > 0 {
    println("✅ User creation test passed")
} else {
    println("❌ User creation test failed")
}

// Test 2: User validation
let is_valid = validate_user(test_user)
if is_valid {
    println("✅ User validation test passed")
}

// Cleanup
cleanup_test_data()
```

### 4. Use Meaningful Output

```ruchy
// Good: Descriptive output
if result == expected {
    println("✅ Calculator.add(2, 3) = 5")
} else {
    println("❌ Calculator.add(2, 3) expected 5, got", result)
}

// Better: Include context
println("Testing Calculator.add with positive integers...")
if result == expected {
    println("✅ PASS: add(2, 3) = 5")
} else {
    println("❌ FAIL: add(2, 3) expected 5, got", result)
}
```

## Advanced Usage

### Custom Test Runners

Create specialized test runners for specific needs:

```ruchy
// tests/custom_test_runner.ruchy
fun run_api_tests() {
    println("🌐 Running API integration tests...")
    
    // Setup test server
    let server = start_test_server()
    
    // Run tests
    test_user_endpoints()
    test_auth_endpoints()
    test_data_endpoints()
    
    // Cleanup
    stop_test_server(server)
    
    println("✅ API tests completed")
}

run_api_tests()
```

### Performance Testing

```ruchy
// tests/performance_test.ruchy
use std::time

fun benchmark_sort_algorithm() {
    let data = generate_random_array(10000)
    let start = time::now()
    
    let sorted = quick_sort(data)
    
    let duration = time::elapsed(start)
    println("Quick sort 10K elements:", duration, "ms")
    
    if duration < 100 {
        println("✅ Performance test passed")
    } else {
        println("❌ Performance test failed: too slow")
    }
}

benchmark_sort_algorithm()
```

## Troubleshooting

### Common Issues

1. **No tests found**
   ```bash
   ⚠️  No .ruchy test files found in tests/
   ```
   - Ensure test files have `.ruchy` extension
   - Check file permissions
   - Verify directory path

2. **Parse errors**
   ```bash
   ❌ math_test.ruchy (0.72ms): Test execution failed
   ```
   - Check syntax in test file
   - Use `ruchy parse test_file.ruchy` to debug
   - Verify file encoding (UTF-8)

3. **Import errors**
   ```bash
   ❌ Module 'utils' not found
   ```
   - Ensure imported modules exist
   - Check module search paths
   - Verify file structure

### Debug Mode

Enable detailed debugging:

```bash
RUST_LOG=debug ruchy test --verbose
```

## Integration with Other Commands

Combine `ruchy test` with other quality tools:

```bash
# Full quality pipeline
ruchy lint src/
ruchy test tests/ --coverage --threshold=80
ruchy prove src/ --check
ruchy score . --min=0.8
```

## Examples

See the [examples directory](../../examples/testing/) for comprehensive test examples and patterns.

## See Also

- [`ruchy lint`](ruchy-lint.md) - Code quality analysis
- [`ruchy prove`](ruchy-prove.md) - Mathematical proof verification
- [`ruchy score`](ruchy-score.md) - Unified quality scoring
- [Testing Guide](../guides/testing.md) - Comprehensive testing strategies