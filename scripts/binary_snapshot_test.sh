#!/usr/bin/env bash
# binary_snapshot_test.sh — one row per behaviour of scripts/binary_snapshot.sh.
# Needs only bash, git, jq: `cargo` and the bin target are stubs written into a
# throwaway repo, so nothing is compiled. The stub binary prints a clap-style
# --help (Commands / Arguments / Options sections, a `[COMMAND]` group, a group
# without it, lower-case `commands:`, a wrapped description, decoy lines, clap's
# auto `help`), so the content rows pin every rule that builds the snapshot line.
# Each refusal and content row goes red when the rule it pins is removed.
# The snapshot-lab block at the end covers binary_snapshot_fresh.sh and snapshot_lab_edge.sh (stub gh, no build).
#
#   bash scripts/binary_snapshot_test.sh
#   BS_SCRIPT=/path/to/mutant.sh bash scripts/binary_snapshot_test.sh   # test a copy
set -uo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SCRIPT="${BS_SCRIPT:-$HERE/binary_snapshot.sh}"
real_jq=$(command -v jq); real_sha=$(command -v sha256sum); real_timeout=$(command -v timeout); real_sort=$(command -v sort); BASH_BIN=$(command -v bash)
T=$(cd "$(mktemp -d)" && pwd -P)
trap 'rm -rf "${T:?}"' EXIT
fail=0 rows=0 OUT="" STDOUT="" ERR="" R=""

# Stub cargo: `metadata` lists $FAKE_BINS bin targets (reverse order, plus a lib
# target), `build` reports the executable among decoy artifacts unless
# FAKE_NOBUILD=1. Every call is logged to $FAKE_ROOT.cargo.
mkdir -p "$T/stub"
cat > "$T/stub/cargo" <<'STUB'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_ROOT.cargo"
if [ "$1" = metadata ] && [ "${FAKE_METAFAIL:-0}" = 1 ]; then exit 2
elif [ "$1" = metadata ]; then
    jq -nc --arg r "$FAKE_ROOT" --argjson n "${FAKE_BINS:-1}" --arg req "${FAKE_REQ:-}" --arg req0 "${FAKE_REQ0:-}" --arg sp "${FAKE_SRCSPACE:-}" \
        '{packages: [{name: "p", targets: ([range($n) as $i | {name: "t\($i)", kind: ["bin"], src_path: "\($r)/src/t\(if $sp == "" then "" else " " end)\($i).rs",
            "required-features": (if $i == 0 and $req0 != "" then ($req0 | split(",")) elif $req != "" then ($req | split(",")) else [] end)}] | reverse)
            + [{name: "libx", kind: ["lib"], src_path: "\($r)/src/lib.rs"}]}]}'
elif [ "${FAKE_NOBUILD:-0}" != 1 ]; then
    t=""; while [ $# -gt 0 ]; do [ "$1" = --bin ] && t=$2; shift; done
    e="$FAKE_ROOT/fakebin/$t"
    if [ "${FAKE_SAMEEXE:-0}" = 1 ]; then cp "$e" "$FAKE_ROOT/fakebin/shared"; e="$FAKE_ROOT/fakebin/shared"; fi
    [ "${FAKE_NOEXEC:-0}" != 1 ] || e="$FAKE_ROOT/noexec"
    jq -nc --arg t "$t" --arg d "$FAKE_ROOT/noexec" '{reason: "compiler-artifact", target: {name: $t}, executable: ($d + "-first")}'
    jq -nc --arg t "$t" --arg e "$e" '{reason: "compiler-artifact", target: {name: $t}, executable: $e}'
    jq -nc --arg t "$t" --arg d "$FAKE_ROOT/noexec" '{reason: "build-script-executed", target: {name: $t}, executable: $d}'
    jq -nc --arg d "$FAKE_ROOT/noexec" '{reason: "compiler-artifact", target: {name: "dep"}, executable: $d}'
    jq -nc --arg t "$t" '{reason: "compiler-artifact", target: {name: $t}, executable: null}'
fi
STUB
cat > "$T/stub/fakebin" <<'STUB'
#!/usr/bin/env bash
echo STDERR-NOISE >&2
# FAKE_HANG="<args>" hangs a call with exactly these args (only the Nth such call if FAKE_HANG_NTH).
if [ -n "${FAKE_HANG:-}" ] && [ "$*" = "$FAKE_HANG" ]; then
    n=$(($(cat "$FAKE_ROOT.hang" 2> /dev/null || echo 0) + 1)); echo "$n" > "$FAKE_ROOT.hang"
    if [ "${FAKE_HANG_NTH:-0}" -eq 0 ] || [ "$FAKE_HANG_NTH" -eq "$n" ]; then sleep 4 >&-; fi
fi
sect() { printf 'Usage: fake %s\n\n%s\n' "$1" "$2"; }
AUTOHELP='  help  Print this message or the help of the given subcommand(s)'
case "$*" in
--version) [ "${FAKE_NOVER:-0}" = 1 ] || echo "fake 1.0"; exit "${FAKE_VERRC:-0}" ;;
mcp) # JSON-RPC over stdio: answers tools/list only if the three requests arrive
    in=$(cat)
    [[ "$in" == *'"id":1,"method":"initialize"'* && "$in" == *'"method":"notifications/initialized"'* && "$in" == *'"id":2,"method":"tools/list"'* ]] || exit 0
    [ "$(printf '%s\n' "$in" | grep -c .)" -eq 3 ] || exit 0   # one JSON-RPC message per line
    [ "$(printf '%s\n' "$in" | grep -c .)" -eq 3 ] || exit 0   # one JSON-RPC message per line
    printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"capabilities":{}}}' \
        '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"beta"},{"name":"alpha"},{"name":"alpha"}]}}' '{"jsonrpc":"2.0","id":3,"result":{"tools":[{"name":"decoy"}]}}' 'not json' ;;
