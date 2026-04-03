# Sub-spec: Visualization Oracle -- Implementation, Additional Features, and References

**Parent:** [visualization-enhanced-transpile-oracle-reporting.md](../visualization-enhanced-transpile-oracle-reporting.md) Sections 6-11

---

## Implementation Priority

Recommended order based on value/effort ratio:

1. **Rich ASCII Reports** (High value, low effort) - Immediate UX improvement [5][7]
2. **Multi-Format Output** (High value, medium effort) - Enables CI/IDE integration [10]
3. **Error Clustering + Pareto** (Medium value, medium effort) - Helps prioritize fixes [8]
4. **SBFL Integration** (High value, medium effort) - Academic rigor, debugging aid [1][2][3]
5. **Convergence Dashboard** (Medium value, high effort) - Auto-fix visibility [6]

**Total estimated effort**: 10-14 days

## File Structure

```
src/
+-- oracle/
|   +-- mod.rs              # Existing
|   +-- classifier.rs       # Existing
|   +-- sbfl.rs            # NEW: SBFL algorithms [1][2][3][4]
|   +-- clustering.rs      # NEW: Error clustering [8]
+-- reporting/
|   +-- mod.rs             # NEW: Report module
|   +-- ascii.rs           # NEW: Rich ASCII rendering [5][7]
|   +-- formats/
|   |   +-- mod.rs         # NEW: Format registry
|   |   +-- human.rs       # NEW: Terminal output (Mieruka) [5]
|   |   +-- json.rs        # NEW: JSON export
|   |   +-- sarif.rs       # NEW: SARIF 2.1.0 [10]
|   |   +-- markdown.rs    # NEW: Markdown export
|   +-- dashboard.rs       # NEW: Convergence dashboard (Jidoka) [6]
|   +-- pareto.rs          # NEW: Pareto analysis [8][9]
```

**Note**: No HTML output - all visualization is text-based per Toyota Way Mieruka principle of "simple, visible indicators" [5].

## Dependencies

```toml
# Cargo.toml additions
[dependencies]
indicatif = "0.17"        # Progress bars (from depyler)
colored = "2.0"           # Terminal colors
unicode-width = "0.1"     # Unicode character width
serde_json = "1.0"        # JSON serialization (existing)
```

## References

- bashrs SBFL: `/home/noah/src/bashrs/rash/src/quality/sbfl.rs`
- bashrs Rich Reports: `/home/noah/src/bashrs/rash/src/quality/report.rs`
- bashrs Lint Reports: `/home/noah/src/bashrs/rash/src/quality/lint_report.rs`
- depyler Report Command: `/home/noah/src/depyler/crates/depyler/src/report_cmd/mod.rs`
- depyler Convergence Reporter: `/home/noah/src/depyler/crates/depyler/src/converge/reporter.rs`

## Acceptance Criteria

- [ ] All 5 features have unit tests (TDD per Extreme TDD Protocol)
- [ ] Property tests for SBFL algorithms (10K+ cases) validating [1][2][3][4]
- [ ] Mutation test coverage >=75% (cargo-mutants)
- [ ] PMAT TDG grade >=A- for all new files
- [ ] Zero clippy warnings
- [ ] Documentation with examples
- [ ] All output is text-based (no HTML) per Mieruka [5]
- [ ] Andon thresholds validated against Deming's control limits [9]

---

## ADDITIONAL FEATURES (From depyler Analysis)

### 6. Delta Debugging / Bisection Mode

**Source**: depyler `src/report_cmd/filter.rs`, `src/report_cmd/mod.rs`

**What it does**: Binary search algorithm to isolate the minimal failing set of .ruchy files in O(log n) time. When you have 1000+ files and some subset causes failures, bisection finds the culprit in ~10 iterations instead of checking each file.

**Academic foundation**:
- [11] Zeller, A., Hildebrandt, R. (2002). "Simplifying and Isolating Failure-Inducing Input." *IEEE TSE*, 28(2), pp. 183-200. DOI:10.1109/32.988498

**CLI integration**:
```bash
ruchy report --bisect                    # Find minimal failing set
ruchy report --bisect --filter "async"   # Bisect only async-related files
ruchy report --fail-fast                 # Stop on first failure
```

