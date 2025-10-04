#!/bin/bash
# Script to migrate Value type usage from old API to new API
# Old: Rc<Vec<T>> and Rc<String>
# New: Rc<[T]> and Rc<str>

set -e

echo "🔧 Migrating Value type usage in test files..."

# Pattern 1: Value::Array(Rc::new(vec![...])) → Value::Array(vec![...].into())
# Pattern 2: Value::String(Rc::new("str".to_string())) → Value::String(Rc::from("str"))
# Pattern 3: Value::Tuple(Rc::new(vec![...])) → Value::Tuple(vec![...].into())

# Find all test files with the old patterns
TEST_FILES=$(grep -r "Value::Array(Rc::new(vec!" tests/ -l 2>/dev/null || true)
TEST_FILES+=" "
TEST_FILES+=$(grep -r "Value::String(Rc::new.*to_string()" tests/ -l 2>/dev/null || true)
TEST_FILES+=" "
TEST_FILES+=$(grep -r "Value::Tuple(Rc::new(vec!" tests/ -l 2>/dev/null || true)

# Remove duplicates
TEST_FILES=$(echo "$TEST_FILES" | tr ' ' '\n' | sort -u | grep -v "^$" || true)

if [ -z "$TEST_FILES" ]; then
    echo "✅ No files need migration"
    exit 0
fi

echo "📁 Files to migrate:"
echo "$TEST_FILES" | sed 's/^/  - /'

echo ""
echo "🔄 Applying migrations..."

for file in $TEST_FILES; do
    echo "  Processing: $file"

    # Create backup
    cp "$file" "$file.bak"

    # Migration 1: Simple one-line Array patterns
    # Value::Array(Rc::new(vec![...])) → Value::Array(vec![...].into())
    perl -i -pe 's/Value::Array\(Rc::new\(vec!\[(.*?)\]\)\)/Value::Array(vec![$1].into())/g' "$file"

    # Migration 2: String patterns
    # Value::String(Rc::new("str".to_string())) → Value::String(Rc::from("str"))
    perl -i -pe 's/Value::String\(Rc::new\("([^"]+)"\.to_string\(\)\)\)/Value::String(Rc::from("$1"))/g' "$file"

    # Migration 3: Simple one-line Tuple patterns
    # Value::Tuple(Rc::new(vec![...])) → Value::Tuple(vec![...].into())
    perl -i -pe 's/Value::Tuple\(Rc::new\(vec!\[(.*?)\]\)\)/Value::Tuple(vec![$1].into())/g' "$file"
done

echo ""
echo "✅ Migration complete!"
echo ""
echo "📊 Verifying compilation..."
if cargo test --no-run --quiet 2>&1 | grep -q "error"; then
    echo "❌ Compilation errors detected - manual fixes needed"
    echo ""
    echo "Common issues:"
    echo "  - Multiline vec![] patterns (need manual .into() addition)"
    echo "  - Nested Value constructions"
    echo ""
    echo "💾 Backups saved as *.bak"
    exit 1
else
    echo "✅ Compilation successful!"
    echo ""
    echo "🗑️  Cleaning up backups..."
    rm -f tests/*.bak
    echo "✅ Migration verified and complete!"
fi
