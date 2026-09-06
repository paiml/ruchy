---
name: ruchy-dogfood
description: Dogfood the ruchy compiler before a release — CLI surface from the built binary, the book/examples corpus through it, wasm32 and a fresh-container install; three read-only worker lanes, one receipt writer.
allowed-tools: Agent, Bash(cargo:*), Bash(git:*), Bash(gh:*), Bash(pmat:*), Bash(docker:*), Bash(bash:*), Bash(jq:*), Bash(find:*), Bash(grep:*), Bash(head:*), Bash(tail:*), Bash(wc:*), Bash(diff:*), Bash(sha256sum:*), Read, Write
---

# ruchy-dogfood — the pre-release protocol for the ruchy compiler

The gate between "CI is green" and a ruchy release. It asks four questions the
test suite does not: does the shipped binary's *verb surface* work, does the
*corpus people actually read* still run through it, does wasm32 still build, and
does a stranger on a clean machine get a working `ruchy`.

**Toyota way, non-negotiable:** any RED gate STOPS the release. Fix the root
cause in this crate. Never `--no-verify`, never `--skip`, never lower a floor to
make a row go green.

The frontmatter carries an explicit `name: ruchy-dogfood`. It has to: a skill
directory named `dogfood` is shadowed by the user-scope `~/.claude/skills/dogfood`
(the fleet's crate-generic protocol), and a shadowed skill is a protocol nobody
runs. This one is ruchy-specific and lives in ruchy's tree, in git.

## Every gate has a SUBJECT

Read this before adding a gate, and before calling a permanently-red one debt.

| gate | subject | when it can pass |
|---|---|---|
| lane 1 surface, lane 2 corpus | **this tree's built binary** | now |
| `scripts/wasm32-build.sh` | **this tree** | now |
| lane 3 fresh-container install | the **published crate**, on a clean host | only AFTER publish |
| the declared gates in `Cargo.toml` | **this tree** | now |

A gate whose subject is a later phase is not broken and is not debt. Before the
version in `Cargo.toml` exists on crates.io, the container lane records
`not-yet-measured` **with the reason**, and never `PASS`. Treating that RED as
debt is how it becomes a step everyone learns to walk past; recording it as PASS
is a lie about a thing that was never measured.

## Status vocabulary and exits

Rows are `PASS` / `FAIL` / `SKIP` / `REPORT` / `not-yet-measured`. A `SKIP` must
record the enumeration that found nothing ("0 files from `find examples -name
'*.ruchy'`") so *no subject* can be told apart from *did not look*. A `REPORT` is
a deliberate non-gating measurement and must carry the number **and** the ticket
it waits on. Do not add `WARN`: a WARN in a release protocol is a step everybody
learns to walk past.

Exits: **0 = GO**, **1 = NO-GO**, **2 = harness failure** (the protocol could not
run: a lane crashed, a required tool is missing, a lane file never appeared),
**3 = the receipt just written is unreadable** (verdict withheld — a verdict
nobody can re-read is not evidence).

**GO iff** every lane is PASS, every declared gate is PASS, and the receipt reads
back. Anything else is NO-GO.

## Shape of a run

The **orchestrator** is the session running this skill (Fable). It spawns at
**depth 1 only** — workers never spawn workers — and at most **three** at a time:

```bash
export CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS=3
```

Three lanes run concurrently. Each is a `paiml-impl-worker`-style **read-only**
worker: it may read anything in the tree and run the built binary, and it may
write **exactly one file — its own**. Each lane's Agent `description` must start
with `<ticket>/ph<i>` (e.g. `PMAT-136/ph1 surface audit`); `receipt-lint.sh`
refuses a receipt whose lane descriptions do not, because a lane that cannot be
attributed to a ticket and a phase cannot be re-run by anyone but its author.

The orchestrator is the **single receipt writer**. Workers never write the
receipt, never commit, never push, never open a PR, never publish.

### Lane 1 — surface + coverage (Sonnet, ≤40 turns)

Scope: `docs/audits/surface_audit.csv`, `out/lane-1.json`.

Enumerate the verb tree **from the built binary**, never from `src/` and never
from the docs: `ruchy --help`, then `--help` for every subcommand it lists,
recursively to a fixed point. A surface read out of the source is a claim about
the code; a surface read out of the binary is a measurement of the release.

Write `docs/audits/surface_audit.csv` with the header
`verb,has_help,smoke_result,covered_by_corpus` — one row per verb, where
`has_help` records that `--help` exited 0 and printed a usage line,
`smoke_result` is the verb's minimal invocation (`PASS` / `FAIL` /
`REPORT <reason>`), and `covered_by_corpus` is `yes` / `no`: whether lane 2's
file set exercises that verb. Then `out/lane-1.json`:

```json
{"lane":1,"verdict":"PASS","verbs":42,"undocumented":[],"uncovered":["wasm"],"rows":[]}
```

A verb with no `--help` is FAIL. A verb the corpus never touches is not FAIL —
it is the `uncovered` list, and it is this lane's most useful output.

### Lane 2 — exercise the covered set (Sonnet)

Scope: `out/lane-2.json`.

Run every `.ruchy` file under `examples/` and under each sibling corpus that
exists — `../ruchy-book`, `../ruchy-cookbook`, `../rosetta-ruchy`,
`../ruchy-repl-demos` — through the **built** binary, each with `timeout 10`
(the hang guard from CLAUDE.md's Ruchy Execution Safety Protocol; a runaway
`ruchy` has cost this project a whole session before).

A **missing corpus is recorded, never silently skipped**: it lands in
`missing_corpora` with the path that was looked for. `scripts/book-corpus.sh`
already performs exactly this enumeration and prints
`files=N pass=N fail=N missing_corpora=[…]`; the lane uses it and adds the
per-file detail.

```json
{"lane":2,"verdict":"PASS","missing_corpora":[],
 "files":[{"path":"examples/01_hello.ruchy","verb":"check","exit":0,
           "stdout_sha256":"…","seconds":0.04}]}
```

Failures under `examples/` are gating — those files are this repo's. Failures
under a sibling corpus are `REPORT` rows naming the corpus: they are another
repo's tree, and their red is that repo's ticket, not a reason to hold this
release. Never drop them silently.

### Lane 3 — wasm32 + fresh container (Sonnet)

Scope: `out/lane-3.json`.

1. `bash scripts/wasm32-build.sh` — the same command the release workflow's WASM
   job runs: `RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --lib
   --target wasm32-unknown-unknown --no-default-features --features wasm-compile`.
   The target's absence is FAIL, never a skip.
2. The fresh-container install, whose subject is the **published artifact**:

```bash
docker run --rm rust:1.91-slim bash -lc \
  'cargo install ruchy --version <v> --locked && ruchy --version'
# then the same again without --locked
```

`<v>` is the version in `Cargo.toml`. Both forms are run: `--locked` proves the
published `Cargo.lock` resolves, unlocked proves the crate still builds against
today's registry — the two fail for different reasons, and a green `--locked` has
hidden a broken unlocked build before.

**Before publish this lane's second half cannot pass.** The version is not on
crates.io yet, so the row is

```json
{"status":"not-yet-measured",
 "reason":"ruchy <v> is not on crates.io; the container's subject is the published artifact (post-publish phase)"}
```

— never `PASS`, never `SKIP`. It is produced for real in the post-publish phase,
and the receipt from that phase is what closes it.

```json
{"lane":3,"verdict":"PASS","wasm32":{"status":"PASS","seconds":91},
 "container":{"status":"not-yet-measured","reason":"…"}}
```

## Rules the lanes obey

1. **Read-only except one file.** A worker that writes anything else — source,
   `Cargo.toml`, another lane's output, the receipt — invalidates the run.
2. **Never push.** No pushes, no PRs, no `cargo publish`, no `gh release` from a
   worker. The orchestrator performs the release, after a GO.
3. **A lane `.err` file, or a lane file that never appears, is NO-GO.** Not a
   retry, not a skip: a lane that did not report did not measure.
4. **Deterministic-tool absence is NO-GO, never a silent skip.**
   `pmat work show <ticket>` must succeed before the lanes start — the ticket
   this run belongs to has to exist. If the installed pmat has no such verb
   (measured 2026-09-06: pmat 3.38.0 exposes `work list|annotate|start|…` and no
   `show`), that is a **FAIL row naming the version**, not a skip. The remedy is
   upstream in pmat; the receipt says so out loud rather than quietly dropping
   the check.
5. **Repo gates are discovered, never copied.** `[package.metadata.dogfood] gates`
   in `Cargo.toml` is the list; `bash scripts/dogfood-gates.sh` runs it and gives
   each gate **its own row**. A declared script that does not exist is a deleted
   gate → `FAIL <script> missing`. An empty or absent list is a clean sweep over
   an empty set → `FAIL no gates declared`. Both are hard FAILs; neither is a
   SKIP. `bash scripts/dogfood-gates.sh --list` prints the list without running
   it, and `tests/dogfood_gates_declared.rs` proves `--list` and the manifest
   agree. Adding a gate is an edit to `Cargo.toml`, never an edit to this file.
6. **One receipt writer.** The orchestrator merges `out/lane-1.json`,
   `out/lane-2.json`, `out/lane-3.json` and the gate rows into
   `.dogfood/receipt-<sha>.json`, where `<sha>` is `git rev-parse HEAD` — the
   commit the receipt describes. It is written as
   `.dogfood/receipt-<sha>.json.partial` and **atomically renamed** on
   completion, so a crashed run leaves no completed receipt rather than a stale
   or truncated one. A partial run never prints GO.
7. **The receipt carries no timestamps.** It is a statement about a commit, not
   about a moment, and a timestamp would make byte-comparison useless.
8. **`--twice` runs the whole protocol twice and diffs the two receipts.** Any
   byte difference is NO-GO: a protocol whose verdict moves while the tree does
   not is measuring something other than the tree. Print the diff.
9. `.dogfood/` is created by this protocol and is in `.gitignore`. Receipts are
   not committed to the tree; they are attached to the release.

## Run it

```bash
export CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS=3
pmat work show <TICKET>                 # the ticket must exist (rule 4)
bash scripts/dogfood-gates.sh --list    # what will run
bash scripts/dogfood-gates.sh           # the declared gates, one row each
# then the three lanes, concurrently, then the receipt
```

Read the receipt back with `jq . .dogfood/receipt-<sha>.json` before printing any
verdict. If it does not parse, exit 3 and withhold the verdict.

## On GO

1. Tag the commit the receipt names (`v<version>`) and push that tag, or run the
   repo's release workflow.
2. Publish, then re-run **lane 3 only** — its container half now has a subject
   that exists — and keep that second receipt.
3. Create the GitHub release from the tag and attach the receipt as
   `dogfood-receipt-<sha>.json`. A release with no receipt attached has no
   evidence, and evidence nobody can find is evidence nobody checked.

## On NO-GO

Stop. Print every red row with its note *before* the verdict — a verdict that
says only "a gate failed" makes the reader hunt for it, and the hunt is where
bypasses start. File the failing gate as a pmat ticket, naming the receipt path
and the row; fix the root cause with five-whys; commit; re-run this protocol.
Never release on a NO-GO.