--help)
    if [ "${FAKE_NONDET:-0}" = 1 ]; then
        n=$(($(cat "$FAKE_ROOT.counter" 2> /dev/null || echo 0) + 1)); echo "$n" > "$FAKE_ROOT.counter"
        extra="      --opt-$n  moves every call"
    fi
    auto="$AUTOHELP"; [ "${FAKE_REALHELP:-0}" != 1 ] || auto=$'  help  Print this page\n  prn   Print this message or the help of a printer'
    [ "${FAKE_WRAPHELP:-0}" != 1 ] || auto=$'  help  Print this message or the help\n        of the given subcommand(s)'
    mcpline='  mcp         Start the MCP server'; [ "${FAKE_NOMCP:-0}" != 1 ] || mcpline='  mcpx        Not the server'
    printf '%s\n' 'Usage: fake [OPTIONS] [FILE]... <INPUT> [COMMAND]' '' 'Commands:' \
        '  run         Run a file' '  grp         Group with nested commands' \
        '                    wrapped description line' '  pick        Pick one' '  chain       Deep chain' \
        "$mcpline" '  bare' '  mig-4to5    Migrate' '  up_load.v2  Upload' "$auto" '' \
        'Arguments:' '  [FILE]...  Files to run' '             [default: x]' '  <INPUT>    Input' '' \
        'Options:' '      --eval <E>       Evaluate' '  -v, --verbose        Verbose, not --noisy-alias' '  -X, --xtra           Upper short' '  -4, --four           Digit short' '      --utf8           Utf' \
        '      --version-info   Info' '  -h, --help           Print help' '  -V, --version        Print version' \
        '        set via the --decoy-flag option' "${extra:-}" '' \
        'Examples:' '  fake run file.fk   Example invocation' '--flush-left  decoy at column 0' '' 'Notes:' '  [NOTE]  decoy'
    exit "${FAKE_HELPRC:-0}" ;;
"grp --help") sect 'grp [COMMAND]' "$(printf 'Commands:\n  sub1  Nested\n  sub2  Leaf\n%s' "$AUTOHELP")" ;;
"grp sub1 --help") sect 'grp sub1 [COMMAND]' "$(printf 'Commands:\n  deep  Leaf\n%s' "$AUTOHELP")" ;;
"pick --help") printf '%s\n' 'Pick one; Usage: fake pick [COMMAND] is not here' '' 'Usage: fake pick <ACTION>' '' 'commands:' '  a  A' '  b  B' "$AUTOHELP" '' 'options:' '  zzz  decoy' '' 'Usage: fake pick [COMMAND] (legacy)' ;;
"chain --help") sect 'chain [COMMAND]' "$(printf 'Commands:\n  l1  One\n%s' "$AUTOHELP")" ;;
"chain l1 --help") sect 'chain l1 [COMMAND]' "$(printf 'Commands:\n  l2  Two\n%s' "$AUTOHELP")" ;;
"chain l1 l2 --help") sect 'chain l1 l2 [COMMAND]' "$(printf 'Commands:\n  l3  Three\n%s' "$AUTOHELP")" ;;
"chain l1 l2 l3 --help") sect 'chain l1 l2 l3 [COMMAND]' "$(printf 'Commands:\n  l4  Four\n%s' "$AUTOHELP")" ;;
*--help) sect "${*% --help} [OPTIONS]" "$(printf 'Options:\n  -h, --help  Print help')" ;;
esac
STUB
# Stub jq: FAKE_JQDUP=1 makes the snapshot jq call emit its row twice.
cat > "$T/stub/jq" <<STUB
#!/usr/bin/env bash
if [ "\${FAKE_JQDUP:-0}" = 1 ] && [[ "\$*" == *binary-snapshot/v1* ]]; then "$real_jq" "\$@"; fi
[ "\${FAKE_JQDROP:-0}" = 1 ] && [[ "\$*" == *binary-snapshot/v1* ]] && exit 0
exec "$real_jq" "\$@"
STUB
# Stub sha256sum: FAKE_SHAEMPTY=1 prints nothing (a hash that came back empty).
cat > "$T/stub/sha256sum" <<STUB
#!/usr/bin/env bash
[ "\${FAKE_SHAEMPTY:-0}" = 1 ] && exit 0
[ "\${FAKE_SHASHORT:-0}" = 1 ] && { echo "abcdef  -"; exit 0; }
[ "\${FAKE_SHALONG:-0}" = 1 ] && { printf "%065d  -\n" 0; exit 0; }
exec "$real_sha" "\$@"
STUB
# Stub sort: logs LC_ALL to $FAKE_ROOT.lc, then runs the real one.
cat > "$T/stub/sort" <<STUB
#!/usr/bin/env bash
echo "\${LC_ALL:-}" >> "\$FAKE_ROOT.lc"
exec "$real_sort" "\$@"
STUB
# Stub timeout: logs its duration argument to $FAKE_ROOT.tmo, then runs the real one.
cat > "$T/stub/timeout" <<STUB
#!/usr/bin/env bash
echo "\$1" >> "\$FAKE_ROOT.tmo"
exec "$real_timeout" "\$@"
STUB
chmod +x "$T/stub/"*

# newrepo: fresh committed throwaway repo holding a copy of the script under test.
newrepo() {
    R="$T/r$rows"; mkdir -p "$R/scripts" "$R/fakebin" "$R/src"
    cp "$SCRIPT" "$R/scripts/binary_snapshot.sh"; : > "$R/Cargo.toml"; : > "$R/noexec"; chmod 644 "$R/noexec"
    for i in 0 1 2 3 4 5 6 7 8 9 10 11; do cp "$T/stub/fakebin" "$R/fakebin/t$i"; echo "# target $i" >> "$R/fakebin/t$i"; done
    git -C "$R" init -q
    git -C "$R" add -A
    git -C "$R" -c user.name=t -c user.email=t@t -c core.hooksPath=/dev/null -c commit.gpgsign=false commit -qm init
}

