# Ruchy Quality Gates

## Enforcement Status
- Pre-commit hooks: ✅ Active
- CI/CD pipeline: ✅ Active
- PMAT integration: ✅ Active
- Documentation sync: ✅ Required

## Quality Metrics
- Cyclomatic complexity: ≤10
- Cognitive complexity: ≤10
- TDG (Technical Debt Grade): ≥A- (85 points)
- Entropy: ≤0.8 (code maintainability)
- Test coverage: ≥80%
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
Zero-tolerance quality settings enforced in pre-commit hook:
- Max cyclomatic complexity: 10
- Max cognitive complexity: 10
- Min TDG grade: A- (85 points)
- Max entropy: 0.8 (maintainability threshold)
- Zero SATD comments allowed
- Min test coverage: 80%

**TDG Components** (all must pass):
- Complexity score (cyclomatic + cognitive)
- Duplication percentage
- Documentation coverage
- Test coverage
- Code style consistency

**Entropy Metrics** (measures code chaos):
- Naming consistency
- Abstraction level uniformity
- Code organization patterns
- Low entropy = predictable, maintainable code
- High entropy = chaotic, hard to understand

## Setup Instructions
Run once to enable all quality gates:
```bash
./scripts/setup-quality.sh
```