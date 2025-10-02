# Migration Guide: 'from' Reserved Keyword

**Breaking Change**: v3.62.12+
**GitHub Issue**: [#23](https://github.com/paiml/ruchy/issues/23)

## Summary

The identifier `from` became a reserved keyword in Ruchy v3.62.12 in preparation for future import syntax (`from module import function`). This breaks backward compatibility with v1.89.0 where `from` was a valid identifier.

## Impact

🔴 **CRITICAL Breaking Change** - Affects code using `from` as:
- Function parameter names
- Local variable names
- Struct field names
- Object literal keys

## Migration Path

### ✗ Before (v1.89.0 - WORKED)

```ruchy
// Graph algorithms
fun dijkstra(graph: Graph, from: i32, to: i32) -> Path {
    // ...
}

// Network code
fun send_packet(from: Address, to: Address, data: Bytes) {
    // ...
}

// Time ranges
struct DateRange {
    from: Date,
    to: Date
}

// Variables
let from = get_start_node();
```

### ✓ After (v3.62.12+ - REQUIRED)

```ruchy
// Graph algorithms - Option 1: Use source/target
fun dijkstra(graph: Graph, source: i32, target: i32) -> Path {
    // ...
}

// Graph algorithms - Option 2: Use from_vertex/to_vertex
fun dijkstra(graph: Graph, from_vertex: i32, to_vertex: i32) -> Path {
    // ...
}

// Network code - Use sender/receiver
fun send_packet(sender: Address, receiver: Address, data: Bytes) {
    // ...
}

// Time ranges - Use start/end or start_date/end_date
struct DateRange {
    start_date: Date,
    end_date: Date
}

// Variables - Use source or start_node
let source = get_start_node();
let start_node = get_start_node();
```

## Recommended Replacements

| Context | Old (`from`) | Recommended Alternatives |
|---------|--------------|-------------------------|
| **Graph algorithms** | `from` | `source`, `from_vertex`, `start_node` |
| **Network/messaging** | `from` | `sender`, `source_addr`, `origin` |
| **Time/date ranges** | `from` | `start`, `start_date`, `begin` |
| **Data transformations** | `from` | `source`, `input`, `original` |

## Error Messages

When using `from` as an identifier, you'll see:

```
Error: 'from' is a reserved keyword (for future import syntax).
Suggestion: Use 'from_vertex', 'source', 'start_node', or similar instead.

Example:
✗ fun shortest_path(from, to) { ... }  // Error
✓ fun shortest_path(source, target) { ... }  // OK

See: https://github.com/paiml/ruchy/issues/23
```

## Rationale

The `from` keyword is reserved for future import syntax:

```ruchy
// Future syntax (not yet implemented)
from std.collections import HashMap, HashSet;
from math import sqrt, pow;
```

This enables Python-style selective imports, improving code clarity and reducing namespace pollution.

## Timeline

- **v1.89.0**: `from` was a valid identifier ✅
- **v3.62.12**: `from` became reserved keyword ❌ (Breaking Change)
- **Future**: Import syntax will be implemented using `from`

## Automated Migration

Use find-and-replace to update your codebase:

### Function Parameters

```bash
# Replace common graph algorithm patterns
sed -i 's/fun \(.*\)(from:/fun \1(source:/g' *.ruchy
sed -i 's/fun \(.*\)(from,/fun \1(source,/g' *.ruchy

# Replace in function bodies
sed -i 's/\bfrom\b/source/g' your_file.ruchy  # Review changes manually!
```

### Struct Fields

```bash
# Replace in struct definitions
sed -i 's/from: /start: /g' *.ruchy
```

**Warning**: Automated replacement may affect string literals or comments. Always review changes manually.

## Testing Your Migration

After migration, verify your code:

```bash
# Check syntax
ruchy check your_file.ruchy

# Run tests
ruchy test run tests/

# Run your program
ruchy run your_file.ruchy
```

## Real-World Example: Dijkstra Algorithm

### Before (v1.89.0)

```ruchy
fun dijkstra(
    graph: [[i32; 25]; 25],
    from: usize,
    to: usize
) -> Option<Path> {
    let mut dist = [MAX; 25];
    dist[from] = 0;

    // Algorithm implementation...
    // Uses 'from' throughout
}
```

### After (v3.62.12+)

```ruchy
fun dijkstra(
    graph: [[i32; 25]; 25],
    source: usize,
    target: usize
) -> Option<Path> {
    let mut dist = [MAX; 25];
    dist[source] = 0;

    // Algorithm implementation...
    // Uses 'source' and 'target' throughout
}
```

## Support

- **Issue Tracker**: [GitHub Issue #23](https://github.com/paiml/ruchy/issues/23)
- **Examples**: See `tests/from_keyword_regression.rs` for comprehensive test cases
- **Workarounds**: All documented in this guide

## Related Breaking Changes

- [#24]: Array references `&[T; N]` with 3+ parameters ✅ FIXED
- [#25]: `mut` in tuple destructuring ✅ FIXED
- [#23]: `from` reserved keyword (this document)

---

**Last Updated**: 2025-10-02
**Ruchy Version**: v3.64.1+