# row NAME EXPECT(ok|refuse) PATTERN [ENV=VAL...] -- ARGS...
# Optional, set per row: PRE (shell text run in the repo first), CHK (function
# run in the repo afterwards; $OUT is the script output, $R the repo).
row() {
    local name=$1 want=$2 pat=$3; shift 3
    local envs=(); while [ "$1" != -- ]; do envs+=("$1"); shift; done; shift
    rows=$((rows + 1)); newrepo; rm -f "${T:?}/o.jsonl"; mkdir -p "$R.tmp"
    [ -z "${PRE:-}" ] || (cd "$R" && "$BASH_BIN" -c "$PRE")
    local rc=0 so; OUT=""; so=$(cd "$R" && env PATH="$T/stub:$PATH" FAKE_ROOT="$R" TMPDIR="$R.tmp" "${envs[@]}" "$BASH_BIN" "$R/scripts/binary_snapshot.sh" "$@" 2> "${T:?}/err") || rc=$?
    STDOUT=$so; ERR=$(cat "$T/err"); OUT="$so$ERR"
    # FAIL: lines and progress belong on stderr, never stdout
    local post_ok=1; [[ "$so" != *FAIL:* && "$so" != *"  snap  "* ]] || post_ok=0
    [ -z "$(ls -A "$R.tmp")" ] || post_ok=0  # the trap removes the work dir on every exit path
    [ -z "${CHK:-}" ] || (cd "$R" && "$CHK") || post_ok=0
    local good=0; if [ "$want" = ok ]; then [ "$rc" -eq 0 ] && good=1; else [ "$rc" -ne 0 ] && good=1; fi
    [ "$post_ok" -eq 1 ] || good=0
    if [ "$good" -eq 1 ] && [[ "$OUT" == *"$pat"* ]]; then
        printf 'ok   %s\n' "$name"
    else
        printf 'FAIL %s (want %s /%s/, rc=%s, check=%s)\n     %s\n' "$name" "$want" "$pat" "$rc" "$post_ok" "$(printf '%s' "$OUT" | tail -n 3)"; fail=$((fail + 1))
    fi
}

# ---- content expectations (the stub binary's surface) ----
CMDS='["bare","chain","chain l1","chain l1 l2","chain l1 l2 l3","grp","grp sub1","grp sub1 deep","grp sub2","mcp","mig-4to5","pick a","pick b","run","up_load.v2"]'
KEYS='["schema","package","target","name","src_path","git_sha","version","version_sha256","binary_sha256","help_sha256","help_exit","commands","options","positionals","routes","tools"]'
sha() { printf '%s' "$1" | sha256sum | cut -d' ' -f1; }
file_sha() { sha256sum < "$1" | cut -d' ' -f1; }
bin_help_sha() { sha "$(FAKE_ROOT="$R" "$R/fakebin/$1" --help 2> /dev/null)"; }
# jq -e over the written line(s): $1 = jq program, $CMDS/$KEYS as --argjson cmds/keys.
snap() { jq -es --argjson cmds "$CMDS" --argjson keys "$KEYS" "$@" "$T/o.jsonl" > /dev/null; }
cargolog() { printf '%s\n' "$@" | cmp -s - "$R.cargo"; }
META="metadata --no-deps --format-version 1 --manifest-path"

check_full() {
    [ "$ERR" = "  snap  p/t0" ] || return 1
    [ "$(sort -u "$R.tmo" | paste -sd,)" = 20,30 ] || return 1
    cargolog "$META $R/Cargo.toml" "build -p p --bin t0 --profile release --manifest-path $R/Cargo.toml --message-format=json" || return 1
    [ "$STDOUT" = "snapshot: 1 bin target(s) -> $T/o.jsonl (HEAD $(git rev-parse --short=9 HEAD))" ] || return 1
    [ -s "$R.lc" ] && [ "$(sort -u "$R.lc")" = C ] || return 1
    snap --arg gs "$(git rev-parse HEAD)" --arg vs "$(sha 'fake 1.0')" --arg bs "$(file_sha "$R/fakebin/t0")" --arg hs "$(bin_help_sha t0)" '
        length == 1 and .[0] == {schema: "binary-snapshot/v1", package: "p", target: "t0", name: "t0", src_path: "src/t0.rs",
            git_sha: $gs, version: "fake 1.0", version_sha256: $vs, binary_sha256: $bs, help_sha256: $hs, help_exit: 0,
            commands: $cmds, options: ["--eval", "--four", "--utf8", "--verbose", "--version-info", "--xtra"], positionals: ["FILE", "INPUT"],
            routes: [], tools: ["alpha", "beta"]} and (.[0] | keys_unsorted) == $keys'
}
nowork() { [ ! -e "$R.cargo" ]; }  # a guard that refuses must stop before cargo runs
check_default() { [ ! -e "$T/o.jsonl" ] && jq -e '.schema == "binary-snapshot/v1"' evidence/binary/snapshot.jsonl > /dev/null; }
check_twice() { [ ! -e evidence ] && [ -z "$(git status --porcelain)" ] && [ "$STDOUT" = "DETERMINISTIC: two snapshots byte-identical (1 lines)." ]; }
check_twice2() { [ "$STDOUT" = "DETERMINISTIC: two snapshots byte-identical (2 lines)." ]; }
check_nofall() {
    [ -z "$(git status --porcelain)" ] && [[ "$OUT" != *dirname* && "$OUT" != *mv:* && "$OUT" != *unrecognized* ]] &&
        [ "$(printf "%s\n" "$STDOUT" | head -n 1)" = "NON-DETERMINISTIC: snapshots differ." ] && [[ "$STDOUT" == *"< {"* && "$STDOUT" == *"> {"* ]]
}
check_diff20() { [ "$(printf "%s\n" "$STDOUT" | wc -l)" -eq 21 ]; }  # header + diff cut at 20 lines
check_zero() { [ "$ERR" = "FAIL: cargo metadata listed 0 bin targets" ] && [ ! -e "$T/o.jsonl" ]; }
check_nobuild() { [ "$ERR" = "FAIL: p/t0 did not build" ]; }
check_notzero() { [[ "$OUT" != *"0 bin targets"* ]]; }  # a cargo failure is not reported as zero targets
check_spaces() { snap '.[0].src_path == "src/t 0.rs" and .[0].name == "t0"'; }
check_req0() { cargolog "$META $R/Cargo.toml" "build -p p --bin t0 --features repl --profile release --manifest-path $R/Cargo.toml --message-format=json" "build -p p --bin t1 --profile release --manifest-path $R/Cargo.toml --message-format=json"; }
check_wrap() { snap '.[0].commands == $cmds'; }
check_two() {
    snap --arg b0 "$(file_sha "$R/fakebin/t0")" --arg b1 "$(file_sha "$R/fakebin/t1")" \
        'map(.target) == ["t0", "t1"] and map(.name) == ["shared", "shared"] and map(.binary_sha256) == [$b0, $b1]' \
        && [[ "$OUT" == *"snapshot: 2 bin target(s)"* ]] && cargolog "$META $R/Cargo.toml" \
            "build -p p --bin t0 --profile release --manifest-path $R/Cargo.toml --message-format=json" \
            "build -p p --bin t1 --profile release --manifest-path $R/Cargo.toml --message-format=json"
}
check_req() { grep -qx "build -p p --bin t0 --features repl,x --profile release --manifest-path $R/Cargo.toml --message-format=json" "$R.cargo"; }
check_dev() { grep -qx "build -p p --bin t0 --profile dev --manifest-path $R/Cargo.toml --message-format=json" "$R.cargo"; }
check_env_tmo() { [ "$(sort -u "$R.tmo" | paste -sd,)" = 7,9 ]; }
check_realhelp() { snap '.[0].commands == (($cmds + ["help", "prn"]) | sort)'; }
check_nomcp() { snap '.[0].tools == [] and .[0].commands == (($cmds - ["mcp"] + ["mcpx"]) | sort)'; }
check_depth1() { snap '.[0].commands == ["bare","chain","grp","mcp","mig-4to5","run","up_load.v2"]'; }
check_verrc() { snap '.[0].version == "fake 1.0" and .[0].tools == ["alpha", "beta"]'; }
check_hang_kids() { snap '.[0].commands == ($cmds - ["grp sub1", "grp sub1 deep", "grp sub2"])'; }
check_hang_usage() { snap '.[0].commands == ($cmds - ["grp"])'; }
check_hang_top() { snap '.[0].commands == [] and .[0].tools == ["alpha", "beta"]'; }
check_hang_opts() { snap '.[0].options == [] and .[0].positionals == ["FILE", "INPUT"] and .[0].commands == $cmds'; }
check_hang_pos() { snap '.[0].positionals == [] and .[0].options == ["--eval", "--four", "--utf8", "--verbose", "--version-info", "--xtra"]'; }
check_hang_mcp() { snap '.[0].tools == [] and .[0].commands == $cmds'; }

