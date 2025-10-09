# Ruchy Cargo Integration Strategy
**Date**: 2025-10-09
**Version**: v3.71.1
**Strategy**: Leverage Rust/Cargo ecosystem instead of building from scratch

---

## Executive Summary

**Decision**: Build Ruchy as a **Cargo-first language** that leverages the entire Rust ecosystem.

**Impact on Timeline**:
- **Before** (custom package manager): 6-12 months, 866+ hours
- **After** (Cargo integration): **3-4 months, 350-400 hours**
- **Savings**: 60% reduction in effort

**Key Insight**: Ruchy already transpiles to Rust. Instead of building a separate package ecosystem, **embrace Cargo as the native package manager**.

---

## 1. Package Management via Cargo

### Current Approach (Transpiler)
Ruchy already generates Rust code:
```ruchy
fun main() {
    println("Hello, Ruchy!");
}
```
↓ transpiles to ↓
```rust
fn main() {
    println!("Hello, Ruchy!");
}
```

### Proposed Approach: Cargo-Native Ruchy

**Ruchy Package = Rust Crate with `.ruchy` source files**

**Example Project Structure**:
```
my-ruchy-app/
├── Cargo.toml          # Standard Cargo manifest
├── src/
│   ├── main.ruchy      # Ruchy source (entry point)
│   ├── lib.ruchy       # Ruchy library code
│   └── utils.ruchy     # Ruchy modules
├── build.rs            # Build script (transpiles .ruchy → .rs)
└── target/             # Standard Cargo build output
```

**Cargo.toml Example**:
```toml
[package]
name = "my-ruchy-app"
version = "0.1.0"
edition = "2021"
build = "build.rs"  # Custom build script for Ruchy

[build-dependencies]
ruchy = "3.71"      # Ruchy transpiler as build dependency

[dependencies]
reqwest = "0.11"    # Use any Rust crate directly!
serde_json = "1.0"
tokio = "1.0"
```

**build.rs (Transpiler Integration)**:
```rust
// Automatically transpile .ruchy files to .rs during build
fn main() {
    ruchy::build_transpiler::transpile_all("src/**/*.ruchy", "src/")
        .expect("Failed to transpile Ruchy files");
}
```

**Workflow**:
```bash
# Standard Cargo commands just work!
cargo new my-app        # Create new Ruchy project
cargo add reqwest       # Add dependencies (any Rust crate)
cargo build             # Build (auto-transpiles .ruchy → .rs)
cargo run               # Run the app
cargo test              # Run tests
cargo publish           # Publish to crates.io
```

**Benefits**:
- ✅ **Zero new infrastructure** - use existing crates.io
- ✅ **Immediate ecosystem** - access to 140,000+ crates
- ✅ **Battle-tested tooling** - Cargo's dependency resolution
- ✅ **Familiar workflow** - Rust developers feel at home
- ✅ **No maintenance burden** - let Cargo team handle it

**Effort**: ~40 hours (build script integration + documentation)

---

## 2. Standard Library via Rust Crates

### Strategy: Thin Ruchy Wrappers Around Rust Crates

Instead of reimplementing standard library, **wrap existing high-quality Rust crates**:

**File I/O** → `std::fs` + `std::io`
```ruchy
// Ruchy wrapper
import fs from "ruchy/std/fs";

fun read_file(path: String) -> Result<String, Error> {
    fs::read_to_string(path)  // Direct delegation to Rust std::fs
}
```

**HTTP Client** → `reqwest` crate
```ruchy
// Ruchy wrapper
import http from "ruchy/std/http";

async fun fetch(url: String) -> Result<String, Error> {
    http::get(url).await?.text().await  // Wraps reqwest
}
```

**JSON** → `serde_json` crate
```ruchy
// Ruchy wrapper
import json from "ruchy/std/json";

fun parse<T>(text: String) -> Result<T, Error> {
    json::from_str(text)  // Wraps serde_json
}
```

**Date/Time** → `chrono` crate
```ruchy
import time from "ruchy/std/time";

fun now() -> DateTime {
    time::Utc::now()  // Wraps chrono
}
```

