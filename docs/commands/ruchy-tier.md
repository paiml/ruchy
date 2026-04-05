# ruchy tier — §14.2 Provability Tier Distribution

The `ruchy tier` command scans a directory (or single file) of Ruchy source
and reports how each function is classified under the **§14.2 four-tier
provability model** (Bronze → Silver → Gold → Platinum). It is the primary
input to the §14.5 falsifier metrics (F1/F2/F4/F11) defined in the
[Ruchy 5.0 Sovereign Platform spec](../specifications/ruchy-5.0-sovereign-platform.md).

## Overview

Every non-test `fun` in Ruchy is assigned exactly one tier based on its
signature (not attributes). `ruchy tier` walks a directory, classifies
each function, and emits a report with:

- **Tier distribution** — counts and percentages of Bronze/Silver/Gold/Platinum
- **§14.5 F1 metric** — % of functions with non-trivial contracts
- **§14.5 F2 metric** — `#[contract_exempt]` density per KLoC
- **§14.5 F4 proxy** — Bronze-tier `pub` function count
- **§14.5 F11 metric** — `#[diff_exempt]` density per KLoC
- **§14.10.6 violations** — Gold/Platinum functions lacking `@total`
- **Totality breakdown** — `@total` / `@partial` / unmarked counts

The tier is derived entirely from the function signature:

| Tier | Signature shape |
|------|-----------------|
| Bronze | `fun f(...) { ... }` — no `requires` / `ensures` |
| Silver | `fun f(...) requires P ensures Q { ... }` |
| Gold | `@gold fun f(...) requires P ensures Q { ... }` |
| Platinum | `@platinum` + YAML contract + Lean theorem |

## Basic Usage

```bash
# Scan a directory, print human summary
ruchy tier src/

# Scan a single file
ruchy tier src/lib.ruchy

# Machine-readable JSON (one line, dashboard-ready)
ruchy tier src/ --json

# Enumerate each classified function
ruchy tier src/ --list

# Both: aggregate JSON + per-function JSON array
ruchy tier src/ --json --list

# Scope to public API only (§14.5 F4 surface area)
ruchy tier src/ --public-only

# Triage: show the 10 files with the most Bronze functions
ruchy tier src/ --by-file --sort-by bronze --top 10
```

## Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `path` | Directory or `.ruchy` file to scan | Required |
| `--json` | Single-line JSON output (21 keys) | `false` |
| `--list` | Enumerate each function with tier + `pub` marker | `false` |
| `--public-only` | Restrict scan to `pub fn` only | `false` |
| `--by-file` | Show per-file tier breakdown (table + JSON array) | `false` |
| `--sort-by <COL>` | Sort `--by-file` by file/bronze/silver/gold/platinum/total | `file` |
| `--top <N>` | Limit `--by-file` to N entries after sorting | none |
| `--parse-timeout-ms <MS>` | Per-file parse timeout (resilience vs parser hangs) | `5000` |
| `--baseline <FILE>` | Regression gate: compare against stored baseline JSON | none |
| `--markdown` | GitHub-flavored markdown report (for PR comments/summaries) | `false` |
| `--fail-under <PCT>` | Exit 1 if `non_bronze_pct` < PCT | none |
| `--fail-under-f1 <PCT>` | Exit 1 if F1 `non_trivial_pct` < PCT | none |
| `--fail-exempt-density-above <PER_KLOC>` | Exit 1 if F2 density > K | none |
| `--fail-diff-exempt-density-above <PER_KLOC>` | Exit 1 if F11 density > K | none |
| `--fail-pub-bronze-above <N>` | Exit 1 if `pub_bronze` count > N | none |
| `--fail-on-totality-violation` | Exit 1 on any §14.10.6 violation | `false` |

## Output Formats

### Human Summary (default)