CHK=check_full row "happy: full snapshot line (every field, key order, cargo calls, timeouts, streams, locale)" ok "snapshot: 1 bin target(s)" LC_ALL=C.UTF-8 -- "$T/o.jsonl"
CHK=check_default row "default output path, nothing else written" ok "-> evidence/binary/snapshot.jsonl (HEAD" --
CHK=check_twice row "twice: deterministic passes, writes nothing" ok "DETERMINISTIC: two snapshots byte-identical (1 lines)." -- --twice
CHK=check_twice2 row "twice: two targets, line count named" ok "DETERMINISTIC" FAKE_BINS=2 -- --twice
CHK=check_diff20 row "twice: diff output cut at 20 lines" refuse "NON-DETERMINISTIC" FAKE_NONDET=1 FAKE_BINS=12 -- --twice
row "twice: differing runs refused"     refuse "NON-DETERMINISTIC"  FAKE_NONDET=1 -- --twice
CHK=check_nobuild row "build failure refused"             refuse "did not build"      FAKE_NOBUILD=1 -- "$T/o.jsonl"
CHK=check_zero row "zero bin targets refused"          refuse "0 bin targets"      FAKE_BINS=0 -- "$T/o.jsonl"
row "line count != target count"        refuse "lines for 1 targets" FAKE_JQDUP=1 -- "$T/o.jsonl"
row "empty --version refused"           refuse "printed nothing for --version" FAKE_NOVER=1 -- "$T/o.jsonl"
row "failing --help refused"            refuse "--help exited 3"    FAKE_HELPRC=3 -- "$T/o.jsonl"
CHK=nowork PRE='echo x >> Cargo.toml' row "dirty tracked file refused" refuse "git tree is dirty" -- "$T/o.jsonl"
CHK=nowork PRE='echo x > stray' row "untracked file refused"       refuse "git tree is dirty" -- "$T/o.jsonl"
PRE='mkdir -p evidence/binary; echo x > evidence/binary/snapshot.jsonl' row "snapshot file itself exempt" ok "snapshot: 1 bin target(s)" -- "$T/o.jsonl"

CHK=check_nobuild row "non-executable build output refused" refuse "did not build" FAKE_NOEXEC=1 -- "$T/o.jsonl"
row "empty sha256 refused"              refuse "sha256 empty or malformed" FAKE_SHAEMPTY=1 -- "$T/o.jsonl"
row "short sha256 refused" refuse "sha256 empty or malformed" FAKE_SHASHORT=1 -- "$T/o.jsonl"
row "over-long sha256 refused" refuse "sha256 empty or malformed" FAKE_SHALONG=1 -- "$T/o.jsonl"
row "missing snapshot line refused" refuse "0 lines for 1 targets" FAKE_JQDROP=1 -- "$T/o.jsonl"
CHK=check_notzero row "cargo metadata failure refused"    refuse "" FAKE_METAFAIL=1 -- "$T/o.jsonl"
CHK=check_nofall row "twice: exits at the refusal (no fall-through, nothing written)" refuse "NON-DETERMINISTIC" FAKE_NONDET=1 -- --twice

