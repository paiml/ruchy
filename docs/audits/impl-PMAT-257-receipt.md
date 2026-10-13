# PMAT-257 — implementation receipt

## Identity

| | |
|---|---|
| ticket | `PMAT-257` (kind=code, `pmat work`), issue #257, epic #238 (RHLGA-1) |
| branch | `PMAT-snapshot-evidence`, off `main` at `f9ff1749` |
| PR | #258 |
| judged commits | `0e2d5b2a` (script + snapshot + roadmap row), `bf482149` (git_sha meaning, comment and criterion only) |
| author model | claude-opus-5-5 |

## Verdict

**DONE** — every acceptance criterion below was measured. Nothing was skipped.

## The defect, measured on main

`pv lint contracts --gate shapes` (pv 0.71.1 e89fb30c4) on a clean `f9ff1749` checkout, exit 1:

```
"gate": "shapes",
"verdict": "Fail",
"rule_id": "PV-ONT-012",
"message": "extractor refused evidence/binary/snapshot.jsonl: a [workspace] root with 1 bin target(s) read zero snapshot lines (absent) — the extractor measured nothing",
reject: lint failed (0/1 armed gates passed)
```

## Acceptance criteria

1. **One `binary-snapshot/v1` line per bin target, written by the script.** Run on t2build from a
   fresh `git clone` checked out at `f9ff1749`, with only `scripts/binary_snapshot.sh` added
   (Rust builds stay off lambda-labs). Verbatim log:

   ```
   f9ff17497 [PMAT-254] README derived + enforced by pv contract and SHACL shape (#256)
   ?? scripts/binary_snapshot.sh
   == snapshot
       Finished `release` profile [optimized] target(s) in 2m 02s
     snap  ruchy/ruchy
   snapshot: 1 bin target(s) -> evidence/binary/snapshot.jsonl (HEAD f9ff17497)
   snap rc=0
   ```

   `sha256sum evidence/binary/snapshot.jsonl` =
   `8e3aa40b392bd480c312702ab98dbd37aa875d12d2a8a61b709fcf9e285c6f13`. That is byte-identical to the
   file an earlier, independent run of the same script wrote. Its `git_sha` is `f9ff1749`, the tree
   it measured, which is the parent of the commit that adds it (see the script header).

2. **`--twice` is deterministic.** Same t2build checkout, verbatim:

   ```
   == twice
     snap  ruchy/ruchy
     snap  ruchy/ruchy
   DETERMINISTIC: two snapshots byte-identical (1 lines).
   twice rc=0
   ```

   The check can fail. A planted control changed line 168 to `cmp -s "$work/a" "$work/a"`, which
   compares a snapshot with itself, and a quorum round drew 3/3 FAIL, each citing
   `scripts/binary_snapshot.sh:167-168` (`docs/audits/quorum-PMAT-257-control.json`).

3. **`bashrs lint scripts/binary_snapshot.sh`** (bashrs 7.4.2), at `bf482149`:

   ```
   Summary: 0 error(s), 34 warning(s), 29 info(s)
   ```

4. **`pv lint contracts --gate shapes` passes on the branch.** At `0e2d5b2a` and again at
   `bf482149`, exit 0:

   ```
   "verdict": "Pass",
   "binary": 1,
   "findings": []
   ```

## Quorum rounds

- control (planted, never pushed): 3/3 FAIL on the planted line, so the lanes discriminate.
- round 1 at `0e2d5b2a`: argon PASS, sonnet PASS, haiku FAIL. Haiku's finding was that `git_sha`
  f9ff1749 ≠ the commit 0e2d5b2a. A file cannot name the commit that contains it, so `bf482149`
  states the meaning in the script header and in the criterion
  (`docs/audits/quorum-PMAT-257-r1.json`).
- round 2 at `bf482149`: argon PASS, sonnet PASS, haiku FAIL. Haiku's three findings were all
  `asserted`: no receipt showed the measurements. This receipt is the answer
  (`docs/audits/quorum-PMAT-257-r2.json`).

Lanes in every round: `gemini-4-argon-high` (agy), `claude-sonnet-5-5`, `claude-haiku-4-5`. Each
round ran under a scratch `PAIML_IMPLEMENT_CONFIG` copy with those lane models and
`agy_tier1_only=false`, because the shared config still names `gemini-3.1-pro-high`, which agy no
longer serves. The shared config was not edited.
