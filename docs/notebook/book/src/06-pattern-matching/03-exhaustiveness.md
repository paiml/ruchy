# Exhaustiveness Checking - Feature 22/41

Exhaustiveness checking ensures that match expressions handle all possible cases. The compiler verifies that no case is missed, preventing runtime errors.

## Why Exhaustiveness Matters

```ruchy
enum Status {
  Pending,
  Active,
  Completed
}

// Good: Exhaustive (all cases handled)
fn describe(status) {
  match status {
    Status::Pending => "Not started",
    Status::Active => "In progress",
    Status::Completed => "Done"
  }
}

// Bad: Non-exhaustive (compiler error if Completed not handled)
fn describe_incomplete(status) {
  match status {
    Status::Pending => "Not started",
    Status::Active => "In progress"
    // Missing: Status::Completed
  }
}
```

**Test Coverage**: ✅ [tests/lang_comp/pattern_matching/exhaustiveness.rs](../../../../tests/lang_comp/pattern_matching/exhaustiveness.rs)

## Wildcard Pattern

Use `_` to catch all remaining cases:

```ruchy
match value {
  1 => "One",
  2 => "Two",
  _ => "Other"  // Catches everything else
}
```

**Expected Output**: Exhaustive with wildcard

## Option Exhaustiveness

```ruchy
let maybe_value = Some(42)

// Good: Exhaustive
match maybe_value {
  Some(value) => f"Got {value}",
  None => "No value"
}

// Also exhaustive with wildcard
match maybe_value {
  Some(value) => f"Got {value}",
  _ => "No value"
}
```

**Expected Output**: Both patterns are exhaustive

## Result Exhaustiveness

```ruchy
let result = divide(10, 2)

// Good: Exhaustive
match result {
  Ok(value) => f"Result: {value}",
  Err(error) => f"Error: {error}"
}
```

**Expected Output**: Exhaustive error handling

## Tuple Exhaustiveness

```ruchy
let pair = (true, false)

// Good: Exhaustive (4 cases: TT, TF, FT, FF)
match pair {
  (true, true) => "Both true",
  (true, false) => "First true",
  (false, true) => "Second true",
  (false, false) => "Both false"
}

// Also exhaustive with wildcard
match pair {
  (true, true) => "Both true",
  _ => "At least one false"
}
```

**Expected Output**: All boolean combinations handled

## Common Patterns

### Catch-All Pattern

```ruchy
match status_code {
  200 => "OK",
  201 => "Created",
  204 => "No Content",
  _ => "Other status"  // Exhaustive catch-all
}
```

**Expected Output**: Handles all possible integers

### Named Catch-All

```ruchy
match status_code {
  200 => "OK",
  201 => "Created",
  other => f"Status: {other}"  // Named binding
}
```

**Expected Output**: Can use the unmatched value

### Ignoring Values

```ruchy
match result {
  Ok(_) => "Success",  // Don't care about value
  Err(_) => "Failed"   // Don't care about error
}
```

**Expected Output**: Exhaustive without binding values

## Nested Exhaustiveness

```ruchy
enum Response {
  Success(Option<i32>),
  Error(String)
}

// Good: Exhaustive nested matching
match response {
  Response::Success(Some(value)) => f"Value: {value}",
  Response::Success(None) => "No value",
  Response::Error(msg) => f"Error: {msg}"
}
```

**Expected Output**: All nested cases handled

## Best Practices

### ✅ Handle All Cases Explicitly

```ruchy
// Good: Clear about all cases
match status {
  Status::Pending => handle_pending(),
  Status::Active => handle_active(),
  Status::Completed => handle_completed()
}

// Acceptable: Explicit catch-all
match status {
  Status::Active => handle_active(),
  _ => handle_other()
}
```

### ✅ Use Wildcards Wisely

```ruchy
// Good: When many cases have same handling
match error_code {
  404 => "Not found",
  500 => "Server error",
  _ => "Unknown error"
}

// Bad: Missing specific cases
match status {
  Status::Active => handle_active(),
  _ => {}  // Silent ignore - probably a bug
}
```

### ✅ Be Explicit When Adding Variants

```ruchy
enum Status {
  Pending,
  Active,
  Completed,
  Cancelled  // New variant added
}

// Good: Compiler forces update when Status changes
match status {
  Status::Pending => ...,
  Status::Active => ...,
  Status::Completed => ...,
  Status::Cancelled => ...  // Must add this
}

// Bad: Wildcard masks missing case
match status {
  Status::Pending => ...,
  Status::Active => ...,
  _ => ...  // Silently catches Cancelled
}
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 98%

Exhaustiveness checking ensures all cases are handled in match expressions, preventing runtime errors and enforcing complete case coverage at compile time.

**Key Takeaways**:
- Compiler verifies all patterns are covered
- Use `_` for catch-all patterns
- Named wildcards when you need the value
- Be explicit about important cases
- Wildcards can hide bugs when enum variants are added
- Exhaustiveness works with Option, Result, enums, tuples

---

[← Previous: Pattern Guards](./02-guards.md) | [Next: Error Handling →](../07-error-handling/01-try-catch.md)
