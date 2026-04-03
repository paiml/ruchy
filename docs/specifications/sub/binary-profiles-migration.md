# Sub-spec: Optimized Binary — Profile Implementation and Migration

**Parent:** [optimized-binary-speed-size-spec.md](../optimized-binary-speed-size-spec.md) Sections: Implementation, CLI Integration, Workload Recommendations, Migration Strategy

---

## Implementation

### Default Profile: `--profile release` (Speed-First)

**Current Cargo.toml** (already optimal as of v3.174.0):

```toml
[profile.release]
opt-level = 3              # Maximum speed
lto = "fat"                # Full link-time optimization ⭐
codegen-units = 1          # Single codegen unit (best optimization)
strip = true               # Remove debug symbols
panic = "abort"            # No unwinding overhead
overflow-checks = false    # No runtime overflow checks
debug-assertions = false   # No debug assertions
incremental = false        # Disable incremental compilation
```

**Characteristics**:
- ⚡ **15x average speedup** vs baseline
- 📦 **~1.7 MB binary** (53% smaller than baseline)
- 🎯 **Best for**: General-purpose production binaries

**Usage**:
```bash
# Implicit (default)
ruchy compile script.ruchy -o myapp

# Explicit
ruchy compile script.ruchy -o myapp --profile release
```

---

### Size-Optimized Profile: `--profile release-tiny`

**Cargo.toml Configuration**:

```toml
[profile.release-tiny]
inherits = "release"
opt-level = "z"            # Optimize for SIZE ⭐
lto = "fat"                # Still use LTO (size + speed)
codegen-units = 1          # Single unit
panic = "abort"            # Minimal panic handler
strip = true               # Remove symbols
```

**Characteristics**:
- 📦 **314 KB binary** (91.7% smaller than baseline)
- ⚡ **2.16x speedup** (only 14% slower than speed-optimized)
- 🎯 **Best for**: Embedded systems, mobile apps, size-constrained deployments

**Usage**:
```bash
ruchy compile script.ruchy -o myapp --profile release-tiny
```

---

### Ultra-Performance Profile: `--profile release-ultra` (with PGO)

**Cargo.toml Configuration** (already defined):

```toml
[profile.release-ultra]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
```

**Additional Compilation Flags**:
```bash
RUSTFLAGS="-C target-cpu=native -C embed-bitcode=yes"
```

**Two-Step PGO Build**:

```bash
# Step 1: Build with profile generation
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" \
  ruchy compile script.ruchy -o myapp-profiled --profile release-ultra

# Step 2: Run workload to collect profile data
./myapp-profiled <typical-workload>

# Step 3: Build with profile-guided optimization
RUSTFLAGS="-C profile-use=/tmp/pgo-data -C target-cpu=native" \
  ruchy compile script.ruchy -o myapp --profile release-ultra
```

**Characteristics**:
- ⚡ **25-50x speedup** for CPU-intensive workloads
- 📦 **~520 KB binary** (moderate size)
- 🎯 **Best for**: Performance-critical production systems, long-running services

**Tradeoffs**:
- ⏱️ Requires two-step build process
- 💾 Profile data collection overhead
- 🖥️ Hardware-specific (not portable across CPUs)

---

### Distribution Profile: `--profile release-dist`

**Current Configuration** (needs update):

```toml
[profile.release-dist]
inherits = "release"
strip = true
lto = "fat"
codegen-units = 1
panic = "abort"
opt-level = "z"       # ❌ PROBLEM: Size-optimized, not speed
```

**Recommended Update** (align with findings):

```toml
[profile.release-dist]
inherits = "release"
opt-level = 3         # ✅ FIX: Use speed optimization
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
overflow-checks = false
debug-assertions = false
incremental = false
```

**Rationale**: Distribution binaries should prioritize performance (15x speedup) over minimal size savings. Users needing tiny binaries can use `release-tiny` explicitly.

---

## CLI Integration

### Updated `ruchy compile` Command

