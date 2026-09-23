# Q4 lowering probe (RHLGA-1)

Binary: `ruchy` built from `270a065c` (debug). 4.2.1 = `~/.cargo/bin/ruchy`.

| Command | Result |
|---|---|
| `ruchy transpile lower.ruchy -o lower.rs && rustc --edition 2021 lower.rs -o lower && ./lower` | exit 0, prints `1 0` |
| `ruchy run lower.ruchy` | prints nothing, exit 0 (RUNMAIN-1) |
| `ruchy run m5.ruchy` (HEAD and 4.2.1) | prints nothing, exit 0 — expected `y` |
| `ruchy run m10.ruchy` (m5 plus an explicit `main()` line) | prints `y` |
