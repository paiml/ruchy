#!/usr/bin/env bash
# binary_snapshot.sh — write evidence/binary/snapshot.jsonl, the tracked record
# `pv lint` (extract:binary, schema "binary-snapshot/v1") reads for every cargo
# bin target of this workspace.
#
# WHY THIS EXISTS. At a [workspace] root that declares bin targets, pv's
# extract:binary refuses an absent or empty snapshot (PV-ONT-012: "read zero
# snapshot lines — the extractor measured nothing"), so `pv lint contracts
# --gate shapes` rejects. The gate never runs a binary; this script does, once,
# and the gate's verdict is then a function of the tree.
#
# FORMAT. One JSON line per bin target, LC_ALL=C sorted by package then target,
# no timestamps. Keys, in this order (schema first), as the schema defines them:
#   schema package target name src_path git_sha version version_sha256
#   binary_sha256 help_sha256 help_exit commands options positionals routes tools
# `commands` is every command path under the binary, space-joined, without the
# binary name: each leaf, plus each non-leaf whose Usage shows `[COMMAND]`.
# clap's auto `help` is not a command. An empty surface is [], never omitted.
# The snapshot is the DEFAULT-FEATURE build. `tools` is asked of the binary over
# stdio (`ruchy mcp`, tools/list); without `--features mcp` it answers "MCP
# support not enabled" and lists none, so [] is what that build exposes.
# `routes` is []: `ruchy serve` serves static files and mounts no route table.
#
# The record names the commit it was built from (git_sha), so regenerate it
# after a change to the CLI surface. Determinism: `--twice` builds once, writes
# two snapshots and compares them byte for byte.
#
#   bash scripts/binary_snapshot.sh            # write evidence/binary/snapshot.jsonl
#   bash scripts/binary_snapshot.sh FILE       # write FILE instead
#   bash scripts/binary_snapshot.sh --twice    # determinism check, writes nothing
#
# Env: SNAPSHOT_PROFILE (default release), SNAPSHOT_MAX_DEPTH (default 4).
set -euo pipefail
export LC_ALL=C

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
PROFILE="${SNAPSHOT_PROFILE:-release}"
MAX_DEPTH="${SNAPSHOT_MAX_DEPTH:-4}"

for tool in cargo jq git sha256sum timeout; do
    command -v "$tool" > /dev/null || { printf 'FAIL: %s missing\n' "$tool" >&2; exit 1; }
done

# One subcommand block of `$@ --help`: "name<TAB>description", clap indent 2.
subcommands() {
    timeout 20 "$@" --help 2>/dev/null | awk '
        /^[Cc]ommands:/ { inb=1; next }
        /^[A-Za-z]/     { inb=0 }
        inb && /^  [a-z][a-z0-9._-]*([[:space:]]|$)/ {
            n=$1; sub(/^  [^ ]+[ ]*/, ""); print n "\t" $0 }' || true
}

# $1 executable, $2 depth, $3.. command path. Prints command paths.
walk() {
    local exe="$1" depth="$2"; shift 2
    local sub desc kids usage
    [ "$depth" -gt "$MAX_DEPTH" ] && return 0
    while IFS=$'\t' read -r sub desc; do
        [ -n "$sub" ] || continue
        if [ "$sub" = help ] && [[ "$desc" == "Print this message or the help"* ]]; then
            continue
        fi
        kids=$(subcommands "$exe" "$@" "$sub" | grep -c . || true)
        if [ "$kids" -eq 0 ]; then
            printf '%s\n' "${*:+$* }$sub"
        else
            usage=$(timeout 20 "$exe" "$@" "$sub" --help 2>/dev/null | grep -m1 '^Usage:' || true)
            if [[ "$usage" == *"[COMMAND]"* ]]; then printf '%s\n' "${*:+$* }$sub"; fi
            walk "$exe" "$((depth + 1))" "$@" "$sub"
        fi
    done < <(subcommands "$exe" "$@")
}

options() {
    { timeout 20 "$1" --help 2>/dev/null || true; } \
        | grep -oE '^ +(-[A-Za-z0-9], )?--[a-z0-9][a-z0-9-]*' \
        | grep -oE -- '--[a-z0-9][a-z0-9-]*$' \
        | grep -vxE -- '--(help|version)' || true
}

positionals() {
    { timeout 20 "$1" --help 2>/dev/null || true; } | awk '
        /^Arguments:/ { inb=1; next }
        /^[A-Za-z]/   { inb=0 }
        inb && /^  [<\[]/ { a=$1; gsub(/[<>\[\]]|\.\.\./, "", a); print a }'
}

# MCP tools, asked of the binary over stdio. A binary with no `mcp` command
# exposes none.
tools() {
    local exe="$1"
    walk "$exe" 1 | grep -qx mcp || return 0
    printf '%s\n' \
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"snapshot","version":"1"}}}' \
        '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        | { timeout 30 "$exe" mcp 2>/dev/null || true; } \
        | jq -r 'select(.id==2) | .result.tools[].name' 2>/dev/null || true
}

arr() { sort -u | jq -R . | jq -sc .; }

