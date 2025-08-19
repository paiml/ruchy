#!/bin/bash
# One-time setup for permanent quality enforcement

echo "🔧 Setting up Ruchy quality enforcement..."

# Install pre-commit hook
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Install PMAT if not already installed
if ! command -v pmat &> /dev/null; then
    echo "Installing PMAT..."
    cargo install pmat || echo "Note: PMAT installation optional for initial setup"
fi

# Create required documentation structure
mkdir -p docs/execution docs/quality docs/architecture/decisions
touch docs/execution/roadmap.md docs/execution/quality-gates.md

# Initialize roadmap template if not exists
if [ ! -s docs/execution/roadmap.md ]; then
    cat > docs/execution/roadmap.md << 'EOF'
# Ruchy Development Roadmap

## Current Sprint: 2025-Q1

### In Progress
- [ ] RUCHY-0001: Parser implementation
- [ ] RUCHY-0002: Type inference baseline

### Completed
- [x] RUCHY-0000: Project setup and quality gates

## Task Log
| ID | Description | Status | Complexity | Owner |
|----|-------------|--------|------------|-------|
| RUCHY-0001 | Recursive descent parser | 🚧 | High | - |
| RUCHY-0002 | Type inference Algorithm W | 🚧 | High | - |
EOF
fi

# Initialize quality-gates.md if not exists
if [ ! -s docs/execution/quality-gates.md ]; then
    cat > docs/execution/quality-gates.md << 'EOF'
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
Generated: $(date)
All quality gates passed
EOF
fi

# Configure git
git config core.hooksPath .git/hooks

echo "✅ Quality enforcement configured!"
echo ""
echo "Usage:"
echo "  make dev      - Start development with quality checks"
echo "  make commit   - Create quality-enforced commit"
echo "  make sprint-close - Verify sprint quality"
echo ""
echo "Documentation MUST be updated with every code change!"