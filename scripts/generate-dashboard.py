#!/usr/bin/env python3
"""
WebAssembly Extreme Quality Assurance Framework v3.0
Quality Dashboard Generator
"""

import json
import argparse
import os
from datetime import datetime
from pathlib import Path

def load_json_file(filepath):
    """Load JSON file safely"""
    try:
        with open(filepath, 'r') as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return None

def parse_coverage_data(coverage_file):
    """Parse coverage JSON data"""
    data = load_json_file(coverage_file)
    if not data:
        return {"line_coverage": 0, "branch_coverage": 0, "function_coverage": 0}

    # Extract coverage metrics (format may vary)
    try:
        if 'data' in data and len(data['data']) > 0:
            totals = data['data'][0].get('totals', {})
            return {
                "line_coverage": totals.get('lines', {}).get('percent', 0),
                "branch_coverage": totals.get('branches', {}).get('percent', 0),
                "function_coverage": totals.get('functions', {}).get('percent', 0)
            }
    except (KeyError, IndexError):
        pass

    return {"line_coverage": 0, "branch_coverage": 0, "function_coverage": 0}

def parse_complexity_data(complexity_file):
    """Parse complexity analysis data"""
    if not os.path.exists(complexity_file):
        return {"violations": 0, "max_complexity": 0}

    violations = 0
    max_complexity = 0

    try:
        with open(complexity_file, 'r') as f:
            for line in f:
                if 'cognitive_complexity' in line or 'cyclomatic_complexity' in line:
                    violations += 1
                # Extract complexity numbers if possible
                if 'complexity of' in line.lower():
                    try:
                        # Extract number after "complexity of"
                        parts = line.split()
                        for i, part in enumerate(parts):
                            if part.isdigit():
                                complexity = int(part)
                                max_complexity = max(max_complexity, complexity)
                                break
                    except ValueError:
                        pass
    except FileNotFoundError:
        pass

    return {"violations": violations, "max_complexity": max_complexity}

def parse_audit_data(audit_file):
    """Parse security audit data"""
    if not os.path.exists(audit_file):
        return {"vulnerabilities": 0, "advisories": []}

    vulnerabilities = 0
    advisories = []

    try:
        with open(audit_file, 'r') as f:
            content = f.read()
            if 'vulnerabilities found' in content:
                # Extract vulnerability count
                for line in content.split('\n'):
                    if 'vulnerabilities found' in line:
                        parts = line.split()
                        for part in parts:
                            if part.isdigit():
                                vulnerabilities = int(part)
                                break

            # Extract advisory information
            if 'ID:' in content:
                for line in content.split('\n'):
                    if line.strip().startswith('ID:'):
                        advisories.append(line.strip())
    except FileNotFoundError:
        pass

    return {"vulnerabilities": vulnerabilities, "advisories": advisories}

def parse_size_data(size_file):
    """Parse binary size data"""
    if not os.path.exists(size_file):
        return {"size_kb": 0, "optimized_size_kb": 0}

    size_kb = 0
    optimized_size_kb = 0

    try:
        with open(size_file, 'r') as f:
            content = f.read()
            for line in content.split('\n'):
                if 'Original:' in line and 'B' in line:
                    # Extract size from line like "Original: 256KB"
                    parts = line.split()
                    for part in parts:
                        if 'KB' in part or 'kB' in part:
                            try:
                                size_kb = int(part.replace('KB', '').replace('kB', ''))
                            except ValueError:
                                pass
                elif 'Optimized:' in line and 'B' in line:
                    parts = line.split()
                    for part in parts:
                        if 'KB' in part or 'kB' in part:
                            try:
                                optimized_size_kb = int(part.replace('KB', '').replace('kB', ''))
                            except ValueError:
                                pass
    except FileNotFoundError:
        pass

    return {"size_kb": size_kb, "optimized_size_kb": optimized_size_kb}

