# Five-class audit pass — ruchy, 2026-09-20

Standing rule: when no `done_when` in the spec is false, run the audit pass and
mint a row per finding with a falsifier. **A session that finds nothing reports
the five probes it ran and their zero counts** — so the zeros are here too.

Tree: `e382f9ab` (main, after #228).

| # | Class | Probe | Result |
|---|---|---|---|
| 1 | computed-but-unexposed | 1896 public struct fields in `src/`; grep each for any `.field` read | **11** never read — reported, not minted (see below) |
| 2 | placeholder-is-most-permissive | workflow steps whose stricter variant is disabled; `unwrap_or` on a permissive value in a gate path | **2** — both minted |
| 3 | gate-that-cannot-fail | `\|\| true`, `continue-on-error: true`, `\|\| echo` across `.github/workflows/` | **4 matches, 1 real** — minted |
| 4 | declared-never-graded | every `test:` in `contracts/*.yaml` resolved against every `fn` declared in `src/` and `tests/` | **0 of 30 contracts dangle** |
| 5 | host ≠ runner | host-specific absolute paths in workflows and the Makefile | **0 in workflows**, 3 in Makefile |

## Minted

**`AUDIT-1` (high) — class 3 and class 2 in one line.** `security.yml` has two
audit steps. Line 24 `cargo audit` can fail, and did today on
`RUSTSEC-2026-0285`. Line 26, named *"Check for known vulnerabilities"*, is
`cargo audit --deny warnings || true` — it **cannot** fail, and it is the
**stricter** of the two: `--deny warnings` escalates unmaintained/unsound/yanked
to errors, which is exactly the class ruchy carries two live exemptions for. The
step named as the thorough check is the disabled one.

**`AUDIT-2` — class 3, test only.** `src/quality/mod.rs:860`:
`count_satd_comments().unwrap_or(0)` then `assert_eq!(count, 0)`. The counter
shells out to `find`; any failure becomes `0` and the assertion passes. The test
cannot distinguish *"there is no SATD"* from *"the counter did not run"*, and `0`
is the more permissive reading. The production gate at line 300 uses `?` and is
unaffected.

## Reported, not minted

**Class 1 — 11 public fields never read via `.field` anywhere in `src/`**:
`CacheKey.content_hash`, `CacheStats.memory_usage_estimate`,
`AntiGamingRules.max_test_ratio`, `LintIssue.issue_type`,
`HanseiReport.overall_accuracy`, `HanseiReport.fix_rate`, and five more. Not
minted as findings because the probe cannot distinguish a genuinely dropped
computation from a field that exists to be serialised — several of these are on
report structs whose consumer is `serde`, which reads them by name and not by
`.field`. **The probe's own limitation is the finding here**: a sharper version
would resolve serde usage before counting. Recorded so the next pass starts from
a known-imprecise number rather than re-deriving it.

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

Probes 1 and 2 are greps over source, not dataflow: probe 1 over-reports
serde-read fields, and probe 2 can only see `unwrap_or` shapes it has patterns
for. Probe 4 is exact — it resolves names against declarations. Probes 3 and 5
are exact over the file set they scan, which is `.github/workflows/` and
`Makefile` only; a swallowed failure inside a script those files *call* is
invisible to this pass.
