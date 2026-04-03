# Sub-spec: Directory Walking — CLI Tools

**Parent:** [multi-threaded-dir-walk-spec.md](../multi-threaded-dir-walk-spec.md) CLI Tools Section

---

## CLI Tools for Sysadmins (No Coding Required)

### Philosophy: Python's `-m` Module Pattern

**Inspiration**: Python's success with CLI utilities that require zero coding:
```bash
python3 -m http.server    # Instant web server
python3 -m json.tool      # Pretty-print JSON
python3 -m pdb script.py  # Interactive debugger
python3 -m venv myenv     # Virtual environment
```

**Ruchy's Approach**: First-class CLI tools for common file operations:
```bash
ruchy find       # Find files (like GNU find, but simpler)
ruchy tree       # Display directory tree (like tree, but faster)
ruchy du         # Disk usage analysis (like du, but clearer)
ruchy count      # Count files/lines/words (like wc, but better)
ruchy rg         # Fast text search (like ripgrep, parallel by default)
```

---

### 1. `ruchy find` - Smart File Finder

**Purpose**: Simplified alternative to GNU `find` with sensible defaults

**Basic Usage**:
```bash
# Find all files in current directory (recursive)
ruchy find

# Find in specific directory
ruchy find /data/logs

# Find specific file types
ruchy find --type f         # Files only
ruchy find --type d         # Directories only

# Find by extension
ruchy find --ext .py        # All Python files
ruchy find --ext .log       # All log files

# Find by name pattern (glob)
ruchy find --name "*.csv"
ruchy find --name "test_*"

# Find by size
ruchy find --size +100M     # Files > 100MB
ruchy find --size -1K       # Files < 1KB
ruchy find --size 10M       # Files exactly 10MB

# Find by modification time
ruchy find --mtime -7       # Modified in last 7 days
ruchy find --mtime +30      # Modified >30 days ago

# Combine filters
ruchy find --ext .log --size +10M --mtime -7
```

**Advanced Usage**:
```bash
# Execute command on each file (parallel)
ruchy find --ext .py --exec "ruchy lint {}"
ruchy find --ext .csv --exec "wc -l {}"

# Output formats
ruchy find --format json    # JSON output for scripting
ruchy find --format csv     # CSV for spreadsheets
ruchy find --format table   # Pretty table (default)

# Parallel processing (automatic on large directories)
ruchy find --parallel       # Use all CPU cores
ruchy find --threads 4      # Explicit thread count

# Depth control
ruchy find --max-depth 2    # Limit recursion
ruchy find --min-depth 1    # Skip root directory

# Hidden files
ruchy find --hidden         # Include hidden files (default: skip)

# Follow symlinks
ruchy find --follow-links   # Follow symbolic links (default: no)
```

**Example Output**:
```bash
$ ruchy find --ext .log --size +10M
🔍 Ruchy Find
📁 Searching: .
🎯 Filters: *.log, size > 10MB

./logs/app.log           15.2 MB    2025-10-19 14:23
./logs/error.log         22.8 MB    2025-10-20 09:15
./logs/archive.log       105.1 MB   2025-10-18 18:42

📊 Found 3 files (143.1 MB total) in 0.8s
```

**Comparison with GNU find**:
```bash
# GNU find (complex)
find /data -type f -name "*.log" -size +10M -mtime -7 -exec wc -l {} \;

# Ruchy find (simple)
ruchy find /data --ext .log --size +10M --mtime -7 --exec "wc -l {}"
```

---

### 2. `ruchy tree` - Visual Directory Structure

**Purpose**: Display directory tree with statistics (like `tree`, but with insights)

**Basic Usage**:
```bash
# Show tree (current directory)
ruchy tree

# Show tree with depth limit
ruchy tree --depth 2

# Show tree for specific directory
ruchy tree /project/src

# Show file sizes
ruchy tree --size

# Show modification times
ruchy tree --mtime

# Show file counts per directory
ruchy tree --stats
```

**Advanced Usage**:
```bash
# Filter by extension
ruchy tree --ext .py       # Only show Python files

# Show hidden files
ruchy tree --hidden

# Exclude patterns
ruchy tree --exclude node_modules --exclude .git

# Output formats
ruchy tree --format json   # JSON for tooling
ruchy tree --format html   # HTML visualization

# Parallel scanning (large directories)
ruchy tree --parallel
```

**Example Output**:
```bash
$ ruchy tree --depth 2 --stats
📁 /project
├── 📂 src (42 files, 15.2 MB)
│   ├── 📂 components (15 files, 5.8 MB)
│   ├── 📂 utils (8 files, 2.1 MB)
│   └── 📄 main.rs (850 lines, 28.5 KB)
├── 📂 tests (28 files, 8.7 MB)
│   ├── 📂 integration (12 files, 4.2 MB)
│   └── 📂 unit (16 files, 4.5 MB)
└── 📄 Cargo.toml (45 lines, 1.2 KB)

📊 Summary: 3 directories, 71 files, 24.1 MB
⏱️  Scanned in 0.2s
```

---

### 3. `ruchy du` - Disk Usage Analysis

**Purpose**: Show disk usage with visual breakdown (clearer than GNU `du`)

**Basic Usage**:
```bash
# Show disk usage (current directory)
ruchy du

# Show disk usage for specific directory
ruchy du /data

# Show top N largest items
ruchy du --top 10

# Minimum size threshold
ruchy du --min 100M        # Only show items > 100MB
```

