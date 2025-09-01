# TDG Real-Time Dashboard Guide

## Overview

The TDG (Technical Debt Gradient) Real-Time Dashboard provides continuous quality monitoring for the Ruchy codebase. It offers real-time insights into code quality metrics, performance profiling, and technical debt trends.

## Features

### Core Capabilities
- **Real-time Metrics**: 5-second automatic updates
- **Storage Monitoring**: Hot/Warm/Cold tier analysis
- **Performance Profiling**: CPU, I/O, Memory, Lock contention detection
- **Interactive Analysis**: Server-Sent Events for live updates
- **Flame Graphs**: Visual performance bottleneck identification
- **Quality Tracking**: File-level TDG scores and trends

### Dashboard Components

1. **System Metrics Panel**
   - Overall TDG score and grade
   - File count and coverage percentage
   - Complexity distribution histogram
   - Real-time trend graphs

2. **File Analysis Grid**
   - Per-file TDG scores
   - Complexity metrics (structural, semantic, cognitive)
   - Duplication percentages
   - Coupling analysis

3. **Performance Profiler**
   - CPU usage patterns
   - Memory allocation tracking
   - I/O bottleneck detection
   - Lock contention analysis

4. **Storage Backend Monitor**
   - Hot tier: Frequently accessed files
   - Warm tier: Moderately accessed files
   - Cold tier: Rarely accessed files
   - Storage optimization recommendations

## Quick Start

### Starting the Dashboard

```bash
# Start dashboard in background with browser auto-open
./scripts/tdg_dashboard.sh start --open

# Start dashboard on custom port
./scripts/tdg_dashboard.sh start --port 3000

# Start with custom update interval (seconds)
./scripts/tdg_dashboard.sh start --interval 10
```

### Interactive Mode

```bash
# Run dashboard in foreground (useful for debugging)
./scripts/tdg_dashboard.sh watch
```

### Managing the Dashboard

```bash
# Check dashboard status
./scripts/tdg_dashboard.sh status

# Stop the dashboard
./scripts/tdg_dashboard.sh stop

# Restart the dashboard
./scripts/tdg_dashboard.sh restart
```

## Makefile Integration

The dashboard is integrated into the project Makefile for convenience:

```bash
# Start TDG dashboard
make tdg-dashboard

# Stop TDG dashboard
make tdg-stop

# Check dashboard status
make tdg-status
```

## Dashboard URL

Once started, the dashboard is accessible at:
- Default: http://localhost:8080
- Custom port: http://localhost:{PORT}

## Interpreting Metrics

### TDG Grades

- **A+ (95-100)**: Excellent - No action needed
- **A (90-94)**: Very Good - Maintain quality
- **A- (85-89)**: Good - Monitor trends
- **B+ (80-84)**: Above Average - Consider improvements
- **B (75-79)**: Average - Plan refactoring
- **B- (70-74)**: Below Average - Priority improvements needed
- **C (60-69)**: Poor - Immediate action required
- **D (50-59)**: Very Poor - Critical refactoring needed
- **F (<50)**: Failing - Emergency intervention required

### Key Metrics to Monitor

1. **Structural Complexity**
   - Cyclomatic complexity per function
   - Target: ≤20 per function
   - Critical: >50 requires immediate refactoring

2. **Semantic Complexity**
   - Cognitive complexity measurements
   - Target: ≤15 per function
   - Critical: >30 indicates confusing code

3. **Code Duplication**
   - Percentage of duplicated code
   - Target: <10%
   - Critical: >20% requires DRY refactoring

4. **Coupling**
   - Module dependency counts
   - Target: <10 imports per module
   - Critical: >20 indicates tight coupling

## Workflow Integration

### Development Workflow

1. **Start of Sprint**
   ```bash
   ./scripts/tdg_dashboard.sh start --open
   ```

2. **During Development**
   - Monitor real-time TDG scores
   - Watch for degradation warnings
   - Use flame graphs to identify hotspots

