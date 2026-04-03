# Module System Specification

## Overview

Ruchy's module system provides code organization, namespace management, and visibility control. It follows Rust's module system design with simplifications appropriate for a scripting language.

## Design Goals

1. **File-based modules**: One file = one module by default
2. **Explicit exports**: Public API clearly marked with `pub`
3. **Simple imports**: Clear, unambiguous import syntax
4. **No circular dependencies**: Enforced at load time
5. **Fast compilation**: Module graph built once, cached

## Syntax

### Module Definition

```ruchy
// math.ruchy - implicitly defines module 'math'
pub fun add(x: i32, y: i32) -> i32 {
    x + y
}

fun internal_helper() {  // private by default
    // ...
}
```

### Inline Modules

```ruchy
mod utils {
    pub fun format(s: String) -> String {
        // ...
    }
}
```

### Importing Modules

```ruchy
// Import specific items
use math::add
use std::collections::{HashMap, HashSet}

// Import all public items
use math::*

// Import with alias
use very::long::module::path as vl

// Import module itself
use math
let sum = math::add(1, 2)
```

### Visibility Rules

- Items are **private by default**
- `pub` makes items visible outside their module
- `pub(crate)` visible within the crate (future)
- Nested visibility follows lexical scope

## File Layout

```
project/
├── main.ruchy          # crate root
├── math.ruchy          # math module
├── utils/
│   ├── mod.ruchy      # utils module
│   ├── string.ruchy   # utils::string
│   └── format.ruchy   # utils::format
└── tests/
    └── math_test.ruchy # tests::math_test
```

## Module Resolution Algorithm

1. **Resolve use statements**:
   ```ruchy
   use math::add  // Look for math.ruchy or math/mod.ruchy
   ```

2. **Search paths** (in order):
   - Current directory
   - Parent directory (up to project root)
   - Standard library path
   - RUCHY_PATH directories

3. **Cache loaded modules**:
   - Each file loaded once per compilation
   - Module exports cached after first resolution

## Implementation Plan

### Phase 1: Basic Modules (RUCHY-0719) — Complete
- [x] Add `mod` keyword to lexer
- [x] Parse inline module definitions (`ExprKind::Module` in AST)
- [x] Parse module declarations (`ExprKind::ModuleDeclaration`)
- [x] Add `pub` visibility modifier (`is_pub` flag on Function, Struct, Class, Trait, Impl)
- [x] Visibility enum (`Public`, `PubCrate`, `PubSuper`, `Private`) in `ast.rs`

### Phase 2: File Modules — Partial
- [x] Implement file-based module loading (`ModuleResolver` in `backend/module_resolver.rs`)
- [x] Add module path resolution (`resolve_modules_for_execution` in `execution_handler.rs`)
- [ ] Cache loaded modules (currently re-resolves on each invocation)
- [ ] Detect circular dependencies (no cycle detection implemented)

### Phase 3: Import System — Implemented
- [x] Parse `use`/`import` statements (`ExprKind::Import`, `ImportAll`, `ImportDefault`)
- [x] Resolve imported symbols (via `ModuleResolver`)
- [x] Support wildcard imports (`ImportAll`)
- [ ] Add import aliases (not yet implemented)

## Examples

### Example 1: Math Library

```ruchy
// math/mod.ruchy
pub mod vec3
pub mod matrix

pub fun lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
```

```ruchy
// math/vec3.ruchy
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fun new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }
    
    pub fun dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}
```

```ruchy
// main.ruchy
use math::vec3::Vec3
use math::lerp

fun main() {
    let v1 = Vec3::new(1.0, 2.0, 3.0)
    let v2 = Vec3::new(4.0, 5.0, 6.0)
    
    let t = lerp(0.0, 1.0, 0.5)
    println(f"t = {t}")
}
```

### Example 2: Standard Library

```ruchy
use std::fs::File
use std::io::{Read, Write}
use std::collections::HashMap

fun process_config(path: String) -> HashMap<String, String> {
    let file = File::open(path)?
    let contents = file.read_to_string()?
    parse_config(contents)
}
```

## Comparison with Other Languages

| Feature | Ruchy | Rust | Python | JavaScript |
|---------|-------|------|--------|------------|
| File = Module | ✓ | ✓ | ✓ | ✓ (ES6) |
| Inline Modules | ✓ | ✓ | ✗ | ✗ |
| Private by Default | ✓ | ✓ | ✗ | ✗ |
| Explicit Exports | pub | pub | __all__ | export |
| Circular Deps | ✗ | ✗ | ✓ | ✓ |
| Module Cache | ✓ | N/A | ✓ | ✓ |

## Error Messages

```ruchy
// Clear, actionable error messages
error: Cannot find module 'math'
  --> main.ruchy:1:5
  |
1 | use math::add
  |     ^^^^ module not found
  |
  = help: Create math.ruchy or math/mod.ruchy
  = help: Available modules: std, utils

error: 'helper' is private
  --> main.ruchy:5:11
  |
5 | let x = math::helper()
  |         ^^^^^^^^^^^^ private function
  |
  = help: Consider making it public: pub fun helper()
```

## Standard Library Modules

```ruchy
std/
├── mod.ruchy      # Re-exports common items
├── fs.ruchy       # File system operations
├── io.ruchy       # Input/output
├── collections.ruchy  # Data structures
├── string.ruchy   # String utilities
├── process.ruchy  # Process management
└── net.ruchy      # Networking (future)
```

## Performance Considerations

1. **Module Graph**: Build once at startup, O(n) modules
2. **Symbol Resolution**: Hash-based lookup, O(1) average
3. **File Loading**: Cached after first read
4. **Incremental Compilation**: Only recompile changed modules (future)

## Security

1. **No arbitrary code execution during import** (unlike Python)
2. **Sandboxed module loading** (future)
3. **Capability-based imports** (future)

## Implementation Status (as of v4.2.1, 2026-04-03)

| Feature | Status | Key Files |
|---------|--------|-----------|
| `mod` keyword / inline modules | Implemented | `ast.rs`, `parser/expressions_helpers/modules.rs` |
| Module declarations (`mod name;`) | Implemented | `ExprKind::ModuleDeclaration` |
| `pub` visibility | Implemented | `visibility_modifiers.rs`, `Visibility` enum |
| File-based module loading | Implemented | `backend/module_resolver.rs` |
| Import statements (`use`/`import`) | Implemented | `ExprKind::Import`, `ImportAll`, `ImportDefault` |
| Formatter support | Implemented | `formatter.rs::format_module_system` |
| Transpiler analysis | Implemented | `transpiler/ast_analysis.rs::contains_imports` |
| Module caching | Not started | Modules re-resolved each invocation |
| Circular dependency detection | Not started | No cycle detection |
| Import aliases (`as`) | Not started | Syntax not yet parsed |