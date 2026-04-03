# Sub-spec: bashrs Shell Transpilation Target

**Parent:** [Ruchy Unified Specification](ruchy-unified-spec.md)
**Version:** 1.0.0
**Date:** 2026-04-03
**Status:** Draft
**Ticket:** SHELL-001

---

## 0. Prerequisites

- **bashrs/rash is not currently a Ruchy dependency** -- it must be added to `Cargo.toml` before any integration work begins.
- **All 6 transpiler files must be created**: `shell_backend.rs`, `shell_ir.rs`, `shell_emitter.rs`, `shell_stdlib.rs`, `shell_safety.rs`, and `shell_tests.rs` under `src/backend/transpiler/` do not exist yet.

---

## 1. Overview

### 1.1 Current State

Ruchy's relationship with bashrs is limited to a `.bashrsignore` file controlling lint
suppression for project shell scripts. There is no code-level integration, no shell code
generation, and no shared AST infrastructure. The transpiler backend exclusively targets
Rust via `proc_macro2` and `quote` token generation.

### 1.2 Vision

Promote bashrs from a passive linting tool to a **first-class compilation target**. The
`--target shell` flag produces POSIX-compatible shell scripts from Ruchy source, using the
same frontend (lexer, parser, type checker) that drives the Rust backend. Ruchy becomes
a single-source language that compiles to both native binaries and portable shell scripts.

### 1.3 bashrs Capabilities (v6.65.0)

| Metric | Value |
|--------|-------|
| Corpus-validated patterns | 17,882 |
| Cross-shell compatibility | 98.8% |
| Lint rules (security) | SEC001-SEC008 |
| Lint rules (determinism) | DET001-DET006 |
| Supported shells | sh, dash, bash, ash, zsh, mksh |
| Parser coverage | Full POSIX.1-2024 + bash 5.x extensions |

### 1.4 Design Principles

1. **Safety over convenience** -- generated shell code is injection-proof by construction
2. **Determinism by default** -- no implicit sources of non-determinism in output
3. **Idempotency guarantee** -- scripts can be re-run without side effects
4. **Cross-shell portability** -- output runs on all 6 target shells without modification
5. **Zero unsafe code** -- consistent with Ruchy's zero-unsafe-Rust policy

## 2. Compilation Target: `--target shell`

### 2.1 Invocation

```bash
ruchy build script.ruchy --target shell -o script.sh
ruchy build script.ruchy --target shell --shells bash,dash  # restrict targets
ruchy build script.ruchy --target shell --strict-posix       # POSIX-only subset
```

### 2.2 Compilation Pipeline

```
Ruchy Source (.ruchy)
  -> [Frontend: Lexer -> Parser -> AST]        # Shared with Rust backend
  -> [Type Checker: Shell-Safety Analysis]      # Validates shell-compatible types
  -> [Shell IR: bashrs Restricted AST]          # Intermediate representation
  -> [Shell Codegen: POSIX Emitter]             # bashrs::bash_transpiler
  -> [Post-Processing: ShellCheck + Verifier]   # Validation pass
  -> POSIX Shell Script (.sh)
```

### 2.3 Shell IR (Intermediate Representation)

The Shell IR is a restricted subset of the bashrs AST that enforces safety invariants at
the type level. A Ruchy AST node that cannot be represented in the Shell IR produces a
compile-time error with a clear diagnostic.

```rust
pub enum ShellIR {
    Assignment { name: ShellIdent, value: ShellExpr },
    Conditional { test: ShellTest, then: Vec<ShellIR>, else_: Option<Vec<ShellIR>> },
    Loop { kind: LoopKind, body: Vec<ShellIR> },
    FunctionDef { name: ShellIdent, params: Vec<ShellIdent>, body: Vec<ShellIR> },
    Command { program: ShellExpr, args: Vec<ShellExpr> },
    Pipeline { stages: Vec<ShellIR> },
    Return { value: Option<ShellExpr> },
    Comment { text: String },
}
```

### 2.4 Safety Guarantees

| Guarantee | Mechanism |
|-----------|-----------|
| No injection | All variables double-quoted; no unquoted `$` expansion |
| Determinism | `$RANDOM`, `$$`, `date +%s` prohibited in generated code |
| Idempotency | File operations use atomic create-or-skip patterns |
| No eval | `eval`, backtick substitution, and `source` never generated |
| Bounded execution | All loops require explicit termination conditions |
| Error propagation | `set -eu` (POSIX) or `set -euo pipefail` (bash/zsh) header on every generated script |