3. **Before Commit**
   - Check dashboard for any red flags
   - Ensure no files dropped below A- grade
   - Review performance metrics

4. **End of Sprint**
   ```bash
   ./scripts/tdg_dashboard.sh stop
   ```

### Continuous Integration

The dashboard can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions integration
- name: Start TDG Dashboard
  run: ./scripts/tdg_dashboard.sh start
  
- name: Run Quality Checks
  run: |
    sleep 10  # Allow dashboard to collect metrics
    curl -s http://localhost:8080/api/metrics > tdg-metrics.json
    
- name: Validate Quality Gates
  run: |
    GRADE=$(jq -r '.overall_grade' tdg-metrics.json)
    if [[ "$GRADE" < "85" ]]; then
      echo "Quality gate failed: Grade $GRADE < 85"
      exit 1
    fi
```

## Troubleshooting

### Dashboard Won't Start

1. Check if port is already in use:
   ```bash
   lsof -i :8080
   ```

2. Check PMAT installation:
   ```bash
   pmat --version
   ```

3. Review log file:
   ```bash
   tail -f .tdg_dashboard.log
   ```

### Dashboard Stops Unexpectedly

1. Check system resources:
   ```bash
   free -h
   df -h
   ```

2. Review error logs:
   ```bash
   grep ERROR .tdg_dashboard.log
   ```

### Metrics Not Updating

1. Verify file changes are saved
2. Check update interval setting
3. Refresh browser (Ctrl+F5)

## Advanced Configuration

### Custom Metrics

Create `.tdg_dashboard.config` for custom settings:

```json
{
  "update_interval": 5,
  "port": 8080,
  "metrics": {
    "complexity_threshold": 20,
    "duplication_threshold": 10,
    "coupling_threshold": 15
  },
  "storage_tiers": {
    "hot_threshold_days": 1,
    "warm_threshold_days": 7,
    "cold_threshold_days": 30
  }
}
```

### API Endpoints

The dashboard exposes REST API endpoints:

- `GET /api/metrics` - Current overall metrics
- `GET /api/files` - Per-file TDG scores
- `GET /api/trends` - Historical trend data
- `GET /api/performance` - Performance profiling data
- `GET /api/storage` - Storage tier analysis

### Export Options

Export dashboard data for reporting:

```bash
# Export to JSON
curl http://localhost:8080/api/export/json > tdg-report.json

# Export to CSV
curl http://localhost:8080/api/export/csv > tdg-report.csv

# Export to Markdown
curl http://localhost:8080/api/export/markdown > tdg-report.md
```

## Best Practices

1. **Regular Monitoring**
   - Start dashboard at beginning of work session
   - Check metrics before major commits
   - Review trends weekly

2. **Team Collaboration**
   - Share dashboard URL during pair programming
   - Include dashboard screenshots in PRs
   - Use metrics in sprint retrospectives

3. **Quality Gates**
   - Set minimum grade thresholds (A- recommended)
   - Block merges if grade drops
   - Track improvement trends over sprints

4. **Performance Optimization**
   - Use flame graphs to identify bottlenecks
   - Monitor memory usage trends
   - Track I/O patterns for optimization

## Toyota Way Integration

The TDG Dashboard embodies Toyota Way principles:

- **Genchi Genbutsu**: Go to the source - real-time metrics
- **Jidoka**: Built-in quality - automatic detection of issues
- **Kaizen**: Continuous improvement - trend tracking
- **Andon**: Visual management - color-coded quality indicators
- **Heijunka**: Level workload - complexity distribution analysis

## Support

For issues or feature requests:
- Check logs: `.tdg_dashboard.log`
- Run diagnostics: `pmat tdg diagnose`
- File issues: https://github.com/paiml/ruchy/issues

## Related Documentation

- [TDG Tracking Guide](./tdg-tracking-guide.md)
- [PMAT Quality Gates](./pmat-quality-gates.md)
- [Development Workflow](./development-workflow.md)