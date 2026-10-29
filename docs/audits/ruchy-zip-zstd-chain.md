# zip / zstd advisory chain (measured 2026-10-29, branch RHLGA-G2c-host-independent-tests)

`cargo audit` (security.yml: whole-workspace Cargo.lock) reports 7 vulnerabilities:
zip 2.4.2 (RUSTSEC-2026-0367/0368/0369, fix >=9.0.0), zstd 0.13.3 (0391, fix >=0.14.3),
zstd-safe 7.2.4 (0398/0403, fix >=8.2.0), zstd-sys 2.0.16 (0410, fix >=2.1.2).

## Chains
- zstd/zstd-safe/zstd-sys: ruchy -> aprender -> aprender-core 0.65.2 -> aprender-common (non-optional `zstd`).
  `aprender-core` already uses `default-features = false`; the zstd dep lives in aprender-common's
  mandatory dependency set, so it cannot be turned off from ruchy. Required by the default build.
- zip: aprender-train (feature `training`, optional) and forjar (feature `infra`, optional). Not in the
  default build, but Cargo.lock holds them and cargo audit scans the lock.
  forjar also pulls zstd 0.13 and bashrs.

## Published versions (crates.io index, 2026-10-29)
Every published aprender-common / aprender-core / aprender-train (<=0.71.4) pins zstd ^0.13 and zip ^2;
forjar 1.33.0 pins zip ^2 and zstd ^0.13. Source 0.72.1 of aprender-common/aprender-train (zstd 0.14, zip 9)
exists in the local registry cache, but the 0.72 facade and train crates do not resolve from crates.io
(`aprender = "0.72"` fails: only <=0.71.4 candidates). So no in-repo bump clears the findings.

## Options
1. Wait for aprender 0.72.x full set + forjar release on zip 9 / zstd 0.14, then bump the Cargo.toml
   requirements (aprender, aprender-core, aprender-compute, aprender-train, aprender-viz, aprender-simulate -> 0.72;
   forjar). Effort: small (one PR + lock refresh), blocked on upstream.
2. Drop optional `training`/`infra` deps: clears zip only (not zstd, which is mandatory via aprender-core). Breaks features.
3. `[patch.crates-io]` zstd/zip: not semver-compatible (0.13 -> 0.14, 2 -> 9); needs forking. Not recommended.
4. Ignore entries: banned.

Recommendation: option 1; ask the aprender owner to publish 0.72.x (compute/core/common/train/viz/simulate + facade) and forjar.
