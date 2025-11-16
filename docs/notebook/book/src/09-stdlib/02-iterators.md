# Iterators - Feature 27/41

Iterators provide a way to process sequences of values lazily. They enable functional programming patterns like map, filter, and fold without creating intermediate collections.

## Creating Iterators

```ruchy
// From arrays
let arr = [1, 2, 3, 4, 5]
let iter = arr.iter()

// From vectors
let vec = vec![1, 2, 3]
let iter = vec.into_iter()

// From ranges
let range = 0..10
```

**Test Coverage**: ✅ [tests/lang_comp/iterators.rs](../../../../../tests/lang_comp/iterators.rs)

### Try It in the Notebook

```ruchy
let sum: i32 = (1..=5).sum()
sum  // Returns: 15
```

**Expected Output**: `15`

## Iterator Adapters

### map()

```ruchy
let doubled: Vec<_> = vec![1, 2, 3]
  .iter()
  .map(|x| x * 2)
  .collect()

doubled  // Returns: [2, 4, 6]
```

**Expected Output**: `[2, 4, 6]`

### filter()

```ruchy
let evens: Vec<_> = vec![1, 2, 3, 4, 5, 6]
  .into_iter()
  .filter(|x| x % 2 == 0)
  .collect()

evens  // Returns: [2, 4, 6]
```

**Expected Output**: `[2, 4, 6]`

### filter_map()

```ruchy
let parsed: Vec<_> = vec!["1", "two", "3"]
  .iter()
  .filter_map(|s| s.parse::<i32>().ok())
  .collect()

parsed  // Returns: [1, 3]
```

**Expected Output**: `[1, 3]`

## Iterator Consumers

### collect()

```ruchy
let vec: Vec<_> = (1..=5).collect()
vec  // Returns: [1, 2, 3, 4, 5]

let set: HashSet<_> = vec![1, 2, 2, 3].into_iter().collect()
set  // Returns: {1, 2, 3}
```

**Expected Output**: `[1, 2, 3, 4, 5]`, `{1, 2, 3}`

### sum() / product()

```ruchy
let sum: i32 = vec![1, 2, 3, 4].iter().sum()
sum  // Returns: 10

let product: i32 = vec![1, 2, 3, 4].iter().product()
product  // Returns: 24
```

**Expected Output**: `10`, `24`

### fold() / reduce()

```ruchy
let sum = (1..=5).fold(0, |acc, x| acc + x)
sum  // Returns: 15

let product = (1..=5).reduce(|acc, x| acc * x)
product  // Returns: Some(120)
```

**Expected Output**: `15`, `Some(120)`

### find() / position()

```ruchy
let found = vec![1, 2, 3, 4].iter().find(|&&x| x > 2)
found  // Returns: Some(&3)

let pos = vec![1, 2, 3, 4].iter().position(|&x| x > 2)
pos  // Returns: Some(2)
```

**Expected Output**: `Some(&3)`, `Some(2)`

### any() / all()

```ruchy
let has_even = vec![1, 3, 5, 6].iter().any(|x| x % 2 == 0)
has_even  // Returns: true

let all_positive = vec![1, 2, 3].iter().all(|x| x > &0)
all_positive  // Returns: true
```

**Expected Output**: `true`, `true`

## Chaining Adapters

```ruchy
let result: Vec<_> = vec![1, 2, 3, 4, 5, 6]
  .into_iter()
  .filter(|x| x % 2 == 0)    // [2, 4, 6]
  .map(|x| x * x)             // [4, 16, 36]
  .take(2)                    // [4, 16]
  .collect()

result  // Returns: [4, 16]
```

**Expected Output**: `[4, 16]`

## Common Patterns

### Transform and Collect

```ruchy
fn square_evens(numbers: Vec<i32>) -> Vec<i32> {
  numbers.into_iter()
    .filter(|x| x % 2 == 0)
    .map(|x| x * x)
    .collect()
}

square_evens(vec![1, 2, 3, 4, 5, 6])
// Returns: [4, 16, 36]
```

**Expected Output**: `[4, 16, 36]`

### Partition

```ruchy
let numbers = vec![1, 2, 3, 4, 5, 6]
let (evens, odds): (Vec<_>, Vec<_>) = numbers
  .into_iter()
  .partition(|x| x % 2 == 0)

evens  // Returns: [2, 4, 6]
odds   // Returns: [1, 3, 5]
```

**Expected Output**: `[2, 4, 6]`, `[1, 3, 5]`

### Enumerate

```ruchy
for (i, value) in vec!["a", "b", "c"].iter().enumerate() {
  println!("{}: {}", i, value)
}
// Prints: 0: a, 1: b, 2: c
```

**Expected Output**: Indexed pairs

### Zip

