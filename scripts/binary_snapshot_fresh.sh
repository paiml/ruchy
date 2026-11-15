#!/usr/bin/env bash
# binary_snapshot_fresh.sh — is evidence/binary/snapshot.jsonl what scripts/binary_snapshot.sh writes today?
# Regenerates the snapshot and compares it with the committed one. git_sha (names the parent of the
# commit that adds the file) and binary_sha256 (follows the floating stable toolchain) are excluded.
#   bash scripts/binary_snapshot_fresh.sh              # regenerate (builds), compare with the committed file
#   bash scripts/binary_snapshot_fresh.sh A.jsonl B.jsonl   # compare two files, no build (stub rows)
# Exit 1: 0 rows on either side, invalid JSON, or a differing row (the diff is printed).
set -euo pipefail
export LC_ALL=C
REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "${tmp:?}"' EXIT
if [ "$#" -eq 2 ]; then
    a_src=$1; b_src=$2
elif [ "$#" -eq 0 ]; then
    a_src="$REPO_ROOT/evidence/binary/snapshot.jsonl"; b_src="$tmp/fresh.jsonl"
    bash "$REPO_ROOT/scripts/binary_snapshot.sh" "$b_src" || exit 1
else
    echo 'usage: binary_snapshot_fresh.sh [COMMITTED REGENERATED]' >&2; exit 2
fi
strip='del(.git_sha,.binary_sha256)'
jq -c "$strip" "$a_src" > "$tmp/a" || { echo "FAIL: jq on committed side"; exit 1; }
jq -c "$strip" "$b_src" > "$tmp/b" || { echo "FAIL: jq on regenerated side"; exit 1; }
na=$(wc -l < "$tmp/a"); nb=$(wc -l < "$tmp/b")
echo "rows: committed=$na regenerated=$nb"
if [ "$na" -lt 1 ] || [ "$nb" -lt 1 ]; then echo 'FAIL: 0 rows on a side, nothing measured'; exit 1; fi
diff "$tmp/a" "$tmp/b" || { echo 'STALE: regenerate with bash scripts/binary_snapshot.sh'; exit 1; }
echo "FRESH: $na row(s) equal"
