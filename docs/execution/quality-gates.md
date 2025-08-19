# Ruchy Quality Gates

## Enforcement Status
- Pre-commit hooks: ✅ Active
- CI/CD pipeline: ✅ Active
- PMAT integration: ✅ Active
- Documentation sync: ✅ Required

## Quality Metrics
- Cyclomatic complexity: <10
- Cognitive complexity: <10
- Test coverage: >80%
- SATD comments: 0
- Lint warnings: 0

## Last Sprint Report
Generated: 2025-08-20
All quality gates passed

## Documentation Requirements
Every code change MUST update at least one of:
- docs/execution/roadmap.md (task status)
- docs/execution/quality-gates.md (quality metrics)
- CHANGELOG.md (features/fixes)
- docs/architecture/decisions/ (ADRs for architectural changes)

## Enforcement Mechanisms

### Pre-commit Hook
Blocks commits without documentation updates.
Located at: `.git/hooks/pre-commit`

### CI/CD Pipeline
GitHub Actions workflow fails PRs missing documentation.
Located at: `.github/workflows/quality-enforcement.yml`

### Makefile Targets
- `make dev` - Checks documentation before development
- `make commit` - Quality-enforced commit with task ID
- `make sprint-close` - Sprint quality verification

### PMAT Configuration
Zero-tolerance quality settings in `pmat.toml`:
- Max cyclomatic complexity: 10
- Max cognitive complexity: 10
- Zero SATD comments allowed
- Min test coverage: 80%

## Setup Instructions
Run once to enable all quality gates:
```bash
./scripts/setup-quality.sh
```