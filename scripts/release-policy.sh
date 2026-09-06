#!/usr/bin/env bash
# PMAT-135 -- release policy gate. CI is the gate; the operator is the publisher.
#
# "CI held a publish credential" is the mechanism behind the 403 that stopped the
# 5.0.0-beta.2 release. "Producer is never the gate": a workflow that gates AND
# publishes is one program attesting to itself. Manual publish restores the
# separation -- CI is the gate (build, clean-room, dogfood, tag, prerelease); the
# operator is the publisher, from their machine, with a scoped token. Consequence:
# no token in the repo, and no crate push from any workflow.
#
# Gates. Each is a function, each prints exactly one of
#     PASS <gate>
#     FAIL <gate>: <reason>
# and any FAIL exits 1. A gate never skips silently: a tool it needs but cannot find
# is "FAIL <gate>: <tool> not installed", an unauthenticated gh is a FAIL, not a pass.
#
#   no-publish-in-ci    no non-comment publish command under .github/workflows/
#   no-registry-secret  no CARGO_REGISTRY_TOKEN / CRATES_TOKEN on the repo or the org
#   receipts-at-tag     the release for --tag carries clean-room, dogfood and
#                       fresh-container receipts naming the tag's 8-char commit SHA
#
# Phase. no-publish-in-ci asks about this tree and runs in CI (release.yml) and in
# the operator dogfood. no-registry-secret needs an admin token GITHUB_TOKEN is not;
# receipts-at-tag asks about receipts attached AFTER the publish (the fresh-container
# receipt installs from crates.io). Those two run only from the operator machine, as
# a [package.metadata.dogfood] gate, and their RED before that phase is "not yet
# measured", never a skip.
#
# Usage:
#   bash scripts/release-policy.sh [--tag <tag>] [--only <gate>] [--gates-dir <dir>]
#   bash scripts/release-policy.sh --self-test
#
# --self-test drives every gate against tests/fixtures/release-policy/ and exits 1
# unless each falsifier turns RED. The asset check reads names on stdin so the
# fixtures can drive it with no network.

set -euo pipefail

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
FIXTURES="$ROOT/tests/fixtures/release-policy"

TAG=""
ONLY=""
GATES_DIR=""   # --gates-dir <dir>: scan one directory (fixtures); default: the CI reach set below
SELF_TEST=0

# Whitespace-tolerant: `cargo  publish` (two spaces) defeated a literal match (quorum on #219).
PUBLISH_RE='cargo[[:space:]]+publish'

# --------------------------------------------------------------------------------------
# reporting
# --------------------------------------------------------------------------------------

pass() { printf 'PASS %s\n' "$1"; }
fail() { printf 'FAIL %s: %s\n' "$1" "$2"; }

require_tool() {
    local gate="$1" tool="$2"
    if ! command -v "$tool" >/dev/null 2>&1; then
        fail "$gate" "$tool not installed"
        return 1
    fi
}

# --------------------------------------------------------------------------------------
# gate: no-publish-in-ci
# --------------------------------------------------------------------------------------

# file:line of every non-comment publish command under a directory.
# The workflow files under a root: .github/ when present (real repo), else every YAML
# under the root (fixtures are flat).
workflow_files() {
    local root="$1"
    if [ -d "$root/.github" ]; then
        find "$root/.github" -type f \( -name '*.yml' -o -name '*.yaml' \) 2> /dev/null
    else
        find "$root" -maxdepth 3 -type f \( -name '*.yml' -o -name '*.yaml' \) 2> /dev/null
    fi
}

# Files CI can reach from those workflows: the workflows themselves and every
# repo script a run step names (`scripts/...`). Listed once each.
reach_files() {
    local root="$1" wf s
    wf=$(workflow_files "$root")
    printf '%s\n' "$wf"
    printf '%s\n' "$wf" | xargs -r grep -hoE 'scripts/[A-Za-z0-9_./-]+' 2> /dev/null | sort -u | while IFS= read -r s; do
        [ -f "$root/$s" ] && printf '%s\n' "$root/$s"
    done
}

