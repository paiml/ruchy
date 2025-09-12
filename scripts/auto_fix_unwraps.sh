#!/bin/bash
# Automatically fix common unwrap() patterns with proper error handling

echo "=== Automated unwrap() fixes ==="
echo

# Fix lock().unwrap() patterns
echo "Fixing lock().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.lock()\.unwrap()/\.lock().expect("Failed to acquire lock")/g' {} \;

# Fix parse().unwrap() patterns
echo "Fixing parse().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.parse()\.unwrap()/\.parse().expect("Failed to parse")/g' {} \;

# Fix to_string().unwrap() patterns  
echo "Fixing to_string().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.to_string()\.unwrap()/\.to_string().expect("Failed to convert to string")/g' {} \;

# Fix to_str().unwrap() patterns
echo "Fixing to_str().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.to_str()\.unwrap()/\.to_str().expect("Failed to convert to str")/g' {} \;

# Fix from_str().unwrap() patterns
echo "Fixing from_str().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.from_str()\.unwrap()/\.from_str().expect("Failed to parse from string")/g' {} \;

# Fix join().unwrap() patterns
echo "Fixing join().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.join()\.unwrap()/\.join().expect("Thread failed to join")/g' {} \;

# Count remaining unwraps
echo
echo "=== Results ==="
echo "Remaining unwrap() calls:"
grep -r '\.unwrap()' src --include='*.rs' | wc -l

# Clean up backup files
find src -name "*.rs.bak" -delete

echo "Done! Remember to:"
echo "1. Review the changes with 'git diff'"
echo "2. Run 'cargo test' to ensure everything still works"
echo "3. Manually fix any complex unwrap patterns that need custom error handling"