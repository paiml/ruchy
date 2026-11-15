#!/usr/bin/env bash
# snapshot_lab_edge.sh — is this red a NEW one (comment) or one already reported (silent)?
# stdin: `gh run list --json databaseId,conclusion` (newest first). $1: current run id. env GITHUB_REPOSITORY.
# Drops the current run and runs that measured nothing (cancelled, skipped, startup_failure, ""); the
# newest remaining decides. Prints exactly one line:
#   NEW red: ...        previous success; or no measured previous; or previous failed but its report job did not succeed
#   CONTINUING red ...  previous failure/timed_out whose report job succeeded
# Exit 1: bad/empty input, an unknown conclusion, or an unreadable job list. Never silent on a doubt.
set -euo pipefail
cur=${1:?usage: snapshot_lab_edge.sh CURRENT_RUN_ID}
repo=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY unset}
in=$(cat)
[ -n "$in" ] || { echo 'FAIL: empty history input' >&2; exit 1; }
sel=$(printf '%s' "$in" | jq -c --arg cur "$cur" '
    map(select((.databaseId | tostring) != $cur)
        | select(((.conclusion // "") | IN("cancelled", "skipped", "startup_failure", "")) | not))
    | .[0] // empty') || { echo 'FAIL: history is not a JSON run list' >&2; exit 1; }
if [ -z "$sel" ]; then echo 'NEW red: no measured previous scheduled run'; exit 0; fi
id=$(printf '%s' "$sel" | jq -r '.databaseId') || exit 1
concl=$(printf '%s' "$sel" | jq -r '.conclusion') || exit 1
case "$concl" in
    success) echo "NEW red: previous scheduled run $id was success" ;;
    failure | timed_out)
        rep=$(gh run view "$id" -R "$repo" --json jobs --jq '[.jobs[] | select(.name == "report") | .conclusion] | .[0] // ""') || { echo "FAIL: cannot read jobs of run $id" >&2; exit 1; }
        if [ "$rep" = success ]; then echo "CONTINUING red since run $id: silent"
        else echo "NEW red: previous scheduled run $id failed and its report job was '${rep:-absent}'"; fi ;;
    *) echo "FAIL: unknown conclusion '$concl' on run $id" >&2; exit 1 ;;
esac
