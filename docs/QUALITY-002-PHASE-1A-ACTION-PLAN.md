# QUALITY-002 Phase 1A: CLI unwrap() Replacement - Action Plan

**Date**: 2025-11-21
**Phase**: 1A - CLI/Binary
**Scope**: 222 unwrap() calls in `src/bin/`
**Estimated Time**: 1 day (4-6 hours)
**Status**: 🚀 **READY TO START**

---

## Quick Start

```bash
# 1. Start with highest priority file (61 unwrap() calls)
code src/bin/handlers/handlers_modules/test_helpers.rs

# 2. Replace unwrap() → expect() with descriptive messages
# Pattern: .unwrap() → .expect("Clear error message")

# 3. Run tests after each file
cargo test --bin ruchy

# 4. Commit when done with each file
git add src/bin/handlers/handlers_modules/test_helpers.rs
git commit -m "[QUALITY-002] Phase 1A: Replace unwrap() in test_helpers.rs (61 calls)"
```

---

## File Priority List (Top 10 Files)

| Priority | File | Calls | Rationale |
|----------|------|-------|-----------|
| **1** | `src/bin/handlers/handlers_modules/test_helpers.rs` | 61 | Test infrastructure - critical for debugging |
| **2** | `src/bin/ruchy.rs` | 40 | Main entry point - user-facing errors |
| **3** | `src/bin/handlers/handlers_modules/prove_helpers.rs` | 37 | Proof system helpers |
| **4** | `src/bin/handlers/commands.rs` | 27 | Command dispatch - CLI errors |
| **5** | `src/bin/handlers/handlers_modules/test.rs` | 22 | Test command handler |
| **6** | `src/bin/handlers/tests.rs` | 16 | Test utilities |
| **7** | `src/bin/handlers/handlers_modules/prove.rs` | 14 | Proof command handler |
| **8** | `src/bin/handlers/mod.rs` | 3 | Handler module root |
| **9** | `src/bin/handlers/build.rs` | 1 | Build command |
| **10** | `src/bin/handlers/add.rs` | 1 | Add command |

**Total**: 222 calls across 10 files

---

## EXTREME TDD Workflow (Per File)

### RED Phase
1. **Identify unwrap() locations**: `grep -n "unwrap()" src/bin/file.rs`
2. **Read surrounding code**: Understand what's being unwrapped
3. **Document expected behavior**: What error message should appear?

### GREEN Phase
1. **Replace unwrap() with expect()**: Add descriptive message
2. **Format consistently**: Use clear, actionable error messages
3. **Save file**: Keep syntax valid

### REFACTOR Phase
1. **Review all expect() messages**: Ensure consistency
2. **Group similar patterns**: Identify common error types
3. **Run rustfmt**: `cargo fmt --all`

### VALIDATE Phase
1. **Run tests**: `cargo test --bin ruchy`
2. **Run clippy**: `cargo clippy --bin ruchy -- -D warnings`
3. **Manual verification**: Intentionally trigger errors, check messages

---

## Replacement Patterns for CLI

### Pattern 1: File Operations
```rust
// ❌ BEFORE
let contents = fs::read_to_string(&path).unwrap();

// ✅ AFTER
let contents = fs::read_to_string(&path)
    .expect(&format!("Failed to read file: {}", path.display()));
```

### Pattern 2: Command-line Arguments
```rust
// ❌ BEFORE
let arg = args.next().unwrap();

// ✅ AFTER
let arg = args.next()
    .expect("Missing required command-line argument");
```

### Pattern 3: Path Operations
```rust
// ❌ BEFORE
let parent = path.parent().unwrap();

// ✅ AFTER
let parent = path.parent()
    .expect(&format!("Path has no parent: {}", path.display()));
```

### Pattern 4: Test Execution
```rust
// ❌ BEFORE
let output = cmd.output().unwrap();

// ✅ AFTER
let output = cmd.output()
    .expect(&format!("Failed to execute test command: {:?}", cmd));
```

### Pattern 5: String Conversion
```rust
// ❌ BEFORE
let s = String::from_utf8(bytes).unwrap();

// ✅ AFTER
let s = String::from_utf8(bytes)
    .expect("Invalid UTF-8 in test output");
```

---

## Step-by-Step: First File (test_helpers.rs)

### Step 1: Open File
```bash
code src/bin/handlers/handlers_modules/test_helpers.rs
```

### Step 2: Find All unwrap() Calls (61 total)
```bash
grep -n "unwrap()" src/bin/handlers/handlers_modules/test_helpers.rs
```

### Step 3: Replace Systematically
Work through file line-by-line:
- Read context around each `unwrap()`
- Determine what failure means
- Replace with descriptive `expect()` message

**Example Section**:
```rust
// Around line 50 (hypothetical)
let test_file = File::open(&path).unwrap();  // Line 50

// Replace with:
let test_file = File::open(&path)
    .expect(&format!("Failed to open test file: {}", path.display()));
```

### Step 4: Test Changes
```bash
# Run tests for this module
cargo test --bin ruchy test_helpers

# Run all CLI tests
cargo test --bin ruchy
```

### Step 5: Verify Error Messages
```bash
# Intentionally trigger an error (e.g., pass invalid file)
# Verify error message is descriptive
```

### Step 6: Commit
```bash
git add src/bin/handlers/handlers_modules/test_helpers.rs
git commit -m "[QUALITY-002] Phase 1A: Replace unwrap() in test_helpers.rs (61/222)

- Replaced 61 unwrap() calls with expect() in test_helpers.rs
- Improved error messages for:
  - File operations (path display)
  - Test execution (command context)
  - Output parsing (UTF-8 errors)

Tests: All passing
Impact: Test errors now show file paths and command details

Progress: Phase 1A: 61/222 (27%)"
```

