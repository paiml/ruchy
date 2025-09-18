#!/bin/bash

# Fix all test files using old Repl::new() API to use new API with PathBuf

echo "Fixing test files to use new Repl::new(PathBuf) API..."

# Find all files using Repl::new() without arguments
find tests/ -name "*.rs" -exec grep -l "Repl::new()" {} \; | while read -r file; do
    echo "Fixing $file..."

    # Replace Repl::new() with Repl::new(std::env::temp_dir())
    sed -i 's/Repl::new()/Repl::new(std::env::temp_dir())/g' "$file"

    # Add std import if not present
    if ! grep -q "use std::" "$file"; then
        # Insert after existing use statements or at top
        if grep -q "^use " "$file"; then
            sed -i '/^use /a use std::env;' "$file"
        else
            sed -i '1iuse std::env;' "$file"
        fi
    elif ! grep -q "use std::env" "$file"; then
        # Add env to existing std imports
        sed -i 's/use std::/use std::{env, /1' "$file"
        sed -i 's/use std::{env, \([^}]*\)}/use std::{env, \1}/g' "$file"
    fi
done

echo "Completed fixing test files."