# How to Run Overnight Mutation Tests

## Quick Start

```bash
cd /home/noah/src/ruchy

# Run overnight mutation tests in background
nohup ./.pmat/run_overnight_mutations.sh > .pmat/overnight_run.log 2>&1 &

# Save the process ID
echo $! > .pmat/overnight_pid.txt

# Verify it started
tail -20 .pmat/overnight_run.log
```

## Monitor Progress

```bash
# Check if still running
ps -p $(cat .pmat/overnight_pid.txt) && echo "✅ Still running" || echo "❌ Completed or stopped"

# View real-time progress
tail -f .pmat/overnight_run.log

# View specific file progress (when it starts)
tail -f .pmat/mutation_logs/*_mutations_*.txt
```

## Expected Timeline

- **Start**: Immediate
- **Duration**: 10-15 hours (7 files × ~1.5-2 hours each)
- **Completion**: Tomorrow morning

## Files Being Tested (in order)

1. eval_pattern.rs (421 lines) - ~78 mutants
2. cache.rs (422 lines)
3. eval_loops.rs (424 lines)
4. eval_method_dispatch.rs (425 lines)
5. safe_arena.rs (430 lines)
6. eval_string.rs (438 lines)
7. inspect.rs (456 lines)

## Check Results Tomorrow

```bash
# View summary of all files
ls -lh .pmat/mutation_logs/

# Quick summary of each file
for f in .pmat/mutation_logs/*.txt; do
  echo "=== $(basename $f) ==="
  grep -E "(Found|mutants tested)" "$f" | tail -2
  echo ""
done

# Count MISSED mutations per file
for f in .pmat/mutation_logs/*.txt; do
  echo "$(basename $f): $(grep '^MISSED' "$f" | wc -l) MISSED mutations"
done
```

## Next Session (After Overnight Run)

See: `NEXT_SESSION_SPRINT_9_PHASE_3_CONTINUATION.md` for complete workflow

**Quick Summary**:
1. Analyze MISSED mutations for each file
2. Fix mutations systematically (one file at a time)
3. Accept test oracle limitations (document them)
4. Commit after each file completion
5. Target: 80%+ coverage with realistic expectations

## Troubleshooting

**If the script stops early**:
```bash
# Check the log for errors
cat .pmat/overnight_run.log

# Find which file was running
tail -50 .pmat/overnight_run.log | grep "Testing:"

# Resume from that file manually
cargo mutants --file src/runtime/<file>.rs --timeout 600 --no-times
```

**If you need to stop it**:
```bash
# Kill the process
kill $(cat .pmat/overnight_pid.txt)

# Results up to that point are saved
ls .pmat/mutation_logs/
```

## Session 3 Achievements (Already Done)

✅ Sprint 8.5: 100% complete (29/29 mutations)
✅ eval_try_catch.rs: 1 test enhanced, test oracle limitations documented
✅ 3554 tests passing (zero regressions)
✅ Overnight infrastructure created

## What to Expect Tomorrow

- 7 complete mutation test reports
- Detailed MISSED mutation analysis for each file
- Systematic path to fixing 30-50 mutations
- ~3-4 hours of productive fixing work
- Sprint 9 Phase 3 completion: 10/10 files

---

**Ready to run!** Just execute the commands above and let it run overnight. 🌙
