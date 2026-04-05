# Tier Baselines — paiml Corpus Repos (Appendix B)

Captured by `ruchy tier <repo> --baseline .ruchy-tier-baselines/<repo>.json`
per §Appendix B of `docs/specifications/ruchy-5.0-sovereign-platform.md`.

These baselines are consumed by CI to detect tier regressions:

```bash
ruchy tier ../ruchy-book --baseline .ruchy-tier-baselines/ruchy-book.json
# ⇒ exits 1 if any metric regresses vs the baseline
```

## Baseline Snapshot (2026-04-05, ruchy 5.0.0-beta.1)

| Repo | Files | LoC | Functions | Bronze | Silver+ | pub Bronze | parse_errors |
|------|------:|----:|----------:|-------:|--------:|-----------:|-------------:|
| ruchy-book | 152 | 3,612 | 216 | 216 | 0 | 0 | 2 |
| ruchy-cookbook | 45 | 8,466 | 399 | 399 | 0 | **80** | 10 |
| ruchy-cli-tools-book | 12 | 1,290 | 67 | 67 | 0 | 0 | 0 |
| tooling-with-ruchy | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ruchy-repl-demos | 76 | 7,199 | 161 | 161 | 0 | 0 | 4 |
| rosetta-ruchy | 198 | 53,110 | 1,689 | 1,689 | 0 | 0 | 29 |
| ruchyruchy | 538 | 172,164 | 3,876 | 3,876 | 0 | 0 | 114 |
| **TOTAL** | **1,021** | **245,841** | **6,408** | **6,408** | **0** | **80** | **159** |

## §14.5 Takeaway

**100% of the 6,408 Ruchy functions in the paiml corpus are Bronze tier.**

Zero have `requires`/`ensures` clauses. The §14.5 falsifier metrics are:
- **F1 non-trivial %**: N/A (no contracts to evaluate)
- **F2 contract_exempt/KLoC**: 0.00 (no escape hatches yet)
- **F4 pub Bronze**: 80 in ruchy-cookbook, 0 elsewhere
- **F11 diff_exempt/KLoC**: 0.00

Per §14.6 deadline schedule:
- **5.0.0** (now): Silver opt-in, Bronze allowed everywhere. ✅ Current state.
- **5.1.0**: Silver default, Bronze warned in user code.
- **5.2.0**: Silver *required* on all `pub fns`. **Current 0% coverage across 6,408 functions is the migration gap.**

## Parse Errors by Repo

Parse failures flag pre-existing parser gaps against language features
in the corpus (not 5.0 regressions). Totals:
- ruchyruchy: 114 (largest corpus)
- rosetta-ruchy: 29
- ruchy-cookbook: 10
- ruchy-repl-demos: 4
- ruchy-book: 2
- Others: 0

Zero parse timeouts across 1,021 files after `PARSER-ACTOR-HANG` fix
(prior to fix, actor-containing files hung indefinitely).

## Regenerating Baselines

```bash
for d in ruchy-book ruchy-cookbook ruchy-cli-tools-book tooling-with-ruchy \
         ruchy-repl-demos rosetta-ruchy ruchyruchy; do
  rm -f ".ruchy-tier-baselines/${d}.json"
  ruchy tier "../$d" --baseline ".ruchy-tier-baselines/${d}.json"
done
```

Re-run whenever a sibling repo adds contracts / migrates to Silver+.
Each file is ≈325 bytes — safe to track in git.
