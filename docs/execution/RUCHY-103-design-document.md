# RUCHY-103: Multi-File Module System Design Document

**Ticket ID**: RUCHY-103  
**Priority**: P1 - HIGH (Strategic Q1 2025 Objective)  
**Status**: Design Phase  
**Document Version**: 1.0  
**Date**: 2025-08-26

---

## Executive Summary

**OBJECTIVE**: Implement multi-file module system enabling `use external_file;` imports for larger Ruchy programs while preserving 100% compatibility with existing inline modules.

**CURRENT STATUS**: ✅ Inline modules fully functional, ❌ Multi-file imports fail with "unresolved module" errors.

**SOLUTION APPROACH**: Extend existing architecture with ModuleLoader component for file system integration while maintaining Toyota Way quality standards.

---

## Problem Analysis

### Current Architecture Assessment

**✅ WORKING PERFECTLY**:
- **Inline Module Syntax**: `mod math { pub fun add(a, b) { a + b } }`
- **Scope Resolution**: `math::add(5, 3)` transpiles to `math::add(5, 3)`
- **Public/Private Visibility**: `pub fun` exports work correctly
- **Nested Modules**: Multi-level hierarchies supported
- **AST Infrastructure**: Complete parsing for Import/Export/Module nodes
- **Transpiler Support**: `transpile_module`, `transpile_import`, `transpile_export` implemented

**❌ MISSING CRITICAL COMPONENTS**:
1. **File System Integration**: No mechanism to load `.ruchy` files from disk
2. **Module Path Resolution**: Cannot resolve `use math;` to `math.ruchy`
3. **Module Caching**: No deduplication of repeatedly imported modules
4. **Dependency Graph**: No circular dependency detection
5. **Standard Library**: No built-in `std::*` modules

### Root Cause Analysis (5 Whys)

1. **Why does `use math;` fail?** → Module loader doesn't exist to find `math.ruchy`
2. **Why no module loader?** → File system integration not implemented
3. **Why not implemented?** → Complex design decisions about module resolution
4. **Why complex?** → Need to handle paths, caching, circular dependencies  
5. **Why not prioritized?** → Inline modules were sufficient for basic book examples

**CONCLUSION**: Need systematic file system integration with proper architecture.

---

## Technical Design

### Architecture Overview

```rust
// New Components to Implement
pub struct ModuleLoader {
    cache: HashMap<String, ParsedModule>,
    search_paths: Vec<PathBuf>,
    dependency_graph: DependencyGraph,
}

pub struct ParsedModule {
    ast: Expr,
    file_path: PathBuf,
    dependencies: Vec<String>,
    last_modified: SystemTime,
}

pub struct DependencyGraph {
    nodes: HashMap<String, Vec<String>>,
}
```

### Implementation Strategy: 3-Phase Approach

#### Phase 1: Core File Loading (Week 1-2)
**Objective**: Basic `use other_file;` functionality working

**Components**:
1. **ModuleLoader Implementation**: File discovery and parsing
2. **Path Resolution Logic**: Find `.ruchy` files in search paths  
3. **Integration with Parser**: Extend existing `parse_import` to handle files
4. **Integration with Transpiler**: Generate correct Rust `mod` declarations

**Success Criteria**:
- [ ] Can import functions across files: `use math; math::add(1, 2)`
- [ ] Proper error messages for missing files
- [ ] Basic test suite demonstrating cross-file functionality

#### Phase 2: Advanced Features (Week 3-4)  
**Objective**: Production-ready module system

**Components**:
1. **Module Caching**: Avoid re-parsing unchanged files
2. **Dependency Graph**: Circular dependency detection
3. **Search Path Management**: Multiple directories, relative paths
4. **Enhanced Error Handling**: Clear messages for common mistakes

**Success Criteria**:
- [ ] Circular dependency detection with clear error messages
- [ ] Performance acceptable for realistic projects (100+ files)
- [ ] Advanced import patterns: `use utils::{helper1, helper2}`