# Content rows: one per rule that builds the line.
CHK=check_two row "two targets sharing one executable name: sorted, each hashed from its own copy" ok "snapshot: 2 bin target(s)" FAKE_BINS=2 FAKE_SAMEEXE=1 -- "$T/o.jsonl"
CHK=check_req row "required-features passed as --features" ok "snapshot: 1 bin" FAKE_REQ=repl,x -- "$T/o.jsonl"
CHK=check_dev row "SNAPSHOT_PROFILE selects the build profile" ok "snapshot: 1 bin" SNAPSHOT_PROFILE=dev -- "$T/o.jsonl"
CHK=check_req0 row "features reset between targets" ok "snapshot: 2 bin" FAKE_BINS=2 FAKE_REQ0=repl -- "$T/o.jsonl"
CHK=check_spaces row "a space in a path field survives the tab-separated hand-off" ok "snapshot: 1 bin" FAKE_SRCSPACE=1 -- "$T/o.jsonl"
CHK=check_wrap row "auto help line wrapped before the end is still skipped" ok "snapshot: 1 bin" FAKE_WRAPHELP=1 -- "$T/o.jsonl"
CHK=check_env_tmo row "SNAPSHOT_TIMEOUT / SNAPSHOT_MCP_TIMEOUT are honoured" ok "snapshot: 1 bin" SNAPSHOT_TIMEOUT=7 SNAPSHOT_MCP_TIMEOUT=9 -- "$T/o.jsonl"
CHK=check_realhelp row "a real command named help, and one whose text starts like the auto line, are kept" ok "snapshot: 1 bin" FAKE_REALHELP=1 -- "$T/o.jsonl"
CHK=check_nomcp row "no mcp command: tools not asked (mcpx is not mcp)" ok "snapshot: 1 bin" FAKE_NOMCP=1 -- "$T/o.jsonl"
CHK=check_depth1 row "SNAPSHOT_MAX_DEPTH=1 stops the walk" ok "snapshot: 1 bin" SNAPSHOT_MAX_DEPTH=1 -- "$T/o.jsonl"
CHK=check_verrc row "a --version that prints but exits non-zero is accepted" ok "snapshot: 1 bin" FAKE_VERRC=3 -- "$T/o.jsonl"

# Hang rows: a call that outlives SNAPSHOT_TIMEOUT yields nothing instead of stalling.
row "hung --version refused"           refuse "printed nothing for --version" SNAPSHOT_TIMEOUT=1 FAKE_HANG=--version -- "$T/o.jsonl"
row "hung --help refused"              refuse "--help exited 124" SNAPSHOT_TIMEOUT=1 FAKE_HANG=--help -- "$T/o.jsonl"
CHK=check_hang_kids row "hung subcommand listing: group treated as leaf" ok "snapshot: 1 bin" SNAPSHOT_TIMEOUT=1 FAKE_HANG="grp --help" FAKE_HANG_NTH=1 -- "$T/o.jsonl"
CHK=check_hang_usage row "hung Usage probe: group not listed, children kept" ok "snapshot: 1 bin" SNAPSHOT_TIMEOUT=1 FAKE_HANG="grp --help" FAKE_HANG_NTH=2 -- "$T/o.jsonl"
CHK=check_hang_top row "hung top-level command listing: commands []" ok "snapshot: 1 bin" SNAPSHOT_TIMEOUT=1 FAKE_HANG=--help FAKE_HANG_NTH=2 -- "$T/o.jsonl"
CHK=check_hang_opts row "hung options call: options []" ok "snapshot: 1 bin" SNAPSHOT_TIMEOUT=1 FAKE_HANG=--help FAKE_HANG_NTH=3 -- "$T/o.jsonl"
CHK=check_hang_pos row "hung positionals call: positionals []" ok "snapshot: 1 bin" SNAPSHOT_TIMEOUT=1 FAKE_HANG=--help FAKE_HANG_NTH=4 -- "$T/o.jsonl"
CHK=check_hang_mcp row "hung mcp call: tools []" ok "snapshot: 1 bin" SNAPSHOT_MCP_TIMEOUT=1 FAKE_HANG=mcp -- "$T/o.jsonl"

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
    CHK=nowork row "missing tool refused: $t" refuse "FAIL: $t missing" "PATH=$(mktool "$t")" -- "$T/o.jsonl"
done


