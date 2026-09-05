#!/usr/bin/env bash
# PMAT-096 -- pre-release gate v2 (ruchy 5.0.0-beta.2 release plan, Z5 / §4).
#
# Every stage prints its own PASS/WARN/FAIL line and writes its measurement into
# docs/specifications/evidence/<date>-dogfood/receipt.json. Nothing scores points and no
# stage reports a number it did not measure: a stage that cannot run is FAIL with a reason.
#
# Stages, in order:
#   1 tests        cargo test --lib -p ruchy
#   2 features     clippy x3 feature sets + fmt (CI toolchain) + cargo audit
#   3 verbs        verb surface derived from the BUILT binary, each run on tests/golden/
#   4 differential three-way vs the 4.2.1 baseline over examples/ + sibling corpora
#   5 clean_room   cargo package -> extract -> --locked and unlocked builds, fresh CARGO_HOME
#   6 package      file count, compressed size, colon paths, Cargo.lock present
#   7 satd         pmat analyze satd --path src
#   8 receipt      write receipt.json, print the summary, exit 1 on no-go
#
# Environment:
#   RUCHY_BASELINE_BIN  path to the 4.2.1 binary (else it is cargo-installed into a cache)
#   RUCHY_BIN           path to the HEAD binary (else built with cargo build --release)
#   CARGO_TARGET_DIR    honoured for HEAD builds; always unset for the clean-room builds

set -u -o pipefail

# --------------------------------------------------------------------------------------
# paths and globals
# --------------------------------------------------------------------------------------

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
CONFIG="$ROOT/scripts/release-gate.toml"
KNOWN_BREAKS="$ROOT/scripts/release-known-breaks.txt"
KNOWN_FIXES="$ROOT/scripts/release-known-fixes.txt"
GOLDEN_DIR="$ROOT/tests/golden"
EVIDENCE_DIR="$ROOT/docs/specifications/evidence/$(date +%Y-%m-%d)-dogfood"
RECEIPT="$EVIDENCE_DIR/receipt.json"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

if [ "${1:-}" = "--worker-diff" ]; then
    WORK=""
else
    WORK=$(mktemp -d "${TMPDIR:-/tmp}/ruchy-gate.XXXXXX") || exit 2
    STAGES="$WORK/stages"
    mkdir -p "$STAGES"
    WARN_FILE="$WORK/warns.txt"
    : > "$WARN_FILE"
    cleanup() {
        [ -n "${WORK:-}" ] && [ -d "$WORK" ] && rm -rf "$WORK"
    }
    trap cleanup EXIT
fi

# --------------------------------------------------------------------------------------
# small helpers
# --------------------------------------------------------------------------------------

say() { printf '%s\n' "$*"; }

hdr() {
    say ""
    say "======================================================================"
    say "$*"
    say "======================================================================"
}

add_warn() {
    printf '%s\n' "$1" >> "$WARN_FILE"
    say "  WARN  $1"
}

# report <stage> <status> <detail>
report() {
    say ""
    say "  [$2] stage $1 -- $3"
    return 0
}

sha_of_stdin() { sha256sum | cut -d' ' -f1; }

need() {
    if ! command -v "$1" > /dev/null 2>&1; then
        say "FATAL: required tool not on PATH: $1"
        exit 2
    fi
}

# --------------------------------------------------------------------------------------
# release-gate.toml (grep/sed parsing; every array must sit on one line)
# --------------------------------------------------------------------------------------

cfg_raw() {
    awk -v sec="[$1]" -v key="$2" '
        /^[[:space:]]*\[/ { insec = ($0 ~ "^[[:space:]]*\\" sec "[[:space:]]*$"); next }
        insec && index($0, key) == 1 {
            line = $0
            sub("^" key "[[:space:]]*=[[:space:]]*", "", line)
            print line
            exit
        }
    ' "$CONFIG"
}

cfg_str() { cfg_raw "$1" "$2" | sed 's/[[:space:]]*$//; s/^"//; s/"$//'; }

cfg_list() {
    cfg_raw "$1" "$2" \
        | sed 's/^\[//; s/\][[:space:]]*$//' \
        | tr ',' '\n' \
        | sed 's/^[[:space:]]*//; s/[[:space:]]*$//; s/^"//; s/"$//' \
        | grep -v '^$'
}

require_cfg() {
    if [ -z "$(cfg_raw "$1" "$2")" ]; then
        say "FATAL: $CONFIG is missing required key [$1].$2"
        exit 2
    fi
}

load_config() {
    [ -f "$CONFIG" ] || { say "FATAL: missing $CONFIG"; exit 2; }
    require_cfg gate ci_toolchain
    require_cfg gate baseline_version
    require_cfg gate satd_max
    require_cfg differential compile_budget_seconds
    require_cfg differential corpora
    require_cfg verbs help_only
    require_cfg verbs expect_nonzero
    CI_TOOLCHAIN=$(cfg_str gate ci_toolchain)
    BASELINE_VERSION=$(cfg_str gate baseline_version)
    SATD_MAX=$(cfg_str gate satd_max)
    COMPILE_BUDGET=$(cfg_str differential compile_budget_seconds)
    case "$SATD_MAX" in ''|*[!0-9]*) say "FATAL: gate.satd_max must be an integer"; exit 2;; esac
    case "$COMPILE_BUDGET" in ''|*[!0-9]*) say "FATAL: differential.compile_budget_seconds must be an integer"; exit 2;; esac
}

