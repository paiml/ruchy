#!/usr/bin/env python3
"""Find unwrap() calls in production code (not tests)."""

import os
import re
from pathlib import Path

def is_test_code(lines, line_num):
    """Check if a line is within test code."""
    # Look backwards for test markers
    for i in range(max(0, line_num - 20), line_num):
        if '#[test]' in lines[i] or '#[cfg(test)]' in lines[i] or 'fn test_' in lines[i]:
            # Check if we're still in the same function
            brace_count = 0
            for j in range(i, min(line_num + 1, len(lines))):
                brace_count += lines[j].count('{') - lines[j].count('}')
                if j < line_num and brace_count == 0:
                    return False  # We've exited the test function
            return brace_count > 0  # Still in test function
    return False

def find_unwraps(filepath):
    """Find unwrap() calls in a file that aren't in test code."""
    unwraps = []
    try:
        with open(filepath, 'r') as f:
            lines = f.readlines()
        
        for i, line in enumerate(lines):
            if '.unwrap()' in line:
                # Skip if it's a comment or doc comment
                stripped = line.strip()
                if stripped.startswith('//') or stripped.startswith('///'):
                    continue
                    
                # Skip if it's in test code
                if not is_test_code(lines, i):
                    unwraps.append((i + 1, line.strip()))
    except:
        pass
    
    return unwraps

def main():
    src_dir = Path('src')
    
    # Files to check (excluding test files)
    files_to_check = []
    for rust_file in src_dir.rglob('*.rs'):
        if '_test.rs' not in str(rust_file) and '/tests/' not in str(rust_file):
            files_to_check.append(rust_file)
    
    production_unwraps = {}
    
    for filepath in files_to_check:
        unwraps = find_unwraps(filepath)
        if unwraps:
            production_unwraps[str(filepath)] = unwraps
    
    # Print results
    total = 0
    for filepath, unwraps in sorted(production_unwraps.items(), key=lambda x: -len(x[1]))[:10]:
        print(f"\n{filepath}: {len(unwraps)} unwraps")
        for line_num, line in unwraps[:3]:  # Show first 3 examples
            print(f"  {line_num}: {line[:80]}...")
        total += len(unwraps)
    
    print(f"\nTotal production unwraps: {total}")

if __name__ == "__main__":
    main()