def generate_dashboard_html(coverage, complexity, audit, size, output_file):
    """Generate HTML dashboard"""

    # Calculate overall score
    coverage_score = (coverage['line_coverage'] + coverage['branch_coverage'] + coverage['function_coverage']) / 3
    complexity_penalty = min(complexity['violations'] * 5, 50)  # Max 50 point penalty
    security_penalty = audit['vulnerabilities'] * 20  # 20 points per vulnerability
    size_penalty = max(0, (size['optimized_size_kb'] - 500) * 0.1)  # Penalty over 500KB

    overall_score = max(0, 100 - complexity_penalty - security_penalty - size_penalty)

    # Grade calculation
    if overall_score >= 90:
        grade = "A"
        grade_color = "#28a745"
    elif overall_score >= 80:
        grade = "B"
        grade_color = "#ffc107"
    elif overall_score >= 70:
        grade = "C"
        grade_color = "#fd7e14"
    else:
        grade = "F"
        grade_color = "#dc3545"

    html_content = f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WASM Quality Dashboard</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(45deg, #1e3c72, #2a5298);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .header p {{
            margin: 10px 0 0 0;
            opacity: 0.9;
        }}
        .dashboard {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            padding: 30px;
        }}
        .metric-card {{
            background: #f8f9fa;
            border-radius: 10px;
            padding: 25px;
            text-align: center;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
            transition: transform 0.3s ease;
        }}
        .metric-card:hover {{
            transform: translateY(-5px);
        }}
        .metric-title {{
            font-size: 1.1em;
            font-weight: 600;
            color: #495057;
            margin-bottom: 15px;
        }}
        .metric-value {{
            font-size: 2.5em;
            font-weight: bold;
            margin-bottom: 10px;
        }}
        .metric-unit {{
            font-size: 0.9em;
            color: #6c757d;
        }}
        .grade-card {{
            background: linear-gradient(45deg, {grade_color}, {grade_color}cc);
            color: white;
            grid-column: span 2;
        }}
        .grade-value {{
            font-size: 4em;
            font-weight: bold;
        }}
        .coverage-good {{ color: #28a745; }}
        .coverage-warning {{ color: #ffc107; }}
        .coverage-danger {{ color: #dc3545; }}
        .complexity-good {{ color: #28a745; }}
        .complexity-warning {{ color: #ffc107; }}
        .security-good {{ color: #28a745; }}
        .security-danger {{ color: #dc3545; }}
        .size-good {{ color: #28a745; }}
        .size-warning {{ color: #ffc107; }}
        .details {{
            padding: 30px;
            background: #f8f9fa;
        }}
        .details h3 {{
            color: #495057;
            border-bottom: 2px solid #dee2e6;
            padding-bottom: 10px;
        }}
        .recommendation {{
            background: #e3f2fd;
            border-left: 4px solid #2196f3;
            padding: 15px;
            margin: 15px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>WebAssembly Quality Dashboard</h1>
            <p>Generated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
        </div>

        <div class="dashboard">
            <div class="metric-card grade-card">
                <div class="metric-title">Overall Grade</div>
                <div class="metric-value grade-value">{grade}</div>
                <div class="metric-unit">{overall_score:.1f}/100</div>
            </div>

            <div class="metric-card">
                <div class="metric-title">Line Coverage</div>
                <div class="metric-value {'coverage-good' if coverage['line_coverage'] >= 80 else 'coverage-warning' if coverage['line_coverage'] >= 60 else 'coverage-danger'}">{coverage['line_coverage']:.1f}</div>
                <div class="metric-unit">%</div>
            </div>

            <div class="metric-card">
                <div class="metric-title">Branch Coverage</div>
                <div class="metric-value {'coverage-good' if coverage['branch_coverage'] >= 90 else 'coverage-warning' if coverage['branch_coverage'] >= 75 else 'coverage-danger'}">{coverage['branch_coverage']:.1f}</div>
                <div class="metric-unit">%</div>
            </div>

            <div class="metric-card">
                <div class="metric-title">Complexity Violations</div>
                <div class="metric-value {'complexity-good' if complexity['violations'] == 0 else 'coverage-warning'}">{complexity['violations']}</div>
                <div class="metric-unit">functions</div>
            </div>

            <div class="metric-card">
                <div class="metric-title">Security Issues</div>
                <div class="metric-value {'security-good' if audit['vulnerabilities'] == 0 else 'security-danger'}">{audit['vulnerabilities']}</div>
                <div class="metric-unit">vulnerabilities</div>
            </div>

            <div class="metric-card">
                <div class="metric-title">Binary Size</div>
                <div class="metric-value {'size-good' if size['optimized_size_kb'] <= 500 else 'size-warning'}">{size['optimized_size_kb']}</div>
                <div class="metric-unit">KB</div>
            </div>
        </div>

        <div class="details">
            <h3>Quality Analysis</h3>

            <div class="recommendation">
                <strong>Coverage Status:</strong>
                {"✅ Excellent coverage" if coverage_score >= 85 else "⚠️ Coverage needs improvement" if coverage_score >= 70 else "❌ Coverage critically low"}
                - Line: {coverage['line_coverage']:.1f}%, Branch: {coverage['branch_coverage']:.1f}%, Function: {coverage['function_coverage']:.1f}%
            </div>

            <div class="recommendation">
                <strong>Complexity Status:</strong>
                {"✅ No complexity violations" if complexity['violations'] == 0 else f"⚠️ {complexity['violations']} functions exceed complexity thresholds"}
                {f" (Max complexity: {complexity['max_complexity']})" if complexity['max_complexity'] > 0 else ""}
            </div>

            <div class="recommendation">
                <strong>Security Status:</strong>
                {"✅ No known vulnerabilities" if audit['vulnerabilities'] == 0 else f"❌ {audit['vulnerabilities']} security vulnerabilities found"}
            </div>

            <div class="recommendation">
                <strong>Size Status:</strong>
                {"✅ Binary size within limits" if size['optimized_size_kb'] <= 500 else f"⚠️ Binary size exceeds 500KB limit by {size['optimized_size_kb'] - 500}KB"}
            </div>

            <h3>Recommendations</h3>
            {generate_recommendations(coverage, complexity, audit, size)}
        </div>
    </div>
</body>
</html>
"""

    with open(output_file, 'w') as f:
        f.write(html_content)

def generate_recommendations(coverage, complexity, audit, size):
    """Generate specific recommendations"""
    recommendations = []

    if coverage['branch_coverage'] < 90:
        recommendations.append("• Increase branch coverage to 90% by adding more test cases for conditional logic")

    if complexity['violations'] > 0:
        recommendations.append("• Refactor functions with high complexity by extracting helper methods")

    if audit['vulnerabilities'] > 0:
        recommendations.append("• Update dependencies to resolve security vulnerabilities")

    if size['optimized_size_kb'] > 500:
        recommendations.append("• Optimize binary size using wasm-opt and remove unused features")

    if not recommendations:
        recommendations.append("• All quality metrics are within acceptable ranges ✅")

    return '<br>'.join(recommendations)

def main():
    parser = argparse.ArgumentParser(description='Generate WASM Quality Dashboard')
    parser.add_argument('--coverage', default='target/coverage/rust.lcov', help='Coverage report file')
    parser.add_argument('--complexity', default='target/complexity/clippy-complexity.log', help='Complexity report file')
    parser.add_argument('--audit', default='target/security/audit-report.txt', help='Security audit file')
    parser.add_argument('--size', default='target/size-analysis.txt', help='Size analysis file')
    parser.add_argument('--output', default='dashboard.html', help='Output HTML file')

    args = parser.parse_args()

    print("🔍 Generating quality dashboard...")

    # Parse all input files
    coverage = parse_coverage_data(args.coverage)
    complexity = parse_complexity_data(args.complexity)
    audit = parse_audit_data(args.audit)
    size = parse_size_data(args.size)

    # Generate dashboard
    generate_dashboard_html(coverage, complexity, audit, size, args.output)

    print(f"✅ Dashboard generated: {args.output}")
    print(f"📊 Metrics: Coverage={coverage['branch_coverage']:.1f}%, Complexity={complexity['violations']} violations, Security={audit['vulnerabilities']} issues, Size={size['optimized_size_kb']}KB")

if __name__ == '__main__':
    main()