# help_only entries are "verb=reason"
help_only_reason() {
    cfg_list verbs help_only | awk -F= -v v="$1" '$1 == v { sub("^[^=]*=", ""); print; exit }'
}

is_help_only() { cfg_list verbs help_only | cut -d= -f1 | grep -qx -- "$1"; }

expect_nonzero_reason() {
    cfg_list verbs expect_nonzero | awk -F= -v v="$1" '$1 == v { sub("^[^=]*=", ""); print; exit }'
}

is_expect_nonzero() { cfg_list verbs expect_nonzero | cut -d= -f1 | grep -qx -- "$1"; }

verb_input() {
    local mapped
    mapped=$(cfg_str verbs "input_$(printf '%s' "$1" | tr ':-' '__')")
    [ -n "$mapped" ] && printf '%s\n' "$mapped" || printf 'hello.ruchy\n'
}

# --------------------------------------------------------------------------------------
# worker: one differential file (invoked through xargs as "$0 --worker-diff <file>")
# --------------------------------------------------------------------------------------

worker_diff() {
    local f="$1" bc hc bt ht br bo hr ho
    cd "$RG_ROOT" || return 0

    timeout 10 "$RG_BASE" check "$f" > /dev/null 2>&1; bc=$?
    timeout 10 "$RG_HEAD" check "$f" > /dev/null 2>&1; hc=$?

    bt=$(timeout 10 "$RG_BASE" transpile "$f" 2> /dev/null | sha_of_stdin)
    ht=$(timeout 10 "$RG_HEAD" transpile "$f" 2> /dev/null | sha_of_stdin)

    bo=$(timeout 10 "$RG_BASE" run "$f" 2> /dev/null); br=$?
    ho=$(timeout 10 "$RG_HEAD" run "$f" 2> /dev/null); hr=$?
    bo=$(printf '%s' "$bo" | sha_of_stdin)
    ho=$(printf '%s' "$ho" | sha_of_stdin)

    printf '%s|%s|%s|%s|%s|%s|%s|%s|%s\n' "$f" "$bc" "$hc" "$bt" "$ht" "$br" "$bo" "$hr" "$ho" \
        >> "$RG_OUT/results.$$"
}

if [ "${1:-}" = "--worker-diff" ]; then
    worker_diff "$2"
    exit 0
fi

# --------------------------------------------------------------------------------------
# stage 1 -- tests
# --------------------------------------------------------------------------------------

stage_tests() {
    hdr "STAGE 1/8  tests -- cargo test --lib -p ruchy"
    local exit_code
    (cd "$ROOT" && command cargo test --lib -p ruchy) 2>&1 | tail -12
    exit_code=${PIPESTATUS[0]}
    local status="PASS"
    [ "$exit_code" -eq 0 ] || status="FAIL"
    jq -n --arg s "$status" --argjson e "$exit_code" \
        '{status: $s, exit: $e}' > "$STAGES/tests.json"
    report tests "$status" "cargo test --lib -p ruchy exit=$exit_code"
}

# --------------------------------------------------------------------------------------
# stage 2 -- feature matrix, fmt, audit
# --------------------------------------------------------------------------------------

clippy_run() {
    local label="$1"; shift
    say "  clippy ($label) ..."
    (cd "$ROOT" && command cargo clippy --all-targets "$@" -- -D warnings) > "$WORK/clippy-$label.log" 2>&1
    local e=$?
    [ "$e" -eq 0 ] || tail -20 "$WORK/clippy-$label.log"
    say "  clippy ($label) exit=$e"
    return "$e"
}

stage_features() {
    hdr "STAGE 2/8  features -- clippy x3, fmt, audit"
    local d a m fmt_exit audit_exit

    clippy_run default; d=$?
    clippy_run all --all-features; a=$?
    clippy_run minimal --no-default-features --features minimal; m=$?

    if rustup run "$CI_TOOLCHAIN" cargo --version > /dev/null 2>&1; then
        (cd "$ROOT" && command cargo "+$CI_TOOLCHAIN" fmt --all -- --check) > "$WORK/fmt.log" 2>&1
        fmt_exit=$?
        say "  fmt (+$CI_TOOLCHAIN) exit=$fmt_exit"
    else
        (cd "$ROOT" && command cargo fmt --all -- --check) > "$WORK/fmt.log" 2>&1
        fmt_exit=$?
        add_warn "features: ci toolchain $CI_TOOLCHAIN not installed; fmt checked with $(rustc --version)"
        say "  fmt (stable) exit=$fmt_exit"
    fi
    [ "$fmt_exit" -eq 0 ] || tail -20 "$WORK/fmt.log"

    if command -v cargo-audit > /dev/null 2>&1 || cargo audit --version > /dev/null 2>&1; then
        (cd "$ROOT" && cargo audit) > "$WORK/audit.log" 2>&1
        audit_exit=$?
        [ "$audit_exit" -eq 0 ] || tail -30 "$WORK/audit.log"
        say "  cargo audit exit=$audit_exit"
    else
        audit_exit=127
        say "  cargo audit NOT INSTALLED -- cannot measure advisories"
    fi

    local status="PASS"
    if [ "$d" -ne 0 ] || [ "$a" -ne 0 ] || [ "$m" -ne 0 ] || [ "$fmt_exit" -ne 0 ] || [ "$audit_exit" -ne 0 ]; then
        status="FAIL"
    fi
    jq -n --arg s "$status" --argjson d "$d" --argjson a "$a" --argjson m "$m" \
        --argjson f "$fmt_exit" --argjson au "$audit_exit" \
        '{status: $s, default: $d, all: $a, minimal: $m, fmt: $f, audit: $au}' \
        > "$STAGES/features.json"
    report features "$status" "default=$d all=$a minimal=$m fmt=$fmt_exit audit=$audit_exit"
}

