#!/bin/bash
# Ruchy Quick Health Check - Runs hourly during business hours

set -e

# Configuration
RUCHY_HOME="/home/noah/src/ruchy"
HEALTH_DIR="$RUCHY_HOME/quality-reports/health"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
HEALTH_FILE="$HEALTH_DIR/health-$TIMESTAMP.txt"

# Create directory
mkdir -p "$HEALTH_DIR"

# Quick health checks
{
    echo "Ruchy Health Check - $(date)"
    echo "================================"
    
    # Check if ruchy is accessible
    if ruchy --version &>/dev/null; then
        echo "✅ Ruchy binary: OK ($(ruchy --version))"
    else
        echo "❌ Ruchy binary: NOT FOUND"
    fi
    
    # Check quality tools
    for tool in test lint score prove; do
        if ruchy $tool --help &>/dev/null; then
            echo "✅ ruchy $tool: OK"
        else
            echo "❌ ruchy $tool: FAILED"
        fi
    done
    
    # Quick test of hello world
    echo 'println("test")' | timeout 5 ruchy repl &>/dev/null && \
        echo "✅ REPL: OK" || echo "❌ REPL: FAILED"
    
    # Check disk space
    DISK_USAGE=$(df -h "$RUCHY_HOME" | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ "$DISK_USAGE" -lt 90 ]; then
        echo "✅ Disk space: ${DISK_USAGE}% used"
    else
        echo "⚠️ Disk space: ${DISK_USAGE}% used (LOW SPACE)"
    fi
    
    echo ""
    echo "Health check complete: $(date)"
} > "$HEALTH_FILE"

# Keep only last 24 hours of health checks
find "$HEALTH_DIR" -name "health-*.txt" -mmin +1440 -delete

# Check for critical issues
if grep -q "❌" "$HEALTH_FILE"; then
    echo "CRITICAL: Health check failures detected" >&2
    cat "$HEALTH_FILE" >&2
    exit 1
fi

exit 0