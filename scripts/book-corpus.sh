#!/usr/bin/env bash
# PMAT-136 -- run the BUILT binary over the examples and the book corpora.
#
# Subject: this tree's examples/ (gating) plus the sibling book repos
# (informational -- they are other repos' trees and their red is their ticket,
# but a MISSING corpus is always recorded, never silently skipped).
#
# Each file is checked with `timeout 10 <bin> check <file>`; the timeout is the
# hang guard (CLAUDE.md, Ruchy Execution Safety Protocol).
#
# Usage:
#   bash scripts/book-corpus.sh            # build (or $RUCHY_BIN) and check every file
#   bash scripts/book-corpus.sh --dry-run  # list the files and corpora, build nothing
#
# Environment:
#   RUCHY_BIN         path to the binary to exercise (else cargo build --release)
#   CARGO_TARGET_DIR  honoured when locating the built binary
#
# Exit: 1 if any file under examples/ fails `check` (or the build fails), else 0.

set -u -o pipefail

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
SIBLINGS="ruchy-book ruchy-cookbook rosetta-ruchy ruchy-repl-demos"
MISSING=""
PRESENT=""

usage() {
    echo "usage: bash scripts/book-corpus.sh [--dry-run]" >&2
}

# Print every *.ruchy file under a directory, sorted; nothing if it is absent.
corpus_files() {
    local dir=$1
    [ -d "$dir" ] || return 0
    find "$dir" -type f -name '*.ruchy' | sort
}

# Split the sibling corpora into PRESENT and MISSING. Called in the current
# shell, never in a command substitution: a subshell would lose MISSING, and a
# corpus that was never looked at would read as a corpus with no findings.
scan_siblings() {
    local name dir parent
    parent=$(dirname "$ROOT")
    for name in $SIBLINGS; do
        dir="$parent/$name"
        if [ -d "$dir" ]; then
            PRESENT="$PRESENT $dir"
        else
            MISSING="$MISSING $name"
        fi
    done
}

# The binary to exercise: $RUCHY_BIN, else a release build of this tree.
resolve_bin() {
    local target_dir
    if [ -n "${RUCHY_BIN:-}" ]; then
        echo "$RUCHY_BIN"
        return 0
    fi
    target_dir=${CARGO_TARGET_DIR:-$ROOT/target}
    (cd "$ROOT" && cargo build --release) >&2 || return 1
    echo "$target_dir/release/ruchy"
}

# Check every file on stdin; report each failure on stderr; echo "<pass> <fail>".
check_files() {
    local bin=$1 label=$2 pass=0 fail=0 file
    while IFS= read -r file; do
        [ -n "$file" ] || continue
        if timeout 10 "$bin" check "$file" > /dev/null 2>&1; then
            pass=$((pass + 1))
        else
            fail=$((fail + 1))
            echo "  fail($label) $file" >&2
        fi
    done
    echo "$pass $fail"
}

summary() {
    local files=$1 pass=$2 fail=$3 missing
    missing=$(echo "$MISSING" | tr -s ' ' ',' | sed 's/^,//; s/,$//')
    echo "files=$files pass=$pass fail=$fail missing_corpora=[$missing]"
}

dry_run() {
    local dir all
    all=$(corpus_files "$ROOT/examples")
    scan_siblings
    for dir in $PRESENT; do
        echo "corpus $dir"
        all="$all"$'\n'$(corpus_files "$dir")
    done
    printf '%s\n' "$all" | sed '/^$/d'
    # not-run, not 0: a dry run has measured nothing, and a zero would read
    # as a measurement that found nothing wrong.
    summary "$(printf '%s\n' "$all" | sed '/^$/d' | wc -l)" not-run not-run
    exit 0
}

full_run() {
    local bin dir counts examples_fail total_pass=0 total_fail=0 files=0
    bin=$(resolve_bin) || { echo "FAIL cannot build or locate the ruchy binary"; exit 1; }
    scan_siblings
    counts=$(corpus_files "$ROOT/examples" | check_files "$bin" examples)
    examples_fail=$(echo "$counts" | cut -d' ' -f2)
    total_pass=$(echo "$counts" | cut -d' ' -f1)
    total_fail=$examples_fail
    for dir in $PRESENT; do
        counts=$(corpus_files "$dir" | check_files "$bin" "$(basename "$dir")")
        total_pass=$((total_pass + $(echo "$counts" | cut -d' ' -f1)))
        total_fail=$((total_fail + $(echo "$counts" | cut -d' ' -f2)))
    done
    files=$((total_pass + total_fail))
    summary "$files" "$total_pass" "$total_fail"
    [ "$examples_fail" -eq 0 ] || exit 1
    exit 0
}

case ${1:-run} in
    --dry-run) dry_run ;;
    run) full_run ;;
    *) usage; exit 2 ;;
esac