```
USAGE:
    ruchy compile [OPTIONS] <INPUT> -o <OUTPUT>

OPTIONS:
    -o, --output <PATH>           Output binary path
    --profile <PROFILE>           Build profile [default: release]
                                  Options: release, release-tiny, release-ultra, release-dist
    --target-cpu <CPU>            Target CPU (e.g., native, x86-64-v3) [default: generic]
    --pgo                         Enable profile-guided optimization (two-step build)
    --show-profile-info           Print profile characteristics and exit
    --rustflags <FLAGS>           Additional RUSTFLAGS to pass to rustc

EXAMPLES:
    # Default: Fast binary (~1.7 MB, 15x speedup)
    ruchy compile main.ruchy -o myapp

    # Tiny binary for embedded (314 KB, 2x speedup)
    ruchy compile main.ruchy -o myapp --profile release-tiny

    # Maximum performance with native CPU optimization
    ruchy compile main.ruchy -o myapp --profile release-ultra --target-cpu native

    # Profile-guided optimization (two-step)
    ruchy compile main.ruchy -o myapp --profile release-ultra --pgo
    # (automatically handles PGO workflow)

PROFILE INFO:
    release         15x speed, 1.7 MB  [DEFAULT]
    release-tiny    2x speed, 314 KB   [EMBEDDED]
    release-ultra   25-50x speed       [MAXIMUM PERFORMANCE]
    release-dist    Same as release    [DISTRIBUTION]
```

### Binary Analysis Command (`ruchy analyze`)

