# Error Handling in Ruchy

Ruchy provides comprehensive error handling mechanisms inspired by Rust, including Result types, the try operator (`?`), and try/catch blocks.

## Result Type

The `Result<T, E>` type represents either success (`Ok`) or failure (`Err`):

```ruchy
fun divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}
```

## Pattern Matching on Results

Use `match` to handle both success and error cases:

```ruchy
let result = divide(10, 2)
match result {
    Ok(value) => println("Success:", value),
    Err(error) => println("Error:", error)
}
```

## The Try Operator (?)

The `?` operator provides early return for error propagation:

```ruchy
fun process_numbers() -> Result<i32, String> {
    let x = divide(100, 5)?  // Returns early if error
    let y = divide(x, 2)?     // Chains error propagation
    Ok(y * 10)
}
```

## Try/Catch Blocks

For exception-style error handling:

```ruchy
fun safe_operation() -> i32 {
    try {
        risky_operation()
    } catch (e) {
        println("Error occurred:", e)
        -1  // Default value
    }
}
```

### Try/Catch with Finally

The `finally` block always executes:

```ruchy
try {
    open_file()
    process_data()
} catch (e) {
    handle_error(e)
} finally {
    close_file()  // Always runs
}
```

### Multiple Catch Clauses

Handle different error types:

```ruchy
try {
    complex_operation()
} catch (io_err) {
    handle_io_error(io_err)
} catch (parse_err) {
    handle_parse_error(parse_err)
}
```

## Error Propagation Patterns

### Early Return with Errors

```ruchy
fun validate_input(data: String) -> Result<Data, String> {
    if data.is_empty() {
        return Err("Input cannot be empty")
    }

    let parsed = parse_data(data)?
    if !is_valid(parsed) {
        return Err("Invalid data format")
    }

    Ok(parsed)
}
```

### Chaining Operations

```ruchy
fun process_file(path: String) -> Result<Output, Error> {
    read_file(path)?
        .parse()?
        .validate()?
        .transform()
}
```

## Custom Error Types

Define your own error types using structs or enums:

```ruchy
struct AppError {
    message: String,
    code: i32
}

enum FileError {
    NotFound(String),
    PermissionDenied,
    IoError { path: String, reason: String }
}
```

## Best Practices

1. **Use Result for recoverable errors**: When the caller can handle the error
2. **Use panic for unrecoverable errors**: When the program cannot continue
3. **Propagate errors with ?**: Don't hide errors unnecessarily
4. **Provide context**: Include helpful error messages
5. **Handle errors at appropriate levels**: Don't catch too early or too late

## Examples

### File Processing with Error Handling

```ruchy
fun process_config(path: String) -> Result<Config, String> {
    try {
        let contents = read_file(path)?
        let config = parse_json(contents)?
        validate_config(config)?
        Ok(config)
    } catch (e) {
        Err(format("Failed to load config: {}", e))
    }
}
```

### Nested Error Handling

```ruchy
fun robust_operation() -> Result<i32, String> {
    try {
        let result = try {
            risky_calculation()
        } catch (calc_err) {
            fallback_calculation()
        }

        Ok(result * 2)
    } catch (e) {
        Err("Complete failure")
    }
}
```

## Transpilation

Ruchy's error handling transpiles to idiomatic Rust:

- `Result<T, E>` → `Result<T, E>`
- `Ok(value)` → `Ok(value)`
- `Err(error)` → `Err(error)`
- `expr?` → `expr?`
- `try/catch` → `match` with Result closures

This ensures zero-cost abstractions and full compatibility with Rust's error handling ecosystem.