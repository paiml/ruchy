#!/bin/bash
# Ruchy Ecosystem Quality Monitoring - Automated Daily Checks
# Run via cron: 0 2 * * * /home/noah/src/ruchy/cron/quality-monitor.sh

set -e

# Configuration
RUCHY_HOME="/home/noah/src/ruchy"
REPORT_DIR="$RUCHY_HOME/quality-reports/daily"
DATE=$(date +%Y%m%d)
REPORT_FILE="$REPORT_DIR/quality-report-$DATE.json"
LOG_FILE="$REPORT_DIR/quality-monitor-$DATE.log"

# Create directories if needed
mkdir -p "$REPORT_DIR"

# Logging function
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Start monitoring
log "Starting Ruchy ecosystem quality monitoring..."

# Initialize report
cat > "$REPORT_FILE" << EOF
{
  "date": "$(date -Iseconds)",
  "version": "1.20.0",
  "components": {}
}
EOF

# Function to check component quality
check_component() {
    local name=$1
    local path=$2
    
    log "Checking $name at $path..."
    
    if [ -d "$path" ]; then
        cd "$path"
        
        # Run tests
        local test_result="unknown"
        if ruchy test . 2>/dev/null | grep -q "All tests passed"; then
            test_result="passed"
        else
            test_result="failed"
        fi
        
        # Get quality score
        local score=$(ruchy score . 2>/dev/null | grep "Score:" | awk '{print $2}' | head -1 || echo "0.0")
        
        # Count lint issues
        local lint_issues=$(ruchy lint . 2>/dev/null | grep -c "issue" || echo "0")
        
        # Update JSON report
        local component_json=$(cat <<JSON
    "$name": {
      "path": "$path",
      "tests": "$test_result",
      "quality_score": "$score",
      "lint_issues": $lint_issues,
      "checked_at": "$(date -Iseconds)"
    }
JSON
)
        
        # Append to report (using jq if available, otherwise sed)
        if command -v jq &> /dev/null; then
            jq ".components += {$component_json}" "$REPORT_FILE" > "$REPORT_FILE.tmp" && mv "$REPORT_FILE.tmp" "$REPORT_FILE"
        else
            # Fallback to sed
            sed -i "s/\"components\": {}/\"components\": {$component_json}/" "$REPORT_FILE"
        fi
        
        log "  Tests: $test_result | Score: $score | Lint: $lint_issues issues"
    else
        log "  WARNING: $path not found"
    fi
}

# Check all ecosystem components
check_component "ruchy-core" "$RUCHY_HOME"
check_component "ruchy-book" "/home/noah/src/ruchy-book"
check_component "ruchyruchy" "/home/noah/src/ruchyruchy"
check_component "rosetta-ruchy" "/home/noah/src/rosetta-ruchy"

# Generate summary
log "Generating quality summary..."

# Calculate average score
avg_score=$(grep "quality_score" "$REPORT_FILE" | grep -o "[0-9]\.[0-9]*" | awk '{sum+=$1} END {print sum/NR}' || echo "0.0")

# Create summary
cat >> "$REPORT_FILE" << EOF
,
  "summary": {
    "average_quality_score": $avg_score,
    "total_components_checked": 4,
    "monitoring_completed_at": "$(date -Iseconds)"
  }
}
EOF

log "Quality monitoring complete. Report saved to $REPORT_FILE"

# Send alerts if quality drops
if (( $(echo "$avg_score < 0.8" | bc -l) )); then
    log "⚠️ WARNING: Average quality score ($avg_score) below threshold (0.8)"
    # Could send email/slack notification here
fi

# Cleanup old reports (keep last 30 days)
find "$REPORT_DIR" -name "quality-report-*.json" -mtime +30 -delete
find "$REPORT_DIR" -name "quality-monitor-*.log" -mtime +30 -delete

log "Monitoring session complete"
exit 0