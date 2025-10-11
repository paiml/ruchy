# Arrays - Feature 13/41

Arrays store ordered collections of values. They're the most common data structure in Ruchy.

## Creating Arrays

```ruchy
let numbers = [1, 2, 3, 4, 5]
let fruits = ["apple", "banana", "cherry"]
let empty = []
let mixed = [1, "two", 3.0, true]
```

**Test Coverage**: ✅ [tests/lang_comp/data_structures/arrays.rs](../../../../tests/lang_comp/data_structures/arrays.rs)

### Try It in the Notebook

```ruchy
let scores = [85, 92, 78, 95, 88]
scores  // Returns: [85, 92, 78, 95, 88]
```

**Expected Output**: `[85, 92, 78, 95, 88]`

## Accessing Elements

Use square brackets with zero-based index:

```ruchy
let fruits = ["apple", "banana", "cherry"]

fruits[0]  // Returns: "apple"
fruits[1]  // Returns: "banana"
fruits[2]  // Returns: "cherry"
```

**Expected Output**: `"apple"`, `"banana"`, `"cherry"`

### Negative Indices

```ruchy
fruits[-1]  // Returns: "cherry" (last item)
fruits[-2]  // Returns: "banana" (second to last)
```

## Array Methods

### `len()` - Length

```ruchy
let nums = [1, 2, 3, 4, 5]
nums.len()  // Returns: 5
```

**Expected Output**: `5`

### `push()` - Add to End

```ruchy
let arr = [1, 2, 3]
arr.push(4)
arr  // Returns: [1, 2, 3, 4]
```

**Expected Output**: `[1, 2, 3, 4]`

### `pop()` - Remove from End

```ruchy
let arr = [1, 2, 3, 4]
let last = arr.pop()

last  // Returns: 4
arr   // Returns: [1, 2, 3]
```

**Expected Output**: `4`, `[1, 2, 3]`

### `append()` - Combine Arrays

```ruchy
let a = [1, 2]
let b = [3, 4]
a.append(b)
a  // Returns: [1, 2, 3, 4]
```

**Expected Output**: `[1, 2, 3, 4]`

### `contains()` - Check Membership

```ruchy
let nums = [1, 2, 3, 4, 5]
nums.contains(3)  // Returns: true
nums.contains(10) // Returns: false
```

**Expected Output**: `true`, `false`

## Iteration

### For Loop

```ruchy
let numbers = [1, 2, 3, 4, 5]
let sum = 0

for n in numbers {
  sum = sum + n
}

sum  // Returns: 15
```

**Expected Output**: `15`

### With Index

```ruchy
let items = ["a", "b", "c"]

for (i, item) in items.enumerate() {
  print(f"{i}: {item}")
}
// Prints: 0: a, 1: b, 2: c
```

## Common Patterns

### Sum

```ruchy
let numbers = [10, 20, 30, 40, 50]
let total = 0

for n in numbers {
  total = total + n
}

total  // Returns: 150
```

### Filter

```ruchy
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let evens = []

for n in numbers {
  if n % 2 == 0 {
    evens.push(n)
  }
}

evens  // Returns: [2, 4, 6, 8, 10]
```

### Map

```ruchy
let numbers = [1, 2, 3, 4, 5]
let doubled = []

for n in numbers {
  doubled.push(n * 2)
}

doubled  // Returns: [2, 4, 6, 8, 10]
```

### Find

```ruchy
let numbers = [5, 12, 8, 130, 44]
let found = null

for n in numbers {
  if n > 100 {
    found = n
    break
  }
}

found  // Returns: 130
```

## Slicing

```ruchy
let arr = [0, 1, 2, 3, 4, 5]

arr[1..4]   // Returns: [1, 2, 3] (exclusive)
arr[1..=4]  // Returns: [1, 2, 3, 4] (inclusive)
arr[..3]    // Returns: [0, 1, 2] (from start)
arr[3..]    // Returns: [3, 4, 5] (to end)
```

## Multi-Dimensional Arrays

```ruchy
let matrix = [
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9]
]

matrix[0][0]  // Returns: 1
matrix[1][2]  // Returns: 6
matrix[2][1]  // Returns: 8
```

### Iterate Matrix

```ruchy
let matrix = [[1, 2], [3, 4]]
let sum = 0

for row in matrix {
  for value in row {
    sum = sum + value
  }
}

sum  // Returns: 10
```

## Array Comparison

```ruchy
[1, 2, 3] == [1, 2, 3]  // Returns: true
[1, 2, 3] == [1, 2, 4]  // Returns: false
```

## Best Practices

### ✅ Use Descriptive Names

```ruchy
let scores = [85, 92, 78]      // Good
let arr = [85, 92, 78]         // Bad
```

### ✅ Check Length Before Access

```ruchy
if arr.len() > 0 {
  let first = arr[0]
}
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 95%

Arrays are ordered collections with zero-based indexing. Use them for lists of similar items.

**Key Takeaways**:
- Zero-based indexing
- Methods: `len()`, `push()`, `pop()`, `append()`, `contains()`
- Iterate with for loops
- Slicing with `[start..end]`
- Can be multi-dimensional

---

[← Previous: Function Definitions](../04-functions/01-definitions.md) | [Next: Tuples →](./02-tuples.md)