#### Phase 3: Standard Library Foundation (Week 5-6)
**Objective**: Built-in modules ecosystem

**Components**:
1. **Standard Library Structure**: `std::collections`, `std::io`, `std::fs`
2. **Built-in Module Loading**: Embedded modules vs external files
3. **Documentation Integration**: Module-aware help system

**Success Criteria**:
- [ ] Basic standard library modules working
- [ ] Clear upgrade path for future standard library expansion

---

## Detailed Implementation Plan

### 1. ModuleLoader Core Implementation

```rust
// src/backend/module_loader.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use anyhow::{Result, bail};
use crate::frontend::parser::Parser;
use crate::frontend::ast::Expr;

pub struct ModuleLoader {
    cache: HashMap<String, ParsedModule>,
    search_paths: Vec<PathBuf>,
    loading_stack: Vec<String>, // For circular dependency detection
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            search_paths: vec![
                PathBuf::from("."),           // Current directory
                PathBuf::from("./src"),       // Source directory
                PathBuf::from("./modules"),   // Modules directory
            ],
            loading_stack: Vec::new(),
        }
    }
    
    pub fn load_module(&mut self, module_name: &str) -> Result<&ParsedModule> {
        // Check circular dependencies
        if self.loading_stack.contains(&module_name.to_string()) {
            bail!("Circular dependency detected: {} -> {}", 
                  self.loading_stack.join(" -> "), module_name);
        }
        
        // Check cache first
        if let Some(cached) = self.cache.get(module_name) {
            if self.is_cache_valid(cached)? {
                return Ok(cached);
            }
        }
        
        // Find and load file
        let file_path = self.resolve_module_path(module_name)?;
        let content = fs::read_to_string(&file_path)?;
        
        // Track loading for circular dependency detection
        self.loading_stack.push(module_name.to_string());
        
        // Parse module
        let mut parser = Parser::new(&content);
        let ast = parser.parse()?;
        
        let parsed_module = ParsedModule {
            ast,
            file_path: file_path.clone(),
            dependencies: self.extract_dependencies(&ast)?,
            last_modified: fs::metadata(&file_path)?.modified()?,
        };
        
        // Load dependencies recursively
        for dep in &parsed_module.dependencies {
            self.load_module(dep)?;
        }
        
        self.loading_stack.pop();
        self.cache.insert(module_name.to_string(), parsed_module);
        
        Ok(self.cache.get(module_name).expect("just inserted"))
    }
    
    fn resolve_module_path(&self, module_name: &str) -> Result<PathBuf> {
        let possible_names = [
            format!("{}.ruchy", module_name),
            format!("{}/mod.ruchy", module_name),
            format!("{}.rchy", module_name),
        ];
        
        for search_path in &self.search_paths {
            for name in &possible_names {
                let candidate = search_path.join(name);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
        
        bail!("Module '{}' not found in search paths: {:?}", 
              module_name, self.search_paths);
    }
    
    fn is_cache_valid(&self, module: &ParsedModule) -> Result<bool> {
        let current_modified = fs::metadata(&module.file_path)?.modified()?;
        Ok(current_modified <= module.last_modified)
    }
    
    fn extract_dependencies(&self, ast: &Expr) -> Result<Vec<String>> {
        // Extract all import statements and return module names
        // This would traverse the AST looking for Import nodes
        // TODO: Implement AST traversal for dependency extraction
        Ok(Vec::new())
    }
}

pub struct ParsedModule {
    pub ast: Expr,
    pub file_path: PathBuf,
    pub dependencies: Vec<String>,
    pub last_modified: SystemTime,
}
```

### 2. Parser Integration

```rust
// Extend src/frontend/parser/utils.rs parse_import function
pub fn parse_import(state: &mut ParserState) -> Result<Expr> {
    // ... existing URL import logic ...
    
    // NEW: Check if this is a simple module name (no :: in path)
    if !path.contains("::") {
        // This might be a file import - let the module loader handle it
        return Ok(Expr::new(
            ExprKind::Import { 
                path: path.clone(), 
                items: vec![ImportItem::Wildcard],
                is_file_import: true, // NEW FIELD
            }, 
            span
        ));
    }
    
    // ... existing inline module logic ...
}
```

