# RHL-0 Phase-0 bindings — the `[U]` rows of RHL-001, bound by reading HEAD

Spec: `docs/specifications/ruchy-high-level-language-interface.md` (RHL-001).
Tree read: `paiml/ruchy` at HEAD `11221f57`, branch `RHL-0-spec-grammar-corpus` off `main` (`be26174c`).

Every row below is `[V]` — the command that measured it is given. A row nobody
re-ran is not a binding, so each is re-runnable from the repository root.

---

## B1 — ruchy's internal AST and the right seam for a new front end

**Bound: the seam is ruchy's PUBLIC crate API, and RHL never touches the AST.**

RHL-001 §0 already forbids a second compiler to Rust: RHL lowers to *ruchy*, and
ruchy's existing transpiler produces the Rust. So the seam RHL needs is not an
internal AST hook at all — it is the two public entry points ruchy already
exports:

```
src/lib.rs:111   pub use backend::{ModuleResolver, Transpiler};
src/lib.rs:112   pub use frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, UnaryOp};
```

and the transpile entry point itself:

```
src/backend/transpiler/mod.rs:375   Transpiler::transpile(&self, expr: &Expr) -> Result<TokenStream>
```

The lowering path is therefore:

```
.rhl ──► intent tree ──► ruchy SOURCE TEXT ──► ruchy::Parser::parse() ──► Expr ──► ruchy::Transpiler::transpile() ──► Rust
                         ^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                         RHL emits this        already built, already tested, unmodified
```

RHL emits ruchy source text and hands it to the existing parser. It constructs no
`Expr` values, patches no transpiler arm, and forks nothing.

**Consequence for §10:** the STOP condition `ast-seam-absent` **does not fire**.
The seam exists, it is public, and it is the coarsest one available — which is the
one that cannot rot when ruchy's internals change.

Measured by:

```bash
grep -n '^pub use' src/lib.rs
pmat query "transpile AST to Rust entry point" --limit 8
```

---

## B2 — what `contracts/` holds

**Bound: 14 YAML contracts, all about PROCESS invariants, none about language semantics.**

```bash
ls contracts/
```

```
checkpoint-v1.yaml        dogfood-receipt-v1.yaml   release-workflow-v1.yaml
clean-room-v1.yaml        eval-dispatch-v1.yaml     roadmap-sot-v1.yaml
dispatch-v1.yaml          examples-parse-v1.yaml    test-feature-gates-v1.yaml
dogfood-gates-v1.yaml     feature-matrix-v1.yaml    transpile-dispatch-v1.yaml
                          release-hygiene-v1.yaml
```

Shape of each (measured on `contracts/roadmap-sot-v1.yaml`): `metadata` ·
`equations` (formula, domain, invariants) · `proof_obligations` ·
`falsification_tests` (each naming a Rust test by function name, with a
`prediction` and an `if_fails`) · `kani_harnesses` · `qa_gate`.

**Consequence for RHL:** RHL-001 §3.4's rule "no term without a contract
template" fits this directory's existing shape without inventing a dialect — an
RHL vocabulary-term contract is another file of the same form, whose
`falsification_tests` name the Rust tests that hold the term to its type and
effect. The `contracts/` directory is currently **process** contracts only, so
RHL contracts are the first *language* contracts to live there; that is an
addition, not a conflict.

---

## B3 — does `pv` run in ruchy's required check?

**Bound: NO. `pv` runs in `ci / gate`, but only as `pv codegen`, and every failure
is swallowed.**

The required status context is `gate` (`.github/workflows/ci.yml:44`), which
re-exports the verdict of the reusable workflow
`paiml/.github/.github/workflows/sovereign-ci.yml@main`. That workflow's `test`,
`lint` and `coverage` jobs each contain a step named
**"Generate contract assertions (pv codegen)"**. Measured content of that step:

- it searches `/usr/local/cargo/bin/pv`, `/usr/local/bin/pv`, then `PATH`;
- **pv absent ⇒ `::warning::` and `exit 0`** — the step passes;
- **pv present ⇒ `"$PV" codegen … || true`** — the failure is discarded.

There is no `pv validate`, no `pv lint`, and no `pv audit` anywhere in the
required check, and none in this repository's `Makefile`, `scripts/`, or
`.git/hooks/pre-commit`.

```bash
gh api repos/paiml/.github/contents/.github/workflows/sovereign-ci.yml --jq .content | base64 -d | grep -n 'pv codegen' -A 22
grep -rnE '(^|[^a-z])pv (check|validate|lint|audit)' Makefile scripts/ .github/ .git/hooks/pre-commit
```

