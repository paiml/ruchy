# BOOK-002: Standard Library Methods (BUG-004)

**Priority**: P0 - CRITICAL  
**Impact**: Would fix ~80+ book examples  
**Duration**: 2 days  
**Coverage Target**: 80%  
**Complexity Target**: All functions < 10 (PMAT enforced)

## Problem Statement

Common standard library methods like `.to_string()`, `format!()`, `.clone()`, `.len()`, `.push()`, `.pop()` are not available, affecting string manipulation and collection examples throughout the book.

## Root Cause Analysis (5 Whys)

1. **Why do stdlib methods fail?** Methods not implemented on Value types
2. **Why not implemented?** Initial focus on core language features
3. **Why core prioritized?** Needed working foundation first (correct)
4. **Why not added incrementally?** Lack of systematic feature tracking
5. **Why no tracking?** Missing book integration testing revealed gap late

## Solution Design

### Phase 1: String Methods (Day 1 Morning)
```rust
// Priority methods that unblock most examples
- to_string() -> String conversion
- len() -> length/size
- trim() -> whitespace removal  
- to_upper() / to_lower() -> case conversion
- split() -> string splitting
- contains() -> substring search
- replace() -> string replacement
```

### Phase 2: Collection Methods (Day 1 Afternoon)
```rust
// Essential collection operations
- push() / pop() -> Vec operations
- insert() / remove() -> HashMap operations
- get() -> safe access
- clone() -> deep copy
- clear() -> empty collection
- is_empty() -> emptiness check
```

### Phase 3: Formatting & Testing (Day 2)
```rust
// String formatting and comprehensive testing
- format!() macro equivalent
- println!() improvements
- Debug trait equivalent
- 80% test coverage
- Performance validation
```

## Test-Driven Development Plan

### RED Phase - Write Failing Tests
```rust
#[test]
fn test_string_to_string() {
    let result = eval("let x = 42; x.to_string()").unwrap();
    assert_eq!(result, Value::String("42".to_string()));
}

#[test]
fn test_string_len() {
    let result = eval("\"hello\".len()").unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_vec_push() {
    let result = eval("let mut v = [1, 2]; v.push(3); v").unwrap();
    assert_eq!(result, Value::Array(vec![
        Value::Int(1), Value::Int(2), Value::Int(3)
    ]));
}

#[test]
fn test_format_macro() {
    let result = eval("format!(\"Hello, {}!\", \"World\")").unwrap();
    assert_eq!(result, Value::String("Hello, World!".to_string()));
}
```

### GREEN Phase - Implement Methods
```rust
// In interpreter/value.rs
impl Value {
    pub fn call_method(&self, method: &str, args: Vec<Value>) -> Result<Value, Error> {
        match (self, method) {
            // String methods
            (Value::String(s), "len") => Ok(Value::Int(s.len() as i64)),
            (Value::String(s), "trim") => Ok(Value::String(s.trim().to_string())),
            (Value::String(s), "to_upper") => Ok(Value::String(s.to_uppercase())),
            (Value::String(s), "to_lower") => Ok(Value::String(s.to_lowercase())),
            
            // Numeric methods
            (Value::Int(n), "to_string") => Ok(Value::String(n.to_string())),
            (Value::Float(f), "to_string") => Ok(Value::String(f.to_string())),
            
            // Array methods
            (Value::Array(arr), "len") => Ok(Value::Int(arr.len() as i64)),
            (Value::Array(arr), "push") if args.len() == 1 => {
                let mut new_arr = arr.clone();
                new_arr.push(args[0].clone());
                Ok(Value::Array(new_arr))
            }
            
            _ => Err(Error::MethodNotFound(method.to_string()))
        }
    }
}
```

### REFACTOR Phase - Ensure Quality
- Group methods by type (string_methods.rs, array_methods.rs)
- Each method in separate function with complexity < 10
- Comprehensive documentation with examples
- Consistent error handling

## Success Metrics

1. **Primary**: 80+ book examples now work with stdlib methods
2. **Secondary**: 80% test coverage on method implementations
3. **Tertiary**: All methods have complexity < 10
4. **Quaternary**: Performance within 10% of native Rust

## Risk Mitigation

- **Risk**: Method name conflicts with user functions
- **Mitigation**: Methods have precedence, document clearly

- **Risk**: Performance overhead from dynamic dispatch
- **Mitigation**: Use inline caching for hot paths

- **Risk**: Mutability confusion
- **Mitigation**: Clear documentation on which methods mutate

## Quality Gates

- [ ] All method implementations have complexity < 10
- [ ] 80% test coverage on stdlib module
- [ ] All methods have doctests
- [ ] Performance benchmarks within acceptable range
- [ ] Book examples using methods pass

## Example Code That Should Work After Fix

```ruchy
// String methods
let name = "Alice";
println(name.to_upper());           // "ALICE"
println(name.len());                 // 5

let num = 42;
let str = num.to_string();          // "42"
println(str + " is the answer");    

// Collection methods
let mut items = [1, 2, 3];
items.push(4);
println(items.len());                // 4

let nums = [1, 2, 3, 4, 5];
let doubled = nums.map(|x| x * 2);
println(doubled);                    // [2, 4, 6, 8, 10]

// Format macro
let msg = format!("Hello, {}!", name);
println(msg);                        // "Hello, Alice!"
```

## Implementation Priority Order

1. **to_string()** - Unblocks most string operations
2. **len()** - Essential for collections and strings
3. **format!()** - String interpolation alternative
4. **push/pop** - Basic collection manipulation
5. **trim/split** - String processing
6. **map/filter/reduce** - Functional operations (may exist)

## Toyota Way Principles Applied

- **Jidoka**: Each method validates inputs, fails fast on errors
- **Genchi Genbutsu**: Test with actual book examples
- **Kaizen**: Add methods incrementally based on usage frequency
- **Respect for People**: Methods match user expectations from other languages
- **Long-term Philosophy**: Foundation for rich standard library