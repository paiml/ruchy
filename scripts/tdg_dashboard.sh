#!/bin/bash
# TDG Real-Time Dashboard Integration Script
# [TDG-001] Continuous quality monitoring with PMAT TDG dashboard

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
DASHBOARD_PORT=8080
UPDATE_INTERVAL=5
LOG_FILE=".tdg_dashboard.log"
PID_FILE=".tdg_dashboard.pid"

# Functions
show_help() {
    echo "TDG Real-Time Dashboard Manager"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  start       Start the TDG dashboard in background"
    echo "  stop        Stop the running dashboard"
    echo "  status      Check dashboard status"
    echo "  restart     Restart the dashboard"
    echo "  watch       Start dashboard in foreground (interactive)"
    echo ""
    echo "Options:"
    echo "  --port PORT         Dashboard port (default: 8080)"
    echo "  --interval SECONDS  Update interval (default: 5)"
    echo "  --open              Open dashboard in browser after starting"
    echo ""
    echo "Examples:"
    echo "  $0 start --open     # Start dashboard and open browser"
    echo "  $0 watch            # Run dashboard interactively"
    echo "  $0 stop             # Stop background dashboard"
}

start_dashboard() {
    local OPEN_BROWSER=$1
    
    # Check if already running
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}⚠️ Dashboard already running (PID: $PID)${NC}"
            echo "Use '$0 stop' to stop it first"
            exit 1
        fi
    fi
    
    echo -e "${BLUE}🚀 Starting TDG Real-Time Dashboard...${NC}"
    echo "Port: $DASHBOARD_PORT"
    echo "Update Interval: ${UPDATE_INTERVAL}s"
    echo ""
    
    # Start dashboard in background
    nohup pmat tdg dashboard \
        --port "$DASHBOARD_PORT" \
        --update-interval "$UPDATE_INTERVAL" \
        > "$LOG_FILE" 2>&1 &
    
    DASHBOARD_PID=$!
    echo "$DASHBOARD_PID" > "$PID_FILE"
    
    # Wait for startup
    echo -n "Starting dashboard."
    for i in {1..5}; do
        sleep 1
        echo -n "."
    done
    echo ""
    
    # Check if dashboard started successfully
    if ps -p "$DASHBOARD_PID" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Dashboard started successfully (PID: $DASHBOARD_PID)${NC}"
        echo ""
        echo "Dashboard URL: http://localhost:$DASHBOARD_PORT"
        echo "Log file: $LOG_FILE"
        echo ""
        echo "Features available:"
        echo "  • Real-time system metrics (5-second updates)"
        echo "  • Storage backend monitoring (Hot/Warm/Cold tiers)"
        echo "  • Performance profiling with flame graphs"
        echo "  • Bottleneck detection (CPU, I/O, Memory, Lock contention)"
        echo "  • Interactive analysis with Server-Sent Events"
        echo ""
        
        if [ "$OPEN_BROWSER" = "true" ]; then
            echo "Opening dashboard in browser..."
            if command -v xdg-open > /dev/null; then
                xdg-open "http://localhost:$DASHBOARD_PORT" 2>/dev/null &
            elif command -v open > /dev/null; then
                open "http://localhost:$DASHBOARD_PORT" 2>/dev/null &
            else
                echo -e "${YELLOW}Could not auto-open browser. Please navigate to: http://localhost:$DASHBOARD_PORT${NC}"
            fi
        fi
    else
        echo -e "${RED}❌ Failed to start dashboard${NC}"
        echo "Check log file for details: $LOG_FILE"
        rm -f "$PID_FILE"
        exit 1
    fi
}

stop_dashboard() {
    if [ ! -f "$PID_FILE" ]; then
        echo -e "${YELLOW}⚠️ Dashboard not running (no PID file found)${NC}"
        exit 0
    fi
    
    PID=$(cat "$PID_FILE")
    
    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${BLUE}🛑 Stopping TDG Dashboard (PID: $PID)...${NC}"
        kill "$PID"
        
        # Wait for shutdown
        for i in {1..5}; do
            if ! ps -p "$PID" > /dev/null 2>&1; then
                break
            fi
            sleep 1
        done
        
        # Force kill if still running
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}Force stopping...${NC}"
            kill -9 "$PID"
        fi
        
        echo -e "${GREEN}✅ Dashboard stopped${NC}"
    else
        echo -e "${YELLOW}⚠️ Dashboard not running (PID $PID not found)${NC}"
    fi
    
    rm -f "$PID_FILE"
}

check_status() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Dashboard is running${NC}"
            echo "PID: $PID"
            echo "URL: http://localhost:$DASHBOARD_PORT"
            echo ""
            echo "Recent log entries:"
            tail -n 5 "$LOG_FILE" 2>/dev/null || echo "No log entries available"
        else
            echo -e "${RED}❌ Dashboard is not running${NC}"
            echo "Stale PID file found: $PID"
            rm -f "$PID_FILE"
        fi
    else
        echo -e "${YELLOW}⚠️ Dashboard is not running${NC}"
    fi
}

watch_dashboard() {
    echo -e "${BLUE}🔍 Starting TDG Dashboard in watch mode...${NC}"
    echo "Port: $DASHBOARD_PORT"
    echo "Update Interval: ${UPDATE_INTERVAL}s"
    echo "Press Ctrl+C to stop"
    echo ""
    
    # Run in foreground
    pmat tdg dashboard \
        --port "$DASHBOARD_PORT" \
        --update-interval "$UPDATE_INTERVAL" \
        --open
}

# Parse command line arguments
COMMAND=$1
shift || true

OPEN_BROWSER=false

while [ $# -gt 0 ]; do
    case "$1" in
        --port)
            DASHBOARD_PORT=$2
            shift 2
            ;;
        --interval)
            UPDATE_INTERVAL=$2
            shift 2
            ;;
        --open)
            OPEN_BROWSER=true
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Execute command
case "$COMMAND" in
    start)
        start_dashboard "$OPEN_BROWSER"
        ;;
    stop)
        stop_dashboard
        ;;
    status)
        check_status
        ;;
    restart)
        stop_dashboard
        sleep 2
        start_dashboard "$OPEN_BROWSER"
        ;;
    watch)
        watch_dashboard
        ;;
    help|--help|-h|"")
        show_help
        ;;
    *)
        echo "Unknown command: $COMMAND"
        show_help
        exit 1
        ;;
esac