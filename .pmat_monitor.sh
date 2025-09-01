#!/bin/bash
# PMAT TDG Real-Time Monitoring Script (CLAUDE.md v2.39.0)
# Implements mandatory real-time quality monitoring requirements

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 PMAT TDG Real-Time Monitoring (v2.39.0)${NC}"

# Check if PMAT TDG dashboard is available
if ! command -v pmat >/dev/null 2>&1; then
    echo "❌ PMAT not found. Install pmat to use TDG monitoring."
    exit 1
fi

# Function to start TDG dashboard
start_dashboard() {
    echo -e "${GREEN}📊 Starting TDG Dashboard...${NC}"
    echo "  - Real-time monitoring with 5-second updates"
    echo "  - Storage backend monitoring (Hot/Warm/Cold tiers)"
    echo "  - Performance profiling with flame graphs"
    echo "  - Interactive analysis with Server-Sent Events"
    
    # Start dashboard in background if not already running
    if ! pgrep -f "pmat tdg dashboard" > /dev/null; then
        pmat tdg dashboard --port 8080 --update-interval 5 --open &
        DASHBOARD_PID=$!
        echo "✅ TDG Dashboard started on http://localhost:8080 (PID: $DASHBOARD_PID)"
    else
        echo "ℹ️ TDG Dashboard already running"
    fi
}

# Function to start MCP server (optional enterprise integration)
start_mcp() {
    echo -e "${GREEN}🔧 Starting PMAT MCP Server...${NC}"
    echo "  - Enterprise-grade analysis with persistence"
    echo "  - System health and performance monitoring" 
    echo "  - Advanced profiling with flame graphs"
    echo "  - Configurable alert system"
    
    if ! pgrep -f "pmat mcp serve" > /dev/null; then
        pmat mcp serve --port 3000 &
        MCP_PID=$!
        echo "✅ MCP Server started on http://localhost:3000 (PID: $MCP_PID)"
    else
        echo "ℹ️ MCP Server already running"
    fi
}

# Function to run baseline TDG check
baseline_check() {
    echo -e "${YELLOW}📋 Running TDG baseline check...${NC}"
    
    # Get current TDG score
    TDG_SCORE=$(pmat tdg . --quiet 2>/dev/null || echo "0")
    if (( $(echo "$TDG_SCORE >= 85" | bc -l 2>/dev/null || echo "1") )); then
        echo "✅ Current TDG Score: $TDG_SCORE (≥85 A- required)"
    else
        echo "⚠️ Current TDG Score: $TDG_SCORE (below 85 A- threshold)"
        echo "Run: pmat tdg . --include-components --format=table"
    fi
    
    # Create baseline if it doesn't exist
    if [ ! -f ".tdg_baseline.json" ]; then
        echo "📝 Creating TDG baseline..."
        pmat tdg . --format=json > .tdg_baseline.json 2>/dev/null || true
    fi
}

# Main execution
case "${1:-start}" in
    "start")
        baseline_check
        start_dashboard
        # start_mcp  # Uncomment for enterprise MCP integration
        echo ""
        echo -e "${GREEN}🎯 PMAT monitoring started successfully!${NC}"
        echo "  - Dashboard: http://localhost:8080"
        echo "  - Use 'Ctrl+C' to stop monitoring"
        ;;
    "stop")
        echo "🛑 Stopping PMAT monitoring..."
        pkill -f "pmat tdg dashboard" && echo "✅ Dashboard stopped" || echo "ℹ️ Dashboard not running"
        pkill -f "pmat mcp serve" && echo "✅ MCP Server stopped" || echo "ℹ️ MCP Server not running"
        ;;
    "status")
        echo "📊 PMAT Monitoring Status:"
        if pgrep -f "pmat tdg dashboard" > /dev/null; then
            echo "  ✅ TDG Dashboard: Running (http://localhost:8080)"
        else
            echo "  ❌ TDG Dashboard: Not running"
        fi
        
        if pgrep -f "pmat mcp serve" > /dev/null; then
            echo "  ✅ MCP Server: Running (http://localhost:3000)"
        else
            echo "  ❌ MCP Server: Not running"
        fi
        
        baseline_check
        ;;
    *)
        echo "Usage: $0 {start|stop|status}"
        echo ""
        echo "Commands:"
        echo "  start   - Start TDG dashboard and monitoring"
        echo "  stop    - Stop all monitoring services"
        echo "  status  - Check monitoring service status"
        exit 1
        ;;
esac