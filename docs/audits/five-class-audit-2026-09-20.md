# Five-class audit pass — ruchy, 2026-09-20

Standing rule: when no `done_when` in the spec is false, run the audit pass and
mint a row per finding with a falsifier. **A session that finds nothing reports
the five probes it ran and their zero counts** — so the zeros are here too.

Tree: `e382f9ab` (main, after #228).

| # | Class | Probe | Result |
|---|---|---|---|
| 1 | computed-but-unexposed | 1896 public struct fields in `src/`; grep each for any `.field` read; split by whether the struct derives `Serialize` | **81** never read: **56 dead** (no `Serialize`) — minted as `AUDIT-3`; 25 serde-read — not defects |
| 2 | placeholder-is-most-permissive | workflow steps whose stricter variant is disabled; `unwrap_or` on a permissive value in a gate path | **2** — both minted |
| 3 | gate-that-cannot-fail | `\|\| true`, `continue-on-error: true`, `\|\| echo` across `.github/workflows/` — **workflows only** | **4 matches, 1 real** — minted as `AUDIT-1`. The Makefile was NOT in this probe; scanned separately after review: **34**, none in any CI path, 5 on developer gates — minted as `AUDIT-4` |
| 4 | declared-never-graded | every `test:` in `contracts/*.yaml` resolved against every `fn` declared in `src/` and `tests/` | **0 of 30 contracts dangle** |
| 5 | host ≠ runner | host-specific absolute paths in workflows and the Makefile | **0 in workflows**, 3 in Makefile |

## Minted

**`AUDIT-1` (high) — class 3 and class 2 in one line.** `security.yml` has two
audit steps. Line 24 `cargo audit` can fail, and did today on
`RUSTSEC-2026-0285`. Line 26, named *"Check for known vulnerabilities"*, is
`cargo audit --deny warnings || true`: it runs every time, and its exit status
**cannot** fail. It is the **stricter** of the two — `--deny warnings` escalates
unmaintained/unsound/yanked to errors, exactly the class ruchy carries two live
exemptions for. So the stricter audit step is the one that cannot fail. Line 24
still gates vulnerabilities; what is masked is warning escalation, not the audit.

> *Corrected after review.* This paragraph first said the step was "named as the
> thorough check" (it is not — that was my gloss, not its name) and called it
> "disabled" (it is not — it runs; its verdict is swallowed). The roadmap row's
> wording was the precise one and is now mirrored here.

**`AUDIT-2` — class 3, test only.** `src/quality/mod.rs:860`:
`count_satd_comments().unwrap_or(0)` then `assert_eq!(count, 0)`. The counter
shells out to `find`; any failure becomes `0` and the assertion passes. The test
cannot distinguish *"there is no SATD"* from *"the counter did not run"*, and `0`
is the more permissive reading. The production gate at line 300 uses `?` and is
unaffected.

## Reported, not minted

**Class 1 — corrected after review: 81, not 11, and 56 of them are dead.**
The first version of this report said *11* and refused to mint them under a
blanket "several are serde-read" exemption. Both review lanes MEASURED that the
two structs I named as examples — `HanseiReport` and `CacheStats` — derive no
`Serialize` at all (`Debug` and `Debug, Clone` respectively), so the exemption
was dodging. And the 11 was not a count: the probe was capped at `hits[:400]`
of 1896 fields. **A truncation reported as a count — the same defect as the
"17 of 17" on CIBASE-1, found the same day.**

Re-run over all 1896 and split by whether the struct actually derives
`Serialize`: **81 never read; 56 on non-serde structs (dead computations); 25 on
serde structs (read by name, not a defect).** The 56 are minted as `AUDIT-3`,
one row for the class with the full list as its content rather than 56 rows.
Many sit under `src/notebook/testing/` and may be scaffolding; the row asks for a
per-struct decision, expose or delete.

**Class 5 — 3 hardcoded `/home/noah/.nvm/versions/node/v22.13.1/bin` paths** in
`Makefile` (1839, 1843, 1862). Not in any CI path, so no runner is affected; they
make those targets work on exactly one machine with exactly one node version. Low
severity, and worth folding into whatever owns Makefile portability rather than
its own row.

## The instance this pass did not have to find

`#228` landed hours earlier and is the cleanest class-3 instance of the day:
`cargo criterion ... || true` reported **success** on a job that ran **zero
benchmarks** for two weeks and ~50 runs, while `main` accumulated 306 green runs
on it. It is not in the table above because it was already fixed — but it is the
reference shape, and its number is the one to quote: the job now fails in
**10m51s**, which is the true state.

## Probe limitations, stated

Probes 1 and 2 are greps over source, not dataflow: probe 1 now splits by the
struct's derives but still cannot see a field read through a macro or a
`Deref`; probe 2 can only see `unwrap_or` shapes it has patterns for. Probe 4 is
exact — it resolves names against declarations. Probe 3 scanned
`.github/workflows/` **only** — an earlier version of this paragraph said it
covered the Makefile too, which it did not, and the review caught the
inconsistency against the table; the Makefile has since been scanned on its own
(`AUDIT-4`). Probe 5 scanned workflows and the Makefile. In all cases a swallowed
failure inside a script those files *call* is invisible to this pass — the
review checked `scripts/release-policy.sh` and found its one `|| true` is a
documented best-effort fallback, not a swallowed gate.
