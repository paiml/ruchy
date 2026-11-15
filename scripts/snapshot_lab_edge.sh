#!/usr/bin/env bash
# snapshot_lab_edge.sh — the report job's whole decision: read the scheduled history, decide whether this
# red is NEW (comment on #299) or CONTINUING (silent), print which one, and comment on a NEW one.
# $1: current run id. env: GITHUB_REPOSITORY, GITHUB_SERVER_URL, GH_TOKEN (for gh).
# Drops the current run and runs that measured nothing (cancelled, skipped, startup_failure, ""); the
# newest remaining decides. Prints exactly one verdict line:
#   NEW red: ...        previous success; or no measured previous; or previous failed but its report job did not succeed
#   CONTINUING red ...  previous failure/timed_out whose report job succeeded
# Exit 1: unreadable history or job list, bad/empty history, an unknown conclusion, a failed comment.
set -euo pipefail
cur=${1:?usage: snapshot_lab_edge.sh CURRENT_RUN_ID}
repo=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY unset}
srv=${GITHUB_SERVER_URL:?GITHUB_SERVER_URL unset}
in=$(gh run list -R "$repo" --workflow snapshot-lab.yml --event schedule --branch main --status completed -L 10 --json databaseId,conclusion) || { echo 'FAIL: cannot read run history' >&2; exit 1; }
[ -n "$in" ] || { echo 'FAIL: empty history input' >&2; exit 1; }
sel=$(printf '%s' "$in" | jq -c --arg cur "$cur" '
    map(select((.databaseId | tostring) != $cur)
        | select(((.conclusion // "") | IN("cancelled", "skipped", "startup_failure", "")) | not))
    | .[0] // empty') || { echo 'FAIL: history is not a JSON run list' >&2; exit 1; }
if [ -z "$sel" ]; then
    verdict='NEW red: no measured previous scheduled run'
else
    id=$(printf '%s' "$sel" | jq -r '.databaseId')
    concl=$(printf '%s' "$sel" | jq -r '.conclusion')
    case "$concl" in
        success) verdict="NEW red: previous scheduled run $id was success" ;;
        failure | timed_out)
            rep=$(gh run view "$id" -R "$repo" --json jobs --jq '[.jobs[] | select(.name == "report") | .conclusion] | .[0] // ""') || { echo "FAIL: cannot read jobs of run $id" >&2; exit 1; }
            if [ "$rep" = success ]; then verdict="CONTINUING red since run $id: silent"
            else verdict="NEW red: previous scheduled run $id failed and its report job was '${rep:-absent}'"; fi ;;
        *) echo "FAIL: unknown conclusion '$concl' on run $id" >&2; exit 1 ;;
    esac
fi
echo "$verdict"
case "$verdict" in
    NEW*) gh issue comment 299 -R "$repo" --body "snapshot-lab RED (new, $verdict): $srv/$repo/actions/runs/$cur" || { echo 'FAIL: cannot comment on #299' >&2; exit 1; } ;;
    CONTINUING*) echo 'silent: red already reported' ;;
esac
