#!/usr/bin/env bash
# PMAT-254 mutation proof: each plant must turn a gate RED; restore is trapped first.
set -u
WT=/mnt/nvme-raid0/tmp/ruchy-readme
export CARGO_TARGET_DIR=/mnt/nvme-raid0/targets/ruchy-readme
cd "$WT" || exit 9
restore() { git -C "$WT" checkout -- README.md examples/readme contracts/ruchy-readme-claims.json; }
trap restore EXIT INT TERM
LOG=/mnt/nvme-raid0/tmp/ruchy-readme-evidence/mutate
mkdir -p "$LOG"
# Test executables are built once (cargo test --no-run): the README is read at run time, and
# build.rs names a rerun-if-changed file absent here, so cargo would relink per plant.
LIBX=$(awk '$1=="lib"{print $2}' /mnt/nvme-raid0/tmp/ruchy-readme-evidence/test-exes.txt)
BINX=$(awk '$1=="test"{print $2}' /mnt/nvme-raid0/tmp/ruchy-readme-evidence/test-exes.txt)
[ -x "$LIBX" ] && [ -x "$BINX" ] || { echo "test exes missing" >&2; exit 8; }
bless() { RUCHY_README_BLESS=1 "$LIBX" readme_contract::test_pmat_254_claims_json_is_fresh >/dev/null 2>&1; }
lib()  { "$LIBX" readme_contract >"$LOG/$1.lib" 2>&1; echo $?; }
bin()  { "$BINX" >"$LOG/$1.bin" 2>&1; echo $?; }
pvg()  { pv lint contracts --gate shapes >"$LOG/$1.pv" 2>&1; echo $?; }
sub()  { python3 -c "import sys;p,a,b=sys.argv[1:];s=open(p).read();assert a in s,(p,a);open(p,'w').write(s.replace(a,b,1))" "$@"; }
row()  { printf '%-4s %-44s lib=%s bin=%s pv=%s\n' "$1" "$2" "$3" "$4" "$5"; }

row M0 "baseline (no plant)" "$(lib M0)" "$(bin M0)" "$(pvg M0)"
sub README.md '**1.91**' '**1.89**'; row M1a "wrong MSRV number, claims NOT re-blessed" "$(lib M1a)" - -; bless
row M1b "wrong MSRV number, claims re-blessed" "$(lib M1b)" - "$(pvg M1b)"; restore
sub README.md 'Plain `cargo install' 'Ruchy has 16,102 tests. Plain `cargo install'; bless
row M1c "hand-typed test count" "$(lib M1c)" - -; restore
sub README.md 'ruchy check examples/readme/match.ruchy' 'ruchy check examples/readme/matchx.ruchy'; bless
row M2a "broken example path in Quick start" "$(lib M2a)" "$(bin M2a)" -; restore
sub README.md '(`examples/readme/match.ruchy`)' '(`examples/readme/matchx.ruchy`)'; bless
row M2b "broken example path on an Examples block" "$(lib M2b)" - -; restore
sub examples/readme/hello.expected 'Hello, Ruchy!' 'Hello, Rust!'
row M3 "example output drift (.expected)" - "$(bin M3)" -; restore
sub README.md 'println(f"First product over 50: {found}")' 'println(f"First product over 50: {found}"'
sub examples/readme/loops.ruchy 'println(f"First product over 50: {found}")' 'println(f"First product over 50: {found}"'
row M4 "example no longer runs (README+file)" "$(lib M4)" "$(bin M4)" -; restore
sub README.md '## MSRV' 'MSRV:'; bless
row M5 "required section removed" "$(lib M5)" - "$(pvg M5)"; restore
sub README.md '| `ruchy wasm` |' '| `ruchy wasm-run` |'; bless
row M6 "Commands table names a missing subcommand" "$(lib M6)" "$(bin M6)" "$(pvg M6)"; restore
sub README.md '(docs/SPECIFICATION.md)' '(docs/SPEC.md)'; bless
row M7 "dead relative link" "$(lib M7)" - "$(pvg M7)"; restore
row M8 "baseline after restore" "$(lib M8)" "$(bin M8)" "$(pvg M8)"
