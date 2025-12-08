Execute one Apex Hunt PDCA cycle. YOU MUST use pmat work commands:

```bash
pmat work start "Apex Hunt: PDCA Cycle - [Defect ID]"
```

PLAN: Scan `tests/transpiler_defect_*_RED.rs`, create `/tmp/repro.ruchy`, verify RED state
DO: Fix transpiler code (use `.to_string()` for &str→String)
CHECK: `ruchy transpile` → `rustc --edition 2021` → execute, `cargo test --lib`
ACT: git commit, then:

```bash
pmat work complete "Apex Hunt: PDCA Cycle - [Defect ID]"
```

GO. Find and fix the next RED transpiler defect.
