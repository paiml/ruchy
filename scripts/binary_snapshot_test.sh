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
real_jq=$(command -v jq); real_sha=$(command -v sha256sum); BASH_BIN=$(command -v bash)
T=$(mktemp -d)
trap 'rm -rf "${T:?}"' EXIT
fail=0 rows=0

# Stub cargo: `metadata` lists $FAKE_BINS bin targets, `build` reports an
# executable unless FAKE_NOBUILD=1. Stub bin: --version/--help follow FAKE_*.
mkdir -p "$T/stub"
cat > "$T/stub/cargo" <<'STUB'
#!/usr/bin/env bash
if [ "$1" = metadata ] && [ "${FAKE_METAFAIL:-0}" = 1 ]; then exit 2
elif [ "$1" = metadata ]; then
    jq -nc --arg r "$FAKE_ROOT" --argjson n "${FAKE_BINS:-1}" \
        '{packages: [{name: "p", targets: [range($n) as $i | {name: "t\($i)", kind: ["bin"], src_path: "\($r)/src/t\($i).rs"}]}]}'
elif [ "${FAKE_NOBUILD:-0}" != 1 ]; then
    t=""; while [ $# -gt 0 ]; do [ "$1" = --bin ] && t=$2; shift; done
    e="$FAKE_ROOT/fakebin/$t"; [ "${FAKE_NOEXEC:-0}" != 1 ] || e="$FAKE_ROOT/noexec"
    jq -nc --arg t "$t" --arg e "$e" '{reason: "compiler-artifact", target: {name: $t}, executable: $e}'
fi
STUB
cat > "$T/stub/fakebin" <<'STUB'
#!/usr/bin/env bash
case "$1" in
--version) [ "${FAKE_NOVER:-0}" = 1 ] || echo "fake 1.0" ;;
--help)
    if [ "${FAKE_NONDET:-0}" = 1 ]; then
        n=$(($(cat "$FAKE_ROOT.counter" 2> /dev/null || echo 0) + 1)); echo "$n" > "$FAKE_ROOT.counter"
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
# Stub sha256sum: FAKE_SHAEMPTY=1 prints nothing (a hash that came back empty).
cat > "$T/stub/sha256sum" <<STUB
#!/usr/bin/env bash
[ "\${FAKE_SHAEMPTY:-0}" = 1 ] && exit 0
exec "$real_sha" "\$@"
STUB
chmod +x "$T/stub/"*

# newrepo: fresh committed throwaway repo holding a copy of the script under test.
newrepo() {
    R="$T/r$rows"; mkdir -p "$R/scripts" "$R/fakebin" "$R/src"
    cp "$SCRIPT" "$R/scripts/binary_snapshot.sh"; : > "$R/Cargo.toml"; : > "$R/noexec"; chmod 644 "$R/noexec"
    for i in 0 1; do cp "$T/stub/fakebin" "$R/fakebin/t$i"; done
    git -C "$R" init -q
    git -C "$R" add -A
    git -C "$R" -c user.name=t -c user.email=t@t -c core.hooksPath=/dev/null -c commit.gpgsign=false commit -qm init
}

# row NAME EXPECT(ok|refuse) PATTERN [ENV=VAL...] -- ARGS...
row() {
    local name=$1 want=$2 pat=$3; shift 3
    local envs=(); while [ "$1" != -- ]; do envs+=("$1"); shift; done; shift
    rows=$((rows + 1)); newrepo
    [ -z "${PRE:-}" ] || (cd "$R" && eval "$PRE")
    local out rc=0
    out=$(cd "$R" && env PATH="$T/stub:$PATH" FAKE_ROOT="$R" "${envs[@]}" "$BASH_BIN" "$R/scripts/binary_snapshot.sh" "$@" 2>&1) || rc=$?
    local post_ok=1; [ -z "${POST:-}" ] || (cd "$R" && eval "$POST") || post_ok=0
    local good=0; if [ "$want" = ok ]; then [ "$rc" -eq 0 ] && good=1; else [ "$rc" -ne 0 ] && good=1; fi
    [ "$post_ok" -eq 1 ] || good=0
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

PRE= row "non-executable build output refused" refuse "did not build" FAKE_NOEXEC=1 -- "$T/o.jsonl"
PRE= row "empty sha256 refused"              refuse "sha256 empty or malformed" FAKE_SHAEMPTY=1 -- "$T/o.jsonl"
PRE= row "cargo metadata failure refused"    refuse "" FAKE_METAFAIL=1 -- "$T/o.jsonl"
PRE= POST='test -z "$(git status --porcelain)" && ! printf "%s\n" "$out" | grep -qE "dirname|mv:|unrecognized"' row "twice: exits at the refusal (no fall-through, nothing written)" refuse "NON-DETERMINISTIC" FAKE_NONDET=1 -- --twice

# Missing-tool rows: one per tool the script requires. The PATH of the run holds
# the stubs and every other tool, but not this one; the refusal must name it.
# (the list below is fixed here, not read from the script, so dropping a tool from the guard goes red.)
mktool() { # $1 = tool left out; prints the PATH dir
    local d="$T/path-$1" t p
    mkdir -p "$d"
    for t in cargo jq git sha256sum timeout awk grep sort cut install mktemp basename dirname cmp diff head tail mkdir mv rm env bash cat; do
        [ "$t" = "$1" ] && continue
        case $t in cargo|jq|sha256sum) cp "$T/stub/$t" "$d/$t" ;; *) p=$(command -v "$t") && ln -sf "$p" "$d/$t" ;; esac
    done
    printf '%s' "$d"
}
for t in cargo jq git sha256sum timeout awk grep sort cut install mktemp basename dirname cmp diff head tail mkdir mv rm; do
    PRE= row "missing tool refused: $t" refuse "FAIL: $t missing" "PATH=$(mktool "$t")" -- "$T/o.jsonl"
done

printf '%s rows, %s failed\n' "$rows" "$fail"
[ "$fail" -eq 0 ]
