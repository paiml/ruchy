#!/bin/bash
# More targeted unwrap fixes with better error messages

echo "=== Targeted unwrap() fixes ==="
echo

# Fix parse().unwrap() patterns
echo "Fixing parse().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.parse()\.unwrap()/\.parse().expect("Failed to parse value")/g' {} \;

# Fix duration_since().unwrap() patterns
echo "Fixing duration_since().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.duration_since(\([^)]*\))\.unwrap()/\.duration_since(\1).unwrap_or_else(|_| std::time::Duration::from_secs(0))/g' {} \;

# Fix checked_sub().unwrap() patterns
echo "Fixing checked_sub().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.checked_sub(\([^)]*\))\.unwrap()/\.checked_sub(\1).expect("Subtraction underflow")/g' {} \;

# Fix to_string().unwrap() patterns
echo "Fixing to_string().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.to_string()\.unwrap()/\.to_string().expect("Failed to convert to string")/g' {} \;

# Fix from_str().unwrap() patterns
echo "Fixing from_str().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.from_str(\([^)]*\))\.unwrap()/\.from_str(\1).expect("Failed to parse from string")/g' {} \;

# Fix join().unwrap() patterns
echo "Fixing join().unwrap() patterns..."
find src -name "*.rs" -type f -exec sed -i.bak \
    's/\.join()\.unwrap()/\.join().expect("Thread join failed")/g' {} \;

# Clean up backup files
find src -name "*.rs.bak" -delete

echo
echo "✅ Targeted unwrap fixes complete"
echo
echo "Remaining unwraps (for manual review):"
rg '\.unwrap\(\)' src --type rust --stats | tail -5