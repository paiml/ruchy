#!/bin/bash
# Web Components Quality Analysis Script
# Target: >80% test coverage for HTML/JS components

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}🌐 Web Components Quality Analysis${NC}"
echo -e "${CYAN}Target: >80% coverage with excellent quality${NC}"
echo "=============================================="

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is required but not installed${NC}"
    echo "Please install Node.js to run web quality analysis"
    exit 1
fi

# Count web files
echo -e "\n${BLUE}📊 Web Component Analysis${NC}"
HTML_FILES=$(find . -name "*.html" -not -path "./target/*" -not -path "./node_modules/*" | wc -l)
JS_FILES=$(find . -name "*.js" -not -path "./target/*" -not -path "./node_modules/*" -not -path "./pkg/*" | wc -l)
CSS_FILES=$(find . -name "*.css" -not -path "./target/*" -not -path "./node_modules/*" | wc -l)

HTML_LINES=$(find . -name "*.html" -not -path "./target/*" -not -path "./node_modules/*" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo 0)
JS_LINES=$(find . -name "*.js" -not -path "./target/*" -not -path "./node_modules/*" -not -path "./pkg/*" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo 0)

echo "  📁 HTML Files: $HTML_FILES"
echo "  📁 JavaScript Files: $JS_FILES"
echo "  📁 CSS Files: $CSS_FILES"
echo "  📝 HTML Lines: $HTML_LINES"
echo "  📝 JavaScript Lines: $JS_LINES"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "\n${BLUE}📦 Installing dependencies...${NC}"
    npm install --silent
fi

# Run HTML linting
echo -e "\n${BLUE}🔍 HTML Linting Analysis${NC}"
if command -v npx &> /dev/null && [ -f ".htmlhintrc" ]; then
    HTML_ERRORS=$(npx htmlhint assets/**/*.html testing/**/*.html 2>/dev/null | grep -c "Error" || echo 0)
    HTML_WARNINGS=$(npx htmlhint assets/**/*.html testing/**/*.html 2>/dev/null | grep -c "Warning" || echo 0)
    
    echo "  ❌ HTML Errors: $HTML_ERRORS"
    echo "  ⚠️  HTML Warnings: $HTML_WARNINGS"
    
    if [ "$HTML_ERRORS" -eq 0 ]; then
        echo -e "  ${GREEN}✅ No HTML errors found${NC}"
    fi
else
    echo "  ⚠️  HTMLHint not configured"
fi

# Run JavaScript linting
echo -e "\n${BLUE}🔍 JavaScript Linting Analysis${NC}"
if command -v npx &> /dev/null && [ -f ".eslintrc.json" ]; then
    JS_ERRORS=$(npx eslint js/**/*.js --format compact 2>/dev/null | grep -c "Error" || echo 0)
    JS_WARNINGS=$(npx eslint js/**/*.js --format compact 2>/dev/null | grep -c "Warning" || echo 0)
    
    echo "  ❌ JS Errors: $JS_ERRORS"
    echo "  ⚠️  JS Warnings: $JS_WARNINGS"
    
    if [ "$JS_ERRORS" -eq 0 ]; then
        echo -e "  ${GREEN}✅ No JavaScript errors found${NC}"
    fi
else
    echo "  ⚠️  ESLint not configured"
fi

# Run tests with coverage
echo -e "\n${BLUE}🧪 Running JavaScript Tests${NC}"
if [ -f "package.json" ] && grep -q "jest" package.json; then
    # Run tests and capture coverage
    npm test --silent 2>/dev/null || echo "Tests completed with some failures"
    
    # Extract coverage metrics
    if [ -f "coverage/coverage-summary.json" ]; then
        echo -e "\n${GREEN}📈 Test Coverage Results${NC}"
        echo "=========================="
        
        # Parse coverage JSON
        LINES_COV=$(cat coverage/coverage-summary.json | grep -A2 '"lines"' | grep '"pct"' | head -1 | sed 's/.*: \([0-9.]*\).*/\1/')
        STATEMENTS_COV=$(cat coverage/coverage-summary.json | grep -A2 '"statements"' | grep '"pct"' | head -1 | sed 's/.*: \([0-9.]*\).*/\1/')
        FUNCTIONS_COV=$(cat coverage/coverage-summary.json | grep -A2 '"functions"' | grep '"pct"' | head -1 | sed 's/.*: \([0-9.]*\).*/\1/')
        BRANCHES_COV=$(cat coverage/coverage-summary.json | grep -A2 '"branches"' | grep '"pct"' | head -1 | sed 's/.*: \([0-9.]*\).*/\1/')
        
        echo "  📊 Line Coverage: ${LINES_COV}%"
        echo "  📊 Statement Coverage: ${STATEMENTS_COV}%"
        echo "  📊 Function Coverage: ${FUNCTIONS_COV}%"
        echo "  📊 Branch Coverage: ${BRANCHES_COV}%"
        
        # Check if target is met
        TARGET=80
        if (( $(echo "$LINES_COV >= $TARGET" | bc -l 2>/dev/null || echo 0) )); then
            echo -e "\n${GREEN}✅ Coverage Target Achieved: ${LINES_COV}% >= ${TARGET}%${NC}"
            COVERAGE_STATUS="PASS"
        else
            echo -e "\n${RED}❌ Coverage Target Not Met: ${LINES_COV}% < ${TARGET}%${NC}"
            COVERAGE_STATUS="FAIL"
        fi
    else
        echo "  ⚠️  Coverage data not available"
        COVERAGE_STATUS="UNKNOWN"
    fi
else
    echo "  ⚠️  Jest not configured for testing"
    COVERAGE_STATUS="SKIP"
fi