# --------------------------------------------------------------------------------------
# stage 3 -- verb surface
# --------------------------------------------------------------------------------------

build_head_binary() {
    if [ -n "${RUCHY_BIN:-}" ]; then
        HEAD_BIN="$RUCHY_BIN"
        say "  HEAD binary from RUCHY_BIN: $HEAD_BIN"
        return 0
    fi
    say "  building the HEAD binary (cargo build --release --bin ruchy) ..."
    (cd "$ROOT" && command cargo build --release --bin ruchy) > "$WORK/build.log" 2>&1
    local e=$?
    if [ "$e" -ne 0 ]; then
        tail -20 "$WORK/build.log"
        HEAD_BIN=""
        return 1
    fi
    HEAD_BIN="$TARGET_DIR/release/ruchy"
    return 0
}

verb_list() {
    "$HEAD_BIN" --help 2>&1 \
        | awk '/^Commands:/{f=1;next} /^Options:/{f=0} f && /^  [a-z]/{print $1}'
}

# run_verb <workdir> <verb> <mode> <args...>  -> prints "exit sha" and returns the exit
run_verb() {
    local wd="$1" verb="$2"; shift 3
    local out e
    out=$( (cd "$wd" && timeout 10 "$HEAD_BIN" "$verb" "$@" < /dev/null 2>/dev/null) )
    e=$?
    printf '%s %s\n' "$e" "$(printf '%s' "$out" | sha_of_stdin)"
    return "$e"
}

stage_verbs() {
    hdr "STAGE 3/8  verbs -- surface derived from the built binary"

    if ! build_head_binary; then
        jq -n '{status: "FAIL", total: null, pass: null, warn: null, fail: null, list: [],
                reason: "cargo build --release --bin ruchy failed; the verb surface could not be measured"}' \
            > "$STAGES/verbs.json"
        report verbs FAIL "the HEAD binary did not build; verb surface not measured"
        return 0
    fi

    local verbs
    verbs=$(verb_list)
    if [ -z "$verbs" ]; then
        jq -n '{status: "FAIL", total: null, pass: null, warn: null, fail: null, list: [],
                reason: "ruchy --help listed no commands"}' > "$STAGES/verbs.json"
        report verbs FAIL "ruchy --help listed no commands"
        return 0
    fi

    # verbs mutate their inputs and their cwd (fmt rewrites, doc/wasm/compile emit files),
    # so they run against a throwaway copy of tests/golden with the temp dir as cwd.
    local wd="$WORK/verbwork"
    mkdir -p "$wd"
    cp -r "$GOLDEN_DIR" "$wd/golden"

    local entries="$WORK/verbs.jsonl"
    : > "$entries"
    local total=0 npass=0 nwarn=0 nfail=0

    local v res e sha reason inputs inp vstatus
    for v in $verbs; do
        total=$((total + 1))
        if is_help_only "$v"; then
            reason=$(help_only_reason "$v")
            res=$(run_verb "$wd" "$v" golden --help)
            e=${res%% *}; sha=${res##* }
            jq -nc --arg v "$v" --arg m help_only --argjson e "$e" --arg sha "$sha" \
                --arg r "$reason" \
                '{verb: $v, mode: "help_only", input: null, exit: $e, stdout_sha256: $sha,
                  status: "WARN", reason: $r}' >> "$entries"
            nwarn=$((nwarn + 1))
            add_warn "verbs: $v is --help only ($reason)"
            continue
        fi

        case "$v" in
            check|parse|transpile|run) inputs=$(cd "$wd/golden" && find . -maxdepth 1 -name '*.ruchy' -printf '%P\n' | sort) ;;
            *) inputs=$(verb_input "$v") ;;
        esac

        vstatus=PASS
        for inp in $inputs; do
            res=$(run_verb "$wd" "$v" golden "golden/$inp")
            e=${res%% *}; sha=${res##* }
            local istatus=PASS ireason=""
            if [ "$e" -ne 0 ]; then
                if is_expect_nonzero "$v"; then
                    istatus=WARN
                    ireason=$(expect_nonzero_reason "$v")
                    add_warn "verbs: $v exits $e on golden/$inp by declaration ($ireason)"
                    [ "$vstatus" = PASS ] && vstatus=WARN
                else
                    istatus=FAIL
                    ireason="exit $e on golden/$inp with no [verbs].expect_nonzero entry"
                    vstatus=FAIL
                fi
            fi
            jq -nc --arg v "$v" --arg i "golden/$inp" --argjson e "$e" --arg sha "$sha" \
                --arg s "$istatus" --arg r "$ireason" \
                '{verb: $v, mode: "golden", input: $i, exit: $e, stdout_sha256: $sha,
                  status: $s, reason: (if $r == "" then null else $r end)}' >> "$entries"
        done
        case "$vstatus" in
            PASS) npass=$((npass + 1)) ;;
            WARN) nwarn=$((nwarn + 1)) ;;
            FAIL) nfail=$((nfail + 1)); say "  FAIL  verb $v" ;;
        esac
    done

    local status="PASS"
    [ "$nfail" -gt 0 ] && status="FAIL"
    jq -n --arg s "$status" --argjson t "$total" --argjson p "$npass" --argjson w "$nwarn" \
        --argjson f "$nfail" --slurpfile l "$entries" \
        '{status: $s, total: $t, pass: $p, warn: $w, fail: $f, list: $l}' \
        > "$STAGES/verbs.json"
    report verbs "$status" "total=$total pass=$npass warn=$nwarn fail=$nfail"
}

