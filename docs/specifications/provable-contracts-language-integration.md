# Sub-spec: Provable Contracts as First-Class Language Feature

**Parent:** docs/SPECIFICATION.md
**Ticket:** CONTRACTS-001
**Status:** Proposed
**Priority:** High

---

## 0. Prerequisites

Current codebase state (as of the writing of this spec):

- **`unsafe_code = "warn"`** in Cargo.toml (not `"forbid"`). Changed from `"forbid"` for
  DEFECT-001-B CallFrame Send impl.
- **No existing parser support for contract keywords.** The Ruchy parser has no
  `ContractClause`, `requires_clause`, `ensures_clause`, `invariant_clause`, or
  `decreases_clause` productions. All four keywords (`requires`, `ensures`, `invariant`,
  `decreases`) are entirely new parser work (greenfield implementation).
- **`generated_contracts.rs` is currently a stub.** Commit `cdee704c` replaced the
  original 5858-line auto-generated file with a 3-line placeholder containing zero
  preconditions and zero postconditions.

## 1. Overview

Provable contracts in Ruchy currently operate at the build-system level: YAML contract
definitions are validated in `build.rs`, macro assertions are emitted to
`generated_contracts.rs`, and trait enforcement is checked at compile time via
`tests/contract_traits.rs`. This spec promotes contracts from build-time machinery to
first-class language syntax with dedicated keywords, graduated verification levels,
and integrated tooling.

**Theoretical Foundations:** Hoare Logic (1969), Meyer's Design by Contract/Eiffel
(1997), SPARK/Ada graduated proof levels (2014), Dafny/Boogie/Z3 (2010), Verus
(2023), F* (2016), Findler-Felleisen blame tracking (2002).

**Design Principle:** Ruchy users write Python-like syntax; the compiler extracts
contracts, transpiles them to Rust assertions or formal verification harnesses, and
enforces them at the appropriate level. No annotation macros, no procedural macro
crates -- contracts are parsed by the Ruchy frontend like any other statement.

## 2. Language Syntax

Four new keywords enter the Ruchy grammar as contract clauses attached to function
declarations, class bodies, and loop constructs.

> **Note (greenfield):** As of this writing, the Ruchy parser has zero existing
> support for contract keywords. There are no `ContractClause`, `requires_clause`,
> or `ensures_clause` AST nodes, no lexer tokens for these keywords, and no grammar
> productions. This section describes entirely new parser work -- not a promotion of
> existing infrastructure.

### 2.1 Keyword Summary

| Keyword | Placement | Semantics |
|---|---|---|
| `requires` | Before function body | Precondition: must hold on entry |
| `ensures` | Before function body | Postcondition: must hold on exit |
| `invariant` | Class body or loop head | Preserved across mutations / iterations |
| `decreases` | Loop head or recursive fn | Termination metric, must strictly decrease |

### 2.2 Grammar Extension

```ebnf
contract_clause  = requires_clause | ensures_clause | invariant_clause | decreases_clause
requires_clause  = "requires" expr
ensures_clause   = "ensures" expr            (* `result` binds return value *)
invariant_clause = "invariant" expr
decreases_clause = "decreases" expr

fun_decl         = "fun" ident "(" params ")" ["->" type] contract_clause* block
class_body       = (field_decl | method_decl | invariant_clause)*
loop_stmt        = "while" expr [invariant_clause] [decreases_clause] block
```

### 2.3 Full Example

```ruchy
fun binary_search(arr: List[int], target: int) -> int
    requires len(arr) > 0
    requires is_sorted(arr)
    ensures result >= -1
    ensures result >= 0 implies arr[result] == target
{
    let lo = 0
    let hi = len(arr) - 1

    while lo <= hi
        invariant 0 <= lo
        invariant hi < len(arr)
        decreases hi - lo + 1
    {
        let mid = lo + (hi - lo) / 2
        if arr[mid] == target {
            return mid
        } else if arr[mid] < target {
            lo = mid + 1
        } else {
            hi = mid - 1
        }
    }
    return -1
}

class BoundedStack
    invariant len(self.items) <= self.capacity
{
    items: List[int]
    capacity: int

    fun push(self, value: int)
        requires len(self.items) < self.capacity
        ensures len(self.items) == old(len(self.items)) + 1
    {
        self.items.append(value)
    }

    fun pop(self) -> int
        requires len(self.items) > 0
        ensures result == old(self.items[len(self.items) - 1])
    {
        return self.items.pop()
    }
}
```

### 2.4 The `old()` and `result` Pseudo-functions