**Logging** → `tracing` crate
```ruchy
import log from "ruchy/std/log";

fun main() {
    log::info("Application started");  // Wraps tracing
}
```

**Benefits**:
- ✅ **Proven reliability** - battle-tested crates
- ✅ **Ongoing maintenance** - Rust community maintains them
- ✅ **Performance** - optimized by experts
- ✅ **Minimal effort** - thin wrappers vs full implementation

**Standard Library Roadmap**:

| Module | Rust Crate | Effort | Priority |
|--------|------------|--------|----------|
| `ruchy/std/fs` | std::fs | 8 hrs | P0 |
| `ruchy/std/http` | reqwest | 12 hrs | P0 |
| `ruchy/std/json` | serde_json | 8 hrs | P0 |
| `ruchy/std/time` | chrono | 8 hrs | P1 |
| `ruchy/std/log` | tracing | 8 hrs | P1 |
| `ruchy/std/regex` | regex | 8 hrs | P1 |
| `ruchy/std/db` | sqlx | 16 hrs | P2 |
| `ruchy/std/crypto` | sha2, aes | 12 hrs | P2 |
| **Total** | | **80 hrs** | |

**Effort**: ~80 hours (vs 300 hours for custom implementation)

---

## 3. Revised Production Timeline

### Phase 1: Cargo Integration (Month 1)

**Week 1-2: Build System Integration**
- [ ] **CARGO-001**: Create build.rs transpiler integration (16 hours)
  - Implement `ruchy::build_transpiler::transpile_all()`
  - Handle incremental compilation
  - Error reporting integration
  - Add `ruchy` as build dependency

- [ ] **CARGO-002**: Project template generation (8 hours)
  - `ruchy new <project>` command (wraps `cargo new`)
  - Default Cargo.toml with Ruchy setup
  - Example .ruchy files
  - README template

- [ ] **CARGO-003**: Dependency integration testing (8 hours)
  - Test using popular Rust crates from Ruchy
  - Verify FFI boundary
  - Document patterns

**Week 3-4: Standard Library Core (P0)**
- [ ] **STD-001**: File I/O module (`ruchy/std/fs`) (8 hours)
- [ ] **STD-002**: HTTP client module (`ruchy/std/http`) (12 hours)
- [ ] **STD-003**: JSON module (`ruchy/std/json`) (8 hours)

**Total**: 60 hours (1 month)

---

### Phase 2: Standard Library Expansion (Month 2)

**Week 1-2: Essential Utilities (P1)**
- [ ] **STD-004**: Time/Date module (`ruchy/std/time`) (8 hours)
- [ ] **STD-005**: Logging module (`ruchy/std/log`) (8 hours)
- [ ] **STD-006**: Regex module (`ruchy/std/regex`) (8 hours)
- [ ] **STD-007**: Environment module (`ruchy/std/env`) (4 hours)

**Week 3-4: DataFrame Resolution**
- [ ] **DF-DECISION**: Evaluate DataFrame options (4 hours)
  - Option A: Complete implementation (83 hours)
  - Option B: Remove and mark experimental (4 hours)
  - Option C: Wrap polars-rs crate (20 hours) ← **RECOMMENDED**

- [ ] **DF-IMPL**: Implement chosen option (4-83 hours)

**Total**: 40-120 hours (depending on DataFrame decision)

---

### Phase 3: Quality & Documentation (Month 3)

**Week 1-2: Quality Stabilization**
- [ ] **QUALITY-018**: Remove duplicate `values_equal()` (1 hour)
- [ ] **QUALITY-019**: Simplify `match_ok_pattern()` CC 36 → <10 (3 hours)
- [ ] **QUALITY-020**: Create validation trait (40 hours)
- [ ] **QUALITY-021**: Extract API client abstraction (20 hours)
- [ ] **QUALITY-022**: DataFrame operations refactoring (20 hours)