**Consequence for RHL-001 §9.3** ("every term, and every compiled unit, carries a
`pv` contract — or the compiler refuses by name"): that rule is **not** enforced by
CI today and cannot be delegated to it. RHL must enforce it **itself**, in a test
that runs inside the gate (see B4). Writing a contract file and assuming CI checks
it would be theatre.

---

## B4 — what `ci / gate` actually runs (not in the spec's `[U]` list; measured because B3 needed it)

**Bound: `cargo test --lib` on the ROOT crate only. Integration tests under
`tests/` are NOT in the required check.**

`sovereign-ci.yml`'s test job sets

```
TEST_SCOPE: ${{ inputs.test_workspace && '--workspace --lib' || '--lib' }}
```

and `.github/workflows/ci.yml` passes only `repo:` — no `test_workspace` — so
`TEST_SCOPE` is `--lib`. The lint job runs `cargo fmt --all -- --check` and
`cargo clippy … -- -D warnings -A unused-variables`.

**Consequence for RHL-0's `done_when`** ("the grammar check is in `ci / gate` with
its ambiguous-production fixture RED"): the grammar check must be a **lib test**
compiled into the root crate. A test placed in `tests/` would never run in the
required check, and a gate that does not run is not a gate. RHL-0's gates
therefore live in `src/rhl_pre_registration/`, declared `#[cfg(test)]` so that no
non-test RHL code enters the crate before RHL-1.

---

## B5 — vocabulary terms and the ontology Σ (RHL-001 §3.4)

**Not bound at RHL-0. Recorded as still `[U]`.**

§3.4 says a vocabulary term is an instance of a Σ class from `paiml-ontology.md`,
"exact binding is `[U]` until Phase 0 reads Σ at HEAD". `paiml-ontology.md` is not
in this repository:

```bash
git ls-files | grep -i 'paiml-ontology'   # no output
```

The `vocab/*.yaml` files shipped by RHL-0 therefore carry `sigma_class: "[U]"` on
every term, alongside a `kind:` drawn from §3.4's own closed list
(`noun | measure | action | unit | entity`). Binding `sigma_class` is work for
RHL-2, where the vocabulary shape enters `pv`. Inventing a Σ class name here
would be an invented threshold, which §10 makes a STOP condition.

---

## B6 — the three disabled-test directories (RHL-001 §1, `[C]`)

**The spec's claim is stale. Filed as `TESTDEBT-1`, not fixed.**

RHL-001 §1 records, citing the repo tree read 2026-09-20, that three
disabled-test directories exist at the root, and RHL-0 is asked to file a ticket
for them. At HEAD none of the three exists:

```bash
ls -d tests.disabled tests_disabled_for_mutation tests_temp_disabled_for_sprint7_mutation
#  ⇒ No such file or directory (×3)
git log --oneline --diff-filter=D -1 -- 'tests.disabled/*'
#  ⇒ 67d2e16e fix(release): package hygiene — drop quarantine dirs, colon paths, use include allowlist
```

They were deleted by `67d2e16e`. Two tracked files of the same debt class do
survive, and `TESTDEBT-1` is filed against those instead:

```bash
git ls-files | grep -i disabled
#  benches/shared_session_performance.rs.disabled
#  src/bin/ruchy-coverage.rs.disabled
#  docs/issues/PMAT-HOOK-DISABLED-CHECKS.md   (a document, not disabled code)
```

---

## B7 — operator rulings taken on RHL-001 §12

Recorded here because §12 asks for them before RHL-0 closes. Each is the
operator's own selection, taken 2026-09-20; the two §12 items he was not asked
remain unruled.

| § | Question | Ruling |
|---|---|---|
| 12.1 | Name and extension | **Keep `RHL` / `.rhl`.** The working name becomes the name. |
| 12.4 | Corpus size N and ambiguous fraction | **N = 40, a quarter ambiguous** (10 of 40) — the spec's own proposal. |
| — | Generator for F1's conflict-free report | **LALRPOP.** An independent generator's own conflict report, not a checker we wrote. Its positive control is `grammar/fixtures/ambiguous.lalrpop`. |
| 12.2 | Canonical artifact: `.rhl` is source, Rust is a build product | Not put to the operator. RHL-0 follows the spec's own **Recommended**. |
| 12.3 | Escape hatch `rust … end` | Not put to the operator. RHL-0 follows the spec's own **Recommended: no in v0**; the grammar has no such production. |
| 12.6 | First vocabularies | Not put to the operator. RHL-0 follows the spec's **proposed** `fleet` and `tickets`. |
| 12.5 | whisper.apr: own or archive | **UNRULED.** Affects RHL-11 only; recorded on that row. |