# >>> snapshot-lab rows: scripts/binary_snapshot_fresh.sh, scripts/snapshot_lab_edge.sh and the workflow text (no build, no network).
# `gh` is a stub: `run list` answers $FAKE_RUNS, `run view` answers $FAKE_JOBS, `issue comment` succeeds; every call is logged.
# BS_FRESH / BS_EDGE / BS_WF test a copy.
FRESH="${BS_FRESH:-$HERE/binary_snapshot_fresh.sh}"; EDGE="${BS_EDGE:-$HERE/snapshot_lab_edge.sh}"
mkdir -p "$T/labstub"
cat > "$T/labstub/gh" <<'STUB'
#!/usr/bin/env bash
echo "$*" >> "$LAB_GHLOG"
case "$1 $2" in
"run list") [ "${FAKE_LISTFAIL:-0}" != 1 ] || exit 1; printf '%s' "$FAKE_RUNS" ;;
"run view") [ "${FAKE_GHFAIL:-0}" != 1 ] || exit 1
    prog=""; while [ $# -gt 0 ]; do if [ "$1" = --jq ]; then prog=$2; fi; shift; done
    printf '%s' "$FAKE_JOBS" | jq -r "$prog" ;;
"issue comment") [ "${FAKE_COMMENTFAIL:-0}" != 1 ] || exit 1 ;;
*) exit 9 ;;
esac
STUB
chmod +x "$T/labstub/gh"
JOBS_OK='{"jobs":[{"name":"snapshot","conclusion":"failure"},{"name":"report","conclusion":"success"}]}'
# lab_check NAME WANT(exit code) PATTERN OUTPUT RC
lab_check() {
    rows=$((rows + 1))
    if [ "$5" -eq "$2" ] && [[ "$4" == *"$3"* ]]; then
        printf 'ok   %s\n' "$1"
    else
        printf 'FAIL %s (want rc=%s /%s/, rc=%s)\n     %s\n' "$1" "$2" "$3" "$5" "$(printf '%s' "$4" | tail -n 3)"; fail=$((fail + 1))
    fi
}
# frow NAME WANT PATTERN COMMITTED_TEXT REGENERATED_TEXT
frow() {
    local rc=0 o; printf '%s' "$4" > "$T/la"; printf '%s' "$5" > "$T/lb"
    o=$("$BASH_BIN" "$FRESH" "$T/la" "$T/lb" 2>&1) || rc=$?
    lab_check "fresh: $1" "$2" "$3" "$o" "$rc"
    [ -z "${PAT2:-}" ] || lab_check "fresh: $1 (also prints)" "$2" "$PAT2" "$o" "$rc"
}
R1='{"schema":"s","name":"n","git_sha":"aaa","binary_sha256":"bbb","help_sha256":"h1"}'$'\n'
R1G='{"schema":"s","name":"n","git_sha":"zzz","binary_sha256":"bbb","help_sha256":"h1"}'$'\n'
R1B='{"schema":"s","name":"n","git_sha":"aaa","binary_sha256":"yyy","help_sha256":"h1"}'$'\n'
R1H='{"schema":"s","name":"n","git_sha":"aaa","binary_sha256":"bbb","help_sha256":"h2"}'$'\n'
R2='{"schema":"s","name":"m","git_sha":"aaa","binary_sha256":"bbb","help_sha256":"k1"}'$'\n'
PAT2='rows: committed=1 regenerated=1' frow "identical rows are fresh" 0 'FRESH: 1 row(s) equal' "$R1" "$R1"
frow "two identical rows are fresh" 0 'FRESH: 2 row(s) equal' "$R1$R2" "$R1$R2"
PAT2='"help_sha256":"h2"' frow "edited help_sha256 is STALE and prints the row" 1 'STALE: regenerate' "$R1" "$R1H"
frow "edit in the second row is STALE" 1 'STALE: regenerate' "$R1$R2" "$R1$R1H"
frow "git_sha-only change is fresh" 0 'FRESH: 1 row(s) equal' "$R1" "$R1G"
frow "binary_sha256-only change is fresh" 0 'FRESH: 1 row(s) equal' "$R1" "$R1B"
frow "extra regenerated row is STALE" 1 'STALE: regenerate' "$R1" "$R1$R2"
R1N='{"schema":"s","name":"other","git_sha":"aaa","binary_sha256":"bbb","help_sha256":"h1"}'$'\n'
frow "edit in the name field is STALE" 1 'STALE: regenerate' "$R1" "$R1N"
PAT2='committed=0 regenerated=1' frow "empty committed refused" 1 'FAIL: 0 rows on a side' '' "$R1"
PAT2='committed=1 regenerated=0' frow "empty regenerated refused" 1 'FAIL: 0 rows on a side' "$R1" ''
frow "0 rows equal 0 rows refused" 1 'FAIL: 0 rows on a side' '' ''
frow "invalid JSON committed refused" 1 'FAIL: jq on committed side' 'not json' "$R1"
frow "invalid JSON regenerated refused" 1 'FAIL: jq on regenerated side' "$R1" 'not json'
rc=0; o=$("$BASH_BIN" "$FRESH" "$T/nonexistent" "$T/la" 2>&1) || rc=$?
lab_check "fresh: missing committed file refused" 1 'FAIL: jq on committed side' "$o" "$rc"
rc=0; o=$("$BASH_BIN" "$FRESH" "$T/la" 2>&1) || rc=$?
lab_check "fresh: one argument is a usage error" 2 'usage:' "$o" "$rc"

mkdir -p "$T/mkstub"; printf '#!/usr/bin/env bash\necho mktemp-stub-failure >&2\nexit 1\n' > "$T/mkstub/mktemp"; chmod +x "$T/mkstub/mktemp"
rc=0; o=$(env PATH="$T/mkstub:$PATH" "$BASH_BIN" "$FRESH" "$T/la" "$T/lb" 2>&1) || rc=$?
lab_check "fresh: a failing mktemp stops the script (exit 1)" 1 'mktemp-stub-failure' "$o" "$rc"
r=0; [[ "$o" != *'FAIL: jq'* ]] || r=1
lab_check "fresh: nothing runs after a failing mktemp" 0 '' '' "$r"
# zrow NAME WANT PATTERN COMMITTED_TEXT REGENERATED_TEXT [ENV=VAL...]: the no-argument path, in a temp repo whose
# scripts/binary_snapshot.sh is a stub that writes $FAKE_REGEN to the file it is given.
zrow() {
    local d="$T/zr$rows" rc=0 o; mkdir -p "$d/scripts" "$d/evidence/binary"
    cp "$FRESH" "$d/scripts/binary_snapshot_fresh.sh"
    printf '%s' "$4" > "$d/evidence/binary/snapshot.jsonl"
    cat > "$d/scripts/binary_snapshot.sh" <<'STUB'
#!/usr/bin/env bash
[ "${FAKE_REGENFAIL:-0}" != 1 ] || { echo regen-stub-failure >&2; exit 3; }
printf '%s' "$FAKE_REGEN" > "${1:?}"
STUB
    o=$(env FAKE_REGEN="$5" "${@:6}" "$BASH_BIN" "$d/scripts/binary_snapshot_fresh.sh" 2>&1) || rc=$?
    ZO=$o; lab_check "fresh (no args): $1" "$2" "$3" "$o" "$rc"
}
zrow "regenerated equals committed -> FRESH" 0 'FRESH: 1 row(s) equal' "$R1" "$R1"
zrow "regenerated differs from committed -> STALE (not self-compared)" 1 'STALE: regenerate' "$R1" "$R1H"
zrow "regenerated differs in the second row -> STALE" 1 'STALE: regenerate' "$R1$R2" "$R1$R1H"
zrow "regenerated git_sha-only change -> FRESH" 0 'FRESH: 1 row(s) equal' "$R1" "$R1G"
zrow "regenerated 0 rows refused" 1 'FAIL: 0 rows on a side' "$R1" ''
zrow "committed 0 rows refused" 1 'FAIL: 0 rows on a side' '' "$R1"
zrow "regeneration failure refused (exit 1, stub message shown)" 1 "regen-stub-failure" "$R1" "$R1" FAKE_REGENFAIL=1
r=0; [[ "$ZO" != *"FAIL: jq"* && "$ZO" != *"rows: committed"* ]] || r=1
lab_check "fresh (no args): a failed regeneration stops before any compare" 0 "" "" "$r"

