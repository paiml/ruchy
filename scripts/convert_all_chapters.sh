#!/usr/bin/env bash
# Convert all MD Book chapters to .rnb format
# Usage: ./scripts/convert_all_chapters.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MD_BOOK_SRC="$SCRIPT_DIR/../docs/notebook/book/src"
OUTPUT_DIR="$SCRIPT_DIR/../notebooks"
CONVERTER="$SCRIPT_DIR/md_to_notebook.rs"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔄 Converting MD Book chapters to .rnb format${NC}"
echo ""

total=0
success=0
failed=0

# Find all .md files in the book source
while IFS= read -r -d '' md_file; do
    ((total++))

    # Get relative path from book src
    rel_path="${md_file#$MD_BOOK_SRC/}"

    # Create output filename (flatten directory structure for simplicity)
    output_name=$(echo "$rel_path" | tr '/' '-' | sed 's/\.md$/.rnb/')
    output_file="$OUTPUT_DIR/$output_name"

    echo -n "Converting $rel_path... "

    # Convert
    if rust-script "$CONVERTER" "$md_file" "$output_file" > /dev/null 2>&1; then
        ((success++))
        echo -e "${GREEN}✅${NC}"
    else
        ((failed++))
        echo "❌ FAILED"
    fi
done < <(find "$MD_BOOK_SRC" -name "*.md" -print0)

echo ""
echo -e "${BLUE}📊 Conversion Summary${NC}"
echo "Total files: $total"
echo -e "${GREEN}Success: $success${NC}"
if [ $failed -gt 0 ]; then
    echo "Failed: $failed"
fi

echo ""
echo -e "${GREEN}✅ Notebooks created in: $OUTPUT_DIR${NC}"
echo ""
echo "Next steps:"
echo "  1. Start notebook server: cargo run --bin ruchy notebook"
echo "  2. Load a notebook via UI or API"
echo "  3. Execute cells and validate outputs"