# A publish line: matches PUBLISH_RE, is not a comment, and is not a --dry-run.
publish_lines() {
    awk -v re="$PUBLISH_RE" '
        {
            rest = $0
            sub(/^[^:]*:[0-9]+:/, "", rest)
            sub(/^[ \t]+/, "", rest)
            if (rest ~ /^#/) next
            if (rest !~ re) next
            if (rest ~ /--dry-run/) next
            match($0, /^[^:]*:[0-9]+:/)
            print substr($0, 1, RLENGTH - 1)
        }'
}

# Makefile recipes of the targets the workflows call (`make <target>`), so a publish
# hidden behind a make target is still a publish from CI.
makefile_recipe_hits() {
    local root="$1" t
    [ -f "$root/Makefile" ] || return 0
    workflow_files "$root" | xargs -r grep -hoE '(^|[^A-Za-z0-9_-])make[[:space:]]+[A-Za-z0-9_-]+' 2> /dev/null \
        | awk '{print $NF}' | sort -u | while IFS= read -r t; do
        awk -v t="$t" '
            $0 ~ "^" t ":" { p = 1; next }
            p && /^\t/ { printf "%s:%d:%s\n", FILENAME, NR, $0; next }
            p { p = 0 }' "$root/Makefile"
    done | publish_lines
}

publish_hits() {
    local root="$1"
    {
        # -H: a single file would otherwise print no filename and the line parser would drop it
        reach_files "$root" | xargs -r grep -HnE "$PUBLISH_RE" 2> /dev/null | grep -v "release-policy.sh:" | publish_lines
        makefile_recipe_hits "$root"
    } | sort -u
}

gate_no_publish_in_ci() {
    local gate="no-publish-in-ci" root="$1" hits
    if [ ! -d "$root" ]; then
        fail "$gate" "root $root does not exist"
        return 1
    fi
    if [ -z "$(workflow_files "$root")" ]; then
        fail "$gate" "no workflow files under $root (a gate over nothing is not a pass)"
        return 1
    fi
    hits=$(publish_hits "$root")
    if [ -n "$hits" ]; then
        fail "$gate" "CI publishes a crate at $(printf '%s' "$hits" | tr '\n' ' ')"
        return 1
    fi
    pass "$gate"
}

# --------------------------------------------------------------------------------------
# gate: no-registry-secret
# --------------------------------------------------------------------------------------

# Repository secrets, plus the paiml org's when that listing is available.
secret_listing() {
    gh secret list 2>/dev/null || return 1
    gh secret list --org paiml 2>/dev/null || true
}

gate_no_registry_secret() {
    local gate="no-registry-secret" listing found
    require_tool "$gate" gh || return 1
    if ! gh auth status >/dev/null 2>&1; then
        fail "$gate" "gh not authenticated"
        return 1
    fi
    if ! listing=$(secret_listing); then
        fail "$gate" "gh secret list failed (no admin read on the repository secrets)"
        return 1
    fi
    found=$(printf '%s\n' "$listing" |
        grep -Eo 'CARGO_REGISTRY_TOKEN|CRATES_TOKEN' | sort -u | tr '\n' ' ' || true)
    if [ -n "$found" ]; then
        fail "$gate" "a registry credential is still configured: $found"
        return 1
    fi
    pass "$gate"
}

# --------------------------------------------------------------------------------------
# gate: receipts-at-tag
# --------------------------------------------------------------------------------------

# $1 = 8-char commit SHA prefix; asset names on stdin; prints the missing receipt kinds.
missing_receipts() {
    local sha="$1" names missing=""
    names=$(cat)
    printf '%s\n' "$names" | grep -Eq "^clean-room-.*${sha}" || missing="${missing}clean-room "
    printf '%s\n' "$names" |
        grep -Eq "^(dogfood-.*${sha}|receipt-.*${sha}.*[.]json)" || missing="${missing}dogfood "
    printf '%s\n' "$names" |
        grep -Eq "^fresh-container-.*${sha}" || missing="${missing}fresh-container "
    printf '%s' "$missing"
}

gate_receipts_at_tag() {
    local gate="receipts-at-tag" tag="$1" sha assets missing
    require_tool "$gate" gh || return 1
    require_tool "$gate" jq || return 1
    if ! sha=$(git -C "$ROOT" rev-parse "${tag}^{commit}" 2>/dev/null); then
        fail "$gate" "tag $tag does not resolve to a commit"
        return 1
    fi
    sha=${sha:0:8}
    if ! assets=$(gh release view "$tag" --json assets --jq '.assets[].name' 2>/dev/null); then
        fail "$gate" "no GitHub release for $tag"
        return 1
    fi
    missing=$(printf '%s\n' "$assets" | missing_receipts "$sha")
    if [ -n "$missing" ]; then
        fail "$gate" "release $tag ($sha) is missing receipts: $missing"
        return 1
    fi
    pass "$gate"
}

# --------------------------------------------------------------------------------------
# self-test: every falsifier must turn RED
# --------------------------------------------------------------------------------------

expect_red() {
    local label="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        printf 'FAIL self-test: %s did not turn RED\n' "$label"
        return 1
    fi
    printf 'PASS self-test: %s turns RED\n' "$label"
}

expect_green() {
    local label="$1"
    shift
    if ! "$@" >/dev/null 2>&1; then
        printf 'FAIL self-test: %s did not stay GREEN\n' "$label"
        return 1
    fi
    printf 'PASS self-test: %s stays GREEN\n' "$label"
}

# Predicate form of receipts-at-tag, driven by a fixture asset listing.
receipts_complete() {
    local sha="$1" listing="$2" missing
    missing=$(missing_receipts "$sha" <"$listing")
    [ -z "$missing" ]
}

self_test_publish() {
    local rc=0
    expect_red "no-publish-in-ci on a workflow that publishes" \
        gate_no_publish_in_ci "$FIXTURES/with-publish" || rc=1
    expect_green "no-publish-in-ci on a workflow that only builds" \
        gate_no_publish_in_ci "$FIXTURES/without-publish" || rc=1
    expect_red "no-publish-in-ci on a missing workflow directory" \
        gate_no_publish_in_ci "$FIXTURES/no-such-directory" || rc=1
    expect_red "no-publish-in-ci on a two-space cargo  publish" \
        gate_no_publish_in_ci "$FIXTURES/with-publish-two-spaces" || rc=1
    expect_red "no-publish-in-ci on a publish inside a script the workflow runs" \
        gate_no_publish_in_ci "$FIXTURES/with-publish-in-script" || rc=1
    expect_red "no-publish-in-ci on a publish inside a make target the workflow calls" \
        gate_no_publish_in_ci "$FIXTURES/with-publish-via-make" || rc=1
    return "$rc"
}

self_test_secret() {
    expect_red "no-registry-secret on a missing tool" \
        require_tool no-registry-secret gh-does-not-exist
}

self_test_receipts() {
    local rc=0 sha="deadbeef"
    expect_red "receipts-at-tag on a release missing the fresh-container receipt" \
        receipts_complete "$sha" "$FIXTURES/assets-missing-fresh-container.txt" || rc=1
    expect_green "receipts-at-tag on a release carrying all three receipts" \
        receipts_complete "$sha" "$FIXTURES/assets-complete.txt" || rc=1
    return "$rc"
}

self_test() {
    local rc=0
    self_test_publish || rc=1
    self_test_secret || rc=1
    self_test_receipts || rc=1
    if [ "$rc" -eq 0 ]; then
        pass "self-test"
    else
        fail "self-test" "a falsifier did not turn RED"
    fi
    return "$rc"
}

# --------------------------------------------------------------------------------------
# driver
# --------------------------------------------------------------------------------------

usage() {
    sed -n '2,28p' "$0"
}

parse_args() {
    while [ "$#" -gt 0 ]; do
        case "$1" in
        --tag)
            TAG="${2:-}"
            shift 2
            ;;
        --only)
            ONLY="${2:-}"
            shift 2
            ;;
        --gates-dir)
            GATES_DIR="${2:-}"
            shift 2
            ;;
        --self-test)
            SELF_TEST=1
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            printf 'FAIL release-policy: unknown argument %s\n' "$1"
            exit 2
            ;;
        esac
    done
}

wants_gate() { [ -z "$ONLY" ] || [ "$ONLY" = "$1" ]; }

run_gates() {
    local rc=0
    if wants_gate no-publish-in-ci; then
        gate_no_publish_in_ci "${GATES_DIR:-$ROOT}" || rc=1
    fi
    if wants_gate no-registry-secret; then
        gate_no_registry_secret || rc=1
    fi
    if [ -n "$TAG" ] && wants_gate receipts-at-tag; then
        gate_receipts_at_tag "$TAG" || rc=1
    fi
    return "$rc"
}

main() {
    parse_args "$@"
    if [ "$SELF_TEST" -eq 1 ]; then
        self_test
        return "$?"
    fi
    run_gates
}

if main "$@"; then
    exit 0
else
    exit 1
fi