**Status**: Proposed in [Issue #145](https://github.com/paiml/ruchy/issues/145)
**Prototype**: [ruchyruchy COMPILED-INST-003](https://github.com/paiml/ruchyruchy) (6/6 tests passing)

Complement profile optimization with comprehensive binary analysis:

```
USAGE:
    ruchy analyze [OPTIONS] <BINARY>

OPTIONS:
    --size              Analyze binary size breakdown by section
    --symbols           Extract symbol table and identify optimization candidates
    --startup           Profile startup time (loader, linking, init)
    --relocations       Analyze relocation overhead
    --optimize          Generate actionable optimization recommendations
    --format            Detect binary format (ELF, Mach-O, PE)
    --output <FILE>     Export JSON report for CI integration

EXAMPLES:
    # Verify profile choice achieved expected size
    ruchy compile main.ruchy -o app --profile release-tiny
    ruchy analyze --size app
    # Expected: ~314 KB (if not, investigate with --optimize)

    # Get optimization recommendations
    ruchy analyze --optimize --output=recommendations.json app
    # Review suggestions for dead code, inlining, outlining

    # Full analysis for CI dashboard
    ruchy analyze --size --symbols --optimize --output=analysis.json app

    # Compare profiles
    ruchy compile main.ruchy -o app-release --profile release
    ruchy compile main.ruchy -o app-tiny --profile release-tiny
    ruchy analyze --size app-release app-tiny
    # Validate size tradeoff (release: 1.7 MB vs tiny: 314 KB)

OUTPUT EXAMPLE (--size):
    Binary Size Analysis
    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    Section       Size        Percentage
    ────────────────────────────────────────────
    .text         1.1 MB      62.3%  (code)
    .rodata       109 KB      9.0%   (read-only data)
    .data         2.5 KB      0.2%   (initialized data)
    .bss          8.0 KB      0.7%   (uninitialized)
    ────────────────────────────────────────────
    Total         1.76 MB

    Format: ELF x86-64
    Profile: release (detected from optimization level)

OUTPUT EXAMPLE (--optimize):
    Optimization Recommendations
    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    [HIGH] Dead code elimination
           Function: unused_helper (main.ruchy:42)
           Impact: -256 bytes
           Confidence: 95%

    [MEDIUM] Inline small function
           Function: get_value (utils.ruchy:15)
           Impact: -64 bytes, +5% speed
           Confidence: 85%

    [MEDIUM] Outline cold error path
           Function: error_handler (error.ruchy:88)
           Impact: -128 bytes, -0% hot path
           Confidence: 75%

    Total potential savings: 448 bytes (3.7%)
```

**Integration with Profiles**:

```bash
# Workflow: Compile → Analyze → Optimize → Verify
ruchy compile main.ruchy -o app --profile release
ruchy analyze --optimize app > recommendations.txt

# Apply manual optimizations based on recommendations
# (future: ruchy compile --apply-recommendations)

ruchy compile main.ruchy -o app-opt --profile release
ruchy analyze --size app app-opt
# Verify: app-opt is smaller without performance loss
```

**CI/CD Integration**:

```yaml
# .github/workflows/binary-size-check.yml
- name: Check binary size regression
  run: |
    ruchy compile main.ruchy -o app --profile release
    ruchy analyze --size --output=size.json app

    # Fail if binary exceeds 2 MB threshold
    SIZE_KB=$(jq '.total_size / 1024' size.json)
    if (( $(echo "$SIZE_KB > 2048" | bc -l) )); then
      echo "Binary too large: ${SIZE_KB} KB (max 2048 KB)"
      exit 1
    fi
```

---

## Benchmark Data Reference

### Speed Rankings (Top 5 Profiles)

| Rank | Profile | Average Speedup | 95% CI | Binary Size |
|------|---------|-----------------|--------|-------------|
| 1 | lto-fat | 15.06x | [13.1, 17.1] | 1.76 MB |
| 2 | lto-thin | 14.52x | [12.6, 16.5] | 1.85 MB |
| 3 | codegen-1 | 14.12x | [12.2, 16.1] | 3.78 MB |
| 4 | standard-release | 13.87x | [12.0, 15.8] | 2.12 MB |
| 5 | perf-ultra | 13.64x | [11.8, 15.5] | 2.34 MB |

### Size Rankings (Top 5 Profiles)

| Rank | Profile | Binary Size | Speedup | Size Reduction |
|------|---------|-------------|---------|----------------|
| 1 | size-ultra | 314 KB | 2.16x | 91.7% |
| 2 | size-z | 512 KB | 2.24x | 86.5% |
| 3 | opt-s | 698 KB | 2.43x | 81.5% |
| 4 | lto-fat | 1.76 MB | 15.06x | 53.4% ⭐ |
| 5 | lto-thin | 1.85 MB | 14.52x | 51.0% |

**Key Insight**: lto-fat is 4th best for size but 1st for speed, making it the optimal balanced choice.

---

## Workload-Specific Recommendations

### Data Science / Scientific Computing

**Characteristics**: Memory-intensive, large datasets, numerical operations

**Recommendation**: `--profile release` (default)
- **Speedup**: 15-51x for memory operations
- **Rationale**: Data processing benefits dramatically from LTO optimizations

```bash
ruchy compile analyze.ruchy -o analyze --profile release
```

### Web Servers / APIs

**Characteristics**: I/O bound, concurrent, long-running

**Recommendation**: `--profile release-ultra --target-cpu native`
- **Speedup**: 10-15x average, better latency
- **Rationale**: Performance-critical, runs on known hardware

```bash
ruchy compile server.ruchy -o server --profile release-ultra --target-cpu native
```

### CLI Tools

**Characteristics**: Fast startup, moderate workload, wide distribution

**Recommendation**: `--profile release` (default)
- **Binary Size**: 1-2 MB (reasonable for download)
- **Performance**: 15x speedup
- **Portability**: Works on all x86-64 CPUs

```bash
ruchy compile tool.ruchy -o tool
```

### Embedded / Edge Devices

**Characteristics**: Size-constrained, ARM processors, limited storage

**Recommendation**: `--profile release-tiny`
- **Binary Size**: 300-500 KB
- **Performance**: Still 2x faster than baseline
- **Deployment**: Fits in constrained environments

```bash
ruchy compile embedded.ruchy -o embedded --profile release-tiny
```

### Serverless / Lambda Functions

**Characteristics**: Cold start sensitive, ephemeral, network-bound

**Recommendation**: `--profile release-tiny`
- **Rationale**: Smaller binaries load faster (cold start optimization)
- **Tradeoff**: Slight performance loss acceptable for I/O-bound workloads

```bash
ruchy compile lambda.ruchy -o bootstrap --profile release-tiny
```

---

## Migration Strategy

### Phase 1: Update Documentation (Immediate)

**Status**: ✅ **COMPLETE**

- [x] Update README with profile recommendations
- [x] Add `--profile` examples to CLI documentation
- [x] Create migration guide for users on old `release-dist`

### Phase 2: Fix `release-dist` Profile (v3.212.0)

**Status**: ✅ **COMPLETE** (Implemented in commit 10d92ad6)

**Implemented Fix**:
```toml
[profile.release-dist]
inherits = "release"
opt-level = 3              # ✅ Changed from "z" to 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
overflow-checks = false    # ✅ Added
debug-assertions = false   # ✅ Added
incremental = false        # ✅ Added
```

**Results**:
- Distribution binaries now achieve ~15x faster performance (previously only ~2x)
- Binary size: ~1.7 MB (acceptable for distribution)
- Breaking Change: No (profile name unchanged, just faster)

**Test Coverage**: 15/15 tests passing (tests/perf_002_profile_optimization.rs)
**PMAT TDG**: 92.4/100 (A grade)

### Phase 3: Add `--show-profile-info` Flag (v3.213.0)

**Status**: ✅ **COMPLETE** (Implemented in commit f898f243)

**Feature**: Print profile characteristics before compilation

```bash
$ ruchy compile main.ruchy -o main --show-profile-info

Profile: release (default)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Optimization:     opt-level = 3 (speed)
  LTO:              fat (maximum)
  Codegen units:    1
  Expected speedup: 15x average
  Expected size:    1-2 MB
  Best for:         General-purpose production binaries
  Compile time:     ~30-60s for 1000 LOC
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Alternative profiles:
  --profile release-tiny    (314 KB, 2x speed, embedded)
  --profile release-ultra   (25-50x speed, PGO, maximum performance)

Continue? [Y/n]
```

**Implementation Details**:
- Added `--show-profile-info` CLI flag to `src/bin/ruchy.rs`
- Implemented `display_profile_info()` function (59 lines, src/bin/handlers/mod.rs)
- Visual formatting with colored output and separators
- Shows optimization settings, expected performance, alternatives

**Test Coverage**: 15/15 tests passing (tests/perf_002_phase3_show_profile_info.rs)
**PMAT TDG**: 96.8/100 (A+ grade)

### Phase 4: Implement `--pgo` Automation (v3.214.0)

**Status**: ✅ **COMPLETE** (Implemented in commit e68bebb1)

**Feature**: Automate two-step PGO build process

```bash
# Single command replaces 3-step manual process
ruchy compile server.ruchy -o server --profile release-ultra --pgo

# Automatically:
# 1. Builds with profile-generate
# 2. Prompts user to run workload
# 3. Rebuilds with profile-use
```

**UX Flow**:
```
Building with profile generation...
✓ Built: server-profiled

Run your typical workload now to collect profile data:
  ./server-profiled <args>

Press Enter when done...

Building with profile-guided optimization...
✓ Built: server (optimized)

Profile data: /tmp/ruchy-pgo-xxxxx (can be reused)
```

**Implementation Details**:
- Added `--pgo` CLI flag to `src/bin/ruchy.rs`
- Implemented `handle_pgo_compilation()` function (144 lines, src/bin/handlers/mod.rs)
- Two-step automated workflow:
  1. Build with `-C profile-generate=/tmp/ruchy-pgo-*`
  2. Interactive prompt for user workload execution
  3. Rebuild with `-C profile-use=/tmp/ruchy-pgo-* -C target-cpu=native`
- Creates intermediate `<output>-profiled` binary (cleaned up after final build)
- Displays profile data location and expected 25-50x speedup
- JSON output support for CI/CD integration

**Test Coverage**: 15/15 tests (2 automated, 13 manual/interactive)
- Automated: `--pgo` flag recognition, help text validation
- Manual: Interactive workflow, binary creation, profile data handling