### 2.5 Type Restrictions for Shell Target

| Ruchy Type | Shell Mapping | Notes |
|------------|---------------|-------|
| `Int` | Integer variable | Arithmetic via `$(( ))` |
| `Float` | Not supported | Compile error; suggest `bc`. **Note:** Approximately 60% of existing Ruchy programs use float arithmetic and would require integer-only refactoring to target shell. |
| `String` | Quoted string | Always double-quoted |
| `Bool` | `0` / `1` | 0 = true for exit codes |
| `List<String>` | Array or positional params | Bash arrays; POSIX positional params |
| `Option<T>` | Empty string / value | Tested via `[ -z "$var" ]` |
| `Result<T, E>` | Exit code + stderr | 0 = Ok, non-zero = Err |
| `HashMap`, `Class`, `Actor` | Not supported | Compile error |

## 3. Shell Standard Library Mapping

### 3.1 String Operations

| Ruchy Function | Shell Equivalent | POSIX |
|----------------|-----------------|-------|
| `str.len()` | `${#var}` | Yes |
| `str.contains(s)` | `case "$var" in *"$s"*) ...` | Yes |
| `str.starts_with(s)` | `case "$var" in "$s"*) ...` | Yes |
| `str.ends_with(s)` | `case "$var" in *"$s") ...` | Yes |
| `str.trim()` | `sed 's/^[[:space:]]*//;s/[[:space:]]*$//'` | Yes |
| `str.to_upper()` | `tr '[:lower:]' '[:upper:]'` | Yes |
| `str.to_lower()` | `tr '[:upper:]' '[:lower:]'` | Yes |
| `str.replace(a, b)` | `sed "s/$a_safe/$b_safe/g"` | Yes |
| `str.split(delim)` | `IFS="$delim" read -ra arr <<< "$var"` | Bash |
| `str.strip_prefix(p)` | `${var#"$p"}` | Yes |
| `str.strip_suffix(s)` | `${var%"$s"}` | Yes |
| `str.is_empty()` | `[ -z "$var" ]` | Yes |
| `str.chars()` | Byte iteration loop | Yes |

### 3.2 File Operations

| Ruchy Function | Shell Equivalent | POSIX |
|----------------|-----------------|-------|
| `file.exists(p)` | `[ -e "$p" ]` | Yes |
| `file.is_file(p)` | `[ -f "$p" ]` | Yes |
| `file.is_dir(p)` | `[ -d "$p" ]` | Yes |
| `file.read_file(p)` | `cat -- "$p"` | Yes |
| `file.write_file(p, s)` | `printf '%s' "$s" > "$p"` | Yes |
| `file.append_file(p, s)` | `printf '%s' "$s" >> "$p"` | Yes |
| `file.copy(src, dst)` | `cp -- "$src" "$dst"` | Yes |
| `file.remove(p)` | `rm -f -- "$p"` | Yes |
| `file.mkdir(p)` | `mkdir -p -- "$p"` | Yes |
| `file.list_dir(p)` | `ls -1 -- "$p"` | Yes |
| `file.read_lines(p)` | `while IFS= read -r line; do ... done < "$p"` | Yes |
| `file.file_size(p)` | `wc -c < "$p"` | Yes |
| `file.is_readable(p)` | `[ -r "$p" ]` | Yes |
| `file.is_writable(p)` | `[ -w "$p" ]` | Yes |
| `file.is_executable(p)` | `[ -x "$p" ]` | Yes |
| `file.realpath(p)` | `readlink -f -- "$p"` | No |
| `file.basename(p)` | `basename -- "$p"` | Yes |
| `file.dirname(p)` | `dirname -- "$p"` | Yes |

### 3.3 Process and Environment Operations

| Ruchy Function | Shell Equivalent | POSIX |
|----------------|-----------------|-------|
| `env(key)` | `${!key}` (bash) / generated case (POSIX) | Bash |
| `env_var_or(key, default)` | `${key:-"$default"}` | Yes |
| `env_set(key, val)` | `export key="$val"` | Yes |
| `arg(n)` | `$1`, `$2`, ... `${n}` | Yes |
| `args()` | `"$@"` | Yes |
| `exec(cmd, args)` | `command -- "$cmd" "$@"` | Yes |
| `capture(cmd, args)` | `output=$(command -- "$cmd" "$@")` | Yes |
| `sleep(secs)` | `sleep "$secs"` | Yes |
| `exit(code)` | `exit "$code"` | Yes |
| `pid()` | Not supported (non-deterministic) | -- |
| `hostname()` | `hostname` | Yes |
| `which(cmd)` | `command -v "$cmd"` | Yes |

