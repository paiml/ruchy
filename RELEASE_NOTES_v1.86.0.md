# Ruchy v1.86.0 Release Notes

## 🎯 Overview
Version 1.86.0 brings significant language enhancements with destructuring patterns, if-let/while-let syntax, and improved developer experience through better quality gates and test infrastructure.

## ✨ New Features

### Pattern Matching & Destructuring
- **Array Destructuring**: `let [a, b, c] = [1, 2, 3]`
- **Tuple Destructuring**: `let (x, y) = (10, 20)`
- **Rest Patterns**: `let [first, ...rest] = [1, 2, 3, 4]`
- **Spread Operator**: `let combined = [...arr1, ...arr2]`
- **Default Values**: `let [a = 10, b = 20] = [1]`
- **Object Destructuring**: `let {name, age} = person`
- **Mixed Patterns**: `let ([a, b], {x, y}) = complex_data`
- **Function Parameters**: `fun process([x, y]) { x + y }`

### If-let & While-let Syntax
- **If-let Pattern Matching**: `if let Some(x) = maybe { ... }`
- **While-let Loops**: `while let Some(item) = iter.next() { ... }`
- **Result Handling**: `if let Ok(value) = result { ... }`
- **Chained Patterns**: `else if let Some(x) = other { ... }`

## 🛠️ Improvements

### Developer Experience
- **PMAT-style Pre-commit Hook**: Cleaner, more informative quality gates
- **Enhanced Error Messages**: Better context for parsing errors
- **Test Infrastructure**: Fixed compilation issues, 898 library tests passing

### Code Quality
- **Refactored Parser**: Reduced complexity in if-expression parsing
- **TDG Score**: Maintained at 94.0/100 (A grade)
- **Coverage**: 49.90% overall coverage
- **Zero Technical Debt**: No TODO/FIXME comments

## 📊 Compatibility

### Book Integration
- **Overall**: 85% compatibility maintained
- **Ch04 Practical Patterns**: Working examples
- **Ch17 Error Handling**: If-let patterns functional
- **One-liners**: 95% success rate

### Working Features
- ✅ Lambdas and closures
- ✅ Higher-order functions
- ✅ Pattern matching (match expressions)
- ✅ Result/Option types
- ✅ List comprehensions
- ✅ String interpolation
- ✅ DataFrames

## 🔧 Technical Details

### Test Results
- **Library Tests**: 898 passing
- **If-let Tests**: 4/7 passing (common cases work)
- **Integration**: Some tests need updates

### Quality Metrics
- **Cyclomatic Complexity**: All functions ≤10
- **Cognitive Complexity**: All functions ≤10
- **SATD Comments**: 0
- **Documentation**: Comprehensive for public APIs

## 🚀 Getting Started

```bash
# Install or update
cargo install ruchy

# Test if-let syntax
echo 'if let Some(x) = Some(42) { println("x: " + x.to_string()) }' > test.ruchy
ruchy test test.ruchy

# Use destructuring
ruchy -e 'let [a, b, ...rest] = [1, 2, 3, 4, 5]; println(rest.to_string())'
```

## 📝 Migration Notes

### Breaking Changes
None - all changes are backward compatible.

### Deprecations
- `ruchy-cli` package is deprecated (use main `ruchy` package)

## 🔮 Next Priorities

1. **Complete If-let**: Support for custom enums and nested patterns
2. **Async/Await**: Implementation of async functions
3. **Traits**: Trait definitions and implementations
4. **Generics**: Generic functions and types

## 🙏 Acknowledgments

Thanks to all contributors and the Ruchy community for continuous support and feedback.

---

**Full Changelog**: https://github.com/paiml/ruchy/compare/v1.85.0...v1.86.0