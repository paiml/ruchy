# The "Self-Evolving Apex Hunt" Protocol for Ruchy

**Role**: Apex Hunt Team (Autonomous MLOps)
**Objective**: Converge on 95% Single-Shot Transpilation Success Rate for ruchy→Rust pipeline.
**New Context**: `ruchy::oracle` now uses `aprender::online::drift::ADWIN` for automatic drift detection. The HotFixLayer monitors accuracy and triggers retraining when drift is detected. Manual model maintenance is obsolete.
**Tooling**: `ruchydbg`, `ruchy transpile`, `ruchy compile`, `pmat work`, `ruchy oracle hansei`

## PROTOCOL: EXECUTE ONE RAPID PDCA CYCLE (Idempotent)

### 1. PLAN (Self-Monitoring Diagnosis):

```bash
# Check State: Does examples/repro_fix.ruchy exist?
ls examples/repro_fix.ruchy 2>/dev/null

# IF PASSING (or missing):
rm -f examples/repro_fix.ruchy

# Turbo Scan & Self-Healing AI Insight:
timeout 30 ruchy oracle hansei --corpus examples/ --format rich

# Note: Oracle automatically monitors DriftStatus via ADWIN:
#   - Stable: Continue with current model
#   - Warning: Flag for monitoring
#   - Drift: Automatic retrain triggered
```

**Target Selection (AI-Prioritized)**: Based on hansei report's:
- **Andon Status**: Check `DriftStatus` output. If `Drift`, prioritize patterns causing regression.
- **Top Error Taxonomies**: E0308 (type mismatch), E0382 (borrow), E0597 (lifetime), TRANSPILE failures
- **Semantic Classification**: Parser/Transpiler/Runtime failure clusters?
- **SBFL Ranking**: Run `ruchydbg analyze ./src -f ochiai` for suspicious code locations

**Create**: Synthesize minimal `examples/repro_fix.ruchy` isolating P0 failure:
```bash
# Verify Red State
timeout 10 ruchy transpile examples/repro_fix.ruchy -o /tmp/repro.rs
rustc --edition 2021 /tmp/repro.rs 2>&1  # Must FAIL
```

### 2. DO (Precision Repair):

```bash
# IF FAILING (Active Red State):
pmat work start "Apex Hunt: Repairing [Error Code] from Oracle Report"

# Strategy A (Missing Pattern):
#   - Check oracle pattern store coverage
#   - Add pattern to src/oracle/patterns.rs

# Strategy B (Transpiler Bug):
#   - Fault Localization:
ruchydbg trace /tmp/repro_fix.ruchy --analyze
ruchydbg analyze ./src/backend/transpiler -f ochiai -o ascii

#   - Fix logic in:
#     src/backend/transpiler/expressions.rs
#     src/backend/transpiler/type_inference.rs
#     src/backend/transpiler/codegen_minimal.rs

# Strategy C (Parser Bug):
ruchydbg tokenize examples/repro_fix.ruchy --analyze
#   - Fix in src/frontend/parser/
```

### 3. CHECK (Verify):

```bash
# Full pipeline validation
timeout 10 ruchy transpile examples/repro_fix.ruchy -o /tmp/repro.rs
rustc --edition 2021 /tmp/repro.rs -o /tmp/repro_bin
/tmp/repro_bin  # Execute

# Constraint: Must pass ALL 15 tools
ruchy check examples/repro_fix.ruchy
ruchy lint examples/repro_fix.ruchy
ruchy run examples/repro_fix.ruchy

# PMAT quality gates
pmat quality-gate --checks=complexity,satd
```

### 4. ACT (Standardize & Track):

```bash
# IF GREEN:
pmat work complete "Apex Hunt: Fixed [Pattern] - Cycle Complete"

git add -A && git commit -m "$(cat <<'EOF'
fix(transpiler): Resolve [Pattern Name] via Oracle-guided analysis

- Root cause: [Brief description]
- Fix: [What was changed]
- Oracle: Pattern automatically learned via HotFixLayer

Refs: TRANSPILER-XXX

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"

# Reflect: Oracle's ADWIN will detect if this fix improves accuracy
echo "Fix committed. Oracle HotFixLayer recorded pattern for online learning."
```

## COMMAND

```
Assess examples/repro_fix.ruchy.
IF FAIL: FIX IT (Use ruchydbg + Oracle).
IF PASS: BREAK IT (Run Hansei → New Repro).
GO.
```

## Key Features

- Uses `aprender::online::drift::ADWIN` for drift detection
- `ruchydbg` for SBFL fault localization (Ochiai/Tarantula formulas)
- `ruchy oracle hansei` for Toyota Way reflection reports
- 15-tool validation pipeline
- PMAT quality gates integration
- Automatic online learning via HotFixLayer