### 3.4 Array and Math Operations

| Ruchy Function | Shell Equivalent | POSIX |
|----------------|-----------------|-------|
| `arr.len()` | `${#arr[@]}` | Bash |
| `arr.push(val)` | `arr+=("$val")` | Bash |
| `arr.join(sep)` | `IFS="$sep"; echo "${arr[*]}"` | Bash |
| `arr.contains(val)` | Loop with string comparison | Yes |
| `arr.is_empty()` | `[ ${#arr[@]} -eq 0 ]` | Bash |
| `x + y`, `x - y`, `x * y` | `$(( x op y ))` | Yes |
| `x / y`, `x % y` | `$(( x / y ))`, `$(( x % y ))` | Yes |
| `abs(x)` | `$(( x < 0 ? -x : x ))` | Yes |
| `min(a, b)`, `max(a, b)` | `$(( a < b ? a : b ))` | Yes |

## 4. Transpiler Integration

### 4.1 Module Structure

New files under `src/backend/transpiler/`:

| File | Purpose |
|------|---------|
| `shell_backend.rs` | Shell codegen entry point |
| `shell_ir.rs` | Shell IR types |
| `shell_emitter.rs` | POSIX code emitter |
| `shell_stdlib.rs` | Built-in function mappings |
| `shell_safety.rs` | Safety analysis pass |
| `shell_tests.rs` | Shell-specific tests |

### 4.2 Backend Selection

```rust
pub enum TranspilationTarget {
    Rust,                       // Existing default
    Shell(ShellConfig),         // New shell target
}

pub struct ShellConfig {
    pub target_shells: Vec<Shell>,
    pub strict_posix: bool,
    pub shellcheck_validate: bool,
}
```

### 4.3 Variable Quoting (Injection Prevention)

Every variable reference in generated code is double-quoted. The emitter has no
code path that produces an unquoted `$` expansion.

```rust
impl ShellEmitter {
    fn emit_var_ref(&self, name: &str) -> String {
        format!("\"${{{}}}\"", name)
    }
    fn emit_cmd_subst(&self, cmd: &str) -> String {
        format!("\"$({})\"", cmd)
    }
}
```

### 4.4 Determinism Rules

The shell backend rejects any AST node that would produce non-deterministic output
at compile time (not runtime).

| Rejected Pattern | Diagnostic |
|-----------------|------------|
| `random()` | "Use explicit seed or remove for shell target" |
| `timestamp()` | "Timestamps vary between runs" |
| `pid()` | "Process IDs are not reproducible" |
| `tempfile()` | "Use explicit path for shell target" |
| `uuid()` | "UUIDs are not reproducible" |
| `HashMap` iteration | "Shell target requires ordered collections" |

## 5. `ruchy purify` Command

### 5.1 Purpose

Ingest legacy bash scripts containing unsafe, non-deterministic, or non-portable
patterns and transform them into safe, deterministic, idempotent POSIX shell scripts.

### 5.2 Invocation

```bash
ruchy purify messy.sh -o clean.sh           # Transform single file
ruchy purify scripts/ -o cleaned/           # Transform directory
ruchy purify messy.sh --report              # Dry run: show what would change
ruchy purify messy.sh --strict              # Reject instead of transform
```

### 5.3 Pipeline

```
Legacy Bash Script -> [bashrs::bash_parser] -> [Safety Analysis]
  -> [Transformation Engine] -> [bashrs::bash_transpiler]
  -> [ShellCheck Validation] -> Clean POSIX Script
```

### 5.4 Security Rules (SEC001-SEC008)

| Rule | Pattern Detected | Transformation |
|------|-----------------|----------------|
| SEC001 | Unquoted variables `$var` | Quote: `"$var"` |
| SEC002 | `eval` usage | Rewrite to direct command |
| SEC003 | Backtick substitution | Rewrite to `$(cmd)` |
| SEC004 | Unvalidated user input | Add input validation |
| SEC005 | `curl \| bash` pipe-to-shell | Extract to file, validate, execute |
| SEC006 | World-writable temp files | Use `mktemp` with restrictive `umask` |
| SEC007 | Path injection via `$PATH` | Use absolute paths for critical commands |
| SEC008 | Glob injection in `[` tests | Quote globs or use `[[` |