### 3. Transpiler Integration

```rust
// Extend src/backend/transpiler/dispatcher.rs
impl Transpiler {
    pub fn set_module_loader(&mut self, loader: ModuleLoader) {
        self.module_loader = Some(loader);
    }
    
    fn transpile_import_with_loader(&mut self, path: &str, items: &[ImportItem], is_file_import: bool) -> Result<TokenStream> {
        if is_file_import {
            // Load the external module
            let module_loader = self.module_loader.as_mut()
                .ok_or_else(|| anyhow!("Module loader not initialized"))?;
            
            let parsed_module = module_loader.load_module(path)?;
            
            // Transpile the loaded module and generate appropriate mod declaration
            let module_ast = &parsed_module.ast;
            let module_tokens = self.transpile_expr(module_ast)?;
            let module_ident = format_ident!("{}", path);
            
            Ok(quote! {
                mod #module_ident {
                    #module_tokens
                }
                use #module_ident::*;
            })
        } else {
            // Use existing inline module transpilation
            Self::transpile_import(path, items)
        }
    }
}
```

### 4. Testing Strategy

```rust
// tests/multi_file_modules_test.rs
#[cfg(test)]
mod multi_file_module_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_basic_file_import() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create math.ruchy
        let math_content = r#"
            pub fun add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;
        fs::write(temp_dir.path().join("math.ruchy"), math_content)?;
        
        // Create main.ruchy that imports math
        let main_content = r#"
            use math;
            
            fun main() {
                let result = math::add(5, 3);
                println(result);
            }
        "#;
        
        let mut loader = ModuleLoader::new();
        loader.add_search_path(temp_dir.path());
        
        let mut transpiler = Transpiler::new();
        transpiler.set_module_loader(loader);
        
        let mut parser = Parser::new(main_content);
        let ast = parser.parse()?;
        
        let result = transpiler.transpile(&ast)?;
        let rust_code = result.to_string();
        
        // Verify generated Rust code
        assert!(rust_code.contains("mod math"));
        assert!(rust_code.contains("fn add"));
        assert!(rust_code.contains("math::add"));
        
        Ok(())
    }
    
    #[test]
    fn test_circular_dependency_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create a.ruchy that imports b
        fs::write(temp_dir.path().join("a.ruchy"), "use b;")?;
        
        // Create b.ruchy that imports a (circular dependency)
        fs::write(temp_dir.path().join("b.ruchy"), "use a;")?;
        
        let mut loader = ModuleLoader::new();
        loader.add_search_path(temp_dir.path());
        
        let result = loader.load_module("a");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Circular dependency"));
        
        Ok(())
    }
}
```

---

## Quality Assurance Plan

### Toyota Way Compliance

1. **Jidoka (Built-in Quality)**:
   - [ ] Comprehensive test suite with TDD methodology
   - [ ] Error detection at module loading boundaries
   - [ ] Quality gates prevent broken module system deployment

2. **Genchi Genbutsu (Go and See)**:
   - [ ] Test with real-world multi-file Ruchy projects
   - [ ] Analyze actual user import patterns from other languages
   - [ ] Direct observation of module loading performance

3. **Kaizen (Continuous Improvement)**:
   - [ ] Start with basic file loading, enhance incrementally
   - [ ] Learn from each test failure to improve architecture
   - [ ] Systematic complexity reduction throughout implementation

### Complexity Management

**MANDATORY**: All functions must maintain <10 cyclomatic complexity
- ModuleLoader methods will use helper functions for path resolution
- Dependency extraction will be separate concern
- Error handling will be centralized

### Test Coverage Requirements

**Baseline**: Maintain 97.4% TDD test suite success rate
**New Tests**: 
- [ ] 15+ unit tests for ModuleLoader components
- [ ] 5+ integration tests for cross-file functionality  
- [ ] 3+ property tests for edge cases (circular deps, missing files)
- [ ] 2+ regression tests for existing inline module compatibility