**Week 3-4: Documentation & Examples**
- [ ] **DOC-001**: Cargo integration guide (8 hours)
- [ ] **DOC-002**: Standard library API docs (12 hours)
- [ ] **DOC-003**: Migration guide from v3.x (8 hours)
- [ ] **DOC-004**: Example projects (12 hours)
  - HTTP server example
  - CLI tool example
  - Data processing example
  - WASM web app example

**Total**: 124 hours

---

### Phase 4: Beta Release (Month 4 - Polish)

**Week 1-2: Testing & Validation**
- [ ] **TEST-001**: End-to-end Cargo workflow tests (16 hours)
- [ ] **TEST-002**: Third-party crate compatibility tests (12 hours)
- [ ] **TEST-003**: Performance benchmarking (16 hours)
- [ ] **TEST-004**: Security audit (20 hours)

**Week 3-4: Release Preparation**
- [ ] **RELEASE-001**: v4.0.0-beta.1 preparation (8 hours)
- [ ] **RELEASE-002**: Migration tooling (16 hours)
- [ ] **RELEASE-003**: Announcement materials (8 hours)
- [ ] **RELEASE-004**: Beta testing program (8 hours)

**Total**: 104 hours

---

## 4. Total Revised Effort

| Phase | Duration | Effort |
|-------|----------|--------|
| **Phase 1**: Cargo Integration + Core Stdlib | 1 month | 60 hrs |
| **Phase 2**: Stdlib Expansion + DataFrame | 1 month | 40-120 hrs |
| **Phase 3**: Quality & Documentation | 1 month | 124 hrs |
| **Phase 4**: Beta Release | 1 month | 104 hrs |
| **Total** | **4 months** | **328-408 hrs** |

**Previous Estimate** (custom package manager): 866+ hours
**New Estimate** (Cargo-based): 328-408 hours
**Savings**: **458-538 hours (60% reduction)**

---

## 5. Cargo Integration Examples

### Example 1: Simple HTTP Server

**project structure**:
```
my-server/
├── Cargo.toml
├── build.rs
└── src/
    └── main.ruchy
```

**Cargo.toml**:
```toml
[package]
name = "my-server"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[build-dependencies]
ruchy = "3.71"

[dependencies]
axum = "0.7"         # Use any Rust web framework
tokio = { version = "1", features = ["full"] }
```

**src/main.ruchy**:
```ruchy
import axum from "axum";
import tokio from "tokio";

async fun hello() -> String {
    "Hello from Ruchy!"
}

#[tokio::main]
async fun main() {
    let app = axum::Router::new()
        .route("/", axum::routing::get(hello));

    axum::Server::bind("0.0.0.0:3000")
        .serve(app.into_make_service())
        .await
        .expect("Server failed");
}
```

**Build & Run**:
```bash
cargo build    # Auto-transpiles .ruchy → .rs
cargo run      # Runs the server
curl localhost:3000  # "Hello from Ruchy!"
```

---

### Example 2: CLI Tool with Dependencies

**Cargo.toml**:
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["blocking"] }
```

**src/main.ruchy**:
```ruchy
import clap from "clap";
import json from "serde_json";
import http from "reqwest";

#[derive(clap::Parser)]
struct Args {
    url: String,
}