```
Provability tier scan: src/
files: 42
loc: 3128
functions: 187
  bronze:   47 (25.1%)
  silver:   128 (68.4%)
  gold:     12 (6.4%)
  platinum: 0 (0.0%)
non-bronze: 74.9%
contract triviality (F1):
  non-trivial: 132
  trivial:     8
  non-trivial %: 94.3%
exemptions (F2):
  #[contract_exempt]: 3
  density / KLoC:     0.96
diff exemptions (F11):
  #[diff_exempt]: 0
  density / KLoC: 0.00
public API (F4 proxy):
  pub Bronze: 2
totality:
  @total:    18
  @partial:  128
  unmarked:  41
parse errors: 0
§14.5 scorecard: F1:OK F2:WARN F4:WARN F11:OK
```

The scorecard line gives single-glance status per §14.5 metric:

| Metric | OK | WARN | FAIL |
|--------|----|------|------|
| F1 non-trivial % | ≥95 | <95 | <50 |
| F2 exempt/KLoC | ≤0.5 | >0.5 | >5.0 |
| F4 pub Bronze | =0 | >0 | (becomes FAIL at 5.2) |
| F11 diff_exempt/KLoC | =0 | >0 | — |

`N/A` appears when a metric cannot be evaluated (e.g., no contracts → no F1).

### JSON Aggregate (`--json`)

Single-line object with 21 keys, stable schema:

```json
{"files":42,"loc":3128,"functions":187,"bronze":47,"silver":128,"gold":12,
 "platinum":0,"non_bronze_pct":74.87,"non_trivial_contracts":132,
 "trivial_contracts":8,"non_trivial_pct":94.29,"contract_exempt":3,
 "exempt_density_per_kloc":0.96,"diff_exempt":0,
 "diff_exempt_density_per_kloc":0.00,"total_marked":18,"partial_marked":128,
 "totality_unmarked":41,"totality_violations":0,"pub_bronze":2,
 "parse_errors":0}
```

### Markdown Report (`--markdown`)

Renders a PR-ready report with status badges:

```markdown
## §14.5 Provability Tier Report

**Files scanned:** 42 (3128 LoC, 187 functions)

### Tier Distribution
| Tier | Count | % |
|------|------:|--:|
| Bronze   | 47 | 25.1% |
| Silver   | 128 | 68.4% |
| ...

### §14.5 Falsifier Scorecard
| Metric | Value | Status |
|--------|------:|:------:|
| F1 non-trivial % | 94.3% | 🟡 WARN |
| F2 exempt / KLoC | 0.96 | 🟡 WARN |
| F4 pub Bronze | 2 | 🟡 WARN |
| F11 diff_exempt / KLoC | 0.00 | 🟢 OK |
```

Usage in GitHub Actions:
```yaml
- run: ruchy tier src/ --markdown >> $GITHUB_STEP_SUMMARY
```

### Per-File Table (`--by-file`)

```
per-file tier breakdown:
  bronze silver gold   platinum total   file
  2      0      0      0        2       src/a.ruchy
  0      1      0      0        1       src/b.ruchy
```

Pair with `--json` to emit a second JSON line:

```json
[{"file":"src/a.ruchy","bronze":2,"silver":0,"gold":0,"platinum":0,"total":2},
 {"file":"src/b.ruchy","bronze":0,"silver":1,"gold":0,"platinum":0,"total":1}]
```

### Per-Function JSON (`--json --list`)

Two lines: aggregate object, then a function array:

```json
{"files":1,"loc":3,"functions":2,...}
[{"name":"a","file":"src/a.ruchy","tier":"bronze","totality":"unknown","pub":true,"non_trivial_contract":false},
 {"name":"b","file":"src/a.ruchy","tier":"silver","totality":"unknown","pub":false,"non_trivial_contract":true}]
```

### List Table (`--list`)

```
functions:
  bronze     unknown    pub  exposed (src/lib.ruchy)
  silver     partial         internal (src/lib.ruchy)
```

Columns: tier, totality, `pub` marker, function name, source file.

## CI Gate Recipes

`ruchy tier` ships with seven CI gate flags (six threshold gates + baseline). All exit non-zero on breach
and print a spec-ticketed error message to stderr.

### Gate: ≥50% non-Bronze (§14.2)

```bash
ruchy tier src/ --fail-under 50
# ⇒ Error if bronze > 50% of functions
```

### Gate: ≥95% non-trivial contracts (§14.5 F1)