**Advanced Usage**:
```bash
# Sort options
ruchy du --sort size       # By size (default)
ruchy du --sort name       # By name
ruchy du --sort count      # By file count

# Output formats
ruchy du --format table    # Table (default)
ruchy du --format chart    # ASCII bar chart
ruchy du --format json     # JSON for scripting

# Depth control
ruchy du --depth 2         # Limit depth

# Include/exclude patterns
ruchy du --exclude node_modules --exclude target

# Parallel scanning (huge directories)
ruchy du --parallel
```

**Example Output**:
```bash
$ ruchy du --top 5 --format chart
💾 Ruchy Disk Usage - /project

node_modules/     ████████████████████████████████████████ 842.5 MB (68%)
target/           ██████████████████░░░░░░░░░░░░░░░░░░░░░ 315.2 MB (25%)
.git/             ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  48.7 MB  (4%)
docs/             █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  22.1 MB  (2%)
src/              █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  15.2 MB  (1%)

📊 Total: 1.24 GB (5 items shown, 42 items total)
⏱️  Scanned in 1.8s (parallel mode, 8 threads)
```

---

### 4. `ruchy count` - File Statistics

**Purpose**: Count files, lines, words, characters (better than `wc`)

**Basic Usage**:
```bash
# Count files in directory
ruchy count

# Count lines in all files
ruchy count --lines

# Count by file type
ruchy count --by-ext

# Filter by extension
ruchy count --ext .rs --lines
ruchy count --ext .py --lines --words
```

**Advanced Usage**:
```bash
# Code statistics (skip comments/blanks)
ruchy count --code         # Only code lines

# Language detection
ruchy count --by-language  # Group by programming language

# Parallel processing
ruchy count --parallel

# Output formats
ruchy count --format table
ruchy count --format json
ruchy count --format csv
```

**Example Output**:
```bash
$ ruchy count --by-language --code
📊 Ruchy Count - Code Statistics

Language    Files    Lines     Code    Comments    Blank    Characters
─────────────────────────────────────────────────────────────────────
Rust          42    15,428   12,015      2,148    1,265    485,392
Python        15     3,842    2,915        645      282    125,458
JavaScript     8     2,156    1,788        245      123     68,542
Markdown       5     1,024      892        n/a      132     38,125
YAML           3       458      412         24       22     12,845
─────────────────────────────────────────────────────────────────────
TOTAL         73    22,908   18,022      3,062    1,824    730,362

⏱️  Scanned in 0.5s (parallel mode, 8 threads)
```

---

### 5. `ruchy rg` - Fast Text Search

**Purpose**: Blazing-fast text search across files (parallel by default, like ripgrep)

**Basic Usage**:
```bash
# Search for pattern in current directory
ruchy rg "error"

# Search in specific directory
ruchy rg "TODO" /project/src

# Case-insensitive search
ruchy rg -i "error"

# Search specific file types
ruchy rg "import" --type py        # Python files
ruchy rg "fn main" --type rust     # Rust files
ruchy rg "error" --ext .log        # By extension

# Regex patterns
ruchy rg "\d{3}-\d{3}-\d{4}"      # Phone numbers
ruchy rg "error|warning|fatal"    # Multiple patterns
```

**Advanced Usage**:
```bash
# Context lines (before/after)
ruchy rg "error" -A 3             # 3 lines after
ruchy rg "error" -B 2             # 2 lines before
ruchy rg "error" -C 2             # 2 lines before & after

# Output control
ruchy rg "error" --count          # Count matches per file
ruchy rg "error" --files-with-matches  # Only filenames
ruchy rg "error" --format json    # JSON output

# Search filters
ruchy rg "error" --max-depth 2    # Limit depth
ruchy rg "error" --hidden         # Include hidden files
ruchy rg "error" --follow         # Follow symlinks

# Exclusions
ruchy rg "error" --exclude node_modules
ruchy rg "error" --exclude-dir .git

# Replace mode (preview only)
ruchy rg "error" --replace "warning" --preview

# Performance
ruchy rg "pattern" --threads 4    # Control parallelism
ruchy rg "pattern" --no-ignore    # Don't respect .gitignore
```

**Example Output**:
```bash
$ ruchy rg "fn main" --type rust
🔍 Ruchy Grep - Searching for "fn main"
📁 Directory: .
🎯 File types: Rust

src/main.rs:15:fn main() {
src/cli/mod.rs:45:    fn main() {
tests/integration_test.rs:10:fn main() {

📊 Found 3 matches in 3 files (0.2s, parallel mode)
```

**With Context**:
```bash
$ ruchy rg "error" -C 2
🔍 Ruchy Grep - Searching for "error"

src/lib.rs:42-    match result {
src/lib.rs:43-        Ok(val) => val,
src/lib.rs:44:        Err(error) => {
src/lib.rs:45-            eprintln!("Error: {}", error);
src/lib.rs:46-            return None;
--
tests/parser.rs:88-    let code = "invalid syntax";
tests/parser.rs:89-    let result = parse(code);
tests/parser.rs:90:    assert!(result.is_err(), "Expected error");
tests/parser.rs:91-
tests/parser.rs:92-    match result {

📊 Found 2 matches in 2 files (0.1s)
```

**Comparison with ripgrep**:
```bash
# ripgrep (complex options)
rg "pattern" --type rust --max-depth 3 --hidden --context 2

# Ruchy rg (same simplicity, better output)
ruchy rg "pattern" --type rust --max-depth 3 --hidden -C 2
```

**Integration with Other Tools**:
```bash
# Chain with ruchy find
ruchy find --ext .py | xargs ruchy rg "TODO"

# Chain with ruchy count
ruchy rg "error" --files-with-matches | ruchy count

# Use in pipelines
ruchy rg "error" --format json | jq '.[] | .path'
```

---