# build_all: builds every bin target and prints "pkg<TAB>tgt<TAB>src<TAB>exe".
# Each target is copied out BEFORE the next build, so two targets sharing an
# executable name can never be hashed as one file.
build_all() {
    local stash="$1" pkg tgt src req exe feat i=0 dest
    cargo metadata --no-deps --format-version 1 --manifest-path "$REPO_ROOT/Cargo.toml" \
        | jq -r --arg r "$REPO_ROOT/" '.packages[] as $p | $p.targets[] | select(.kind | index("bin"))
              | [$p.name, .name, (.src_path | ltrimstr($r)), ((.["required-features"] // []) | join(","))] | @tsv' \
        | sort \
        | while IFS=$'\t' read -r pkg tgt src req; do
            feat=(); [ -n "$req" ] && feat=(--features "$req")
            exe=$(cargo build -p "$pkg" --bin "$tgt" "${feat[@]}" --profile "$PROFILE" \
                    --manifest-path "$REPO_ROOT/Cargo.toml" --message-format=json \
                | jq -r --arg t "$tgt" 'select(.reason=="compiler-artifact" and .target.name==$t and .executable) | .executable' \
                | tail -1)
            if [ -z "$exe" ] || [ ! -x "$exe" ]; then
                printf 'FAIL: %s/%s did not build\n' "$pkg" "$tgt" >&2; exit 1
            fi
            # numbered stash dirs: no metadata-derived string becomes a path
            i=$((i + 1)); dest="$stash/$i"
            mkdir -p "$dest"
            cp "$exe" "$dest/"
            printf '%s\t%s\t%s\t%s\n' "$pkg" "$tgt" "$src" "$dest/${exe##*/}"
        done
}

# write_snapshot BUILT OUT: one line per built target.
write_snapshot() {
    local built="$1" out="$2" sha pkg tgt src exe ver help hrc tmp
    sha=$(git -C "$REPO_ROOT" rev-parse HEAD)
    tmp="$out.tmp"; : > "$tmp"
    while IFS=$'\t' read -r pkg tgt src exe; do
        ver=$(timeout 20 "$exe" --version 2>/dev/null || true)
        hrc=0; help=$(timeout 20 "$exe" --help 2>/dev/null) || hrc=$?
        jq -nc --arg package "$pkg" --arg target "$tgt" --arg name "$(basename "$exe")" --arg src "$src" \
            --arg sha "$sha" --arg ver "$ver" \
            --arg vs "$(printf '%s' "$ver" | sha256sum | cut -d' ' -f1)" \
            --arg bs "$(sha256sum "$exe" | cut -d' ' -f1)" \
            --arg hs "$(printf '%s' "$help" | sha256sum | cut -d' ' -f1)" \
            --argjson hrc "$hrc" \
            --argjson commands "$(walk "$exe" 1 | arr)" \
            --argjson options "$(options "$exe" | arr)" \
            --argjson positionals "$(positionals "$exe" | arr)" \
            --argjson routes '[]' \
            --argjson tools "$(tools "$exe" | arr)" \
            '{schema: "binary-snapshot/v1", package: $package, target: $target, name: $name, src_path: $src,
              git_sha: $sha, version: $ver, version_sha256: $vs, binary_sha256: $bs, help_sha256: $hs,
              help_exit: $hrc, commands: $commands, options: $options, positionals: $positionals,
              routes: $routes, tools: $tools}' >> "$tmp"
        printf '  snap  %s/%s\n' "$pkg" "$tgt" >&2
    done < "$built"
    mv -f "$tmp" "$out"
}

work=$(mktemp -d)
trap 'rm -rf "${work:?}"' EXIT
build_all "$work/bin" > "$work/built"
n=$(grep -c . "$work/built" || true)
# Vacuity: a workspace that builds no bin target has nothing to snapshot, and a
# snapshot of zero lines is exactly what the gate refuses.
[ "$n" -ge 1 ] || { printf 'FAIL: cargo metadata listed 0 bin targets\n' >&2; exit 1; }

if [ "${1:-}" = "--twice" ]; then
    write_snapshot "$work/built" "$work/a"
    write_snapshot "$work/built" "$work/b"
    if cmp -s "$work/a" "$work/b"; then
        printf 'DETERMINISTIC: two snapshots byte-identical (%s lines).\n' "$(grep -c . "$work/a")"
        exit 0
    fi
    printf 'NON-DETERMINISTIC: snapshots differ.\n'; diff "$work/a" "$work/b" | head -20 || true
    exit 1
fi

out="${1:-$REPO_ROOT/evidence/binary/snapshot.jsonl}"
mkdir -p "$(dirname "$out")"
write_snapshot "$work/built" "$out"
[ "$(grep -c . "$out")" -eq "$n" ] || { printf 'FAIL: %s lines for %s targets\n' "$(grep -c . "$out")" "$n" >&2; exit 1; }
printf 'snapshot: %s bin target(s) -> %s (HEAD %s)\n' "$n" "${out#"$REPO_ROOT"/}" "$(git -C "$REPO_ROOT" rev-parse --short=9 HEAD)"
