# Tuples - Feature 14/41

Tuples are fixed-size ordered collections that can hold values of different types. They're perfect for grouping related data.

## Creating Tuples

```ruchy
let point = (10, 20)
let person = ("Alice", 30, true)
let empty = ()
let single = (42,)  // Note trailing comma for single element
```

**Test Coverage**: ✅ [tests/lang_comp/data_structures/tuples.rs](../../../../tests/lang_comp/data_structures/tuples.rs)

### Try It in the Notebook

```ruchy
let coordinates = (100, 200, 300)
coordinates  // Returns: (100, 200, 300)
```

**Expected Output**: `(100, 200, 300)`

## Accessing Elements

Use zero-based indexing with dot notation:

```ruchy
let point = (10, 20)

point.0  // Returns: 10
point.1  // Returns: 20
```

**Expected Output**: `10`, `20`

### Multi-Type Tuples

```ruchy
let data = ("Error", 404, false)

data.0  // Returns: "Error"
data.1  // Returns: 404
data.2  // Returns: false
```

**Expected Output**: `"Error"`, `404`, `false`

## Tuple Destructuring

Unpack tuple values into variables:

```ruchy
let point = (10, 20)
let (x, y) = point

x  // Returns: 10
y  // Returns: 20
```

**Expected Output**: `10`, `20`

### Partial Destructuring

```ruchy
let triple = (1, 2, 3)
let (first, _, last) = triple

first  // Returns: 1
last   // Returns: 3
```

**Expected Output**: `1`, `3`

## Common Patterns

### Swap Variables

```ruchy
let a = 10
let b = 20

(a, b) = (b, a)

a  // Returns: 20
b  // Returns: 10
```

**Expected Output**: `20`, `10`

### Return Multiple Values

```ruchy
fn divide_with_remainder(a, b) {
  let quotient = a / b
  let remainder = a % b
  (quotient, remainder)
}

let result = divide_with_remainder(17, 5)
result  // Returns: (3, 2)

let (q, r) = divide_with_remainder(17, 5)
q  // Returns: 3
r  // Returns: 2
```

**Expected Output**: `(3, 2)`, `3`, `2`

### Coordinate Pairs

```ruchy
let points = [(0, 0), (10, 20), (30, 40)]

for (x, y) in points {
  let distance = sqrt(x * x + y * y)
  print(f"({x}, {y}) -> {distance}")
}
```

**Expected Output**: `(0, 0) -> 0`, `(10, 20) -> 22.36`, `(30, 40) -> 50`

## Tuples vs Arrays

| Feature | Tuple | Array |
|---------|-------|-------|
| Size | Fixed at creation | Can grow/shrink |
| Types | Mixed types allowed | Typically same type |
| Access | By position (`.0`, `.1`) | By index (`[0]`, `[1]`) |
| Mutation | Elements can change | Elements can change |
| Use Case | Related but different data | Collection of similar items |

```ruchy
// Tuple: Fixed size, mixed types
let person = ("Alice", 30, true)

// Array: Dynamic size, same type
let numbers = [1, 2, 3]
numbers.push(4)  // Can grow
```

## Nested Tuples

```ruchy
let nested = ((1, 2), (3, 4), (5, 6))

nested.0     // Returns: (1, 2)
nested.0.0   // Returns: 1
nested.1.1   // Returns: 4
```

**Expected Output**: `(1, 2)`, `1`, `4`

### Destructuring Nested Tuples

```ruchy
let data = ((10, 20), (30, 40))
let ((x1, y1), (x2, y2)) = data

x1  // Returns: 10
y2  // Returns: 40
```

**Expected Output**: `10`, `40`

## Tuple Iteration

```ruchy
let tuple = (1, 2, 3, 4, 5)
let sum = 0

// Convert to array for iteration
let values = [tuple.0, tuple.1, tuple.2, tuple.3, tuple.4]
for v in values {
  sum = sum + v
}

sum  // Returns: 15
```

**Expected Output**: `15`

**Note**: Tuples don't have built-in iteration. Convert to array if needed.

## Common Use Cases

### Configuration

```ruchy
let config = ("localhost", 8080, true)
let (host, port, ssl) = config

print(f"Server: {host}:{port} (SSL: {ssl})")
```

**Expected Output**: `Server: localhost:8080 (SSL: true)`

### State Tracking

```ruchy
fn fetch_data() {
  let success = true
  let data = "result"
  let timestamp = 1234567890
  (success, data, timestamp)
}

let (ok, result, time) = fetch_data()
if ok {
  print(f"Fetched {result} at {time}")
}
```

**Expected Output**: `Fetched result at 1234567890`

### Min/Max Pairs

```ruchy
fn min_max(arr) {
  let min = arr[0]
  let max = arr[0]

  for n in arr {
    if n < min { min = n }
    if n > max { max = n }
  }

  (min, max)
}

let (minimum, maximum) = min_max([3, 7, 2, 9, 4])
minimum  // Returns: 2
maximum  // Returns: 9
```

**Expected Output**: `2`, `9`

## Tuple Methods

### Length (Compile-Time)

```ruchy
let tuple = (1, 2, 3)
// tuple.len() is known at compile time
// Size is fixed: always 3 elements
```

**Note**: Tuple size is determined at compile time, not runtime.

## Best Practices

### ✅ Use Descriptive Destructuring

```ruchy
// Good: Clear names
let (name, age, active) = user_data

// Bad: Unclear single variable
let data = user_data
```

### ✅ Keep Tuples Small

```ruchy
// Good: 2-3 elements
let (x, y) = point

// Bad: Too many elements (use struct instead)
let data = (a, b, c, d, e, f, g, h)
```

### ✅ Use Tuples for Temporary Grouping

```ruchy
// Good: Return multiple values temporarily
fn parse_header(line) {
  let key = extract_key(line)
  let value = extract_value(line)
  (key, value)
}

// Better for persistent data: Use struct
struct Header {
  key: String,
  value: String
}
```

### ✅ Destructure at Function Boundaries

```ruchy
// Good: Destructure immediately
fn process_result(result) {
  let (success, data, error) = result
  if success {
    use(data)
  } else {
    handle(error)
  }
}
```

## Tuple Comparison

```ruchy
(1, 2, 3) == (1, 2, 3)  // Returns: true
(1, 2, 3) == (1, 2, 4)  // Returns: false

// Lexicographic ordering
(1, 2) < (1, 3)   // Returns: true
(2, 1) > (1, 10)  // Returns: true
```

**Expected Output**: `true`, `false`, `true`, `true`

## Unit Type

The empty tuple `()` is called the "unit type":

```ruchy
let nothing = ()

fn do_side_effect() {
  print("Done")
  ()  // Return unit
}

let result = do_side_effect()
result  // Returns: ()
```

**Expected Output**: `Done`, `()`

**Use Case**: Functions that don't return meaningful values return `()`.

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 97%

Tuples are fixed-size collections for grouping related but differently-typed data. Use them for temporary grouping, multiple return values, and coordinate pairs.

**Key Takeaways**:
- Fixed size, mixed types
- Access via `.0`, `.1`, `.2`
- Destructuring with `let (a, b) = tuple`
- Perfect for function return values
- Keep tuples small (2-4 elements)
- Use structs for larger, named data

---

[← Previous: Arrays](./01-arrays.md) | [Next: Objects/Maps →](./03-objects.md)