```bash
ruchy tier src/ --fail-under-f1 95
# ⇒ Error if < 95% of contracted fns have non-trivial clauses
```

### Gate: ≤0.5 exempt density per KLoC (§14.5 F2)

```bash
ruchy tier src/ --fail-exempt-density-above 0.5
# ⇒ Error if #[contract_exempt] density > 0.5 per KLoC
```

### Gate: ≤1.0 `#[diff_exempt]` per KLoC (§14.5 F11)

```bash
ruchy tier src/ --fail-diff-exempt-density-above 1.0
# ⇒ Error if #[diff_exempt] density > 1.0 per KLoC
# F11 target: publish 0 #[diff_exempt] without ticket
```

### Gate: 0 pub Bronze (§14.5 F4)

```bash
ruchy tier src/ --fail-pub-bronze-above 0
# ⇒ Error if any pub function is Bronze
# Use during 5.0→5.2 migration with a declining ceiling
```

### Gate: no regressions vs baseline

```bash
# First run: capture baseline
ruchy tier src/ --baseline .ruchy/tier-baseline.json

# Subsequent runs: compare
ruchy tier src/ --baseline .ruchy/tier-baseline.json
# ⇒ baseline OK: no regressions vs .ruchy/tier-baseline.json
# ⇒ (or) Error: N baseline regression(s) detected
```

Commit the baseline JSON to version control. CI re-runs `ruchy tier`
with `--baseline` on every PR; any metric regression (bronze count up,
non-trivial % down, etc.) fails the build.

### Gate: no §14.10.6 totality violations

```bash
ruchy tier src/ --fail-on-totality-violation
# ⇒ Error if any Gold/Platinum fn lacks @total / @corecursive
```

### Combined CI pipeline example

```bash
ruchy tier src/ \
  --fail-under 50 \
  --fail-under-f1 80 \
  --fail-exempt-density-above 1.0 \
  --fail-diff-exempt-density-above 1.0 \
  --fail-pub-bronze-above 5 \
  --fail-on-totality-violation
```

## Metric Definitions (§14.5)

| # | Metric | Falsifier range |
|---|--------|-----------------|
| F1 | `non_trivial_pct` of contracted fns | < 50% at 5.2 → "mandatory" gate is performative |
| F2 | `#[contract_exempt]` density per KLoC | > 5/KLoC → gate is routinely bypassed |
| F4 | stdlib Bronze count (`pub_bronze` proxy) | ≥ 1 after 5.2 → stdlib isn't provable |
| F11 | `#[diff_exempt]` density per KLoC | ≥ 1 published w/o ticket → differential gate has hole |

## §14.10.6 Totality Rule

Gold and Platinum functions **must** be annotated `@total` or
`@corecursive(justification = "...")`. `ruchy tier` reports violations
to stderr and (with `--fail-on-totality-violation`) exits non-zero:

```
§14.10.6 violations: 1 Gold/Platinum function(s) lack @total:
  compute_fib (src/math.ruchy) is gold but has unknown
```

## Limitations (current implementation)

- **F1 triviality** is detected **syntactically** (`requires true` /
  `ensures true`). Genuine SMT-based tautology detection (Z3 `P ↔ true`)
  is a future sprint.
- **F4 is a proxy**: `pub_bronze` approximates "stdlib Bronze count → 0".
  True stdlib-vs-user-code distinction requires package metadata.
- No AST-to-tier bridge for method definitions inside `impl` blocks
  (functions in `fun` position only).
- **Parse timeouts**: a file whose parser exceeds `--parse-timeout-ms`
  (default 5000) is counted in `parse_timeouts` and skipped. This
  indicates a parser bug (infinite loop) in the underlying `ruchy`
  frontend, not in `ruchy tier` itself.

## See Also

- [Ruchy 5.0 Sovereign Platform spec](../specifications/ruchy-5.0-sovereign-platform.md) — §14 Provability Mandate
- [provable-contracts integration](../specifications/provable-contracts-language-integration.md) — Pillar 1 sub-spec
- `ruchy contracts check` — YAML contract synchronization
- `ruchy suggest-contracts` — LLM-inferred contract suggestions
