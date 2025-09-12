#!/usr/bin/env python3
"""
Analyze cyclomatic complexity of Rust functions
"""
import os
import re
from collections import defaultdict

def count_complexity_indicators(function_body):
    """Count complexity indicators in a function body"""
    complexity = 1  # Base complexity
    
    # Count control flow keywords
    patterns = [
        r'\bif\b', r'\belse\b', r'\bfor\b', r'\bwhile\b', r'\bloop\b',
        r'\bmatch\b', r'=>', r'\?', r'\.unwrap\(', r'\.expect\(',
        r'\breturn\b', r'\bbreak\b', r'\bcontinue\b',
        r'&&', r'\|\|'
    ]
    
    for pattern in patterns:
        matches = re.findall(pattern, function_body)
        complexity += len(matches)
    
    return complexity

def analyze_rust_file(filepath):
    """Analyze complexity of functions in a Rust file"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Find function definitions
    function_pattern = r'(pub\s+)?(async\s+)?fn\s+(\w+)[^{]*\{((?:[^{}]|\{[^{}]*\})*)\}'
    
    functions = []
    for match in re.finditer(function_pattern, content):
        fn_name = match.group(3)
        fn_body = match.group(4)
        
        # Skip test functions
        if fn_name.startswith('test_'):
            continue
            
        complexity = count_complexity_indicators(fn_body)
        
        if complexity > 10:  # Only report high complexity
            functions.append((fn_name, complexity, filepath))
    
    return functions

# Analyze all Rust files
high_complexity_functions = []

for root, dirs, files in os.walk('src'):
    # Skip test directories
    if 'tests' in root or 'test' in root:
        continue
        
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            functions = analyze_rust_file(filepath)
            high_complexity_functions.extend(functions)

# Sort by complexity
high_complexity_functions.sort(key=lambda x: x[1], reverse=True)

# Report findings
print("=== HIGH COMPLEXITY FUNCTIONS (>10) ===\n")
print(f"Found {len(high_complexity_functions)} functions with cyclomatic complexity >10\n")

# Show top 20
for i, (fn_name, complexity, filepath) in enumerate(high_complexity_functions[:20], 1):
    rel_path = filepath.replace('src/', '')
    print(f"{i:2}. {fn_name:30} complexity: {complexity:3} in {rel_path}")

# Group by file
by_file = defaultdict(list)
for fn_name, complexity, filepath in high_complexity_functions:
    by_file[filepath].append((fn_name, complexity))

print("\n=== FILES WITH MOST COMPLEX FUNCTIONS ===\n")
file_complexities = [(f, sum(c for _, c in fns), len(fns)) 
                     for f, fns in by_file.items()]
file_complexities.sort(key=lambda x: x[1], reverse=True)

for filepath, total_complexity, count in file_complexities[:10]:
    rel_path = filepath.replace('src/', '')
    print(f"{rel_path:50} {count:2} functions, total complexity: {total_complexity}")