| Name | Context | Meaning |
|---|---|---|
| `result` | `ensures` clause | Binds the return value of the function |
| `old(expr)` | `ensures` clause | Evaluates `expr` in the pre-state (before function body executes) |

`old()` transpiles to a let-binding captured before the function body. `result` is
the variable holding the return value in the generated wrapper.

> **Ownership note:** `old(expr)` requires cloning the expression's value before the
> function body executes. For complex types (e.g., `List[int]`, `Map[str, T]`,
> user-defined classes), this inserts `.clone()` calls that may be expensive. The
> transpiler should emit a warning when `old()` is used on non-`Copy` types with
> large expected allocations. A future optimization could use copy-on-write
> (`Cow<T>`) or reference snapshots where provably safe.

## 3. SPARK-Inspired Graduated Enforcement

Ruchy adopts a five-level verification model inspired by SPARK/Ada. Each level
subsumes all lower levels. Functions default to Silver unless overridden.

### 3.1 Level Definitions

| Level | Name | What Is Verified | Cost | Tooling |
|---|---|---|---|---|
| 0 | Stone | Valid Ruchy syntax, type-checks | Zero | `ruchy check` |
| 1 | Bronze | Data flow: no uninitialized reads, no unused assignments | Zero | `ruchy lint --flow` |
| 2 | Silver | Absence of Runtime Errors (AoRTE): no panics, no overflows, no OOB | Near-zero | Rust type system + `debug_assert!` |
| 3 | Gold | Functional contracts verified on annotated paths | Moderate | SMT solver (`src/proving/smt.rs`) |
| 4 | Platinum | Full formal proof of all contracts and termination | High | Kani / Verus backend |

### 3.2 Default Level

Silver is the default. Rust's type system already provides AoRTE guarantees for safe
code. The transpiler emits `debug_assert!` for `requires`/`ensures` in debug builds,
giving Silver-level checking with zero release overhead.

### 3.3 Per-Function Override

```ruchy
#[prove(gold)]
fun transfer(from: Account, to: Account, amount: u64)
    requires from.balance >= amount
    ensures from.balance == old(from.balance) - amount
    ensures to.balance == old(to.balance) + amount
{
    from.balance -= amount
    to.balance += amount
}

#[prove(platinum)]
fun crypto_verify(sig: Signature, msg: Bytes) -> bool
    requires len(msg) > 0
    ensures result == true implies valid_signature(sig, msg)
{
    // ...
}
```

### 3.4 Level Selection Guidelines

- **Stone/Bronze:** Prototypes, internal helpers
- **Silver (default):** Application logic, data science pipelines
- **Gold:** Financial calculations, library public APIs
- **Platinum:** Cryptographic primitives, safety-critical control

## 4. Transpiler Integration

The transpiler (`src/backend/transpiler/`) maps each contract keyword to Rust code
whose form depends on the enforcement level.

### 4.1 Silver (Default): Debug Assertions

```ruchy
fun abs(x: int) -> int
    requires true       # trivial, elided
    ensures result >= 0
{
    if x < 0 { return -x }
    return x
}
```

Transpiles to:

```rust
fn abs(x: i64) -> i64 {
    // requires: trivial, elided
    let __result = {
        if x < 0 { return -x; }
        x
    };
    debug_assert!(__result >= 0, "ensures violated: result >= 0");
    __result
}
```

### 4.2 Gold: SMT Verification Harness

For `#[prove(gold)]` functions, the transpiler emits an SMT query module alongside
the runtime code. The existing `src/proving/smt.rs` module provides the SMT backend
(Z3/CVC5 interface), but it is not currently wired to the transpiler. A new
transpiler-to-prover bridge must be built to route Gold-level queries to the solver.

```rust
// Emitted to src/proving/generated/abs_proof.rs
#[cfg(feature = "prove")]
fn prove_abs() -> SmtResult {
    let mut solver = SmtSolver::with_backend(SmtBackend::Z3);
    solver.declare("x", "Int");
    solver.assert("(>= (ite (< x 0) (- x) x) 0)");
    solver.check_sat()
}
```

### 4.3 Platinum: Kani/Verus Backend