---

## Risk Assessment

### Technical Risks

1. **Performance Impact**
   - **Risk**: File I/O may slow down compilation
   - **Mitigation**: Aggressive caching, lazy loading
   - **Contingency**: Feature flag to disable multi-file modules

2. **Compatibility Regression**
   - **Risk**: Changes break existing inline modules
   - **Mitigation**: Comprehensive regression test suite
   - **Contingency**: Rollback mechanism with feature flags

3. **Complex Error Messages**
   - **Risk**: File import errors confuse users
   - **Mitigation**: Clear, actionable error messages with suggestions
   - **Contingency**: Enhanced documentation and examples

### Process Risks

1. **Scope Creep**
   - **Risk**: Attempting full standard library in Phase 1
   - **Mitigation**: Strict phase boundaries, clear success criteria
   - **Contingency**: MVP first, advanced features later

2. **Quality Debt**
   - **Risk**: Rushing implementation compromises architecture
   - **Mitigation**: Toyota Way discipline, complexity gates
   - **Contingency**: Technical debt sprint if quality metrics decline

---

## Success Metrics

### Phase 1 Success (Week 1-2)
- [ ] Basic `use other_file;` imports working
- [ ] Cross-file function calls: `other_file::function()`
- [ ] Clear error messages for missing files
- [ ] Zero regressions in existing inline module tests
- [ ] All new code <10 complexity

### Phase 2 Success (Week 3-4)
- [ ] Module caching reduces repeated file parsing by 80%
- [ ] Circular dependency detection with helpful error messages
- [ ] Advanced import patterns: `use utils::{helper1, helper2}`
- [ ] Performance acceptable for 100+ file projects (<500ms total load time)

### Phase 3 Success (Week 5-6)
- [ ] Basic standard library modules: `std::io`, `std::collections`
- [ ] Documentation integration showing available modules
- [ ] Clear upgrade path for future standard library expansion
- [ ] User feedback indicates excellent developer experience

---

## Timeline and Milestones

### Week 1: Foundation
- [ ] **Day 1-2**: ModuleLoader basic implementation
- [ ] **Day 3-4**: Parser integration for file imports
- [ ] **Day 5-6**: Basic transpiler integration
- [ ] **Day 7**: End-to-end test with simple file import working

### Week 2: Core Functionality
- [ ] **Day 8-9**: Path resolution algorithm with search paths
- [ ] **Day 10-11**: Error handling and user-friendly messages
- [ ] **Day 12-13**: Comprehensive test suite development
- [ ] **Day 14**: Integration with existing ruchy-book examples

### Week 3-4: Advanced Features
- [ ] **Week 3**: Module caching and dependency graph
- [ ] **Week 4**: Performance optimization and complex import patterns

### Week 5-6: Standard Library Foundation
- [ ] **Week 5**: Basic standard library modules
- [ ] **Week 6**: Documentation and ecosystem preparation

---

## Next Actions

### Immediate (Next 2 Days)
1. **Create ModuleLoader skeleton**: Basic struct and interfaces
2. **Extend AST**: Add `is_file_import` field to Import node
3. **Basic path resolution**: Simple file discovery algorithm
4. **First integration test**: Demonstrate basic file import

### Short-term (Week 1)  
1. **Implement core ModuleLoader**: Complete file loading and parsing
2. **Parser integration**: Extend `parse_import` for file imports
3. **Transpiler integration**: Generate correct Rust modules
4. **Testing infrastructure**: TDD test suite foundation

---

**Status**: 🚀 **READY TO IMPLEMENT**  
**Next Action**: Create ModuleLoader skeleton and first integration test  
**Confidence Level**: HIGH - Clear architecture, proven foundation, systematic approach

**Quality Commitment**: Every line of code will follow Toyota Way principles with comprehensive testing and complexity management. No shortcuts, no technical debt, no regressions.