# erow NAME WANT PATTERN RUNS_JSON [ENV=VAL...]  (current run id 999). EXP=yes|no: must the run comment on #299?
# Default: yes for a want-0 row whose verdict starts NEW (pattern or not), no otherwise.
# GHPAT: a string the gh log must contain. NOVIEW=1: no `run view` call allowed. Every row also pins the history query.
LISTCMD='run list -R o/r --workflow snapshot-lab.yml --event schedule --branch main --status completed -L 10 --json databaseId,conclusion'
erow() {
    local name=$1 want=$2 pat=$3 hist=$4; shift 4; local rc=0 o r
    : > "$T/ghlog"
    o=$(env PATH="$T/labstub:$PATH" GITHUB_REPOSITORY=o/r GITHUB_SERVER_URL=https://x.test LAB_GHLOG="$T/ghlog" FAKE_RUNS="$hist" FAKE_JOBS="$JOBS_OK" "$@" "$BASH_BIN" "$EDGE" 999 2>&1) || rc=$?
    lab_check "edge: $name" "$want" "$pat" "$o" "$rc"
    local log exp="${EXP:-}"; log=$(cat "$T/ghlog")
    [ "$want" -eq 0 ] || [ -n "$exp" ] || exp=no
    [ -n "$exp" ] || { case "$o" in NEW*) exp=yes ;; *) exp=no ;; esac; }
    r=0; [ "$(printf '%s\n' "$log" | head -n 1)" = "$LISTCMD" ] || r=1
    lab_check "edge: $name (first gh call is the pinned history query)" 0 '' '' "$r"
    r=0
    if [ "$exp" = yes ]; then [[ "$log" == *"issue comment 299 -R o/r --body snapshot-lab RED (new, ${o%%$'\n'*}): https://x.test/o/r/actions/runs/999"* ]] || r=1
    else [[ "$log" != *"issue comment"* ]] || r=1; fi
    lab_check "edge: $name (comment on #299: $exp)" 0 '' '' "$r"
    r=0; [ -z "${GHPAT:-}" ] || [[ "$log" == *"$GHPAT"* ]] || r=1
    [ -z "${NOVIEW:-}" ] || [[ "$log" != *"run view"* ]] || r=1
    lab_check "edge: $name (job-read calls)" 0 '' '' "$r"
}
NOVIEW=1 erow "previous success -> NEW, no jobs read" 0 'NEW red: previous scheduled run 5 was success' '[{"databaseId":5,"conclusion":"success"}]'
GHPAT='run view 5 -R o/r --json jobs' erow "previous failure, report ok -> CONTINUING" 0 'CONTINUING red since run 5: silent' '[{"databaseId":5,"conclusion":"failure"}]'
erow "previous timed_out, report ok -> CONTINUING" 0 'CONTINUING red since run 5: silent' '[{"databaseId":5,"conclusion":"timed_out"}]'
NOVIEW=1 erow "empty history -> NEW" 0 'NEW red: no measured previous scheduled run' '[]'
erow "cancelled then failure -> CONTINUING on the failure" 0 'CONTINUING red since run 5: silent' '[{"databaseId":6,"conclusion":"cancelled"},{"databaseId":5,"conclusion":"failure"}]'
NOVIEW=1 erow "only cancelled -> NEW" 0 'NEW red: no measured previous scheduled run' '[{"databaseId":6,"conclusion":"cancelled"},{"databaseId":5,"conclusion":"cancelled"}]'
NOVIEW=1 erow "current run (failure) dropped, older success -> NEW" 0 'NEW red: previous scheduled run 5 was success' '[{"databaseId":999,"conclusion":"failure"},{"databaseId":5,"conclusion":"success"}]'
erow "current run (failure) dropped, older failure -> CONTINUING" 0 'CONTINUING red since run 5: silent' '[{"databaseId":999,"conclusion":"failure"},{"databaseId":5,"conclusion":"failure"}]'
NOVIEW=1 erow "newest remaining decides: success newer than failure" 0 'NEW red: previous scheduled run 6 was success' '[{"databaseId":6,"conclusion":"success"},{"databaseId":5,"conclusion":"failure"}]'
erow "newest remaining decides: failure newer than success" 0 'CONTINUING red since run 6: silent' '[{"databaseId":6,"conclusion":"failure"},{"databaseId":5,"conclusion":"success"}]'
NOVIEW=1 erow "skipped is skipped over" 0 'NEW red: previous scheduled run 5 was success' '[{"databaseId":6,"conclusion":"skipped"},{"databaseId":5,"conclusion":"success"}]'
NOVIEW=1 erow "startup_failure is skipped over" 0 'NEW red: previous scheduled run 5 was success' '[{"databaseId":6,"conclusion":"startup_failure"},{"databaseId":5,"conclusion":"success"}]'
NOVIEW=1 erow "empty conclusion (running) is skipped over" 0 'NEW red: previous scheduled run 5 was success' '[{"databaseId":6,"conclusion":""},{"databaseId":5,"conclusion":"success"}]'
NOVIEW=1 erow "null conclusion is skipped over" 0 'NEW red: previous scheduled run 5 was success' '[{"databaseId":6,"conclusion":null},{"databaseId":5,"conclusion":"success"}]'
erow "previous failure, report job failed -> NEW (lost edge)" 0 "NEW red: previous scheduled run 5 failed and its report job was 'failure'" '[{"databaseId":5,"conclusion":"failure"}]' FAKE_JOBS='{"jobs":[{"name":"snapshot","conclusion":"failure"},{"name":"report","conclusion":"failure"}]}'
erow "previous failure, report job skipped -> NEW" 0 "its report job was 'skipped'" '[{"databaseId":5,"conclusion":"failure"}]' FAKE_JOBS='{"jobs":[{"name":"report","conclusion":"skipped"}]}'
erow "previous failure, no report job -> NEW" 0 "its report job was 'absent'" '[{"databaseId":5,"conclusion":"failure"}]' FAKE_JOBS='{"jobs":[{"name":"snapshot","conclusion":"failure"}]}'
erow "previous timed_out, report job failed -> NEW" 0 "its report job was 'failure'" '[{"databaseId":5,"conclusion":"timed_out"}]' FAKE_JOBS='{"jobs":[{"name":"report","conclusion":"failure"}]}'
erow "jobs read fails -> exit 1, no comment" 1 'FAIL: cannot read jobs of run 5' '[{"databaseId":5,"conclusion":"failure"}]' FAKE_GHFAIL=1
for c in neutral action_required stale; do
    erow "unknown conclusion $c -> exit 1, no comment" 1 "FAIL: unknown conclusion '$c' on run 5" "[{\"databaseId\":5,\"conclusion\":\"$c\"}]"
