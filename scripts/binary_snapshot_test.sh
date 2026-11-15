#!/usr/bin/env bash
# binary_snapshot_test.sh — one row per behaviour of scripts/binary_snapshot.sh.
# Needs only bash, git, jq: `cargo` and the bin target are stubs written into a
# throwaway repo, so nothing is compiled. Each refusal row goes red when the
# guarded line in the script is removed.
#
#   bash scripts/binary_snapshot_test.sh
#   BS_SCRIPT=/path/to/mutant.sh bash scripts/binary_snapshot_test.sh   # test a copy
set -uo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SCRIPT="${BS_SCRIPT:-$HERE/binary_snapshot.sh}"
real_jq=$(command -v jq)
T=$(mktemp -d)
trap 'rm -rf "${T:?}"' EXIT
fail=0 rows=0

# Stub cargo: `metadata` lists $FAKE_BINS bin targets, `build` reports an
# executable unless FAKE_NOBUILD=1. Stub bin: --version/--help follow FAKE_*.
mkdir -p "$T/stub"
cat > "$T/stub/cargo" <<'STUB'
#!/usr/bin/env bash
if [ "$1" = metadata ]; then
    jq -nc --arg r "$FAKE_ROOT" --argjson n "${FAKE_BINS:-1}" \
        '{packages: [{name: "p", targets: [range($n) as $i | {name: "t\($i)", kind: ["bin"], src_path: "\($r)/src/t\($i).rs"}]}]}'
elif [ "${FAKE_NOBUILD:-0}" != 1 ]; then
    t=""; while [ $# -gt 0 ]; do [ "$1" = --bin ] && t=$2; shift; done
    jq -nc --arg t "$t" --arg e "$FAKE_ROOT/fakebin/$t" '{reason: "compiler-artifact", target: {name: $t}, executable: $e}'
fi
STUB
cat > "$T/stub/fakebin" <<'STUB'
#!/usr/bin/env bash
case "$1" in
--version) [ "${FAKE_NOVER:-0}" = 1 ] || echo "fake 1.0" ;;
--help)
    if [ "${FAKE_NONDET:-0}" = 1 ]; then
        n=$(($(cat "$FAKE_ROOT/counter" 2> /dev/null || echo 0) + 1)); echo "$n" > "$FAKE_ROOT/counter"
        extra="      --opt-$n  moves every call"
    fi
    printf 'Usage: fake [OPTIONS]\n\nOptions:\n      --eval <E>  x\n%s\n  -h, --help  h\n' "${extra:-}"
    exit "${FAKE_HELPRC:-0}" ;;
esac
STUB
# Stub jq: FAKE_JQDUP=1 makes the snapshot jq call emit its row twice.
cat > "$T/stub/jq" <<STUB
#!/usr/bin/env bash
if [ "\${FAKE_JQDUP:-0}" = 1 ] && [[ "\$*" == *binary-snapshot/v1* ]]; then "$real_jq" "\$@"; fi
exec "$real_jq" "\$@"
STUB
chmod +x "$T/stub/"*

# newrepo: fresh committed throwaway repo holding a copy of the script under test.
newrepo() {
    R="$T/r$rows"; mkdir -p "$R/scripts" "$R/fakebin" "$R/src"
    cp "$SCRIPT" "$R/scripts/binary_snapshot.sh"; : > "$R/Cargo.toml"
    for i in 0 1; do cp "$T/stub/fakebin" "$R/fakebin/t$i"; done
    git -C "$R" init -q
    git -C "$R" add -A
    git -C "$R" -c user.name=t -c user.email=t@t -c core.hooksPath=/dev/null commit -qm init
}

# row NAME EXPECT(ok|refuse) PATTERN [ENV=VAL...] -- ARGS...
row() {
    local name=$1 want=$2 pat=$3; shift 3
    local envs=(); while [ "$1" != -- ]; do envs+=("$1"); shift; done; shift
    rows=$((rows + 1)); newrepo
    [ -z "${PRE:-}" ] || (cd "$R" && eval "$PRE")
    local out rc=0
    out=$(env PATH="$T/stub:$PATH" FAKE_ROOT="$R" "${envs[@]}" bash "$R/scripts/binary_snapshot.sh" "$@" 2>&1) || rc=$?
    local good=0; if [ "$want" = ok ]; then [ "$rc" -eq 0 ] && good=1; else [ "$rc" -ne 0 ] && good=1; fi
    if [ "$good" -eq 1 ] && [[ "$out" == *"$pat"* ]]; then
        printf 'ok   %s\n' "$name"
    else
        printf 'FAIL %s (want %s /%s/, rc=%s)\n     %s\n' "$name" "$want" "$pat" "$rc" "$(printf '%s' "$out" | tail -n 3)"; fail=$((fail + 1))
    fi
}

PRE= row "happy: one bin, one line"          ok     "snapshot: 1 bin target(s)" -- "$T/h.jsonl"
PRE= row "twice: deterministic passes"       ok     "DETERMINISTIC: two"        -- --twice
PRE= row "twice: differing runs refused"     refuse "NON-DETERMINISTIC"  FAKE_NONDET=1 -- --twice
PRE= row "build failure refused"             refuse "did not build"      FAKE_NOBUILD=1 -- "$T/o.jsonl"
PRE= row "zero bin targets refused"          refuse "0 bin targets"      FAKE_BINS=0 -- "$T/o.jsonl"
PRE= row "line count != target count"        refuse "lines for 1 targets" FAKE_JQDUP=1 -- "$T/o.jsonl"
PRE= row "empty --version refused"           refuse "printed nothing for --version" FAKE_NOVER=1 -- "$T/o.jsonl"
PRE= row "failing --help refused"            refuse "--help exited 3"    FAKE_HELPRC=3 -- "$T/o.jsonl"
PRE='echo x >> Cargo.toml' row "dirty tracked file refused" refuse "git tree is dirty" -- "$T/o.jsonl"
PRE='echo x > stray' row "untracked file refused"       refuse "git tree is dirty" -- "$T/o.jsonl"
PRE='mkdir -p evidence/binary; echo x > evidence/binary/snapshot.jsonl' row "snapshot file itself exempt" ok "snapshot: 1 bin target(s)" -- "$T/o.jsonl"

printf '%s rows, %s failed\n' "$rows" "$fail"
[ "$fail" -eq 0 ]
