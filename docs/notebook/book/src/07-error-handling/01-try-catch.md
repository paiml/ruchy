# Try-Catch - Feature 23/41

Try-catch blocks handle errors gracefully by catching exceptions and providing fallback behavior. They prevent crashes and enable error recovery.

## Basic Try-Catch

```ruchy
try {
  let result = risky_operation()
  result
} catch error {
  f"Error occurred: {error}"
}
```

**Test Coverage**: ✅ [tests/lang_comp/error_handling/try_catch.rs](../../../../../tests/lang_comp/error_handling/try_catch.rs)

### Try It in the Notebook

```ruchy
try {
  let x = 10 / 2
  x
} catch error {
  0  // Fallback value
}
// Returns: 5
```

**Expected Output**: `5`

## Catching Specific Errors

```ruchy
try {
  parse_int("not a number")
} catch error {
  if error.contains("parse") {
    0  // Default for parse errors
  } else {
    throw error  // Re-throw other errors
  }
}
```

**Expected Output**: `0` (parse error caught)

## Try-Catch with Finally

```ruchy
let file = open("data.txt")

try {
  let content = file.read()
  process(content)
} catch error {
  log(f"Error: {error}")
  null
} finally {
  file.close()  // Always runs
}
```

**Expected Output**: File closed regardless of error

## Common Patterns

### Safe Division

```ruchy
fn safe_divide(a, b) {
  try {
    a / b
  } catch error {
    0  // Return 0 on division by zero
  }
}

safe_divide(10, 0)  // Returns: 0
safe_divide(10, 2)  // Returns: 5
```

**Expected Output**: `0`, `5`

### Safe Parsing

```ruchy
fn parse_or_default(s, default) {
  try {
    parse_int(s)
  } catch error {
    default
  }
}

parse_or_default("42", 0)     // Returns: 42
parse_or_default("invalid", 0) // Returns: 0
```

**Expected Output**: `42`, `0`

### Resource Cleanup

```ruchy
fn with_file(path, callback) {
  let file = open(path)
  try {
    callback(file)
  } catch error {
    log(f"Error: {error}")
    null
  } finally {
    file.close()
  }
}
```

**Expected Output**: File always closed

## Nested Try-Catch

```ruchy
try {
  try {
    risky_operation()
  } catch inner_error {
    // Handle inner error
    fallback_operation()  // May also throw
  }
} catch outer_error {
  // Handle outer error
  ultimate_fallback()
}
```

**Expected Output**: Multiple error recovery layers

## Try as Expression

```ruchy
let result = try { parse_int("42") } catch error { 0 }

result  // Returns: 42
```

**Expected Output**: `42`

## Best Practices

### ✅ Use Try-Catch for Recoverable Errors

```ruchy
// Good: Recoverable error
let config = try {
  load_config("config.json")
} catch error {
  default_config()
}

// Bad: Should use Result instead
fn load_config(path) -> Config {
  try {
    read_file(path)
  } catch error {
    // Silently swallowing errors
  }
}
```

### ✅ Always Clean Up Resources

```ruchy
// Good: Finally ensures cleanup
try {
  use_resource()
} finally {
  cleanup()
}

// Bad: Cleanup might not run
try {
  use_resource()
}
cleanup()  // Skipped if error occurs
```

### ✅ Catch Specific Error Types

```ruchy
// Good: Handle different errors differently
try {
  operation()
} catch error {
  match error.type {
    "NetworkError" => retry(),
    "ValidationError" => use_default(),
    _ => throw error
  }
}
```

## Try-Catch vs Result

| Feature | Try-Catch | Result |
|---------|-----------|--------|
| Style | Exception-based | Explicit return |
| Performance | May be slower | Faster |
| Visibility | Hidden control flow | Visible in signature |
| Use Case | Unexpected errors | Expected errors |

```ruchy
// Try-Catch: For unexpected errors
try {
  network_call()
} catch error {
  log(error)
}

// Result: For expected failures
fn divide(a, b) -> Result<i32, String> {
  if b == 0 {
    Err("Division by zero")
  } else {
    Ok(a / b)
  }
}
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 95%

Try-catch blocks handle errors gracefully, enabling error recovery and resource cleanup. Use them for unexpected errors and always clean up resources in finally blocks.

**Key Takeaways**:
- `try { code } catch error { fallback }`
- `finally` block always executes
- Use for unexpected, recoverable errors
- Prefer Result for expected failures
- Always clean up resources
- Catch specific error types when possible

---

[← Previous: Exhaustiveness](../06-pattern-matching/03-exhaustiveness.md) | [Next: Option Type →](./02-option.md)