**Implementation**:
```rust
/// Bisection state for Delta Debugging [11]
pub struct BisectionState {
    pub files: Vec<PathBuf>,
    pub low: usize,
    pub high: usize,
    pub iteration: usize,
    pub max_iterations: usize,  // Safety limit: 20
    pub result: Option<Vec<PathBuf>>,
}

impl BisectionState {
    /// O(log n) complexity: ~11 iterations for 1671 files
    pub fn advance(&mut self, failure_in_first_half: bool) {
        let mid = (self.low + self.high) / 2;
        if failure_in_first_half {
            self.high = mid;
        } else {
            self.low = mid + 1;
        }
        self.iteration += 1;
    }
}
```

**Sample output**:
```
BISECTION MODE: Delta Debugging [Zeller & Hildebrandt, 2002]

Starting with 1671 files

Iteration 1: Testing 836 files (range 0-835)
Iteration 2: Testing 418 files (range 836-1253)
Iteration 3: Testing 209 files (range 836-1044)
...
Iteration 11: Testing 1 file (range 1023-1023)

BISECTION COMPLETE
Isolated 1 failing file in 11 iterations:
-> examples/advanced/closure_capture.ruchy
  Error: E0382 - borrow of moved value: `captured`

O(log n) complexity: 11 iterations for 1671 files
```

**Implementation complexity**: Medium (2-3 days)

---

### 7. Semantic Tagging & Corpus Filtering

**Source**: depyler `src/report_cmd/filter.rs`, `src/corpus/semantic.rs`

**What it does**: Automatically tags .ruchy files by language features used (Closure, Generic, Async, Match, Enum, Struct, Trait, etc.), then allows filtering and analysis by tag. Enables targeted debugging: "show me all failures involving closures."

**CLI integration**:
```bash
ruchy report --tag Closure              # Filter to closure-using files
ruchy report --tag Generic --tag Trait  # Multiple tags (OR)
ruchy report --filter "test_"           # Regex/glob pattern filter
ruchy report --sample 50                # Random sample of 50 files
ruchy report --limit 100                # Process first 100 files only
```

**Semantic tags for Ruchy**:
```rust
pub enum SemanticTag {
    // Type system
    Generic,      // <T>, where T: Trait
    Trait,        // trait Foo { }
    Enum,         // enum Color { }
    Struct,       // struct Point { }

    // Control flow
    Match,        // match expr { }
    IfLet,        // if let Some(x) = ...
    WhileLet,     // while let Some(x) = ...
    Loop,         // loop { }

    // Closures & functions
    Closure,      // |x| x + 1
    AsyncFn,      // async fun foo() { }

    // Memory & ownership
    Borrow,       // &x, &mut x
    Lifetime,     // 'a, 'static
    Box,          // Box::new()
    Rc,           // Rc::new()
    Arc,          // Arc::new()

    // Collections
    Vec,          // vec![], Vec::new()
    HashMap,      // HashMap::new()

    // I/O
    FileIO,       // File::open, read, write
    StdIO,        // println!, stdin

    // Domain classification
    Core,         // Basic Ruchy features
    Stdlib,       // Standard library usage
    External,     // External crate usage
}
```

**Sample output**:
```
SEMANTIC ANALYSIS BY TAG

Tag Distribution (41 examples):
  Closure      xxxxxxxxxxxx........ 12 files (29%)  -> 8 pass, 4 fail
  Generic      xxxxxxxxxx.......... 10 files (24%)  -> 7 pass, 3 fail
  Match        xxxxxxxx............ 8 files (20%)   -> 8 pass, 0 fail
  Struct       xxxxxx.............. 6 files (15%)   -> 6 pass, 0 fail
  Async        xxxx................ 4 files (10%)   -> 2 pass, 2 fail
  Lifetime     xx.................. 1 file  (2%)    -> 0 pass, 1 fail  WARNING

PROBLEM AREAS (Pass Rate < 80%):
  Closure:   67% pass rate -> Focus on borrow checker in closures
  Async:     50% pass rate -> Focus on async/await transpilation
  Lifetime:   0% pass rate -> CRITICAL - lifetime inference broken
```

**Implementation complexity**: Medium (2-3 days)

---

### 8. 5-Phase Corpus Pipeline with Blocker Priority

**Source**: depyler `crates/depyler-corpus/src/lib.rs`, `src/taxonomy.rs`