For `#[prove(platinum)]`, the transpiler emits Kani proof harnesses or Verus
annotations that are verified during CI.

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_abs() {
    let x: i64 = kani::any();
    let result = abs(x);
    assert!(result >= 0);
}
```

### 4.4 Invariant and Decreases Emission

| Keyword | Transpiler Output |
|---|---|
| `invariant` (class) | Assertion in every `&mut self` method epilogue |
| `invariant` (loop) | `debug_assert!` at loop entry and end of each iteration |
| `decreases` (loop) | Shadow variable + `debug_assert!(new_metric < old_metric)` per iteration |
| `decreases` (recursion) | Parameter comparison assertion at recursive call site |

### 4.5 YAML Contract Synchronization (`ruchy contracts sync`)

> **Important:** YAML generation cannot happen inside the transpiler or `build.rs`,
> because `build.rs` runs before compilation and has no access to parsed Ruchy ASTs.
> Instead, a separate CLI command performs this extraction.

```bash
ruchy contracts sync src/ -o contracts/   # Extract contracts from .ruchy to YAML
ruchy contracts sync --check src/         # Verify YAML is up-to-date (CI gate)
```

The `ruchy contracts sync` command parses `.ruchy` source files, extracts contract
clauses from the AST, and emits YAML files compatible with the `provable-contracts`
crate format. This runs as an explicit developer action (or CI step), not as part
of the build pipeline.

```yaml
# Generated by `ruchy contracts sync` from binary_search.ruchy
version: "1.0"
contract: binary-search-v1
equations:
  - name: binary_search
    requires:
      - "len(arr) > 0"
      - "is_sorted(arr)"
    ensures:
      - "result >= -1"
      - "result >= 0 implies arr[result] == target"
```

## 5. Contracts as Documentation

### 5.1 Contract Form (Eiffel Pattern)

Eiffel pioneered the idea that contracts ARE documentation. Ruchy adopts this: the
`ruchy doc` command extracts `requires`/`ensures`/`invariant` clauses and renders
them as structured API documentation.

### 5.2 ruchy doc Integration

```bash
ruchy doc src/           # Generate HTML docs with contract sections
ruchy doc --json src/    # Machine-readable contract extraction
ruchy doc --verify src/  # Verify all Gold/Platinum contracts, then generate docs
```

The `--json` output feeds downstream tools (LLM inference, CI dashboards, binding
validators) without parsing HTML.

## 6. Blame Tracking (Findler-Felleisen)

### 6.1 Caller vs. Callee Fault

When a contract violation occurs at runtime, the error message must identify blame:

| Violation | Fault | Example |
|---|---|---|
| `requires` fails | **Caller** | Passed empty array to `binary_search` |
| `ensures` fails | **Callee** | `binary_search` returned wrong index |
| `invariant` fails | **Callee** | `BoundedStack.push` broke capacity bound |

### 6.2 Error Message Format

Error messages include: source location, function name, violated clause text, blame
assignment (CALLER or CALLEE), and runtime values of relevant variables.

### 6.3 Higher-Order Contract Wrapping

When a function is passed as an argument, its contracts travel with it. The
transpiler wraps higher-order function arguments with contract-checking closures.
If a passed function `f` violates an `ensures` clause, blame falls on the **caller**
who supplied `f`, following Findler-Felleisen's higher-order contract semantics.

## 7. LLM-Assisted Contract Inference

### 7.1 Automated Contract Suggestion

```bash
ruchy suggest-contracts src/math.ruchy    # Infer contracts from signatures + bodies
ruchy suggest-contracts --apply src/      # Infer and insert into source files
```

**Inference Pipeline:**

| Stage | Technique | Reference |
|---|---|---|
| Signature analysis | Type-driven precondition templates | Heuristic |
| Body analysis | Weakest-precondition calculus | Dijkstra (1975) |
| NL2Contract | LLM translates docstrings to formal specs | arXiv:2510.12702 |
| AutoVerus | Automated proof generation for Rust/Verus | arXiv:2409.13082 |

> **Infrastructure note:** LLM-assisted contract inference requires external LLM API
> access (e.g., Anthropic or OpenAI). This feature is not implementable without
> infrastructure for model invocation -- an API key configuration mechanism, a network
> client, and a prompt/response pipeline. Implementation is deferred until the `ruchy`
> CLI has a plugin or external-tool integration system.

### 7.2 Inference Example

For `fun divide(a: float, b: float) -> float`, the tool infers
`requires b != 0.0` (high confidence) and
`ensures b > 0.0 implies (result >= 0.0) == (a >= 0.0)` (medium confidence).

### 7.3 Confidence Levels

| Confidence | Action | Example |
|---|---|---|
| High (>0.9) | Auto-insert with `--apply` | `requires len(arr) > 0` for index access |
| Medium (0.5-0.9) | Suggest in comments | `# suggested: ensures result >= 0` |
| Low (<0.5) | Omit unless `--all` flag | Speculative invariants |

## 8. Integration with Existing Systems

### 8.1 Current Infrastructure Map

