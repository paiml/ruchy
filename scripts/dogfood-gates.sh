#!/usr/bin/env bash
# PMAT-136 -- run the release gates this repo DECLARES, one row each.
#
# The dogfood protocol never carries a copy of a repo's own gates. It reads them
# from Cargo.toml:
#
#   [package.metadata.dogfood]
#   gates = ["scripts/release-policy.sh", "scripts/wasm32-build.sh", ...]
#
# Both vacuity guards are hard FAILs, never skips:
#   * a declared script that does not exist is a DELETED gate -> FAIL <script> missing
#   * an empty or absent gates list is a clean sweep over an empty set
#     -> FAIL no gates declared
#
# Usage:
#   bash scripts/dogfood-gates.sh --list   # print the declared gates, one per line
#   bash scripts/dogfood-gates.sh          # run each, print "PASS|FAIL <script> <seconds>"
#
# Exit: 0 every gate passed, 1 any FAIL, 2 usage error.
# The gates list is read with awk/sed -- no TOML library, so this script has no
# build step and runs before anything is compiled.

set -u -o pipefail

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
MANIFEST="$ROOT/Cargo.toml"

usage() {
    echo "usage: bash scripts/dogfood-gates.sh [--list] [--manifest <Cargo.toml>]" >&2
}

# Print the strings of the `gates` array in [package.metadata.dogfood], one per line.
# Comments are dropped first: a `]` or a quoted name inside a comment must neither end
# the array early nor become a phantom gate (quorum on #220).
extract_gates() {
    awk '
        { sub(/#.*$/, "") }
        /^[[:space:]]*\[/ { in_table = ($0 ~ /^[[:space:]]*\[package\.metadata\.dogfood\]/); next }
        !in_table { next }
        /^[[:space:]]*gates[[:space:]]*=/ { collecting = 1 }
        collecting { buf = buf $0; if ($0 ~ /\]/) { collecting = 0 } }
        END { print buf }
    ' "$MANIFEST" | grep -o '"[^"]*"' | sed 's/"//g'
}

# Run one gate. Prints its row; returns 0 on PASS, 1 on FAIL.
run_gate() {
    local gate=$1
    local start log status elapsed
    if [ ! -f "$ROOT/$gate" ]; then
        echo "FAIL $gate missing"
        return 1
    fi
    start=$SECONDS
    log=$(bash "$ROOT/$gate" 2>&1)
    status=$?
    elapsed=$((SECONDS - start))
    if [ "$status" -eq 0 ]; then
        echo "PASS $gate ${elapsed}s"
        return 0
    fi
    echo "FAIL $gate ${elapsed}s"
    printf '%s\n' "$log" | tail -20 | sed 's/^/    | /'
    return 1
}

main() {
    local mode=run gates failed gate
    while [ $# -gt 0 ]; do
        case "$1" in
            --list) mode="list"; shift ;;
            --manifest) MANIFEST=${2:-}; [ -n "$MANIFEST" ] || { usage; exit 2; }; shift 2 ;;
            run) shift ;;
            *) usage; exit 2 ;;
        esac
    done

    gates=$(extract_gates)
    if [ -z "$gates" ]; then
        echo "FAIL no gates declared"
        exit 1
    fi

    if [ "$mode" = "list" ]; then
        printf '%s\n' "$gates"
        exit 0
    fi

    failed=0
    while IFS= read -r gate; do
        [ -n "$gate" ] || continue
        run_gate "$gate" || failed=$((failed + 1))
    done <<< "$gates"

    echo "gates=$(printf '%s\n' "$gates" | wc -l) failed=$failed"
    [ "$failed" -eq 0 ] || exit 1
    exit 0
}

main "$@"