fun main() {
    let args = Args::parse();

    // Use Rust crate directly
    let response = http::blocking::get(args.url)
        .expect("Request failed")
        .text()
        .expect("Failed to read response");

    // Parse JSON
    let data: json::Value = json::from_str(&response)
        .expect("Invalid JSON");

    println("Response: {}", data);
}
```

**Usage**:
```bash
cargo run -- https://api.github.com/users/octocat
```

---

## 6. Migration Path for Existing Ruchy Code

### v3.x (Current) → v4.x (Cargo-based)

**Before (v3.x)**:
```bash
# Standalone Ruchy scripts
ruchy run script.ruchy
ruchy compile script.ruchy
```

**After (v4.x)**:
```bash
# Cargo project
cargo new my-project
# Move script.ruchy to src/main.ruchy
cargo build   # Transpiles automatically
cargo run
```

**Migration Tool** (auto-convert):
```bash
ruchy migrate script.ruchy --to-cargo
# Creates:
# - Cargo.toml
# - build.rs
# - src/main.ruchy (from script.ruchy)
```

**Backward Compatibility**:
- Keep `ruchy run` for quick scripts
- Keep `ruchy compile` for one-off builds
- Add `cargo` workflow as the recommended approach

---

## 7. Benefits Summary

### Immediate Benefits
1. **Access to 140,000+ crates** from day 1
2. **Zero package registry infrastructure** to build/maintain
3. **Familiar tooling** for Rust developers
4. **Battle-tested dependency resolution**
5. **Automatic security advisories** via cargo-audit

### Long-term Benefits
1. **No package ecosystem split-brain** (one ecosystem, not two)
2. **Easier Rust interop** (Ruchy ↔ Rust is seamless)
3. **Lower maintenance burden** (Cargo team handles tooling)
4. **Natural migration path** to/from Rust
5. **Production-ready from day 1** (leverage Rust's maturity)

### Strategic Benefits
1. **Faster to production** (4 months vs 12 months)
2. **Lower risk** (proven infrastructure vs custom)
3. **Better positioning** ("Rust-based scripting language with Cargo support")
4. **Ecosystem leverage** (not starting from zero)

---

## 8. Risks & Mitigation

### Risk 1: Rust Dependency Hell
**Problem**: Users might struggle with Cargo.toml syntax
**Mitigation**:
- Provide `ruchy add <crate>` wrapper around `cargo add`
- Clear documentation with examples
- Error messages that explain Cargo concepts

### Risk 2: Build Script Complexity
**Problem**: build.rs might become complex
**Mitigation**:
- Keep transpiler integration simple
- Provide templates and generators
- Document common patterns

### Risk 3: Ecosystem Confusion
**Problem**: "Is this Rust or Ruchy?"
**Mitigation**:
- Clear branding: "Ruchy - A Rust-based scripting language"
- Document the relationship explicitly
- Position as a strength, not a weakness

### Risk 4: Transpiler Performance
**Problem**: Build times might be slow with transpilation
**Mitigation**:
- Implement incremental compilation
- Cache transpiled .rs files
- Optimize transpiler performance
- Benchmark and monitor

---

## 9. Success Criteria

### Phase 1 Success (Month 1)
- ✅ Can `cargo new` a Ruchy project
- ✅ Can `cargo add` any Rust crate
- ✅ Can `cargo build` and auto-transpile works
- ✅ Can use at least 3 popular crates (reqwest, serde_json, tokio)

### Phase 2 Success (Month 2)
- ✅ Core stdlib modules working (fs, http, json)
- ✅ DataFrame decision made and implemented
- ✅ Can build real applications (HTTP server, CLI tool)

### Phase 3 Success (Month 3)
- ✅ Quality debt reduced below 30 violations
- ✅ Complete API documentation
- ✅ 5+ working example projects

### Phase 4 Success (Month 4)
- ✅ v4.0.0-beta.1 released
- ✅ Can publish Ruchy crates to crates.io
- ✅ Security audit complete
- ✅ Performance benchmarks published

---

## 10. Next Steps

### Immediate (This Week)
1. ✅ **Create this strategy document**
2. [ ] **CARGO-001**: Prototype build.rs integration (16 hours)
3. [ ] **TEST**: Validate concept with simple example
4. [ ] **DOC**: Update roadmap with Cargo strategy

### Week 2
1. [ ] **CARGO-002**: Create project template generator
2. [ ] **STD-001**: Implement fs module
3. [ ] **EXAMPLE**: Build HTTP server proof-of-concept

### Month 1 Goal
- Working Cargo integration
- Core stdlib (fs, http, json)
- At least 2 working example projects

---

**Strategy Status**: ✅ **Approved for Implementation**

**Timeline**: 4 months to v4.0.0-beta.1

**Effort**: 328-408 hours (60% reduction from original plan)

**Next Action**: Begin CARGO-001 (build.rs integration prototype)