### 5.5 Determinism Rules (DET001-DET006)

| Rule | Pattern Detected | Transformation |
|------|-----------------|----------------|
| DET001 | `$RANDOM` usage | Compile error or explicit seed parameter |
| DET002 | `date +%s` for logic | Accept if logging-only; error otherwise |
| DET003 | `$$` PID usage | Replace with caller-provided ID |
| DET004 | Unsorted `glob *` results | Pipe through `sort` |
| DET005 | `find` without `-maxdepth` | Add explicit depth bound |
| DET006 | Locale-dependent `sort`/`tr` | Add `LC_ALL=C` prefix |

### 5.6 Example Transformation

Input (`messy.sh`):
```bash
#!/bin/bash
for f in *.txt; do
  data=`cat $f`
  if [ $data = "important" ]; then
    cp $f /tmp/backup_$$_$RANDOM
  fi
done
```

Output after `ruchy purify messy.sh`:
```sh
#!/bin/sh
set -eu
for f in ./*.txt; do
  [ -e "$f" ] || continue
  data=$(cat -- "$f")
  if [ "$data" = "important" ]; then
    backup_dir="${BACKUP_DIR:?'BACKUP_DIR must be set'}"
    cp -- "$f" "${backup_dir}/$(basename -- "$f")"
  fi
done
```

Applied: SEC001 (quoting), SEC003 (backticks), SEC008 (glob prefix), DET001
(`$RANDOM` removed), DET003 (`$$` removed), `--` flag injection prevention.

## 6. Shell Validation Pipeline

### 6.1 Three-Stage Validation

**Stage 1 -- Static Analysis (compile-time):** Type safety for shell-compatible types,
determinism verification, injection-safety proof via quoting rules.

**Stage 2 -- ShellCheck Integration (post-generation):** Run ShellCheck on generated
`.sh` files with `--severity=style` (all warnings as errors).

**Stage 3 -- Cross-Shell Matrix Testing (CI):** Execute generated scripts on all 6
target shells, compare stdout/stderr/exit-code, flag any divergence.

### 6.2 Cross-Shell Compatibility Matrix

| Feature | sh | dash | bash | ash | zsh | mksh |
|---------|---:|-----:|-----:|----:|----:|-----:|
| `set -eu` | Yes | Yes | Yes | Yes | Yes | Yes |
| `pipefail` (bash/zsh extension) | No | No | Yes | No | Yes | No |
| `$()` substitution | Yes | Yes | Yes | Yes | Yes | Yes |
| `${var:-default}` | Yes | Yes | Yes | Yes | Yes | Yes |
| `${#var}` length | Yes | Yes | Yes | Yes | Yes | Yes |
| `[ ]` tests | Yes | Yes | Yes | Yes | Yes | Yes |
| `$(( ))` arithmetic | Yes | Yes | Yes | Yes | Yes | Yes |
| Arrays `arr=()` | No | No | Yes | No | Yes | Yes |
| `[[ ]]` extended test | No | No | Yes | No | Yes | Yes |
| `local` keyword | Ext | Yes | Yes | Yes | Yes | Yes |
| Here-strings `<<<` | No | No | Yes | No | Yes | Yes |

With `--strict-posix`, the emitter uses only features supported by all 6 shells.

### 6.3 bashrs Verifier Integration

```rust
pub struct ShellVerifier {
    shellcheck: ShellCheckRunner,
    bashrs: BashrsAnalyzer,
}

impl ShellVerifier {
    pub fn verify(&self, script: &Path) -> Result<VerificationReport> {
        let sc_result = self.shellcheck.run(script)?;
        let br_result = self.bashrs.analyze(script)?;
        Ok(VerificationReport {
            shellcheck_issues: sc_result.issues,
            safety_violations: br_result.violations,
            determinism_score: br_result.determinism_score,
            compatibility: br_result.cross_shell_compat,
        })
    }
}
```

## 7. CLI Commands