```ruchy
let names = vec!["Alice", "Bob", "Charlie"]
let ages = vec![30, 25, 35]

let pairs: Vec<_> = names.iter()
  .zip(ages.iter())
  .collect()

pairs  // Returns: [("Alice", 30), ("Bob", 25), ("Charlie", 35)]
```

**Expected Output**: Paired tuples

### Flatten

```ruchy
let nested = vec![vec![1, 2], vec![3, 4], vec![5, 6]]
let flat: Vec<_> = nested.into_iter().flatten().collect()

flat  // Returns: [1, 2, 3, 4, 5, 6]
```

**Expected Output**: `[1, 2, 3, 4, 5, 6]`

## Range Iterators

```ruchy
// Exclusive range
let r1: Vec<_> = (0..5).collect()
r1  // Returns: [0, 1, 2, 3, 4]

// Inclusive range
let r2: Vec<_> = (0..=5).collect()
r2  // Returns: [0, 1, 2, 3, 4, 5]

// Step by
let r3: Vec<_> = (0..10).step_by(2).collect()
r3  // Returns: [0, 2, 4, 6, 8]
```

**Expected Output**: Various ranges

## take() / skip() / take_while() / skip_while()

```ruchy
let vec = vec![1, 2, 3, 4, 5]

vec.iter().take(3).collect()       // [1, 2, 3]
vec.iter().skip(2).collect()       // [3, 4, 5]
vec.iter().take_while(|&&x| x < 4).collect()  // [1, 2, 3]
vec.iter().skip_while(|&&x| x < 3).collect()  // [3, 4, 5]
```

**Expected Output**: Various slices

## Iterator Performance

### Lazy Evaluation

```ruchy
// No computation until collect()
let iter = (1..1_000_000)
  .map(|x| x * 2)
  .filter(|x| x % 3 == 0)
  .take(10)

// Computation happens here
let result: Vec<_> = iter.collect()
```

**Expected Output**: Only computes 10 elements

### Zero-Cost Abstractions

```ruchy
// Iterator chain (zero allocation)
let sum: i32 = (1..=100)
  .filter(|x| x % 2 == 0)
  .map(|x| x * x)
  .sum()

// Equivalent manual loop
let mut sum = 0
for x in 1..=100 {
  if x % 2 == 0 {
    sum += x * x
  }
}
// Both have same performance!
```

**Expected Output**: Same performance characteristics

## Custom Iterators

```ruchy
struct Counter {
  count: usize,
  max: usize
}

impl Counter {
  fn new(max: usize) -> Self {
    Counter { count: 0, max }
  }
}

impl Iterator for Counter {
  type Item = usize

  fn next(&mut self) -> Option<Self::Item> {
    if self.count < self.max {
      self.count += 1
      Some(self.count)
    } else {
      None
    }
  }
}

let counter = Counter::new(5)
let sum: usize = counter.sum()
sum  // Returns: 15 (1+2+3+4+5)
```

**Expected Output**: `15`

## Best Practices

### ✅ Use Iterators for Transformations

```ruchy
// Good: Functional, clear
let squared: Vec<_> = numbers
  .iter()
  .map(|x| x * x)
  .collect()

// Bad: Imperative, verbose
let mut squared = Vec::new()
for x in &numbers {
  squared.push(x * x)
}
```

### ✅ Chain Adapters for Readability

```ruchy
// Good: Clear pipeline
users
  .iter()
  .filter(|u| u.active)
  .map(|u| u.name)
  .collect()

// Bad: Nested loops
let mut names = Vec::new()
for user in &users {
  if user.active {
    names.push(user.name)
  }
}
```

### ✅ Use fold() for Complex Reductions

```ruchy
// Good: Single pass
let stats = numbers.iter().fold((0, 0, 0), |(sum, count, max), &x| {
  (sum + x, count + 1, max.max(x))
})

// Bad: Multiple passes
let sum: i32 = numbers.iter().sum()
let count = numbers.len()
let max = numbers.iter().max().unwrap()
```

### ✅ Prefer iter() over into_iter() When Possible

```ruchy
// Good: Borrow, reusable
let sum: i32 = vec.iter().sum()
let product: i32 = vec.iter().product()

// Bad: Move, can't reuse
let sum: i32 = vec.into_iter().sum()
// vec is now moved, can't use again
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 94%

Iterators provide lazy, composable transformations over sequences. They enable functional programming patterns with zero-cost abstractions.

**Key Takeaways**:
- Adapters: map, filter, filter_map, take, skip
- Consumers: collect, sum, fold, find, any, all
- Lazy evaluation: No work until consumed
- Zero-cost: Same performance as manual loops
- Chain adapters for readable pipelines
- Custom iterators via Iterator trait

---

[← Previous: Collections](./01-collections.md) | [Next: I/O →](./03-io.md)
