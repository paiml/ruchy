# Ruchy Parser Limitations Discovered During Lambda Runtime Development

**GitHub Issue**: https://github.com/paiml/ruchy/issues/137

**Date**: 2025-11-05
**Project**: ruchy-lambda (AWS Lambda custom runtime)
**Ruchy Version**: v3.182.0+
**Severity**: Medium - Blocks idiomatic Rust patterns in systems programming

## Summary

While implementing a pure Ruchy Lambda runtime (`ruchy-lambda/crates/runtime-pure`), we discovered several parser limitations that prevent writing idiomatic systems code. These limitations force workarounds in build.rs instead of clean Ruchy source code.

## Context

**Goal**: Implement AWS Lambda Runtime API HTTP client in 100% Ruchy code that transpiles to Rust.

**Challenge**: Ruchy parser fails on several common Rust patterns needed for systems programming (networking, I/O, error handling).

## Discovered Limitations

### 1. ❌ Vector Macro Syntax (`vec![...]`) Not Supported

**What we tried**:
```ruchy
fun http_post(&self, path: &str, body: &str) -> bool {
    let mut buffer = vec![0u8; 1024];  // ❌ Parse error
    // ...
}
```

**Error**:
```
Error: Failed to parse input
Caused by: Expected RightBrace, found Let
```

**Workaround**:
```ruchy
// Works but verbose
let mut buffer = Vec::new();
let mut temp_buf = [0u8; 1024];  // This works!
// Manually copy bytes
let mut i = 0usize;
while i < n {
    buffer.push(temp_buf[i]);
    i = i + 1;
}
```

**Impact**: Makes byte buffer handling extremely verbose and unidiomatic.

---

### 2. ❌ Module Declarations (`mod foo;`) Not Supported

**What we tried**:
```ruchy
mod http_client;  // Import Rust module

pub struct Runtime {
    // ...
}
```

**Error**:
```
Error: Failed to transpile to Rust
Caused by: Unsupported expression kind: ModuleDeclaration { name: "http_client" }
```

**Workaround**: Inject Rust module via build.rs post-processing:
```rust
// build.rs
let http_client_code = std::fs::read_to_string("src/http_client.rs")?;
let module = format!("mod http_client {{\n{}\n}}", http_client_code);
transpiled = format!("{}{}", module, transpiled);
```

**Impact**: Cannot compose Ruchy code with hand-written Rust modules. Forces all-or-nothing approach.

---

### 3. ⚠️ Module Path Separator (`.` vs `::`) Transpilation Bug

**What we wrote**:
```ruchy
let result = http_client::http_get(&endpoint, &path);
```

**What transpiler generated**:
```rust
let result = http_client.http_get(&endpoint, &path);  // ❌ Wrong!
```

**Error**:
```
error[E0423]: expected value, found module `http_client`
help: use the path separator to refer to an item
```

**Workaround**: String replacement in build.rs:
```rust
transpiled = transpiled.replace("http_client.http_get(", "http_client::http_get(");
```

**Impact**: Breaks module function calls. Requires brittle string replacements.

---

### 4. ❌ Complex Use Statements Not Supported

**What we tried**:
```ruchy
use std::io::{Read, Write};  // ❌ Parse error
use std::net::TcpStream;
```

**Error**:
```
Error: Failed to parse input
Caused by: Expected RightBrace, found Let
```

**What works**:
```ruchy
use std::io::Read;   // ✅ One import per line
use std::io::Write;
use std::net::TcpStream;
```

**Impact**: Verbose imports, but manageable workaround.

---

### 5. ⚠️ Transpiler Generates Mock Stubs for `std::net::TcpStream`

**What we wrote**:
```ruchy
use std::net::TcpStream;

let stream = TcpStream::connect(&endpoint)?;
```

**What transpiler generated**:
```rust
mod net {
    pub use std::net::*;
    pub struct TcpListener;
    pub struct TcpStream;
    impl TcpStream {
        pub fn connect(addr: String) -> Result<Self, String> {
            println!("Would connect to: {}", addr);  // ❌ Mock!
            Ok(TcpStream)
        }
    }
}
```

**Workaround**: Strip stub modules in build.rs:
```rust
if let Some(start) = transpiled.find("mod net {") {
    transpiled.replace_range(start..end, "");
}
transpiled = format!("use std::net::TcpStream;\n{}", transpiled);
```

**Impact**: Real `std::net::TcpStream` is shadowed by non-functional stub. Requires post-processing to use real stdlib.

---

## What We Need

### High Priority