**What it does**: Structured analysis pipeline with Toyota Way 5S methodology. Classifies errors by blocker priority (P0-CRITICAL to P3-LOW) based on frequency impact.

**Pipeline phases**:
```
Phase 1: Artifact Cleaning (5S)     -> Remove .rs, target/, Cargo.toml
Phase 2: Transpilation              -> .ruchy -> .rs conversion
Phase 3: Compilation Verification   -> rustc on generated .rs files
Phase 4: Error Taxonomy             -> Classify by category + priority
Phase 5: Report Generation          -> Terminal, JSON, SARIF, Markdown
```

**Blocker priority calculation** (from [8] Juran's Pareto):
```rust
pub enum BlockerPriority {
    P0Critical,  // >20% of corpus OR >=50 occurrences
    P1High,      // >10% of corpus OR >=20 occurrences
    P2Medium,    // >5% of corpus OR >=10 occurrences
    P3Low,       // <5% of corpus AND <10 occurrences
}

impl BlockerPriority {
    pub fn from_frequency(count: usize, total: usize) -> Self {
        let pct = (count as f64 / total as f64) * 100.0;
        match (pct, count) {
            (p, c) if p > 20.0 || c >= 50 => Self::P0Critical,
            (p, c) if p > 10.0 || c >= 20 => Self::P1High,
            (p, c) if p > 5.0 || c >= 10 => Self::P2Medium,
            _ => Self::P3Low,
        }
    }
}
```

**Implementation complexity**: Medium-High (3-4 days)

---

## Updated File Structure

```
src/
+-- oracle/
|   +-- mod.rs              # Existing
|   +-- classifier.rs       # Existing
|   +-- sbfl.rs            # NEW: SBFL algorithms [1][2][3][4]
|   +-- clustering.rs      # NEW: Error clustering [8]
+-- corpus/                 # NEW: Corpus analysis module
|   +-- mod.rs             # NEW: Corpus analyzer
|   +-- bisect.rs          # NEW: Delta debugging [11]
|   +-- filter.rs          # NEW: Semantic filtering
|   +-- tags.rs            # NEW: Semantic tag extraction
|   +-- taxonomy.rs        # NEW: Error taxonomy + blocker priority
+-- reporting/
|   +-- mod.rs             # NEW: Report module
|   +-- ascii.rs           # NEW: Rich ASCII rendering [5][7]
|   +-- formats/
|   |   +-- mod.rs         # NEW: Format registry
|   |   +-- human.rs       # NEW: Terminal output (Mieruka) [5]
|   |   +-- json.rs        # NEW: JSON export
|   |   +-- sarif.rs       # NEW: SARIF 2.1.0 [10]
|   |   +-- markdown.rs    # NEW: Markdown export
|   +-- dashboard.rs       # NEW: Convergence dashboard (Jidoka) [6]
|   +-- pareto.rs          # NEW: Pareto analysis [8][9]
```

## Full Reference List

1. Jones, J.A., Harrold, M.J., Stasko, J. (2002). "Visualization of test information to assist fault localization." *ICSE '02*. IEEE.
2. Abreu, R., Zoeteweij, P., van Gemund, A.J.C. (2007). "On the accuracy of spectrum-based fault localization." *TAICPART-MUTATION '07*. IEEE.
3. Wong, W.E., Debroy, V., Gao, R., Li, Y. (2014). "The DStar method for effective software fault localization." *IEEE TSE*, 40(8).
4. Xie, X., Chen, T.Y., Kuo, F.C., Xu, B. (2013). "A theoretical analysis of the risk evaluation formulas for spectrum-based fault localization." *TOSEM*, 22(4).
5. Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press.
6. Liker, J.K. (2004). *The Toyota Way: 14 Management Principles*. McGraw-Hill.
7. Imai, M. (1986). *Kaizen: The Key to Japan's Competitive Success*. McGraw-Hill.
8. Juran, J.M. (1988). *Juran on Planning for Quality*. Free Press.
9. Deming, W.E. (1986). *Out of the Crisis*. MIT Press.
10. Pearson, S., Campos, J., Just, R., et al. (2017). "Evaluating and improving fault localization." *ICSE '17*. IEEE.
11. Zeller, A., Hildebrandt, R. (2002). "Simplifying and Isolating Failure-Inducing Input." *IEEE TSE*, 28(2). **Delta Debugging**.