done
erow "history is not JSON -> exit 1, no comment" 1 'FAIL: history is not a JSON run list' 'not json'
erow "history is a JSON object, not a list -> exit 1" 1 'FAIL: history is not a JSON run list' '{"databaseId":5}'
erow "empty history output -> exit 1" 1 'FAIL: empty history input' ''
erow "history read fails -> exit 1, no comment" 1 'FAIL: cannot read run history' '[]' FAKE_LISTFAIL=1
EXP=yes erow "comment fails on a NEW red -> exit 1" 1 'FAIL: cannot comment on #299' '[{"databaseId":5,"conclusion":"success"}]' FAKE_COMMENTFAIL=1
rc=0; o=$(printf '[]' | env PATH="$T/labstub:$PATH" GITHUB_REPOSITORY=o/r GITHUB_SERVER_URL=https://x.test "$BASH_BIN" "$EDGE" 2>&1) || rc=$?
lab_check "edge: no current run id -> exit 1" 1 'usage:' "$o" "$rc"
rc=0; o=$(env -u GITHUB_REPOSITORY PATH="$T/labstub:$PATH" GITHUB_SERVER_URL=https://x.test "$BASH_BIN" "$EDGE" 999 2>&1) || rc=$?
lab_check "edge: no GITHUB_REPOSITORY -> exit 1" 1 'GITHUB_REPOSITORY unset' "$o" "$rc"
rc=0; o=$(env -u GITHUB_SERVER_URL PATH="$T/labstub:$PATH" GITHUB_REPOSITORY=o/r "$BASH_BIN" "$EDGE" 999 2>&1) || rc=$?
lab_check "edge: no GITHUB_SERVER_URL -> exit 1" 1 'GITHUB_SERVER_URL unset' "$o" "$rc"

# Workflow rows are TEXT assertions on the files (grep), not runs of the workflow: they pin what the YAML says.
WF="${BS_WF:-$HERE/../.github/workflows/snapshot-lab.yml}"
wf_has() { rc=0; grep -Eq -- "$2" "${3:-$WF}" || rc=1; lab_check "workflow text: $1" 0 '' '' "$rc"; }
wf_lacks() { rc=0; grep -Eq -- "$2" "${3:-$WF}" && rc=1; lab_check "workflow text: $1" 0 '' '' "$rc"; }
wf_count() { local c; c=$(grep -Ec -- "$2" "$WF"); rc=0; [ "$c" = "$3" ] || rc=1; lab_check "workflow text: $1 (x$3)" 0 '' '' "$rc"; }
wf_has "workflow name" "^name: snapshot-lab$"
wf_has "snapshot job timeout 60" "^    timeout-minutes: 60$"
wf_has "report job timeout 10" "^    timeout-minutes: 10$"
wf_count "checkout in both jobs" "^      - uses: actions/checkout@v4$" 2
wf_count "ubuntu-latest in both jobs" "^    runs-on: ubuntu-latest$" 2
wf_has "snapshot job installs the stable toolchain" "^      - uses: dtolnay/rust-toolchain@stable$"
wf_has "snapshot job caches cargo with the release+repl key" "^          key: snapshot-release-repl$"
wf_has "report job runs only for schedule, first attempt, on failure" "^    if: \\$\\{\\{ failure\\(\\) && github.run_attempt == 1 && github.event_name == 'schedule' \\}\\}$"
wf_has "report job needs snapshot" "^    needs: snapshot$"
wf_has "cron schedule 04:45" "^    - cron: '45 4 \\* \\* \\*'$"
wf_has "triggers: schedule" "^  schedule:$"
wf_has "triggers: workflow_dispatch" "^  workflow_dispatch: \\{\\}$"
wf_lacks "no push / pull_request / merge_group / release trigger" "^  (push|pull_request|pull_request_target|merge_group|release|workflow_run):"
wf_lacks "no || true, continue-on-error or if: always()" "\\|\\| true|continue-on-error|always\\(\\)"
wf_lacks "no || true or continue-on-error in the new scripts" "\\|\\| true|continue-on-error" "$FRESH"
wf_lacks "no || true or continue-on-error in the edge script" "\\|\\| true|continue-on-error" "$EDGE"
wf_has "top-level permissions: contents: read" "^permissions:$"
wf_has "top-level contents: read" "^  contents: read$"
wf_has "report job asks for actions: read" "^      actions: read$"
wf_has "report job asks for issues: write" "^      issues: write$"
wf_has "report job asks for contents: read" "^      contents: read$"
wf_lacks "no write permission beyond issues: write" "(contents|actions|pull-requests|checks|statuses|packages|id-token): write"
wf_has "report job runs the edge script with the run id" "^        run: bash scripts/snapshot_lab_edge.sh \"\\\$GITHUB_RUN_ID\"$"
wf_has "report job passes the token" "^          GH_TOKEN: \\\$\\{\\{ github.token \\}\\}$"
wf_has "report job checks out the scripts" "^      - uses: actions/checkout@v4$"
wf_has "snapshot job runs the stub rows" "^        run: bash scripts/binary_snapshot_test.sh$"
wf_has "snapshot job runs the fresh check" "^        run: bash scripts/binary_snapshot_fresh.sh$"
wf_has "edge script comments on #299 only" "gh issue comment 299 " "$EDGE"
wf_lacks "edge script never closes or edits an issue" "gh issue (close|edit|reopen)" "$EDGE"
wf_lacks "workflow itself calls no gh (outside comments)" "^[^#]*gh (issue|run|api)"
# <<< snapshot-lab rows

printf '%s rows, %s failed\n' "$rows" "$fail"
[ "$fail" -eq 0 ]
