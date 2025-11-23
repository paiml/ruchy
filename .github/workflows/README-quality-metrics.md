# Quality Metrics Tracking Workflow - Ruchy

**O(1) Quality Gates - CI/CD Integration (Phase 3.4)**

## Overview

This workflow automatically tracks quality metrics (lint, test-fast, test-pre-commit, coverage, binary size) on every push/PR and provides trend analysis to detect regressions early.

Integrated with Ruchy's Certeza Three-Tiered Testing Framework.

## Features

- **Automatic Metric Recording**: Records lint, test-fast, test-pre-commit, coverage, and binary size durations automatically
- **Trend Analysis**: Analyzes 30-day trends to detect regressions
- **PR Warnings**: Posts warnings to PRs when metrics are trending toward threshold breaches
- **Artifact Storage**: Keeps metrics data for 90 days
- **Reports**: Generates detailed metric reports for each CI run
- **Rust Project Score**: Weekly comprehensive quality assessment on main branch

## How It Works

### 1. Metric Recording

The workflow runs Ruchy's Makefile targets, measures their duration, and records them:

```bash
START=$(date +%s%3N)
make lint
END=$(date +%s%3N)
DURATION=$((END - START))
pmat record-metric lint $DURATION
```

Metrics tracked:
- **lint**: Clippy linting time
- **test-fast**: TDD cycle test duration (target: <5min)
- **test-pre-commit**: Pre-commit test duration (target: <30s)
- **coverage**: Coverage analysis time (target: <10min)
- **binary-size**: Ruchy binary size (target: <10MB)

### 2. Trend Analysis

After recording, `pmat show-metrics --trend` analyzes the last 30 days:

```
📊 Quality Metrics Trends (30 days)

lint
  Direction: ↑ Regressing
  Mean: 23390.50
  Std Dev: 2156.30
  Slope: 235.46/day
  Recommendations:
    • ⚠️ WARNING: Approaching threshold in ~15 days
    • Remove unused dependencies (saves ~2-3s)
```

### 3. Regression Detection (PRs only)

For pull requests, the workflow predicts threshold breaches:

```bash
pmat predict-quality --all --failures-only --format json > regressions.json
```

If regressions are detected, it posts a comment to the PR with:
- Which metrics are regressing
- Predicted days until threshold breach
- Specific recommendations to fix the issue

### 4. Artifacts

All metrics data is uploaded as artifacts:
- **quality-metrics**: Raw `.pmat-metrics/` data
- **metrics-report**: Markdown report with trends and analysis
- **rust-project-score**: Weekly comprehensive quality score (main branch only)

## Manual Usage

You can also record metrics manually in CI:

```yaml
- name: Record custom metric
  run: |
    pmat record-metric my-metric 42.5
```

Or with a custom timestamp:

```bash
pmat record-metric lint 25000 --timestamp 1763906533
```

## View Trends Locally

```bash
# Show all metric trends
pmat show-metrics --trend

# Show specific metric
pmat show-metrics --trend --metric lint

# JSON output
pmat show-metrics --trend --format json

# Show only regressing metrics
pmat show-metrics --trend --failures-only
```

## Thresholds

From `.pmat-metrics.toml`:
- **lint**: ≤30s (30,000ms) - Ruchy target: <30s pre-commit
- **test-fast**: ≤5min (300,000ms) - Ruchy target: <5min TDD cycle
- **test-pre-commit**: ≤30s (30,000ms) - Ruchy target: <30s pre-commit
- **coverage**: ≤10min (600,000ms) - Ruchy target: <10min
- **binary-size**: ≤10MB (10,000,000 bytes) - Keep Ruchy lean

## Integration with Certeza Framework

This workflow integrates with Ruchy's Certeza Three-Tiered Testing Framework:

- **Tier 1 (On-Save, <1s)**: Not tracked (too fast for CI)
- **Tier 2 (On-Commit, 1-5min)**: `test-fast`, `test-pre-commit` metrics
- **Tier 3 (On-Merge/Nightly, hours)**: `coverage`, mutation testing (not in CI)

## Architecture

```
GitHub Actions
     ↓
  measure duration (Makefile targets)
     ↓
  pmat record-metric
     ↓
  .pmat-metrics/trends/
     ↓
  pmat show-metrics --trend
     ↓
  PageRank hot metrics + ML predictions
     ↓
  PR comment (if regressing)
```

## Toyota Way Principles

- **Jidoka** (Built-in Quality): Automated regression detection
- **Andon Cord**: Stop-the-line PR warnings when quality degrades
- **Kaizen**: Continuous improvement via trend tracking
- **Genchi Genbutsu**: Direct measurement of actual build/test performance
- **Muda** (Waste Elimination): Fast trend analysis without re-running tests

## Local Development Workflow

```bash
# Run lint and record metric
make lint
pmat record-metric lint $(measure_time)

# Run fast tests and record metric
make test-fast
pmat record-metric test-fast $(measure_time)

# Check for regressions before committing
pmat predict-quality --all

# View current trends
pmat show-metrics --trend
```

## Pre-Commit Hooks

PMAT hooks are installed with TDG enforcement:

```bash
pmat hooks install --tdg-enforcement
```

This adds:
- **pre-commit**: O(1) quality validation (<30ms) + TDG quality checks
- **post-commit**: Baseline auto-update

## See Also

- **Phase 2**: O(1) Quality Gates (pre-commit validation)
- **Phase 3.1**: CLI integration (`pmat record-metric`)
- **Phase 3.2**: PageRank hot metrics
- **Phase 3.4**: CI/CD integration (this workflow)
- **Phase 4.1**: Predictive threshold breach detection
- **Specifications**:
  - `docs/specifications/quick-test-build-O(1)-checking.md`
  - `docs/specifications/improve-testing-quality-using-certeza-concepts.md`
- **Rust Project Score**: `pmat rust-project-score --full`
