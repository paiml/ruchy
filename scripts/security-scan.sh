#!/bin/bash
# scripts/security-scan.sh
# WebAssembly Extreme Quality Assurance Framework v3.0
# Security Scanning Script

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Starting security scan...${NC}"

# Create output directory
mkdir -p target/security

echo -e "${YELLOW}Phase 1: Dependency vulnerability scan${NC}"
if cargo audit --version &> /dev/null; then
    cargo audit --json > target/security/audit-report.json 2>/dev/null || {
        echo -e "${YELLOW}Warning: Some vulnerabilities found or audit database needs update${NC}"
        cargo audit > target/security/audit-report.txt 2>&1 || true
    }
    echo -e "${GREEN}✓ Dependency audit complete${NC}"
else
    echo -e "${YELLOW}Warning: cargo-audit not installed${NC}"
    echo "Install with: cargo install cargo-audit"
fi

echo -e "${YELLOW}Phase 2: Unsafe code analysis${NC}"
if command -v cargo-geiger &> /dev/null; then
    cargo geiger --all-features --output-format Json > target/security/geiger-report.json 2>/dev/null || true
    cargo geiger --all-features > target/security/geiger-report.txt 2>&1 || true

    # Check for unsafe code
    UNSAFE_COUNT=$(cargo geiger --all-features --output-format Json 2>/dev/null | \
                   jq -r '.packages[].unsafety.used.functions.unsafe // 0' | \
                   awk '{sum += $1} END {print sum}' 2>/dev/null || echo "0")

    if [ "$UNSAFE_COUNT" -gt 0 ]; then
        echo -e "${YELLOW}Warning: Found $UNSAFE_COUNT unsafe functions${NC}"
    else
        echo -e "${GREEN}✓ No unsafe code found${NC}"
    fi
else
    echo -e "${YELLOW}Warning: cargo-geiger not installed${NC}"
    echo "Install with: cargo install cargo-geiger"

    # Alternative: grep for unsafe code
    echo -e "${YELLOW}Using alternative unsafe code detection...${NC}"
    grep -r "unsafe" src/ > target/security/unsafe-grep.txt 2>/dev/null || true
    UNSAFE_LINES=$(wc -l < target/security/unsafe-grep.txt 2>/dev/null || echo "0")

    if [ "$UNSAFE_LINES" -gt 0 ]; then
        echo -e "${YELLOW}Found $UNSAFE_LINES lines containing 'unsafe'${NC}"
    else
        echo -e "${GREEN}✓ No unsafe code patterns found${NC}"
    fi
fi

echo -e "${YELLOW}Phase 3: License compliance${NC}"
cargo license > target/security/license-report.txt 2>/dev/null || {
    echo -e "${YELLOW}Warning: cargo-license not available, using alternative${NC}"

    # Alternative: Check common license files
    find . -name "LICENSE*" -o -name "COPYING*" > target/security/license-files.txt
    echo "License files found:" >> target/security/license-report.txt
    cat target/security/license-files.txt >> target/security/license-report.txt
}

echo -e "${YELLOW}Phase 4: WASM-specific security checks${NC}"

# Check for potential WASM security issues
cat > target/security/wasm-security-check.sh << 'EOF'
#!/bin/bash
# WASM-specific security patterns to check

echo "Checking WASM security patterns..."

# Check for unsafe WASM bindings
grep -r "unsafe" src/wasm* 2>/dev/null || true

# Check for potential memory leaks in WASM code
grep -r "forget\|mem::" src/wasm* 2>/dev/null || true

# Check for proper error handling in WASM bindings
grep -r "unwrap\|expect" src/wasm* 2>/dev/null || true

echo "WASM security check complete"
EOF

chmod +x target/security/wasm-security-check.sh
./target/security/wasm-security-check.sh > target/security/wasm-security.txt 2>&1

echo -e "${YELLOW}Phase 5: Generate security report${NC}"
cat > target/security/security-report.md << EOF
# Security Analysis Report

Generated: $(date)

## Vulnerability Scan
$(if [ -f target/security/audit-report.txt ]; then echo "✓ Dependency audit completed"; grep -c "warning\|error" target/security/audit-report.txt 2>/dev/null || echo "0"; echo " issues found"; else echo "⚠ Dependency audit not available"; fi)

## Unsafe Code Analysis
$(if [ -f target/security/geiger-report.txt ]; then echo "✓ Geiger scan completed"; else echo "⚠ Geiger scan not available"; fi)
$(if [ -f target/security/unsafe-grep.txt ]; then echo "Found $(wc -l < target/security/unsafe-grep.txt) lines with 'unsafe'"; fi)

## License Compliance
$(if [ -f target/security/license-report.txt ]; then echo "✓ License report generated"; else echo "⚠ License report not available"; fi)

## WASM Security
$(if [ -f target/security/wasm-security.txt ]; then echo "✓ WASM security check completed"; else echo "⚠ WASM security check not available"; fi)

## Recommendations

EOF

# Check for critical issues
CRITICAL_ISSUES=0

# Check audit results
if [ -f target/security/audit-report.txt ]; then
    AUDIT_ISSUES=$(grep -c "error" target/security/audit-report.txt 2>/dev/null || echo "0")
    if [ "$AUDIT_ISSUES" -gt 0 ]; then
        echo -e "${RED}Critical: Found $AUDIT_ISSUES security vulnerabilities${NC}"
        CRITICAL_ISSUES=$((CRITICAL_ISSUES + AUDIT_ISSUES))
        echo "🚨 CRITICAL: $AUDIT_ISSUES security vulnerabilities found" >> target/security/security-report.md
    fi
fi

# Summary
if [ $CRITICAL_ISSUES -eq 0 ]; then
    echo -e "${GREEN}✓ Security scan passed - no critical issues${NC}"
    echo "✅ PASS: No critical security issues found" >> target/security/security-report.md
else
    echo -e "${RED}❌ Security scan found $CRITICAL_ISSUES critical issues${NC}"
    echo "❌ FAIL: $CRITICAL_ISSUES critical security issues found" >> target/security/security-report.md
    echo -e "${RED}Review target/security/ for details${NC}"
    exit 1
fi

echo -e "${GREEN}Security scan complete${NC}"