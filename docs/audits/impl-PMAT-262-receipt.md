# PMAT-262 implementation receipt

Measured on t2build (cargo 1.99.0), 2026-10-14 00:19-00:30Z. Logs: `t2build:~/build/i38-262-run1.log`, `i38-262-run2.log`, `i38-262-run3.log`.
Two branches carry this ticket: `PMAT-262-libyaml-dev-paths` (from main f9ff1749, item 1) and
`PMAT-262-null-rows-on-261` (stacked on #261 head 89f6e0a3, items 1 and 2), because `src/yaml_emit.rs` exists only on #261.

## 1. provable-contracts 0.3.1 -> 0.4

`cargo update -p provable-contracts@0.3.1` after the Cargo.toml change:

```
      Adding aprender-contracts v0.64.0
      Adding aprender-contracts-macros v0.64.0
    Updating provable-contracts v0.3.1 -> v0.4.0
    Removing provable-contracts-macros v0.3.1
```

`cargo test --test contract_traits`: `test result: ok. 9 passed; 0 failed` (main + bump, and #261 + bump).

The bump does not remove the advisory path. provable-contracts 0.4.0's `src/lib.rs` is `pub use upstream::*;` with
`upstream = { package = "aprender-contracts", version = "0.64.0" }`; the crates.io index lists serde_yaml `^0.9` as kind
`normal` and kind `build` for aprender-contracts 0.70.2, 0.70.3 and 0.71.1 (and the registry sources of 0.61.0 to 0.69.1).
`cargo tree -i unsafe-libyaml -e normal,build,dev --target all` on #261 + this ticket:

```
unsafe-libyaml v0.2.11
├── serde_yaml v0.9.34+deprecated
│   └── aprender-contracts v0.64.0
│       └── provable-contracts v0.4.0
│           [dev-dependencies]
│           └── ruchy v5.0.0-beta.2
│   [build-dependencies]
│   └── aprender-contracts v0.64.0 (*)
└── serde_yaml_ng v0.10.0
    └── jugar-probar v1.0.4
        [dev-dependencies]
        └── ruchy v5.0.0-beta.2
    [build-dependencies]
    ├── aprender-compute v0.65.2
    │   ├── aprender-core v0.65.2
    │   │   ├── aprender v0.65.2
    │   │   │   └── ruchy v5.0.0-beta.2
    │   │   └── ruchy v5.0.0-beta.2
    │   └── ruchy v5.0.0-beta.2
    └── aprender-core v0.65.2 (*)
    [dev-dependencies]
    └── ruchy v5.0.0-beta.2
```

## 2. Null rows (stacked branch only)

Mutants applied to line 138 of `src/yaml_emit.rs` (`|| matches!(v, "null" | "Null" | "NULL" | "~")`), test
`yaml_io::yaml_io_tests::test_sec2_writer_is_byte_identical_to_the_oracle`:

| tree | mutant | result |
|---|---|---|
| #261 as-is (no new rows) | drop `"null"` | `ok. 7 passed; 0 failed` (all yaml_io tests): survives |
| #261 + this ticket | none | `ok. 7 passed; 0 failed` |
| #261 + this ticket | drop `"null"` | `FAILED`, panicked at src/yaml_io_tests.rs:221:9 |
| #261 + this ticket | drop `"Null"` | `FAILED`, panicked at src/yaml_io_tests.rs:221:9 |
| #261 + this ticket | drop `"NULL"` | `FAILED`, panicked at src/yaml_io_tests.rs:221:9 |

## 3. Side effect on the stacked branch's lock

`cargo update -p provable-contracts@0.3.1` on #261's lock also re-points 13 `cfg(windows)` edges back to
`windows-sys 0.61.2` (9 from 0.59.0, 4 from 0.60.2). No package version changes. main's lock has the 0.61.2 layout, and a
plain re-resolve of #261's Cargo.toml from its parent's lock gives the same layout (13 edges differ from #261's lock).
The main-based branch's lock diff touches no windows-sys edge.

No advisory ignore, audit.toml, deny.toml or advisory-ignores.yaml change.
