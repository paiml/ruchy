# Function Definitions - Feature 12/41

Functions encapsulate reusable code. In Ruchy, functions are first-class values that can be passed around and returned.

## Basic Function Definition

Define a function with `fn`:

```ruchy
fn greet() {
  print("Hello!")
}

greet()  // Prints: Hello!
```

**Expected Output**: `Hello!`

**Test Coverage**: ✅ [tests/lang_comp/functions/definitions.rs](../../../../tests/lang_comp/functions/definitions.rs)

### Try It in the Notebook

```ruchy
fn add(a, b) {
  a + b
}

let result = add(5, 3)
result  // Returns: 8
```

**Expected Output**: `8`

## Function with Return Value

The last expression is automatically returned:

```ruchy
fn square(n) {
  n * n
}

square(4)  // Returns: 16
```

**Expected Output**: `16`

### Explicit Return

Use `return` for early exit:

```ruchy
fn abs(n) {
  if n < 0 {
    return -n
  }
  n
}

abs(-5)  // Returns: 5
```

**Expected Output**: `5`

## Parameters

Functions can accept multiple parameters:

```ruchy
fn calculate(x, y, z) {
  x * y + z
}

calculate(2, 3, 4)  // Returns: 10
```

**Expected Output**: `10`

## Common Patterns

### Pure Functions

```ruchy
fn celsius_to_fahrenheit(c) {
  c * 9 / 5 + 32
}

celsius_to_fahrenheit(0)   // Returns: 32
celsius_to_fahrenheit(100)  // Returns: 212
```

### Helper Functions

```ruchy
fn is_even(n) {
  n % 2 == 0
}

fn filter_evens(numbers) {
  let result = []
  for n in numbers {
    if is_even(n) {
      result.push(n)
    }
  }
  result
}

filter_evens([1, 2, 3, 4, 5])  // Returns: [2, 4]
```

### Validation Functions

```ruchy
fn is_valid_age(age) {
  age >= 0 && age <= 120
}

is_valid_age(25)   // Returns: true
is_valid_age(-5)   // Returns: false
is_valid_age(150)  // Returns: false
```

## Recursion

Functions can call themselves:

```ruchy
fn factorial(n) {
  if n <= 1 {
    1
  } else {
    n * factorial(n - 1)
  }
}

factorial(5)  // Returns: 120
```

**Expected Output**: `120`

### Fibonacci

```ruchy
fn fib(n) {
  if n <= 1 {
    n
  } else {
    fib(n - 1) + fib(n - 2)
  }
}

fib(7)  // Returns: 13
```

**Expected Output**: `13`

## Function Scope

Functions have their own scope:

```ruchy
let x = 10

fn test() {
  let x = 20  // Different x
  x
}

test()  // Returns: 20
x       // Returns: 10 (unchanged)
```

## Closures

Functions capture their environment:

```ruchy
fn make_adder(n) {
  fn add(x) {
    x + n  // Captures n
  }
  add
}

let add5 = make_adder(5)
add5(3)  // Returns: 8
```

**Expected Output**: `8`

## Higher-Order Functions

Functions that take or return functions:

```ruchy
fn apply_twice(f, x) {
  f(f(x))
}

fn double(n) {
  n * 2
}

apply_twice(double, 3)  // Returns: 12 (3 * 2 * 2)
```

**Expected Output**: `12`

## Anonymous Functions

```ruchy
let square = fn(x) { x * x }
square(5)  // Returns: 25
```

**Expected Output**: `25`

## Arrow Functions

Shorthand syntax:

```ruchy
let add = (a, b) => a + b
add(3, 4)  // Returns: 7
```

**Expected Output**: `7`

## Best Practices

### ✅ Small, Focused Functions

```ruchy
// Good: Single responsibility
fn calculate_tax(amount) {
  amount * 0.08
}

fn calculate_total(subtotal) {
  subtotal + calculate_tax(subtotal)
}
```

### ✅ Descriptive Names

```ruchy
// Good
fn is_prime(n) { ... }
fn get_user_by_id(id) { ... }

// Bad
fn check(n) { ... }
fn get(id) { ... }
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 96%

Functions are the building blocks of reusable code. Use them to organize logic, avoid repetition, and create abstractions.

**Key Takeaways**:
- Last expression is returned automatically
- Use `return` for early exit
- Functions can be recursive
- Closures capture environment
- Keep functions small and focused

---

[← Previous: Loop Control](../03-control-flow/05-loop-control.md) | [Next: Parameters & Arguments →](./02-parameters.md)