1. **Support `vec![]` macro syntax** - Critical for systems programming
   - `vec![0u8; 1024]` for fixed-size buffers
   - `vec![1, 2, 3]` for initialization

2. **Support `mod foo;` declarations** - Enable Rust interop
   - Allow importing external Rust modules
   - Support `mod foo { ... }` inline modules

3. **Fix module path separator** - Correctness issue
   - `foo::bar()` should transpile to `foo::bar()`, not `foo.bar()`

### Medium Priority

4. **Support complex use statements** - Ergonomics
   - `use std::io::{Read, Write, BufReader}`
   - `use std::net::{TcpStream, TcpListener}`

5. **Disable stub generation for stdlib types** - Correctness
   - Don't generate mocks for `std::net::TcpStream`
   - Don't generate mocks for `std::io::Read/Write`
   - Or provide flag to disable: `--no-stubs`

---

## Test Case

See `ruchy-lambda/crates/runtime-pure/src/lib.ruchy` for full reproduction.

**Minimal reproduction**:
```ruchy
use std::net::TcpStream;
use std::io::{Read, Write};  // ❌ Parse error

pub struct Client {
    endpoint: String,
}

impl Client {
    pub fun connect(&self) -> bool {
        let stream = TcpStream::connect(&self.endpoint);
        if stream.is_err() {
            return false;
        }

        let mut buf = vec![0u8; 1024];  // ❌ Parse error
        true
    }
}
```

**To test**:
```bash
cd ruchy-lambda/crates/runtime-pure
ruchy transpile src/lib.ruchy
```

---

## Current Workaround Strategy

We're currently using a **hybrid approach**:
- **Ruchy**: High-level API (struct, methods, control flow)
- **Rust**: Low-level I/O (HTTP client in `http_client.rs`)
- **build.rs**: Post-processing glue (inject modules, fix paths)

This works but defeats the purpose of "100% Ruchy" systems programming.

---

## Impact on Ruchy Lambda Project

**Current status**: ✅ Working with workarounds
- Hybrid Ruchy+Rust runtime compiles successfully
- HTTP client in pure Rust, API in Ruchy
- Post-processing in build.rs handles limitations

**Ideal future**: Pure Ruchy implementation
- All runtime logic in `.ruchy` files
- No build.rs post-processing hacks
- Demonstrates Ruchy as systems programming language

**Composition**: Currently ~40% Ruchy, ~60% Rust (HTTP client)
**Goal**: ~90% Ruchy, ~10% Rust (only AWS SDK bindings)

---

## Related Work

**Similar projects using Ruchy for systems code**:
- `../ruchy-book` Chapter 21: Scientific benchmarks (fibonacci achieves 82% of C performance)
- `../ruchyruchy`: JIT compiler written in Ruchy (1,257 tests)

**Expected use cases**:
- AWS Lambda runtimes
- HTTP servers/clients
- CLI tools
- Embedded systems (future)

---

## Proposed Solutions

### Option 1: Extend Parser (Preferred)

Add support for missing Rust syntax:
- `vec![]` macro (already have `println!()` support)
- `mod foo;` declarations
- Complex `use` statements: `use foo::{bar, baz}`

### Option 2: Rust Interop Feature

Add explicit Rust interop syntax:
```ruchy
// Import entire Rust module
extern mod http_client;

// Call Rust function
let result = http_client::http_get(&endpoint, &path);
```

### Option 3: Disable Stub Generation Flag

```bash
ruchy transpile --no-stubs src/lib.ruchy
```

Don't generate mocks for `std::*` types.

---

## Files for Reference

All code in `ruchy-lambda` repository:
- `crates/runtime-pure/src/lib.ruchy` - Pure Ruchy attempt
- `crates/runtime-pure/src/http_client.rs` - Rust workaround
- `crates/runtime-pure/build.rs` - Post-processing hacks
- `crates/runtime-pure/examples/bootstrap.ruchy` - Pure Ruchy bootstrap (works!)

---

## Questions

1. Is `vec![]` macro support planned?
2. Should Ruchy support `mod foo;` or have explicit `extern mod` syntax?
3. Is the module path separator bug (`::` → `.`) known?
4. Can we disable stub generation for `std::net::*`?

---

## Priority Assessment

**Blocking**: No - workarounds exist
**Important**: Yes - limits Ruchy's systems programming story
**Timeline**: Would like for v3.183.0 (next release)

---

**Contact**: Noah (ruchy-lambda project maintainer)
**Tested with**: Ruchy v3.182.0, 4,031 tests passing