# --------------------------------------------------------------------------------------
# stage 4 -- three-way differential (and the transpile sub-receipt)
# --------------------------------------------------------------------------------------

resolve_baseline() {
    if [ -n "${RUCHY_BASELINE_BIN:-}" ] && [ -x "${RUCHY_BASELINE_BIN}" ]; then
        BASE_BIN="$RUCHY_BASELINE_BIN"
        say "  baseline from RUCHY_BASELINE_BIN: $BASE_BIN ($("$BASE_BIN" --version 2>&1 | head -1))"
        return 0
    fi
    local root="$TARGET_DIR/baseline"
    if [ -x "$root/bin/ruchy" ]; then
        BASE_BIN="$root/bin/ruchy"
        say "  baseline from cache: $BASE_BIN"
        return 0
    fi
    say "  installing the $BASELINE_VERSION baseline into $root ..."
    if ! command cargo install ruchy --version "$BASELINE_VERSION" --locked --root "$root" \
        > "$WORK/baseline-install.log" 2>&1 || [ ! -x "$root/bin/ruchy" ]; then
        tail -20 "$WORK/baseline-install.log"
        BASE_BIN=""
        return 1
    fi
    BASE_BIN="$root/bin/ruchy"
    return 0
}

collect_corpus() {
    local out="$1" d
    : > "$out"
    (cd "$ROOT" && find examples -name '*.ruchy' -type f \
        -not -path '*/target/*' -not -path '*/node_modules/*' | sort) >> "$out"
    for d in $(cfg_list differential corpora); do
        if [ -d "$ROOT/$d" ]; then
            (cd "$ROOT" && find "$d" -name '*.ruchy' -type f \
                -not -path '*/target/*' -not -path '*/node_modules/*' | sort) >> "$out"
        else
            add_warn "differential: corpus $d is not present next to the repo; it was not measured"
        fi
    done
}

known_listed() { # <file> <path>
    awk -v p="$2" '$1 == p { found = 1 } END { exit(found ? 0 : 1) }' "$1"
}

untracked_snapshot() { (cd "$ROOT" && git status --porcelain --untracked-files=all | sort); }

