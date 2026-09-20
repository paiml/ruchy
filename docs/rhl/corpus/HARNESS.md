# RHL-0 corpus harness — the fixed interface contract

Spec: `docs/specifications/ruchy-high-level-language-interface.md` (RHL-001), falsifier
**F8** (§7) — the headline hypothesis. F8 is a pre-registered A/B on the SAME task
corpus with the SAME model:

- **path A** — English → the model writes Rust directly.
- **path B** — English → the model writes RHL → the compiler produces the Rust.

The claim (RHL is more robust than English) holds only if B beats A. For that to be a
real measurement, both arms must be scored by the SAME pre-written tests, and neither
arm may be advantaged by the harness itself. A quorum of three independent reviewers
unanimously found that a corpus without a fixed interface contract makes that
comparison impossible — without one, each arm's tests could silently assume a
different shape for "the world the task reads" and "the effects the task produced,"
and the comparison would measure the harness, not the language. This document is
that contract, and it is written and committed **before** any compiler exists, so
that neither arm's success can later be tuned to fit an implementation. See
`docs/rhl/phase0-bindings.md` binding B7 for the operator's ruling that fixed the
corpus at N = 40 tasks, exactly 10 of them ambiguous.

## The entrypoint every task implements

```rust
pub fn run(ctx: &mut TaskCtx) -> Result<Report, TaskError>;
```

Both arms produce a function with exactly this signature, named `run`. A task's
`tests.rs` calls nothing else to exercise a solution.

## `TaskCtx` — the mocked world a task reads

```rust
pub struct TaskCtx { /* private fields; read only through the accessors below */ }

pub struct HostRecord {
    pub name: String,
    pub cpu_pct: f64,
    pub online: bool,
}

pub struct RepoRecord {
    pub name: String,           // "owner/repo"
    pub open_prs: u32,
    pub default_branch: String,
}

impl TaskCtx {
    pub fn new(now_unix: u64) -> Self;
    pub fn with_disk_free(self, path: impl Into<String>, bytes: u64) -> Self;
    pub fn with_host(self, host: HostRecord) -> Self;
    pub fn with_repo(self, repo: RepoRecord) -> Self;

    // Accessors. Each call appends one entry to the read log below, so a test
    // (or a future auditor comparing the two arms) can see exactly what a
    // task looked at, and nothing more.
    pub fn disk_free(&mut self, path: &str) -> Option<u64>;
    pub fn hosts(&mut self) -> &[HostRecord];
    pub fn host(&mut self, name: &str) -> Option<&HostRecord>;
    pub fn repos(&mut self) -> &[RepoRecord];
    pub fn repo(&mut self, name: &str) -> Option<&RepoRecord>;
    pub fn now_unix(&mut self) -> u64;

    /// The audit trail: one string per accessor call above, in call order,
    /// e.g. `"disk_free(/var/lib/docker)"`, `"hosts()"`, `"now_unix()"`.
    pub fn reads(&self) -> &[String];
}
```

A test declares the mocked world once, via the `with_*` builders, before calling
`run`. `run` may only read that world back through the accessors above — disk free
per path, declared hosts, declared repos, the clock — and every one of those reads
is recorded. Nothing under `TaskCtx` is mutated by a task except its own read log.

## `Report` — the effects a task produced

```rust
pub struct Report {
    pub tickets: Vec<TicketDraft>,
    pub values: std::collections::BTreeMap<String, String>,
}

pub struct TicketDraft {
    pub repo: String,
    pub title: String,
    pub labels: Vec<String>,
}
```

Effects are DATA, never real side effects. A task that "files a ticket" pushes a
`TicketDraft` onto `report.tickets`; it never calls a real issue tracker, and a task
that reports a scalar writes it into `report.values`. This is what lets a test
assert on behavior without either arm performing real I/O, and it is what makes a
Rust-direct solution and an RHL-lowered solution comparable on the same axis: a
`Report` is the same shape regardless of which arm produced it.

## `TaskError` — the refusal shape

```rust
pub enum TaskError {
    /// The English genuinely underdetermines the task; `reason` names which
    /// reading is unresolved.
    Ambiguous { reason: String },
    /// `ctx` does not satisfy a precondition the task stated (e.g. a named
    /// host is absent).
    Refused { reason: String },
}
```

F8's positive control (RHL-001 §7) requires path B to refuse or clarify on every
task the corpus marks ambiguous, 100% of the time. `TaskError::Ambiguous` is that
refusal, and it is exactly what an ambiguous task's `tests.rs` asserts `run`
returns — never a guess at one of the two readings.

## The closed reference rule

A task's `tests.rs` may reference only `run`, `TaskCtx`, `HostRecord`, `RepoRecord`,
`Report`, `TicketDraft`, `TaskError`, and ordinary Rust/`std`. It must never name
anything arm-specific — nothing that would exist in a Rust-direct solution but not
an RHL-lowered one, or vice versa. Concretely, the identifiers `Transpiler`,
`Parser`, `rhl`, and `ruchy` must never appear in a `tests.rs`: naming any of them
would let one arm's own machinery leak into the shared harness and advantage that
arm before either has written a line. `src/rhl_pre_registration/corpus_gate.rs`
enforces this by direct substring search over every committed `tests.rs`.

## Nothing here compiles yet

These types are DECLARED by this document, not implemented anywhere in this
repository. They are implemented at row RHL-8 (`docs/specifications/ruchy-high-level-language-interface.md`
§8), when the A/B measurement itself is built. Until then, every task's `tests.rs`
is expected to FAIL TO COMPILE — `corpus_gate.rs` checks only that each one PARSES
as syntactically valid Rust (`syn::parse_file`) and contains at least one `#[test]`
function. That is what proves the corpus is not vacuous (a directory of empty or
garbled files would satisfy a file-count check and measure nothing) without
requiring the compiler to exist first.

## Per-task files

Each `docs/rhl/corpus/NN-slug/` (`NN` = `01`..`40`, no gaps, no duplicates) holds
exactly three files:

- `task.yaml` — `id`, `title`, `ambiguous: true|false`, `entrypoint: "run"`, and,
  when `ambiguous: true`, `ambiguity:` naming exactly which two readings of the
  English are both consistent with what was said.
- `intent.en` — the English statement, one short paragraph, phrased the way the
  fleet operator would actually say it.
- `tests.rs` — the pre-written Rust tests against the contract above.

## Domain

The fleet's own small jobs and checks — disk watch, runner inventory, pin check,
stale-PR sweep, and the like (RHL-001 §8). Ground truth already exists for these,
so every task that is later solved correctly is dogfood, and every task is small
enough that a correct answer is one short job.

## Do not write a solution here

RHL-0 pre-registers the TASKS, not the solutions. No `.rhl` program and no
Rust implementation of `run` is committed alongside a task; either would
contaminate the A/B this corpus exists to make fair.