| Component | Location | Role | Change Required |
|---|---|---|---|
| `build.rs` contract validation | `build.rs:35-109` | YAML binding enforcement | Consume YAML from `ruchy contracts sync` (not from transpiler) |
| `generated_contracts.rs` | `src/generated_contracts.rs` | Stub (0 assertions, commit `cdee704c`) | Rebuild via transpiler emission or `ruchy contracts sync` |
| SMT prover module | `src/proving/smt.rs` | Z3/CVC5 interface (not wired to transpiler) | New transpiler-to-prover bridge required |
| Refinement checker | `src/proving/refinement.rs` | Type refinement verification | Platinum-level dispatch |
| Contract traits | `tests/contract_traits.rs` | Compile-time trait checks | Unchanged; contracts generate trait impls |
| `provable-contracts` crate | `Cargo.toml` dev-dependency | Trait definitions | Upstream: add `ContractAware` trait |

### 8.2 Migration Path

> **Starting point:** `generated_contracts.rs` is currently a 3-line stub (commit
> `cdee704c`). The parser has zero contract keyword support. The SMT prover module
> exists but is not connected to the transpiler. This migration builds everything
> from scratch.

**Phase 1 (Silver):** Greenfield parser work: add lexer tokens and grammar
productions for `requires`, `ensures`, `invariant`, `decreases`. Add
`ContractClause` AST nodes. Transpiler emits `debug_assert!`. Existing `build.rs`
pipeline unchanged. No new dependencies.

**Phase 2 (Gold):** Build transpiler-to-prover bridge connecting contract AST nodes
to `src/proving/smt.rs`. Transpiler emits SMT queries to `src/proving/`. `ruchy
prove` command invokes solver. Requires Z3 installed (optional dependency).

**Phase 3 (Platinum):** Kani/Verus backend integration. `ruchy prove --platinum`
generates and checks proof harnesses. CI-only; not required for local dev.

**Phase 4 (Tooling):** `ruchy contracts sync` for YAML generation, `ruchy doc
--verify`, blame tracking in error messages. `ruchy suggest-contracts` deferred
until LLM API infrastructure is available (see Section 7).

### 8.3 AST Node Addition

```rust
/// Contract clause in the AST
#[derive(Debug, Clone, PartialEq)]
pub enum ContractClause {
    Requires(Expr),
    Ensures { result_binding: Option<String>, condition: Expr },
    Invariant(Expr),
    Decreases(Expr),
}

/// Extended function declaration
pub struct FunDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub contracts: Vec<ContractClause>,  // NEW
    pub body: Block,
    pub proof_level: ProofLevel,         // NEW
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ProofLevel {
    Stone,
    Bronze,
    #[default]
    Silver,
    Gold,
    Platinum,
}
```

## 9. Quality Gates

### 9.1 Pre-Commit (Mandatory)

All Silver-level contracts must pass. This is enforced automatically because Silver
contracts transpile to `debug_assert!` which executes in `cargo test`.

```bash
# Pre-commit hook addition
cargo test --lib --release=false    # debug_assert! active
ruchy lint --contracts src/         # Verify contract syntax
```

### 9.2 CI Pipeline

| Gate | Level | Trigger | Timeout |
|---|---|---|---|
| Syntax | Stone | Every push | 30s |
| Data flow | Bronze | Every push | 60s |
| Debug assertions | Silver | Every push | 5min |
| SMT verification | Gold | PR to main, tagged functions only | 15min |
| Formal proof | Platinum | Release branch, tagged functions only | 60min |

### 9.3 Release Criteria

| Component | Required Level | Rationale |
|---|---|---|
| Core stdlib (`List`, `Map`, `String`) | Gold | Public API must have verified contracts |
| Cryptographic functions | Platinum | Safety-critical, no margin for error |
| Parser/Transpiler internals | Silver | Covered by Rust type system + tests |
| User-facing CLI | Bronze | Data flow correctness sufficient |
| Examples and tutorials | Stone | Syntax validity only |

### 9.4 PMAT Integration

Contract coverage becomes a PMAT quality dimension:

```bash
pmat query --contracts src/           # Show contract density per function
pmat tdg . --include-contracts        # TDG score includes contract coverage
```

Functions with Gold/Platinum contracts receive a TDG bonus. Functions in high-risk
modules (identified by `pmat query --churn`) without contracts receive a penalty.

### 9.5 Metrics

Target: >60% contract density on public functions, 100% pass rate at Silver/Gold/Platinum
for tagged functions, 100% blame accuracy. Measured via `ruchy doc --json` and proof exit codes.