stage_differential() {
    hdr "STAGE 4/8  differential -- HEAD vs $BASELINE_VERSION over examples/ + sibling corpora"

    if [ -z "${HEAD_BIN:-}" ] || [ ! -x "${HEAD_BIN:-/nonexistent}" ]; then
        jq -n '{status: "FAIL", reason: "no HEAD binary; the differential was not measured"}' \
            > "$STAGES/differential.json"
        jq -n '{status: "FAIL", reason: "no HEAD binary; transpile output was not measured"}' \
            > "$STAGES/transpile.json"
        report differential FAIL "no HEAD binary; not measured"
        report transpile FAIL "no HEAD binary; not measured"
        return 0
    fi
    if ! resolve_baseline; then
        jq -n --arg v "$BASELINE_VERSION" \
            '{status: "FAIL", reason: ("no " + $v + " baseline binary; the differential was not measured")}' \
            > "$STAGES/differential.json"
        jq -n '{status: "FAIL", reason: "no baseline binary; transpile output was not measured"}' \
            > "$STAGES/transpile.json"
        report differential FAIL "no $BASELINE_VERSION baseline; not measured"
        report transpile FAIL "no baseline; not measured"
        return 0
    fi

    local list="$WORK/corpus.txt"
    collect_corpus "$list"
    local files
    files=$(wc -l < "$list" | tr -d ' ')
    say "  $files .ruchy files to compare (check exit, transpile text, run exit+stdout)"

    local before after
    before=$(untracked_snapshot)

    local outdir="$WORK/diffout"
    mkdir -p "$outdir"
    RG_ROOT="$ROOT" RG_BASE="$BASE_BIN" RG_HEAD="$HEAD_BIN" RG_OUT="$outdir" \
        xargs -a "$list" -P "$(nproc)" -n 1 -I '{}' "$SCRIPT_DIR/pre-release-gate.sh" --worker-diff '{}'
    cat "$outdir"/results.* 2>/dev/null | sort > "$WORK/results.txt"

    local measured
    measured=$(wc -l < "$WORK/results.txt" | tr -d ' ')
    if [ "$measured" -ne "$files" ]; then
        add_warn "differential: $measured of $files files produced a result row"
    fi

    # classify
    local both_pass=0 both_fail=0 fixed=0
    local checkreg="$WORK/check_regressions.txt" runreg="$WORK/run_regressions.txt"
    local stdoutdiff="$WORK/run_stdout_diffs.txt" nondet="$WORK/nondet.txt"
    local transdiff="$WORK/transpile_diffs.txt"
    : > "$checkreg"; : > "$runreg"; : > "$stdoutdiff"; : > "$nondet"; : > "$transdiff"
    local identical=0

    local f bc hc bt ht br bo hr ho
    while IFS='|' read -r f bc hc bt ht br bo hr ho; do
        [ -n "$f" ] || continue
        if [ "$bc" -eq 0 ] && [ "$hc" -eq 0 ]; then
            both_pass=$((both_pass + 1))
            if [ "$bt" = "$ht" ]; then
                identical=$((identical + 1))
            else
                printf '%s\n' "$f" >> "$transdiff"
            fi
        elif [ "$bc" -eq 0 ] && [ "$hc" -ne 0 ]; then
            printf '%s\n' "$f" >> "$checkreg"
        elif [ "$bc" -ne 0 ] && [ "$hc" -eq 0 ]; then
            fixed=$((fixed + 1))
        else
            both_fail=$((both_fail + 1))
        fi

        if [ "$br" -eq 0 ] && [ "$hr" -ne 0 ]; then
            printf '%s\n' "$f" >> "$runreg"
        elif [ "$br" -eq 0 ] && [ "$hr" -eq 0 ] && [ "$bo" != "$ho" ]; then
            # a baseline that disagrees with itself is nondeterminism, not a diff
            local r1 r2
            r1=$( (cd "$ROOT" && timeout 10 "$BASE_BIN" run "$f" 2>/dev/null) | sha_of_stdin)
            r2=$( (cd "$ROOT" && timeout 10 "$BASE_BIN" run "$f" 2>/dev/null) | sha_of_stdin)
            if [ "$r1" != "$r2" ] || [ "$r1" != "$bo" ]; then
                printf '%s\n' "$f" >> "$nondet"
            else
                printf '%s\n' "$f" >> "$stdoutdiff"
            fi
        fi
    done < "$WORK/results.txt"

    while read -r f; do
        add_warn "differential: $f is nondeterministic on the $BASELINE_VERSION baseline; stdout not compared"
    done < "$nondet"

    # known lists
    local unknown_break="$WORK/unknown_breaks.txt" unknown_fix="$WORK/unknown_fixes.txt"
    : > "$unknown_break"; : > "$unknown_fix"
    while read -r f; do
        known_listed "$KNOWN_BREAKS" "$f" || printf '%s\n' "$f" >> "$unknown_break"
    done < <(cat "$checkreg" "$runreg")
    while read -r f; do
        known_listed "$KNOWN_FIXES" "$f" || printf '%s\n' "$f" >> "$unknown_fix"
    done < "$stdoutdiff"

    # compiled-binary comparison over examples/, time-boxed
    local compiled_compared=0 compiled_skipped=0
    local budget_end=$(( $(date +%s) + COMPILE_BUDGET ))
    local cdir="$WORK/compile"
    mkdir -p "$cdir"
    local ex be he bout hout
    while read -r ex; do
        case "$ex" in examples/*) ;; *) continue ;; esac
        if [ "$(date +%s)" -ge "$budget_end" ]; then
            compiled_skipped=$((compiled_skipped + 1))
            continue
        fi
        (cd "$ROOT" && timeout 60 "$BASE_BIN" compile "$ex" -o "$cdir/base.bin" >/dev/null 2>&1); be=$?
        (cd "$ROOT" && timeout 60 "$HEAD_BIN" compile "$ex" -o "$cdir/head.bin" >/dev/null 2>&1); he=$?
        if [ "$be" -ne 0 ] || [ "$he" -ne 0 ]; then
            continue
        fi
        bout=$( (cd "$ROOT" && timeout 10 "$cdir/base.bin" 2>/dev/null) | sha_of_stdin)
        hout=$( (cd "$ROOT" && timeout 10 "$cdir/head.bin" 2>/dev/null) | sha_of_stdin)
        compiled_compared=$((compiled_compared + 1))
        if [ "$bout" != "$hout" ]; then
            if ! known_listed "$KNOWN_FIXES" "$ex"; then
                printf '%s\n' "$ex" >> "$unknown_fix"
                printf '%s\n' "$ex" >> "$stdoutdiff"
            fi
        fi
    done < <(grep '^examples/' "$WORK/results.txt" | cut -d'|' -f1)
    rm -f "$cdir/base.bin" "$cdir/head.bin"
    if [ "$compiled_skipped" -gt 0 ]; then
        add_warn "differential: the ${COMPILE_BUDGET}s compiled-binary budget ran out; $compiled_skipped examples were not compiled-compared"
    fi

    # the corpus runs from the repo root (as the P0 measurement did), so any artefact a
    # sample wrote into the tree is removed here -- a dirty tree would sink the clean room.
    after=$(untracked_snapshot)
    local stray p
    stray=$(comm -13 <(printf '%s\n' "$before") <(printf '%s\n' "$after") | sed 's/^?? //')
    for p in $stray; do
        add_warn "differential: running the corpus from the repo root created $p; the gate removed it"
        rm -f "$ROOT/$p" 2> /dev/null
    done

    local status="PASS"
    if [ -s "$unknown_break" ] || [ -s "$unknown_fix" ]; then
        status="FAIL"
        say "  unlisted regressions:"; sed 's/^/    /' "$unknown_break"
        say "  unlisted stdout diffs:"; sed 's/^/    /' "$unknown_fix"
    fi

    jq -n --arg s "$status" --argjson files "$files" --argjson bp "$both_pass" \
        --argjson bf "$both_fail" --argjson fixed "$fixed" \
        --argjson cc "$compiled_compared" --argjson cs "$compiled_skipped" \
        --rawfile cr "$checkreg" --rawfile rr "$runreg" --rawfile sd "$stdoutdiff" \
        --rawfile nd "$nondet" --rawfile ub "$unknown_break" --rawfile uf "$unknown_fix" \
        --arg kb "$(cut -d' ' -f1 "$KNOWN_BREAKS" | grep -v '^#' | grep -v '^$' | tr '\n' ' ')" \
        --arg kf "$(cut -d' ' -f1 "$KNOWN_FIXES" | grep -v '^#' | grep -v '^$' | tr '\n' ' ')" \
        '
        def lines: split("\n") | map(select(length > 0));
        {status: $s, files: $files, both_pass: $bp,
         check_regressions: ($cr | lines), run_regressions: ($rr | lines),
         run_stdout_diffs: ($sd | lines),
         known_breaks: ($kb | split(" ") | map(select(length > 0))),
         known_fixes: ($kf | split(" ") | map(select(length > 0))),
         unlisted_regressions: ($ub | lines), unlisted_stdout_diffs: ($uf | lines),
         nondeterministic: ($nd | lines),
         fixed: $fixed, both_fail: $bf,
         compiled_compared: $cc, compiled_skipped_budget: $cs}
        ' > "$STAGES/differential.json"
    report differential "$status" \
        "files=$files both_pass=$both_pass both_fail=$both_fail fixed=$fixed check_reg=$(wc -l < "$checkreg" | tr -d ' ') run_reg=$(wc -l < "$runreg" | tr -d ' ') stdout_diff=$(wc -l < "$stdoutdiff" | tr -d ' ') compiled=$compiled_compared skipped=$compiled_skipped"

    jq -n --argjson i "$identical" --rawfile td "$transdiff" \
        '{status: "PASS", identical: $i,
          differs: ($td | split("\n") | map(select(length > 0)))}' \
        > "$STAGES/transpile.json"
    report transpile PASS \
        "identical=$identical differs=$(wc -l < "$transdiff" | tr -d ' ') (informational)"
}

# --------------------------------------------------------------------------------------
# stage 5/6 -- clean room and package hygiene
# --------------------------------------------------------------------------------------

crate_version() {
    awk '/^\[workspace\.package\]/{f=1;next} /^\[/{f=0} f && /^version[[:space:]]*=/ {
        gsub(/^version[[:space:]]*=[[:space:]]*"/, ""); gsub(/"$/, ""); print; exit }' "$ROOT/Cargo.toml"
}

stage_clean_room() {
    hdr "STAGE 5/8  clean_room -- cargo package, then --locked and unlocked builds"
    local version
    version="$CRATE_VERSION"

    if [ -n "$(cd "$ROOT" && git status --porcelain)" ]; then
        (cd "$ROOT" && git status --porcelain | head -10)
        jq -n '{status: "FAIL", locked_exit: null, unlocked_exit: null, binary_version: null,
                reason: "the working tree is dirty; cargo package -p ruchy was not run"}' \
            > "$STAGES/clean_room.json"
        jq -n '{status: "FAIL", files: null, bytes_compressed: null, colon_paths: null,
                has_lock: null, reason: "the working tree is dirty; cargo package -p ruchy was not run"}' \
            > "$STAGES/package.json"
        report clean_room FAIL "dirty working tree; cargo package not run (never --allow-dirty)"
        report package FAIL "dirty working tree; cargo package not run"
        return 0
    fi

    say "  cargo package -p ruchy (with verification build) ..."
    (cd "$ROOT" && command cargo package -p ruchy) > "$WORK/package.log" 2>&1
    local pkg_exit=$?
    if [ "$pkg_exit" -ne 0 ]; then
        tail -25 "$WORK/package.log"
        jq -n --argjson e "$pkg_exit" \
            '{status: "FAIL", locked_exit: null, unlocked_exit: null, binary_version: null,
              reason: ("cargo package -p ruchy exited " + ($e | tostring))}' \
            > "$STAGES/clean_room.json"
        jq -n --argjson e "$pkg_exit" \
            '{status: "FAIL", files: null, bytes_compressed: null, colon_paths: null, has_lock: null,
              reason: ("cargo package -p ruchy exited " + ($e | tostring))}' \
            > "$STAGES/package.json"
        report clean_room FAIL "cargo package -p ruchy exit=$pkg_exit"
        report package FAIL "cargo package -p ruchy exit=$pkg_exit"
        return 0
    fi

    stage_package "$version"

    local crate="$TARGET_DIR/package/ruchy-$version.crate"
    local ex="$WORK/cleanroom"
    mkdir -p "$ex"
    tar xzf "$crate" -C "$ex" || {
        jq -n '{status: "FAIL", locked_exit: null, unlocked_exit: null, binary_version: null,
                reason: "the .crate could not be extracted"}' > "$STAGES/clean_room.json"
        report clean_room FAIL "the .crate could not be extracted"
        return 0
    }
    local src="$ex/ruchy-$version"

    say "  clean-room build 1/2: --locked, fresh CARGO_HOME ..."
    local home1 locked_exit
    home1=$(mktemp -d "$WORK/cargohome1.XXXX")
    (cd "$src" && env -u CARGO_TARGET_DIR CARGO_HOME="$home1" \
        command cargo build --release --locked) > "$WORK/cleanroom-locked.log" 2>&1
    locked_exit=$?
    [ "$locked_exit" -eq 0 ] || tail -25 "$WORK/cleanroom-locked.log"
    say "  clean-room --locked exit=$locked_exit"

    local bin_version=""
    if [ -x "$src/target/release/ruchy" ]; then
        bin_version=$("$src/target/release/ruchy" --version 2>&1 | awk '{print $NF}')
        say "  clean-room binary reports version: $bin_version"
    fi

    say "  clean-room build 2/2: unlocked (fresh resolution), fresh CARGO_HOME ..."
    local ex2="$WORK/cleanroom2" home2 unlocked_exit
    mkdir -p "$ex2"
    tar xzf "$crate" -C "$ex2"
    local src2="$ex2/ruchy-$version"
    home2=$(mktemp -d "$WORK/cargohome2.XXXX")
    (cd "$src2" && env -u CARGO_TARGET_DIR CARGO_HOME="$home2" \
        command cargo build --release) > "$WORK/cleanroom-unlocked.log" 2>&1
    unlocked_exit=$?
    [ "$unlocked_exit" -eq 0 ] || tail -25 "$WORK/cleanroom-unlocked.log"
    say "  clean-room unlocked exit=$unlocked_exit"

    local bin_version2=""
    if [ -x "$src2/target/release/ruchy" ]; then
        bin_version2=$("$src2/target/release/ruchy" --version 2>&1 | awk '{print $NF}')
    fi

    local status="PASS"
    if [ "$locked_exit" -ne 0 ] || [ "$unlocked_exit" -ne 0 ] \
        || [ "$bin_version" != "$version" ] || [ "$bin_version2" != "$version" ]; then
        status="FAIL"
    fi
    jq -n --arg s "$status" --argjson l "$locked_exit" --argjson u "$unlocked_exit" \
        --arg bv "$bin_version" --arg bv2 "$bin_version2" --arg v "$version" \
        '{status: $s, locked_exit: $l, unlocked_exit: $u, binary_version: $bv,
          unlocked_binary_version: $bv2, expected_version: $v}' \
        > "$STAGES/clean_room.json"
    report clean_room "$status" \
        "locked=$locked_exit unlocked=$unlocked_exit version=$bin_version/$bin_version2 expected=$version"
}

stage_package() {
    hdr "STAGE 6/8  package -- hygiene of the packaged crate"
    local version="$1"
    local crate="$TARGET_DIR/package/ruchy-$version.crate"
    local listing="$WORK/pkg-list.txt"
    (cd "$ROOT" && command cargo package --list -p ruchy) > "$listing" 2> "$WORK/pkg-list.err"
    local list_exit=$?
    if [ "$list_exit" -ne 0 ] || [ ! -f "$crate" ]; then
        tail -10 "$WORK/pkg-list.err"
        jq -n '{status: "FAIL", files: null, bytes_compressed: null, colon_paths: null,
                has_lock: null, reason: "cargo package --list failed or the .crate is missing"}' \
            > "$STAGES/package.json"
        report package FAIL "cargo package --list exit=$list_exit, crate present=$([ -f "$crate" ] && echo yes || echo no)"
        return 0
    fi
    local files bytes colons has_lock
    files=$(grep -c . "$listing")
    bytes=$(stat -c %s "$crate")
    colons=$(grep -c ':' "$listing")
    if grep -qx 'Cargo.lock' "$listing"; then has_lock=true; else has_lock=false; fi

    local status="PASS"
    [ "$colons" -eq 0 ] || status="FAIL"
    [ "$bytes" -le 10485760 ] || status="FAIL"
    [ "$has_lock" = true ] || status="FAIL"
    jq -n --arg s "$status" --argjson f "$files" --argjson b "$bytes" --argjson c "$colons" \
        --argjson h "$has_lock" \
        '{status: $s, files: $f, bytes_compressed: $b, colon_paths: $c, has_lock: $h}' \
        > "$STAGES/package.json"
    report package "$status" \
        "files=$files bytes_compressed=$bytes (limit 10485760) colon_paths=$colons has_lock=$has_lock"
}

# --------------------------------------------------------------------------------------
# stage 7 -- SATD
# --------------------------------------------------------------------------------------

stage_satd() {
    hdr "STAGE 7/8  satd -- pmat analyze satd --path src"
    if ! command -v pmat > /dev/null 2>&1; then
        jq -n --argjson m "$SATD_MAX" \
            '{status: "FAIL", count: null, max: $m, reason: "pmat is not on PATH; SATD was not measured"}' \
            > "$STAGES/satd.json"
        report satd FAIL "pmat is not installed; SATD not measured"
        return 0
    fi
    (cd "$ROOT" && NO_COLOR=1 pmat analyze satd --path src) > "$WORK/satd.log" 2>&1
    local e=$? count
    count=$(sed -n 's/^Total violations:[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$WORK/satd.log" | head -1)
    if [ -z "$count" ]; then
        tail -15 "$WORK/satd.log"
        jq -n --argjson m "$SATD_MAX" --argjson e "$e" \
            '{status: "FAIL", count: null, max: $m,
              reason: ("pmat analyze satd exited " + ($e | tostring) + " without a \"Total violations\" line; SATD was not measured")}' \
            > "$STAGES/satd.json"
        report satd FAIL "pmat analyze satd produced no violation count (exit $e); not measured"
        return 0
    fi
    local status="PASS"
    [ "$count" -le "$SATD_MAX" ] || status="FAIL"
    jq -n --arg s "$status" --argjson c "$count" --argjson m "$SATD_MAX" \
        '{status: $s, count: $c, max: $m}' > "$STAGES/satd.json"
    grep -A5 '^Top Violations' "$WORK/satd.log" | sed 's/^/  /'
    report satd "$status" "count=$count max=$SATD_MAX"
}

# --------------------------------------------------------------------------------------
# stage 8 -- receipt
# --------------------------------------------------------------------------------------

stage_receipt() {
    hdr "STAGE 8/8  receipt -- $RECEIPT"
    mkdir -p "$EVIDENCE_DIR"
    local head_sha
    head_sha=$(cd "$ROOT" && git rev-parse HEAD 2>/dev/null || echo unknown)

    jq -n \
        --arg version "$CRATE_VERSION" \
        --arg head "$head_sha" \
        --arg baseline "$BASELINE_VERSION" \
        --slurpfile tests "$STAGES/tests.json" \
        --slurpfile features "$STAGES/features.json" \
        --slurpfile verbs "$STAGES/verbs.json" \
        --slurpfile differential "$STAGES/differential.json" \
        --slurpfile transpile "$STAGES/transpile.json" \
        --slurpfile clean_room "$STAGES/clean_room.json" \
        --slurpfile package "$STAGES/package.json" \
        --slurpfile satd "$STAGES/satd.json" \
        --rawfile warns "$WARN_FILE" \
        '
        {schema_version: 1, version: $version, head: $head, baseline: $baseline,
         stages: {tests: $tests[0], features: $features[0], verbs: $verbs[0],
                  differential: $differential[0], transpile: $transpile[0],
                  clean_room: $clean_room[0], package: $package[0], satd: $satd[0]},
         warns: ($warns | split("\n") | map(select(length > 0)))}
        | .verdict = (if ([.stages[].status] | any(. == "FAIL")) then "no-go" else "go" end)
        ' > "$RECEIPT"
    local e=$?
    if [ "$e" -ne 0 ]; then
        say "FATAL: the receipt could not be assembled (jq exit $e)"
        exit 2
    fi
    say "  wrote $RECEIPT"
}

print_summary() {
    local verdict
    verdict=$(jq -r '.verdict' "$RECEIPT")
    hdr "SUMMARY  ruchy pre-release gate v2 (PMAT-096)"
    say "  version   $(jq -r '.version' "$RECEIPT")"
    say "  head      $(jq -r '.head' "$RECEIPT")"
    say "  baseline  $(jq -r '.baseline' "$RECEIPT")"
    say ""
    jq -r '.stages | to_entries[] | "  \(.value.status | (. + "    ")[0:5]) \(.key)"' "$RECEIPT"
    say ""
    say "  warns ($(jq -r '.warns | length' "$RECEIPT")):"
    jq -r '.warns[] | "    - " + .' "$RECEIPT"
    say ""
    say "  VERDICT: $verdict"
    say "  receipt: $RECEIPT"
    say ""
    [ "$verdict" = "go" ]
}

# --------------------------------------------------------------------------------------
# main
# --------------------------------------------------------------------------------------

main() {
    need jq
    need sha256sum
    need timeout
    need git
    load_config
    [ -f "$KNOWN_BREAKS" ] || { say "FATAL: missing $KNOWN_BREAKS"; exit 2; }
    [ -f "$KNOWN_FIXES" ] || { say "FATAL: missing $KNOWN_FIXES"; exit 2; }
    [ -d "$GOLDEN_DIR" ] || { say "FATAL: missing $GOLDEN_DIR"; exit 2; }
    CRATE_VERSION=$(crate_version)
    [ -n "$CRATE_VERSION" ] || { say "FATAL: no [workspace.package] version in Cargo.toml"; exit 2; }

    hdr "ruchy pre-release gate v2 -- PMAT-096"
    say "  root      $ROOT"
    say "  version   $CRATE_VERSION"
    say "  baseline  $BASELINE_VERSION"
    say "  config    $CONFIG"
    say "  started   $(date -Is)"

    stage_tests
    stage_features
    stage_verbs
    stage_differential
    stage_clean_room
    stage_satd
    stage_receipt

    if print_summary; then
        exit 0
    fi
    exit 1
}

main "$@"
