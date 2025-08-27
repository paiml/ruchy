#!/bin/bash
# Setup script for Ruchy quality monitoring cron jobs

set -e

echo "🚀 Ruchy Quality Monitoring Setup"
echo "================================="

# Check if running as appropriate user
if [ "$USER" != "noah" ]; then
    echo "⚠️  Warning: Running as $USER, expected noah"
    echo "   Adjust paths in crontab.conf if needed"
fi

# Make scripts executable
echo "Setting up executable permissions..."
chmod +x /home/noah/src/ruchy/cron/*.sh

# Create report directories
echo "Creating report directories..."
mkdir -p /home/noah/src/ruchy/quality-reports/{daily,weekly,monthly,health}

# Test scripts
echo "Testing monitoring scripts..."
if /home/noah/src/ruchy/cron/health-check.sh; then
    echo "✅ Health check script works"
else
    echo "❌ Health check script failed"
    exit 1
fi

# Install crontab
echo ""
echo "Current crontab:"
crontab -l 2>/dev/null || echo "  (empty)"

echo ""
echo "To install the monitoring cron jobs, run:"
echo "  crontab /home/noah/src/ruchy/cron/crontab.conf"
echo ""
echo "Or to append to existing crontab:"
echo "  (crontab -l 2>/dev/null; cat /home/noah/src/ruchy/cron/crontab.conf) | crontab -"
echo ""
echo "To verify installation:"
echo "  crontab -l"
echo ""
echo "To remove monitoring jobs:"
echo "  crontab -r"
echo ""
echo "Manual test commands:"
echo "  /home/noah/src/ruchy/cron/health-check.sh      # Quick health check"
echo "  /home/noah/src/ruchy/cron/quality-monitor.sh   # Full quality scan"
echo ""
echo "✅ Setup complete! Quality monitoring is ready to deploy."