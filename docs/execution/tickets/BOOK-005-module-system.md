# BOOK-005: Basic Module System Support (FEATURE-001)

**Priority**: P1 - HIGH  
**Impact**: Enables multi-file examples and real programs  
**Duration**: 2-3 days  
**Coverage Target**: 80%  
**Complexity Target**: All functions < 10 (PMAT enforced)

## Problem Statement

No module system exists, preventing multi-file programs and proper code organization. This blocks all book examples that demonstrate modular programming, libraries, and real-world project structure.

## Root Cause Analysis (5 Whys)

1. **Why no module system?** Not implemented yet
2. **Why not implemented?** Focus on single-file execution first
3. **Why single-file focus?** REPL and scripting use cases prioritized
4. **Why those prioritized?** Faster iteration for language development
5. **Why not added now?** Complexity and design decisions delayed

## Solution Design

### Phase 1: Basic Import/Export (Day 1)
```rust
// Minimal module system
- use statements for importing
- mod declarations for modules
- pub visibility for exports
- File-based module discovery
```

### Phase 2: Module Resolution (Day 2)
```rust
// Path resolution and loading
- Resolve module paths
- Load and parse module files  
- Cache parsed modules
- Handle circular dependencies
```

### Phase 3: Symbol Resolution (Day 3)
```rust
// Name resolution and linking
- Build symbol tables
- Resolve imports to exports
- Type check across modules
- Generate combined AST
```

## Minimal Viable Module System

### Syntax Support
```ruchy
// math.ruchy
pub fun add(a, b) {
    a + b
}

pub fun multiply(a, b) {
    a * b
}

// main.ruchy
use math;

fun main() {
    let result = math::add(2, 3);
    println(result);
}
```

### Alternative: Simple Include
```ruchy
// Even simpler: just include files
include "math.ruchy";

fun main() {
    let result = add(2, 3);  // Functions available directly
    println(result);
}
```

## Test-Driven Development Plan

### RED Phase - Write Failing Tests
```rust
#[test]
fn test_basic_import() {
    create_file("math.ruchy", "pub fun add(a, b) { a + b }");
    let code = "use math; math::add(2, 3)";
    let result = eval_with_modules(code).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_module_visibility() {
    create_file("utils.ruchy", r#"
        fun private_helper() { 42 }
        pub fun public_func() { private_helper() }
    "#);
    let code = "use utils; utils::public_func()";
    let result = eval_with_modules(code).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_circular_dependency_detection() {
    create_file("a.ruchy", "use b; pub fun fa() { b::fb() }");
    create_file("b.ruchy", "use a; pub fun fb() { a::fa() }");
    let result = compile_with_modules("use a; a::fa()");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("circular"));
}
```

### GREEN Phase - Implement Basics
```rust
// In parser.rs
fn parse_use_statement(&mut self) -> Result<Import, ParseError> {
    self.expect_keyword("use")?;
    let module = self.expect_identifier()?;
    self.expect_semicolon()?;
    Ok(Import { module })
}

// In module_loader.rs
pub struct ModuleLoader {
    cache: HashMap<String, Module>,
    search_paths: Vec<PathBuf>,
}

impl ModuleLoader {
    pub fn load(&mut self, name: &str) -> Result<Module, Error> {
        if let Some(module) = self.cache.get(name) {
            return Ok(module.clone());
        }
        
        let path = self.resolve_module_path(name)?;
        let content = fs::read_to_string(&path)?;
        let module = self.parse_module(name, &content)?;
        
        self.cache.insert(name.to_string(), module.clone());
        Ok(module)
    }
}

// In resolver.rs  
pub fn resolve_symbols(modules: &[Module]) -> Result<SymbolTable, Error> {
    let mut symbols = SymbolTable::new();
    
    // First pass: collect all exports
    for module in modules {
        for item in &module.exports {
            symbols.define(item.qualified_name(), item)?;
        }
    }
    
    // Second pass: resolve imports
    for module in modules {
        for import in &module.imports {
            symbols.resolve_import(import)?;
        }
    }
    
    Ok(symbols)
}
```

### REFACTOR Phase - Ensure Quality
- Separate module loading from parsing
- Extract path resolution logic
- Ensure complexity < 10 for all functions
- Add caching and optimization

## Success Metrics

1. **Primary**: Multi-file book examples compile and run
2. **Secondary**: 80% test coverage on module system
3. **Tertiary**: Module loading performance < 100ms
4. **Quaternary**: Clear error messages for module issues

## Risk Mitigation

- **Risk**: Complex dependency graphs
- **Mitigation**: Start with single-level imports only

- **Risk**: Platform-specific path issues
- **Mitigation**: Use rust's PathBuf consistently

- **Risk**: Performance with many modules
- **Mitigation**: Cache aggressively, load lazily

## Quality Gates

- [ ] Module loader functions have complexity < 10
- [ ] 80% test coverage on module system
- [ ] Multi-file examples in book work
- [ ] Module loading < 100ms for typical project
- [ ] Circular dependency detection works

## Example Code That Should Work After Fix

### Example 1: Math Library
```ruchy
// lib/math.ruchy
pub fun add(a, b) { a + b }
pub fun sub(a, b) { a - b }
pub fun mul(a, b) { a * b }
pub fun div(a, b) { a / b }

// main.ruchy
use lib::math;

fun main() {
    let x = math::add(10, 5);
    let y = math::mul(x, 2);
    println(y);  // 30
}
```

### Example 2: Project Structure
```
project/
├── main.ruchy
├── utils/
│   ├── mod.ruchy
│   ├── strings.ruchy
│   └── numbers.ruchy
└── models/
    ├── mod.ruchy
    └── user.ruchy
```

### Example 3: Standard Library
```ruchy
// Using future stdlib modules
use std::io;
use std::fs;
use std::collections::HashMap;

fun main() {
    let contents = fs::read_file("data.txt");
    io::println(contents);
}
```

## Implementation Phases

### MVP Phase (This Sprint)
- Basic `use` statements
- File-based modules
- Simple name resolution
- No nested modules

### Phase 2 (Future)
- Nested modules
- Re-exports
- Visibility modifiers
- Module aliases

### Phase 3 (Future)
- Package management
- External dependencies
- Version resolution
- Publishing system

## Alternative: Include System

If module system too complex for sprint:

```rust
// Simple include preprocessor
fn preprocess_includes(code: &str) -> Result<String, Error> {
    let mut result = String::new();
    for line in code.lines() {
        if line.starts_with("include ") {
            let file = extract_include_path(line)?;
            let content = fs::read_to_string(file)?;
            result.push_str(&content);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    Ok(result)
}
```

## Toyota Way Principles Applied

- **Jidoka**: Detect circular dependencies automatically
- **Genchi Genbutsu**: Test with real multi-file projects
- **Kaizen**: Start simple, enhance incrementally
- **Respect for People**: Clear errors for module issues
- **Long-term Philosophy**: Foundation for package ecosystem