| Command | Purpose | Key Options |
|---------|---------|-------------|
| `ruchy build <src> --target shell` | Compile Ruchy to `.sh` | `-o`, `--shells`, `--strict-posix`, `--chmod` |
| `ruchy purify <script.sh>` | Clean legacy bash to safe POSIX | `-o`, `--report`, `--strict`, `--rules` |
| `ruchy lint <script.sh>` | Shell-specific linting | `--severity`, `--rules`, `--format` |
| `ruchy test --shell` | Test generated shell scripts | `--shells`, `--matrix`, `--timeout` |
| `ruchy check --target shell` | Type-check for shell compat | Same as `check` |

### 7.1 `ruchy build --target shell`

```
ruchy build <SOURCE> --target shell [OPTIONS]
  -o, --output <PATH>       Output file (default: <source>.sh)
  --shells <LIST>           Target shells (default: all)
  --strict-posix            POSIX-only features
  --no-shellcheck           Skip ShellCheck validation
  --header <TEXT>           Custom header comment
  --chmod                   Make output executable
```

### 7.2 `ruchy purify`

```
ruchy purify <INPUT> [OPTIONS]
  -o, --output <PATH>       Output path (default: stdout)
  --report                  Dry run showing transformations
  --strict                  Reject unfixable patterns
  --rules <LIST>            Apply specific rules (e.g., SEC001,DET003)
  --ignore <LIST>           Skip specific rules
```

## 8. Testing Requirements

### 8.1 Corpus-Driven Testing

Validate generated shell code against the bashrs corpus of 17,882 patterns. Each
generated script must pass all applicable bashrs lint rules without suppression.

```rust
#[test]
fn test_shell_001_corpus_validation() {
    for pattern in bashrs::load_corpus().patterns() {
        let script = generate_shell_for_pattern(pattern);
        let result = bashrs::validate(&script);
        assert!(result.is_ok(), "Pattern {} failed: {:?}", pattern.id, result);
    }
}
```

### 8.2 Property Tests for Determinism

```rust
proptest! {
    #[test]
    fn test_shell_determinism(input in arb_ruchy_program()) {
        let shell1 = compile_to_shell(&input);
        let shell2 = compile_to_shell(&input);
        prop_assert_eq!(shell1, shell2, "Same input must produce identical output");
    }

    #[test]
    fn test_shell_idempotency(script in arb_shell_script()) {
        let run1 = execute_shell(&script);
        let run2 = execute_shell(&script);
        prop_assert_eq!(run1, run2, "Re-execution must produce same result");
    }

    #[test]
    fn test_shell_quoting(input in ".*") {
        let quoted = shell_quote(&input);
        prop_assert!(!quoted.contains_unquoted_expansion());
    }
}
```

### 8.3 Cross-Shell Matrix Tests

```rust
const SHELLS: &[&str] = &["sh", "dash", "bash", "ash", "zsh", "mksh"];

#[test]
fn test_shell_cross_compat_matrix() {
    for script in &collect_generated_scripts() {
        let results: Vec<_> = SHELLS.iter()
            .filter(|s| which(s).is_ok())
            .map(|s| (*s, execute_with_shell(s, script)))
            .collect();
        for pair in results.windows(2) {
            assert_eq!(pair[0].1.stdout, pair[1].1.stdout,
                "{} vs {} diverged", pair[0].0, pair[1].0);
        }
    }
}
```

### 8.4 Mutation Tests for Safety Rules

```bash
cargo mutants --file src/backend/transpiler/shell_safety.rs --timeout 300
# Target: >= 85% kill rate; surviving mutant in quoting/determinism = P0 bug
```

### 8.5 Coverage Targets

| Component | Line | Branch | Mutation Kill |
|-----------|------|--------|---------------|
| `shell_safety.rs` | 100% | 100% | >= 95% |
| `shell_emitter.rs` | >= 95% | >= 90% | >= 85% |
| `shell_ir.rs` | >= 95% | >= 90% | >= 85% |
| `shell_stdlib.rs` | >= 90% | >= 85% | >= 80% |
| `shell_backend.rs` | >= 90% | >= 85% | >= 80% |

### 8.6 Test Naming Convention

```
test_shell_<NNN>_<section>_<feature>_<scenario>

Examples:
  test_shell_001_stdlib_string_trim_whitespace()
  test_shell_002_safety_quoting_special_chars()
  test_shell_003_determinism_no_random()
  test_shell_004_purify_sec001_unquoted_variable()
  test_shell_005_compat_dash_arithmetic()
```
