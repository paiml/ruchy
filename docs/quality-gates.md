# Quality Gates Documentation

## Overview

Ruchy enforces strict quality gates through automated pre-commit hooks following the Toyota Way principle of "building quality in" rather than "inspecting quality out". 

**CRITICAL**: These gates are MANDATORY and cannot be bypassed. The command `git commit --no-verify` is FORBIDDEN.

## Quality Gate Hierarchy

### Gate 1: PMAT Complexity Analysis
- **Current Baseline**: Max cyclomatic 133, cognitive 237 (legacy code being refactored)
- **New Code Requirement**: Max cyclomatic 50, cognitive 75
- **Goal**: Prevent introduction of new high-complexity code
- **Command**: `pmat analyze complexity --top-files 5 --format=detailed`

### Gate 2: Zero SATD Policy
- **Requirement**: ZERO TODO/FIXME/HACK comments allowed
- **Philosophy**: Technical debt must be tracked in issues, not code
- **Command**: `pmat analyze satd --format=detailed`

### Gate 3: Dead Code Analysis
- **Current State**: ~47% dead code in src/ (parser module organization)
- **Warning Threshold**: >60% dead code
- **Command**: `pmat analyze dead-code --path src/`

### Gate 4: TDG Tracking
- **Purpose**: Track Technical Debt Gradient over time
- **Requirement**: No degradation without explicit override
- **Override**: Add `[TDG-OVERRIDE]` to commit message with justification

### Gate 5: Lint Check
- **Requirement**: ZERO clippy warnings
- **Flag**: `-D warnings` (all warnings treated as errors)
- **Command**: `make lint`

### Gate 6: Test Execution
- **Requirement**: ALL tests must pass
- **Current Count**: 606+ tests
- **Command**: `cargo test`

### Gate 7: Basic Functionality
- **Requirement**: REPL must execute basic print statement
- **Test**: `echo 'println("Hello")' | ruchy repl`
- **Purpose**: Prevent completely broken interpreter commits

### Gate 8: Language Compatibility
- **Requirement**: One-liner compatibility tests must pass
- **Purpose**: Prevent language feature regressions
- **Command**: `make compatibility`

## Pre-commit Hook Installation

The pre-commit hook is automatically installed at `.git/hooks/pre-commit`. To ensure it's executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Manual Quality Check

To run all quality gates manually before committing:

```bash
bash .git/hooks/pre-commit
```

## TDG Baseline Management

To update the TDG baseline after intentional improvements:

```bash
pmat tdg . --format=json > .tdg_baseline.json
```

## Troubleshooting

### "Complexity violations detected"
- Run `pmat analyze complexity --top-files 5` to identify hot spots
- Refactor functions with complexity >50
- Extract helper functions to reduce complexity

### "SATD violations detected"
- Remove all TODO/FIXME/HACK comments
- Create GitHub issues for tracking instead
- Use descriptive comments without debt markers

### "TDG degradation detected"
- Review changes for unnecessary complexity
- If degradation is justified, add `[TDG-OVERRIDE]` to commit message
- Include clear justification for the override

### "Tests failing"
- Run `cargo test` to see detailed failures
- Fix all failing tests before committing
- Never commit with `#[ignore]` on failing tests

## Toyota Way Principles

1. **Stop the Line**: Any quality gate failure blocks the commit
2. **Build Quality In**: Prevent defects at the source
3. **Continuous Improvement**: Baseline metrics improve over time
4. **Respect for People**: Automated gates prevent human error
5. **Long-term Philosophy**: No shortcuts for short-term gains

## Current Quality Baseline (v1.29.1)

- **Complexity**: Max cyclomatic 133 (target: <50 for new code)
- **SATD**: 0 violations (maintained)
- **Dead Code**: ~47% in src/ (parser module organization)
- **Test Coverage**: 606+ tests passing
- **Lint**: Zero warnings enforced
- **Language Compatibility**: 100% one-liner compatibility

## Future Improvements

1. Reduce legacy complexity below 50 cyclomatic
2. Achieve <10% dead code through refactoring
3. Implement coverage threshold enforcement (>80%)
4. Add performance regression detection
5. Integrate security vulnerability scanning