# Accessibility check
echo -e "\n${BLUE}♿ Accessibility Analysis${NC}"
echo "========================"
HTML_COUNT=$(find . -name "*.html" -not -path "./target/*" -not -path "./node_modules/*" | wc -l)
if [ "$HTML_COUNT" -gt 0 ]; then
    # Check for ARIA attributes
    ARIA_COUNT=$(grep -r "aria-\|role=" assets/*.html 2>/dev/null | wc -l || echo 0)
    ALT_COUNT=$(grep -r 'alt=' assets/*.html 2>/dev/null | wc -l || echo 0)
    
    echo "  🏷️  ARIA attributes found: $ARIA_COUNT"
    echo "  🖼️  Alt attributes found: $ALT_COUNT"
    
    if [ "$ARIA_COUNT" -gt 0 ] && [ "$ALT_COUNT" -gt 0 ]; then
        echo -e "  ${GREEN}✅ Basic accessibility features present${NC}"
    else
        echo -e "  ${YELLOW}⚠️  Consider adding more accessibility features${NC}"
    fi
fi

# Performance analysis
echo -e "\n${BLUE}⚡ Performance Analysis${NC}"
echo "======================"
# Check for minification
MINIFIED_JS=$(find . -name "*.min.js" -not -path "./node_modules/*" | wc -l)
MINIFIED_CSS=$(find . -name "*.min.css" -not -path "./node_modules/*" | wc -l)

echo "  📦 Minified JS files: $MINIFIED_JS"
echo "  📦 Minified CSS files: $MINIFIED_CSS"

# Check for lazy loading
LAZY_LOADING=$(grep -r 'loading="lazy"' assets/*.html 2>/dev/null | wc -l || echo 0)
echo "  🦥 Lazy loading attributes: $LAZY_LOADING"

# Check for service worker
if [ -f "js/sw.js" ] || [ -f "sw.js" ]; then
    echo -e "  ${GREEN}✅ Service Worker present${NC}"
else
    echo -e "  ${YELLOW}⚠️  No Service Worker found${NC}"
fi

# Security analysis
echo -e "\n${BLUE}🔒 Security Analysis${NC}"
echo "==================="
# Check for inline scripts
INLINE_SCRIPTS=$(grep -r '<script>' assets/*.html 2>/dev/null | grep -v 'src=' | wc -l || echo 0)
INLINE_STYLES=$(grep -r 'style=' assets/*.html 2>/dev/null | wc -l || echo 0)

echo "  ⚠️  Inline scripts: $INLINE_SCRIPTS"
echo "  ⚠️  Inline styles: $INLINE_STYLES"

if [ "$INLINE_SCRIPTS" -eq 0 ] && [ "$INLINE_STYLES" -eq 0 ]; then
    echo -e "  ${GREEN}✅ No inline scripts or styles (good for CSP)${NC}"
else
    echo -e "  ${YELLOW}⚠️  Consider moving inline code to external files${NC}"
fi

# Quality summary
echo -e "\n${BOLD}🏆 Quality Summary${NC}"
echo "=================="
echo "Target: >80% coverage + excellent quality"
echo ""

# Calculate overall score
QUALITY_SCORE=0
QUALITY_CHECKS=0

# Coverage score
if [ "$COVERAGE_STATUS" = "PASS" ]; then
    QUALITY_SCORE=$((QUALITY_SCORE + 1))
    echo -e "  ✅ Test Coverage: PASS"
else
    echo -e "  ❌ Test Coverage: NEEDS IMPROVEMENT"
fi
QUALITY_CHECKS=$((QUALITY_CHECKS + 1))

# Linting score
if [ "$HTML_ERRORS" -eq 0 ] && [ "$JS_ERRORS" -eq 0 ]; then
    QUALITY_SCORE=$((QUALITY_SCORE + 1))
    echo -e "  ✅ Linting: PASS"
else
    echo -e "  ⚠️  Linting: WARNINGS"
fi
QUALITY_CHECKS=$((QUALITY_CHECKS + 1))

# Accessibility score
if [ "$ARIA_COUNT" -gt 0 ] && [ "$ALT_COUNT" -gt 0 ]; then
    QUALITY_SCORE=$((QUALITY_SCORE + 1))
    echo -e "  ✅ Accessibility: PASS"
else
    echo -e "  ⚠️  Accessibility: NEEDS IMPROVEMENT"
fi
QUALITY_CHECKS=$((QUALITY_CHECKS + 1))

# Security score
if [ "$INLINE_SCRIPTS" -eq 0 ]; then
    QUALITY_SCORE=$((QUALITY_SCORE + 1))
    echo -e "  ✅ Security: PASS"
else
    echo -e "  ⚠️  Security: REVIEW NEEDED"
fi
QUALITY_CHECKS=$((QUALITY_CHECKS + 1))

# Calculate percentage
QUALITY_PERCENT=$((QUALITY_SCORE * 100 / QUALITY_CHECKS))

echo ""
echo -e "${BOLD}Overall Quality Score: ${QUALITY_PERCENT}%${NC}"

if [ "$QUALITY_PERCENT" -ge 80 ]; then
    echo -e "${GREEN}🎉 EXCELLENT - Quality target achieved!${NC}"
    EXIT_CODE=0
elif [ "$QUALITY_PERCENT" -ge 60 ]; then
    echo -e "${YELLOW}📈 GOOD - Some improvements needed${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}⚠️  NEEDS WORK - Significant improvements required${NC}"
    EXIT_CODE=1
fi

echo -e "\n${BLUE}📁 Reports:${NC}"
echo "  📊 Coverage Report: coverage/lcov-report/index.html"
echo "  📝 Test Results: coverage/test-report.html"

echo -e "\n${GREEN}✨ Analysis Complete${NC}"
exit $EXIT_CODE