---

## Quality Gates (Before Each Commit)

**MANDATORY checklist**:

```bash
# 1. All tests pass
cargo test --bin ruchy
echo "✅ Tests: $?"

# 2. No unwrap() in modified files (verify replacement complete)
git diff --name-only | xargs grep -n "unwrap()" || echo "✅ No unwrap() in changed files"

# 3. Clippy passes
cargo clippy --bin ruchy -- -D warnings
echo "✅ Clippy: $?"

# 4. Format code
cargo fmt --all
echo "✅ Format: Done"

# 5. PMAT TDG (optional, pre-commit hook will run this)
pmat tdg src/bin/ --min-grade A-
echo "✅ TDG: $?"
```

---

## Progress Tracking

### Daily Log Format

**Morning** (Start of day):
```
📊 QUALITY-002 Phase 1A Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Today's Goal: Files 1-3 (138 calls)
- [ ] test_helpers.rs (61 calls)
- [ ] ruchy.rs (40 calls)
- [ ] prove_helpers.rs (37 calls)
```

**Evening** (End of day):
```
📊 QUALITY-002 Phase 1A Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Completed: 2/10 files (101/222 calls, 45%)
- [x] test_helpers.rs (61 calls) ✅
- [x] ruchy.rs (40 calls) ✅
- [ ] prove_helpers.rs (37 calls) - Tomorrow

Tests: 4036/4036 passing
Commits: 2
```

### Update Specification
After each file, update `docs/specifications/quality-002-unwrap-replacement-spec.md`:

```markdown
## Progress Tracking

**Phase 1A: CLI/Binary** (In Progress)
- [x] test_helpers.rs - 61 calls ✅ (Commit: abc123)
- [x] ruchy.rs - 40 calls ✅ (Commit: def456)
- [ ] prove_helpers.rs - 37 calls
- [ ] commands.rs - 27 calls
...

**Total Progress**: 101/3697 (2.7%)
```

---

## Commit Message Template

```
[QUALITY-002] Phase 1A: Replace unwrap() in <filename> (<N>/222)

- Replaced <N> unwrap() calls with expect() in <module>
- Improved error messages for:
  - <error_type_1> (<description>)
  - <error_type_2> (<description>)
  - <error_type_3> (<description>)

Examples:
Before: "called `Option::unwrap()` on a `None` value"
After: "Failed to read test file: tests/example.ruchy: No such file"

Tests: <PASSING>/<TOTAL> passing
Impact: <user_facing_improvement>

Progress: Phase 1A: <DONE>/222 (<PERCENT>%)
```

---

## Time Estimates (Per File)

| File | Calls | Est. Time | Complexity |
|------|-------|-----------|------------|
| test_helpers.rs | 61 | 1.5 hours | Medium (many file ops) |
| ruchy.rs | 40 | 1 hour | High (main entry point) |
| prove_helpers.rs | 37 | 1 hour | Medium |
| commands.rs | 27 | 45 min | Medium |
| test.rs | 22 | 30 min | Low |
| tests.rs | 16 | 30 min | Low |
| prove.rs | 14 | 30 min | Low |
| mod.rs | 3 | 15 min | Low |
| build.rs | 1 | 5 min | Low |
| add.rs | 1 | 5 min | Low |

**Total**: ~6 hours (one work day with breaks and testing)

---

## Success Criteria (Phase 1A Complete)

- ✅ All 222 unwrap() calls in `src/bin/` replaced
- ✅ All tests passing (4036+/4036+)
- ✅ Error messages verified manually for top 3 files
- ✅ Clippy warnings: None introduced
- ✅ TDG score: Maintained or improved
- ✅ All commits atomic and descriptive
- ✅ Specification updated with progress

**Next Phase**: Phase 1B - Core Compilation Pipeline (Parser, Type Inference, VM)

---

## Troubleshooting

### Issue: Tests Fail After Replacement

**Solution**:
1. Revert changes: `git checkout -- src/bin/file.rs`
2. Review test expectations: Some tests may check error messages
3. Update test assertions to match new expect() messages
4. Re-run tests

### Issue: Too Many Changes in One File

**Solution**:
1. Commit partial progress: Replace 20-30 calls at a time
2. Test incrementally: Run tests after each batch
3. Split commits: Multiple commits per file is OK

### Issue: Unclear What Error Message Should Be

**Solution**:
1. Read surrounding code for context
2. Look at what data is available (variables, function args)
3. Ask: "What would help me debug this?"
4. Include: Operation, file/path, expected vs actual

---

## Commands Reference

```bash
# Start work
pmat work continue QUALITY-002

# Find unwrap() calls
grep -rn "unwrap()" src/bin/

# Test after changes
cargo test --bin ruchy

# Check quality
cargo clippy --bin ruchy -- -D warnings
cargo fmt --all
pmat tdg src/bin/

# Commit changes
git add <file>
git commit -m "[QUALITY-002] Phase 1A: ..."

# Check progress
grep "unwrap()" src/bin/*.rs | wc -l  # Remaining calls
```

---

## Next Steps

1. **Start Now**: Open `src/bin/handlers/handlers_modules/test_helpers.rs`
2. **First Replacement**: Find first `unwrap()` and replace
3. **Test Immediately**: `cargo test test_helpers`
4. **Repeat**: Continue through all 61 calls in file
5. **Commit**: When file is complete
6. **Move to Next**: `src/bin/ruchy.rs` (40 calls)

---

**Status**: 🚀 **READY TO START** - Begin with test_helpers.rs (61 calls)

**Time**: ~6 hours total for Phase 1A (222 calls)

**Command**: `code src/bin/handlers/handlers_modules/test_helpers.rs`
