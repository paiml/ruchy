# Multi-Threaded Directory Walking Specification (STDLIB-005)

**Version**: 2.0.0
**Status**: HARDENED (Post-Critical Review)
**Created**: 2025-10-20
**Updated**: 2025-10-20 (Critical review applied)
**Target Release**: v3.100.0
**Ticket**: STDLIB-005

## Critical Review Applied (Toyota Way: Jidoka - Stop the Line)

This specification underwent rigorous critical review using Five Whys and Genchi Genbutsu principles. The following **5 fundamental design flaws** were identified and corrected:

1. ✅ **Memory Scalability Defect**: walk_parallel() eager collection → OOM on large directories
   - **Fix**: Documented defect, proposed iterator-based API for v2.0
2. ✅ **Abstraction Cost Not Measured**: "Zero-cost" claim was unvalidated
   - **Fix**: Renamed to "high-performance", added <1µs overhead benchmark as quality gate
3. ✅ **Concurrency Testing Gap**: No systematic concurrency testing
   - **Fix**: Added loom tests, thread sanitizer, stress tests (3 new test categories)
4. ✅ **Security Testing Gap**: No directory traversal/symlink attack tests
   - **Fix**: Added 5 security test categories (directory traversal, symlink bombs, unicode, path injection, TOCTOU)
5. ✅ **Missing Algorithm Justification**: Design not grounded in CS research
   - **Fix**: Added theoretical foundations section with peer-reviewed algorithm references (Aho-Corasick, Blumofe & Leiserson work-stealing, Thompson NFA)

**Kaizen Result**: Quality gates increased from 7 → 16, mutation coverage 80% → 90%, estimated time 10-14h → 14-18h

## Overview

Multi-threaded directory walking and fast text search for Ruchy, combining:
- **Directory walking**: Python's `os.walk()` ergonomics + Rust's `walkdir` + `rayon` performance
- **Text search**: Ripgrep's speed with Ruchy's simplicity via `grep` crate
- **CLI tools**: 5 production-ready sysadmin utilities (find, tree, du, count, rg)
- **Programmatic API**: 6 functions for data science, data engineering, and automation workflows

## Design Philosophy

1. **Simple by default, powerful when needed**
   - **CLI tools for sysadmins** (no coding required, like `python -m http.server`)
   - Basic walk() for simple cases
   - Advanced options for complex scenarios
   - Parallel processing built-in, not bolted-on

2. **High-performance abstraction** (corrected from "zero-cost")
   - Direct wrapping of `walkdir` crate (proven, battle-tested)
   - Parallel iteration via `rayon` (optimal work-stealing scheduler based on Cilk work-stealing)
   - **⚠️ Reality Check**: Interpreter boundary crossings are NOT zero-cost
   - **Quality Gate**: Boundary overhead must be <1µs per item (measured via benchmarks)
   - **Justification**: "Zero-cost" applies to compiled Rust, not interpreter FFI calls

3. **Data pipeline friendly**
   - Iterator-based API (chainable, composable)
   - Integrates with existing array methods (map, filter, reduce)
   - Natural fit for ETL workflows

4. **Three ways to use** (following Ruchy hybrid pattern from http-server-mvp-spec):
   - **CLI** - Quick sysadmin tasks (like GNU find, tree, du, wc)
   - **Import** - Programmatic usage in .ruchy scripts
   - **Task Runner** - Workflow automation via ruchy.yaml

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

## CLI Implementation Architecture

### Pattern: Dual-Use Functions (Following http-server-mvp-spec)

**Key Insight**: CLI commands internally use the programmatic API

```rust
// CLI command implementation
pub fn cli_find(args: FindArgs) -> Result<(), Error> {
    // Parse CLI arguments
    let options = WalkOptions {
        max_depth: args.max_depth,
        follow_links: args.follow_links,
        parallel: args.parallel,
        // ...
    };

    // Use programmatic API
    let entries = walk_with_options(&args.path, options)?;

    // Apply CLI-specific filtering
    let filtered = entries
        .into_iter()
        .filter(|e| matches_extension(e, &args.ext))
        .filter(|e| matches_size(e, &args.size))
        .collect::<Vec<_>>();

    // Format output
    print_results(&filtered, &args.format);

    Ok(())
}
```

**Benefits**:
1. **Dogfooding**: CLI proves the API works
2. **Code reuse**: Zero duplication between CLI and library
3. **Testing**: CLI tests validate programmatic API
4. **Consistency**: Same behavior in both modes

---

### CLI Module Structure

```
src/
├── cli/
│   ├── mod.rs           # CLI argument parsing (clap)
│   ├── find.rs          # ruchy find implementation
│   ├── tree.rs          # ruchy tree implementation
│   ├── du.rs            # ruchy du implementation
│   ├── count.rs         # ruchy count implementation
│   ├── rg.rs            # ruchy rg implementation (text search)
│   └── formatters/      # Output formatting (table, JSON, CSV)
│       ├── table.rs
│       ├── json.rs
│       └── chart.rs
└── runtime/
    ├── eval_dir_walk.rs  # Core walk implementation (used by CLI)
    └── eval_search.rs    # Text search implementation (grep/rg functionality)
```

---

## API Design (Programmatic Usage)

### Basic Functions

#### 1. `walk(path: String) -> Array<FileEntry>`

**Purpose**: Recursively walk directory tree (single-threaded)

**Example**:
```ruby
# Simple recursive walk
for entry in walk("/data/logs") {
    println("Found: {}", entry.path)
}

# With filtering
let files = walk("/data/logs")
    .filter(|e| e.is_file())
    .filter(|e| e.path.ends_with(".log"))

println("Found {} log files", files.len())
```

**Returns**: Array of `FileEntry` objects with:
- `path: String` - Full path
- `name: String` - File/directory name
- `is_file: Bool` - Is regular file
- `is_dir: Bool` - Is directory
- `is_symlink: Bool` - Is symbolic link
- `size: Int` - File size in bytes (0 for directories)
- `depth: Int` - Depth from root (0 = root)

---

#### 2. `walk_parallel(path: String, callback: Closure) -> Array<Any>`

**Purpose**: Walk directory tree with parallel processing (multi-threaded)

**Example**:
```ruby
# Count words in all text files (parallel)
let results = walk_parallel("/data/docs", |entry| {
    if entry.is_file() && entry.path.ends_with(".txt") {
        let content = read_file(entry.path)
        return {
            path: entry.path,
            words: content.split(" ").len()
        }
    }
})

# Aggregate results
let total_words = results
    .filter(|r| r != nil)
    .map(|r| r.words)
    .reduce(0, |acc, x| acc + x)

println("Total words: {}", total_words)
```

**Behavior**:
- Automatically uses all available CPU cores
- Work-stealing scheduler (optimal load balancing)
- Callback executed in parallel for each entry
- Returns array of callback results (preserves order)

**⚠️ CRITICAL DESIGN ISSUE (Identified in Review):**

This API has a **memory scalability defect** for large directory trees:
- **Problem**: Collects ALL results into array before returning → OOM on millions of files
- **Root Cause**: Eager collection prevents lazy evaluation and composability
- **Risk**: User closures can access shared state → **race conditions** (no guardrails)

**RECOMMENDED ALTERNATIVE** (Iterator-Based API):
```ruby
# Lazy, composable, memory-efficient
let total = walk("/data")
    .par_iter()  # Parallel iterator (lazy evaluation)
    .filter(|e| e.is_file() && e.path.ends_with(".txt"))
    .map(|e| read_file(e.path).split(" ").len())
    .reduce(0, |acc, x| acc + x)

# Benefits:
# - No intermediate array allocation (constant memory)
# - Composable parallel operations (rayon pattern)
# - Pure functional style (no shared state)
```

**Implementation Decision**:
- **v1.0**: Keep callback API for simplicity (document memory limits)
- **v2.0**: Add parallel iterator API for composability and scalability
- **Documentation**: Warn users about memory usage on large directories (>1M files)
- **Safety**: Document that closures must NOT access shared mutable state

---

#### 3. `walk_with_options(path: String, options: Object) -> Array<FileEntry>`

**Purpose**: Advanced directory walking with fine-grained control

**Example**:
```ruby
# Advanced walk with options
let entries = walk_with_options("/data", {
    max_depth: 3,              # Limit recursion depth
    follow_links: false,       # Don't follow symlinks
    min_depth: 1,              # Skip root directory
    parallel: true,            # Use parallel processing
    threads: 8,                # Explicit thread count
    filter: |e| !e.name.starts_with("."),  # Skip hidden files
    sort_by: "name"            # Sort results (name, size, date)
})

for entry in entries {
    println("{} - {} bytes", entry.path, entry.size)
}
```

**Options Object**:
```ruby
{
    max_depth: Int?,           # Maximum recursion depth (default: unlimited)
    min_depth: Int?,           # Minimum depth to include (default: 0)
    follow_links: Bool?,       # Follow symbolic links (default: false)
    parallel: Bool?,           # Enable parallel processing (default: false)
    threads: Int?,             # Thread count (default: num_cpus)
    filter: Closure?,          # Filter predicate (called per entry)
    sort_by: String?           # Sort results: "name", "size", "date" (default: none)
}
```

---

### Utility Functions

#### 4. `glob(pattern: String) -> Array<String>`

**Purpose**: Find files matching glob pattern (wraps `glob` crate)

**Example**:
```ruby
# Find all Python files
let py_files = glob("**/*.py")

# Find all log files from 2025
let logs_2025 = glob("/var/log/**/2025-*.log")

# Count files by extension
let stats = {}
for file in glob("/project/**/*") {
    let ext = file.split(".").last()
    stats[ext] = (stats[ext] || 0) + 1
}
```

---

#### 5. `find(path: String, predicate: Closure) -> Array<FileEntry>`

**Purpose**: Find files matching predicate (convenience wrapper)

**Example**:
```ruby
# Find large files (>100MB)
let large_files = find("/data", |e| {
    e.is_file() && e.size > 100_000_000
})

# Find recently modified files (last 7 days)
let recent = find("/logs", |e| {
    e.is_file() && e.modified_days_ago() < 7
})
```

---

#### 6. `search(pattern: String, path: String, options: Object?) -> Array<SearchMatch>`

**Purpose**: Fast parallel text search across files (wraps ripgrep functionality)

**Example**:
```ruby
# Basic text search
let matches = search("error", "/var/log")

for match in matches {
    println("{}:{}: {}", match.path, match.line_num, match.line)
}

# Search with options
let matches = search("TODO", "/project/src", {
    case_insensitive: true,
    file_types: ["rs", "py"],
    max_depth: 3,
    context_lines: 2
})

# Count matches per file
let counts = search("error", "/logs", { count_only: true })
for result in counts {
    println("{}: {} matches", result.path, result.count)
}

# Regex search
let phone_numbers = search(r"\d{3}-\d{3}-\d{4}", "/data")
```

**SearchMatch Object**:
```ruby
{
    path: String,           # File path
    line_num: Int,          # Line number (1-indexed)
    line: String,           # Matched line content
    column: Int,            # Column of match
    match_text: String,     # The matched text
    before: Array<String>?, # Context lines before (if requested)
    after: Array<String>?   # Context lines after (if requested)
}
```

**Options Object**:
```ruby
{
    case_insensitive: Bool?,    # Case-insensitive search (default: false)
    file_types: Array<String>?, # File extensions to search
    max_depth: Int?,            # Maximum recursion depth
    context_lines: Int?,        # Lines of context (before & after)
    count_only: Bool?,          # Only return counts (default: false)
    parallel: Bool?,            # Parallel search (default: true)
    follow_links: Bool?,        # Follow symlinks (default: false)
    hidden: Bool?               # Include hidden files (default: false)
}
```

---

## Use Case Examples

### Example 1: ETL Pipeline - Process CSV Files in Parallel

```ruby
# Extract: Find all CSV files
let csv_files = walk("/data/raw")
    .filter(|e| e.is_file() && e.path.ends_with(".csv"))

# Transform: Process each CSV in parallel
let results = walk_parallel("/data/raw", |entry| {
    if entry.is_file() && entry.path.ends_with(".csv") {
        let df = DataFrame.from_csv(read_file(entry.path))

        # Data cleaning
        df = df.filter(|row| row["amount"] > 0)

        # Aggregate
        return {
            file: entry.name,
            total: df.sum("amount"),
            count: df.len()
        }
    }
})

# Load: Write summary
let summary = DataFrame.new(results.filter(|r| r != nil))
write_file("/data/processed/summary.csv", summary.to_csv())
```

### Example 2: Log Analysis - Find Errors Across Directory Tree

```ruby
# Find all errors in log files (parallel)
let errors = walk_parallel("/var/log", |entry| {
    if entry.is_file() && entry.path.ends_with(".log") {
        let content = read_file(entry.path)
        let error_lines = content.split("\n")
            .filter(|line| line.contains("ERROR"))

        if error_lines.len() > 0 {
            return {
                file: entry.path,
                errors: error_lines,
                count: error_lines.len()
            }
        }
    }
})

# Generate report
let error_files = errors.filter(|e| e != nil)
println("Found errors in {} files", error_files.len())

for file_errors in error_files {
    println("\n{}: {} errors", file_errors.file, file_errors.count)
    for line in file_errors.errors {
        println("  {}", line)
    }
}
```

### Example 3: Data Science - Build Training Dataset from Images

```ruby
# Find all images recursively
let image_paths = walk("/dataset/images")
    .filter(|e| e.is_file())
    .filter(|e| {
        let ext = e.path.split(".").last()
        ext == "jpg" || ext == "png" || ext == "jpeg"
    })
    .map(|e| e.path)

println("Found {} images", image_paths.len())

# Process images in parallel (simulate with metadata)
let dataset = walk_parallel("/dataset/images", |entry| {
    if entry.is_file() {
        let ext = entry.path.split(".").last()
        if ext == "jpg" || ext == "png" || ext == "jpeg" {
            # Extract label from directory name
            let parts = entry.path.split("/")
            let label = parts[parts.len() - 2]

            return {
                path: entry.path,
                label: label,
                size: entry.size,
                width: 224,  # Would come from image reading
                height: 224
            }
        }
    }
})

# Create DataFrame for ML pipeline
let df = DataFrame.new(dataset.filter(|d| d != nil))
println("Dataset ready: {} samples", df.len())
```

### Example 4: Code Analysis - Count Lines of Code

```ruby
# Count lines of code by language
let stats = {}

walk_parallel("/project", |entry| {
    if entry.is_file() {
        let ext = entry.path.split(".").last()

        # Only count code files
        if ["rs", "py", "js", "go"].contains(ext) {
            let content = read_file(entry.path)
            let lines = content.split("\n").len()

            return { ext: ext, lines: lines }
        }
    }
}).filter(|r| r != nil).for_each(|result| {
    stats[result.ext] = (stats[result.ext] || 0) + result.lines
})

# Print summary
println("Lines of code by language:")
for lang in stats.keys().sort() {
    println("  {}: {}", lang, stats[lang])
}
```

### Example 5: Security Audit - Find Sensitive Data

```ruby
# Search for potential security issues (parallel)
let patterns = [
    "password",
    "api_key",
    "secret",
    "TODO.*security",
    r"\d{3}-\d{2}-\d{4}"  # SSN pattern
]

for pattern in patterns {
    println("\n🔍 Searching for: {}", pattern)

    let matches = search(pattern, "/project/src", {
        case_insensitive: true,
        file_types: ["py", "js", "rs", "env"],
        context_lines: 2
    })

    if matches.len() > 0 {
        println("⚠️  Found {} matches", matches.len())

        for match in matches {
            println("  {}:{}", match.path, match.line_num)
            println("    {}", match.line.trim())
        }
    } else {
        println("✅ No matches found")
    }
}

# Generate security report
let all_issues = []
for pattern in patterns {
    let results = search(pattern, "/project", {
        count_only: true,
        case_insensitive: true
    })

    for result in results {
        all_issues.push({
            pattern: pattern,
            file: result.path,
            count: result.count
        })
    }
}

# Summary
println("\n📊 Security Audit Summary")
println("Total files with issues: {}", all_issues.len())
let total = all_issues.map(|i| i.count).reduce(0, |a, x| a + x)
println("Total potential issues: {}", total)
```

---

## Theoretical Foundations and Algorithm References

**Purpose**: Ground design decisions in peer-reviewed computer science research (per critical review).

### 1. Work-Stealing Scheduler (Rayon)

**Algorithm**: Cilk-style work-stealing based on **[Blumofe & Leiserson, 1999]**

**Key Properties**:
- **Time Complexity**: T_p ≤ T_1/p + O(T_∞) where:
  - T_1 = sequential execution time
  - T_∞ = critical path length (span)
  - p = number of processors
- **Space Complexity**: O(p × T_∞) (provably efficient)
- **Locality**: LIFO for local tasks (cache-friendly), FIFO for stolen tasks

**Implementation Details**:
```rust
// Rayon's work-stealing deque
// - Each thread has local deque (LIFO for own work)
// - Steals from other threads' tails (FIFO for stolen work)
// - Lock-free chase-lev deque algorithm [Chase & Lev, 2005]
```

**References**:
- Blumofe, R. D., & Leiserson, C. E. (1999). "Scheduling multithreaded computations by work stealing." *Journal of the ACM*, 46(5), 720-748.
- Chase, D., & Lev, Y. (2005). "Dynamic circular work-stealing deque." *SPAA '05*.

### 2. Fast Text Search (grep crate / ripgrep)

**Algorithms**:

**2.1 Multi-Pattern Matching: Aho-Corasick [1975]**
- **Purpose**: Find multiple patterns simultaneously in O(n + m + z) time
  - n = text length
  - m = total pattern length
  - z = number of matches
- **Used For**: Searching for multiple keywords (e.g., "error", "warning", "fatal")

**2.2 SIMD String Searching**
- **Vectorized memchr**: Uses SSE2/AVX2 instructions for byte scanning
- **Performance**: Up to 16x speedup on modern CPUs
- **Implementation**: `memchr` crate (used by ripgrep)

**2.3 Regular Expression Engines**
- **Thompson NFA Construction**: Linear time regex matching (no catastrophic backtracking)
- **DFA Caching**: Lazy DFA construction for frequently-used patterns
- **Hybrid Approach**: Switch between NFA and DFA based on pattern complexity

**References**:
- Aho, A. V., & Corasick, M. J. (1975). "Efficient string matching: an aid to bibliographic search." *Communications of the ACM*, 18(6), 333-340.
- Cox, R. (2007). "Regular Expression Matching Can Be Simple And Fast." https://swtch.com/~rsc/regexp/
- Thompson, K. (1968). "Programming Techniques: Regular expression search algorithm." *Communications of the ACM*, 11(6), 419-422.

### 3. Parallel Iterator Abstraction

**Design Pattern**: Iterator-based parallelism [Dean & Ghemawat, 2004 (MapReduce)]

**Key Insight**: Lazy evaluation + parallel execution = composable high-performance pipelines

```rust
// Composable pipeline (no intermediate collections)
walk("/data")
    .par_iter()           // Parallel iterator
    .filter(|e| test(e))  // Parallel filter (no sync needed)
    .map(|e| process(e))  // Parallel map (pure function)
    .reduce(|| 0, |a, b| a + b)  // Parallel reduce (associative)
```

**Performance**: O(n/p) with p processors (optimal speedup for embarrassingly parallel workloads)

**References**:
- Dean, J., & Ghemawat, S. (2004). "MapReduce: Simplified data processing on large clusters." *OSDI '04*.
- Rayon documentation: https://docs.rs/rayon/latest/rayon/

### 4. Concurrency Correctness

**Testing Strategy**: Systematic Exploration via Loom [Kokologiannakis et al., 2019]

**Problem**: Standard tests explore O(1) thread interleavings out of exponentially many possible schedules

**Solution**: Loom systematically explores all possible schedules under a bounded context switch model

**Theoretical Foundation**: DPOR (Dynamic Partial Order Reduction) [Flanagan & Godefroid, 2005]

**References**:
- Flanagan, C., & Godefroid, P. (2005). "Dynamic partial-order reduction for model checking software." *POPL '05*.
- Kokologiannakis, M., Raad, A., & Vafeiadis, V. (2019). "Model checking for weakly consistent libraries." *PLDI '19*.

---

## Implementation Strategy

### Phase 1: Basic Walk (Single-threaded)
**Pattern**: Zero-cost abstraction over `walkdir` crate

```rust
// src/runtime/eval_builtin.rs
fn eval_walk(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk", args, 1)?;

    match &args[0] {
        Value::String(path) => {
            use walkdir::WalkDir;

            let entries: Vec<Value> = WalkDir::new(path.as_ref())
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|entry| {
                    create_file_entry(&entry)
                })
                .collect();

            Ok(Value::from_array(entries))
        }
        _ => Err(InterpreterError::TypeError(
            "walk() expects string path".to_string()
        ))
    }
}

fn create_file_entry(entry: &walkdir::DirEntry) -> Value {
    let metadata = entry.metadata().unwrap();

    Value::Object(Rc::new(HashMap::from([
        ("path".to_string(), Value::from_string(entry.path().display().to_string())),
        ("name".to_string(), Value::from_string(entry.file_name().to_string_lossy().to_string())),
        ("is_file".to_string(), Value::Bool(metadata.is_file())),
        ("is_dir".to_string(), Value::Bool(metadata.is_dir())),
        ("is_symlink".to_string(), Value::Bool(metadata.file_type().is_symlink())),
        ("size".to_string(), Value::Integer(metadata.len() as i64)),
        ("depth".to_string(), Value::Integer(entry.depth() as i64)),
    ])))
}
```

### Phase 2: Parallel Processing
**Pattern**: Rayon's `par_bridge()` for automatic parallelism

```rust
fn eval_walk_parallel(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk_parallel", args, 2)?;

    use rayon::prelude::*;
    use walkdir::WalkDir;

    match (&args[0], &args[1]) {
        (Value::String(path), Value::Closure { .. }) => {
            let results: Vec<Value> = WalkDir::new(path.as_ref())
                .into_iter()
                .filter_map(|e| e.ok())
                .par_bridge()  // Enable parallel processing
                .map(|entry| {
                    let file_entry = create_file_entry(&entry);

                    // Call user's closure with entry
                    eval_closure(&args[1], &[file_entry])
                        .unwrap_or(Value::Nil)
                })
                .collect();

            Ok(Value::from_array(results))
        }
        _ => Err(InterpreterError::TypeError(
            "walk_parallel(path, callback) expects string and closure".to_string()
        ))
    }
}
```

### Phase 3: Advanced Options
**Pattern**: Object destructuring for optional parameters

```rust
fn eval_walk_with_options(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk_with_options", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(path), Value::Object(opts)) => {
            use walkdir::WalkDir;

            let mut walker = WalkDir::new(path.as_ref());

            // Apply options
            if let Some(Value::Integer(max)) = opts.get("max_depth") {
                walker = walker.max_depth(*max as usize);
            }

            if let Some(Value::Integer(min)) = opts.get("min_depth") {
                walker = walker.min_depth(*min as usize);
            }

            if let Some(Value::Bool(follow)) = opts.get("follow_links") {
                walker = walker.follow_links(*follow);
            }

            let use_parallel = opts.get("parallel")
                .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                .unwrap_or(false);

            let entries = walker.into_iter()
                .filter_map(|e| e.ok());

            if use_parallel {
                use rayon::prelude::*;
                let results: Vec<Value> = entries
                    .par_bridge()
                    .map(|e| create_file_entry(&e))
                    .collect();
                Ok(Value::from_array(results))
            } else {
                let results: Vec<Value> = entries
                    .map(|e| create_file_entry(&e))
                    .collect();
                Ok(Value::from_array(results))
            }
        }
        _ => Err(InterpreterError::TypeError(
            "walk_with_options(path, options) expects string and object".to_string()
        ))
    }
}
```

---

## Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
walkdir = "2.5"      # Recursive directory traversal
rayon = "1.10"       # Data parallelism
glob = "0.3"         # Glob pattern matching (for glob() function)
num_cpus = "1.16"    # CPU core detection
grep = "0.3"         # Fast text search (ripgrep library)
regex = "1.10"       # Regex pattern matching (for search() function)
```

---

## Testing Requirements (EXTREME TDD)

### Test Suite Structure

```
tests/
└── stdlib_dir_walk_test.rs          # Main test suite
    ├── Basic walk() tests            (10 tests)
    ├── Parallel walk_parallel() tests (8 tests)
    ├── Advanced walk_with_options() (12 tests)
    ├── Utility glob()/find() tests   (6 tests)
    ├── Text search() tests           (8 tests)
    ├── Integration tests             (6 tests)
    ├── Property tests                (4 tests, 40K cases)
    ├── Concurrency tests (NEW)       (3 tests - loom, sanitizer, stress)
    ├── Security tests (NEW)          (5 tests - traversal, symlinks, unicode, injection, TOCTOU)
    ├── Performance benchmarks (NEW)  (2 tests - overhead, speedup)
    └── Error handling tests          (6 tests)

Total: 70 tests + 2 benchmarks (INCREASED from 60)
```

### Test Categories

#### 1. Basic Walk Tests (10 tests)
```rust
#[test]
fn test_stdlib005_walk_basic() {
    // Create test directory structure
    let temp_dir = create_test_tree();

    let code = format!(r#"
        let entries = walk("{}")
        assert(entries.len() > 0)
        println("Found {} entries", entries.len())
    "#, temp_dir.path().display());

    ruchy_cmd().arg("-e").arg(code)
        .assert().success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stdlib005_walk_filter_files() {
    let code = r#"
        let files = walk("/tmp/test")
            .filter(|e| e.is_file)

        for f in files {
            println("File: {}", f.path)
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_walk_depth() {
    let code = r#"
        let entries = walk("/tmp/test")

        let depths = entries.map(|e| e.depth).unique()
        assert(depths.contains(0))  # Root
        assert(depths.contains(1))  # First level
    "#;
    // ... assertions
}
```

#### 2. Parallel Processing Tests (8 tests)
```rust
#[test]
fn test_stdlib005_walk_parallel_callback() {
    let code = r#"
        let results = walk_parallel("/tmp/test", |entry| {
            if entry.is_file {
                return { path: entry.path, size: entry.size }
            }
        })

        let files = results.filter(|r| r != nil)
        assert(files.len() > 0)
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_walk_parallel_word_count() {
    // Real-world example: count words in parallel
    let code = r#"
        let totals = walk_parallel("/tmp/docs", |entry| {
            if entry.path.ends_with(".txt") {
                let content = read_file(entry.path)
                return content.split(" ").len()
            }
        })

        let total = totals.filter(|t| t != nil).reduce(0, |a, x| a + x)
        println("Total words: {}", total)
    "#;
    // ... assertions
}
```

#### 3. Advanced Options Tests (12 tests)
```rust
#[test]
fn test_stdlib005_walk_max_depth() {
    let code = r#"
        let entries = walk_with_options("/tmp/test", {
            max_depth: 2
        })

        let max_depth = entries.map(|e| e.depth).max()
        assert(max_depth <= 2)
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_walk_follow_symlinks() {
    let code = r#"
        let entries = walk_with_options("/tmp/test", {
            follow_links: true
        })

        let symlinks = entries.filter(|e| e.is_symlink)
        # Should include symlink targets
    "#;
    // ... assertions
}
```

#### 4. Text Search Tests (8 tests)
```rust
#[test]
fn test_stdlib005_search_basic() {
    // Create test files with known content
    let temp_dir = create_test_files_with_content();

    let code = format!(r#"
        let matches = search("error", "{}")

        assert(matches.len() > 0)
        for match in matches {{
            println("{{}}:{{}}: {{}}", match.path, match.line_num, match.line)
        }}
    "#, temp_dir.path().display());

    ruchy_cmd().arg("-e").arg(code)
        .assert().success()
        .stdout(predicate::str::contains("error"));
}

#[test]
fn test_stdlib005_search_case_insensitive() {
    let code = r#"
        let matches = search("ERROR", "/tmp/test", {
            case_insensitive: true
        })

        # Should match "error", "Error", "ERROR"
        assert(matches.len() > 0)
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_with_context() {
    let code = r#"
        let matches = search("target", "/tmp/test", {
            context_lines: 2
        })

        for match in matches {
            # Should have before and after context
            assert(match.before != nil)
            assert(match.after != nil)
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_file_types() {
    let code = r#"
        let matches = search("fn main", "/project", {
            file_types: ["rs", "rust"]
        })

        # Should only search Rust files
        for match in matches {
            assert(match.path.ends_with(".rs"))
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_regex_pattern() {
    let code = r#"
        # Search for phone numbers
        let matches = search(r"\d{3}-\d{3}-\d{4}", "/data")

        for match in matches {
            println("Found phone: {}", match.match_text)
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_count_only() {
    let code = r#"
        let counts = search("TODO", "/project", {
            count_only: true
        })

        for result in counts {
            println("{}: {} matches", result.path, result.count)
        }
    "#;
    // ... assertions
}
```

#### 5. Property Tests (4 tests, 10K cases each)
```rust
proptest! {
    #[test]
    fn prop_walk_never_panics(depth in 0..5usize, file_count in 0..20usize) {
        let temp = create_random_tree(depth, file_count);
        let code = format!(r#"
            let entries = walk("{}")
            assert(entries.len() >= 0)
        "#, temp.path().display());

        ruchy_cmd().arg("-e").arg(code)
            .assert().success();
    }

    #[test]
    fn prop_parallel_same_results(depth in 1..4usize) {
        // Verify walk() and walk_parallel() return same entries
        let temp = create_test_tree(depth);

        let code = format!(r#"
            let serial = walk("{}").map(|e| e.path).sort()

            let parallel = walk_parallel("{}", |e| e.path).sort()

            assert_eq(serial, parallel)
        "#, temp.path().display(), temp.path().display());

        ruchy_cmd().arg("-e").arg(code)
            .assert().success();
    }
}
```

#### 6. Concurrency Safety Tests (CRITICAL - Identified in Review)

**Problem**: Standard tests don't reliably expose race conditions due to non-deterministic thread interleavings.

**Required Testing Tools**:

```rust
// 1. Loom - Systematic concurrency testing (explores all interleavings)
#[cfg(test)]
#[cfg(loom)]
mod concurrency_tests {
    use loom::thread;
    use loom::sync::Arc;

    #[test]
    fn test_walk_parallel_no_data_races() {
        loom::model(|| {
            // Test that walk_parallel doesn't cause data races
            // when closures access shared state (even though they shouldn't)
            let shared = Arc::new(loom::sync::Mutex::new(0));

            // This should FAIL if walk_parallel allows unsafe access
            walk_parallel("/test", |entry| {
                let mut val = shared.lock().unwrap();
                *val += 1;
            });
        });
    }
}

// 2. Thread Sanitizer - Dynamic race detection
// Run ALL tests with: RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test

// 3. Stress Testing - High-contention scenarios
#[test]
fn stress_test_parallel_walk_contention() {
    // Create large directory tree (10K files)
    let temp = create_large_tree(10_000);

    // Run parallel walk with high contention
    for _ in 0..100 {
        let _ = walk_parallel(temp.path(), |entry| {
            // Simulate work
            thread::sleep(Duration::from_micros(1));
            entry.path
        });
    }
}
```

**Quality Gate Requirements**:
1. ✅ **Loom tests**: All concurrency patterns verified under systematic exploration
2. ✅ **Thread Sanitizer**: Zero data races detected in test suite
3. ✅ **Stress tests**: No panics or deadlocks under high contention (100 iterations)

#### 7. Security Tests (CRITICAL - Identified in Review)

**Problem**: Directory walking is a common attack vector. Must test for malicious inputs.

**Attack Vectors to Test**:

```rust
#[test]
fn security_test_directory_traversal_attack() {
    // Attempt to escape via ../ sequences
    let result = ruchy_cmd()
        .arg("-e")
        .arg(r#"walk("../../../../../../etc/passwd")"#)
        .assert();

    // Should either sanitize path or reject with clear error
    // MUST NOT expose system files outside project directory
}

#[test]
fn security_test_symlink_bomb() {
    // Create circular symlink structure
    let temp = TempDir::new().unwrap();
    let a = temp.path().join("a");
    let b = temp.path().join("b");

    std::os::unix::fs::symlink(&b, &a).unwrap();
    std::os::unix::fs::symlink(&a, &b).unwrap();

    let code = format!(r#"
        walk_with_options("{}", {{
            follow_links: true,
            max_depth: 100
        }})
    "#, temp.path().display());

    // MUST detect cycle and terminate (not infinite loop)
    ruchy_cmd().arg("-e").arg(code)
        .assert()
        .success(); // Or controlled failure
}

#[test]
fn security_test_unicode_normalization() {
    // Test Unicode homograph attacks (e.g., Cyrillic 'а' vs Latin 'a')
    // Create files with visually similar but different Unicode paths
    // Ensure walk() reports them as distinct files
}

#[test]
fn security_test_path_injection() {
    // Test for command injection via malicious filenames
    let temp = TempDir::new().unwrap();
    let evil_file = temp.path().join("; rm -rf /");
    File::create(&evil_file).unwrap();

    let code = format!(r#"
        walk("{}").for_each(|e| {{
            println("File: {{}}", e.path)
        }})
    "#, temp.path().display());

    // MUST NOT execute injected commands
    ruchy_cmd().arg("-e").arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("; rm -rf /"));
}

#[test]
fn security_test_time_of_check_time_of_use() {
    // Test TOCTOU vulnerabilities
    // Create file, walk() finds it, delete it, callback accesses it
    // MUST handle gracefully (not panic or expose undefined behavior)
}
```

**Security Quality Gates**:
1. ✅ **Directory traversal**: All `../` escape attempts blocked or sanitized
2. ✅ **Symlink loops**: Detected and terminated within max_depth limit
3. ✅ **Unicode attacks**: Filenames properly validated and normalized
4. ✅ **Path injection**: No command execution via malicious filenames
5. ✅ **TOCTOU races**: Graceful handling of filesystem changes during walk

### Mutation Testing Requirements

**Target**: ≥90% mutation coverage (INCREASED from 80% per critical review)

**Rationale**: For foundational libraries dealing with concurrency and filesystem operations, subtle bugs have severe consequences. Higher mutation coverage provides confidence that boundary conditions and concurrent interactions are thoroughly tested.

**Strategy**: File-level mutation testing
```bash
# Test walk() implementation
cargo mutants --file src/runtime/eval_dir_walk.rs \
    --test stdlib_dir_walk_test \
    --timeout 300

# Expected: ≥80% coverage
```

**Critical Mutations to Catch**:
1. **Match arm deletions**: All filter conditions must be tested
2. **Boolean negations**: Test is_file vs is_dir boundaries
3. **Boundary conditions**: Test depth limits (max_depth, min_depth)
4. **Iterator transformations**: Verify parallel vs serial behavior
5. **Option handling**: Test all walk_with_options parameters

---

## Quality Gates (MANDATORY)

Following stdlib quality protocol from `docs/execution/roadmap.yaml`:

### Gates (ALL BLOCKING - Updated per Critical Review):

**Functional Correctness**:
1. ✅ **Unit Tests**: ≥60 tests passing (100% pass rate)
2. ✅ **Property Tests**: 4 tests × 10K cases = 40K total validations
3. ✅ **Integration Tests**: 6 real-world scenario tests passing (including security audit)

**Concurrency Correctness** (NEW - Critical):
4. ✅ **Loom Tests**: All concurrency patterns verified under systematic exploration
5. ✅ **Thread Sanitizer**: Zero data races detected (`RUSTFLAGS="-Zsanitizer=thread"`)
6. ✅ **Stress Tests**: No panics/deadlocks under high contention (100 iterations)

**Security** (NEW - Critical):
7. ✅ **Directory Traversal**: All `../` escape attempts blocked/sanitized
8. ✅ **Symlink Loops**: Detected and terminated within max_depth
9. ✅ **Unicode Attacks**: Filenames properly validated and normalized
10. ✅ **Path Injection**: No command execution via malicious filenames
11. ✅ **TOCTOU Races**: Graceful handling of filesystem changes during walk

**Quality & Performance**:
12. ✅ **Mutation Tests**: ≥90% coverage (INCREASED from 80% per review)
13. ✅ **Complexity**: All functions ≤10 cyclomatic complexity
14. ✅ **Abstraction Overhead**: <1µs per item (measured via benchmarks) (NEW)
15. ✅ **Parallel Speedup**: ≥2x faster than serial (4+ cores)
16. ✅ **Documentation**: All 6 functions have doctests + examples + algorithm references

### Performance Benchmarks

**CRITICAL (Per Review)**: Benchmark abstraction overhead to validate "high-performance" claim.

```rust
#[test]
#[ignore]
fn bench_abstraction_overhead() {
    // Measure Ruchy-to-Rust boundary crossing cost per item
    use std::time::Instant;

    let temp = create_large_tree(10_000); // 10K files

    // Benchmark: Pure Rust (baseline)
    let start = Instant::now();
    let rust_count = walkdir::WalkDir::new(temp.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .count();
    let rust_time = start.elapsed();

    // Benchmark: Ruchy wrapper
    let start = Instant::now();
    let code = format!(r#"
        let entries = walk("{}")
        println("Count: {{}}", entries.len())
    "#, temp.path().display());
    ruchy_cmd().arg("-e").arg(code).assert().success();
    let ruchy_time = start.elapsed();

    // Calculate per-item overhead
    let overhead_per_item = (ruchy_time - rust_time).as_micros() as f64 / rust_count as f64;

    println!("Pure Rust: {:?} ({} files)", rust_time, rust_count);
    println!("Ruchy wrapper: {:?}", ruchy_time);
    println!("Overhead per item: {:.3}µs", overhead_per_item);

    // QUALITY GATE: Overhead must be <1µs per item
    assert!(
        overhead_per_item < 1.0,
        "Abstraction overhead {:.3}µs exceeds 1µs budget",
        overhead_per_item
    );
}

#[test]
#[ignore]  // Run with: cargo test --test stdlib_dir_walk_test -- --ignored
fn bench_parallel_speedup() {
    use std::time::Instant;

    let temp = create_large_tree(1000); // 1000 files

    // Serial walk
    let start = Instant::now();
    let code = format!(r#"
        let results = walk("{}")
            .filter(|e| e.is_file)
            .map(|e| {{ path: e.path, size: e.size }})
    "#, temp.path().display());
    ruchy_cmd().arg("-e").arg(code).assert().success();
    let serial_time = start.elapsed();

    // Parallel walk
    let start = Instant::now();
    let code = format!(r#"
        let results = walk_parallel("{}", |e| {{
            if e.is_file {{ path: e.path, size: e.size }}
        }})
    "#, temp.path().display());
    ruchy_cmd().arg("-e").arg(code).assert().success();
    let parallel_time = start.elapsed();

    let speedup = serial_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);

    // Verify ≥2x speedup on 4+ core systems
    if num_cpus::get() >= 4 {
        assert!(speedup >= 2.0, "Expected ≥2x speedup, got {:.2}x", speedup);
    }
}
```

---

## Roadmap Entry (YAML Format)

```yaml
- id: "STDLIB-005"
  title: "Multi-Threaded Directory Walking + Text Search (HARDENED)"
  status: "PLANNED (Post-Critical Review)"
  priority: "🔴 HIGH"
  estimated_time: "14-18h (INCREASED from 10-14h due to security/concurrency hardening)"
  dependencies:
    - "STDLIB-004"  # Array methods needed for chaining
  functions: 6
  cli_tools: 5
  module: "Directory traversal with parallel processing + fast text search (security-hardened)"
  tests:
    unit: 70  # INCREASED from 60
    concurrency: 3  # NEW: loom, thread sanitizer, stress
    security: 5     # NEW: traversal, symlinks, unicode, injection, TOCTOU
    benchmarks: 2   # NEW: abstraction overhead, parallel speedup
    interpreter: 45 # INCREASED from 38
    transpiler: 12  # INCREASED from 10
    integration: 6
    property: 4
    property_cases: 40000
    mutation_target: "≥90%"  # INCREASED from 80%
  quality:
    complexity_max: 10
    tdg_target: "A-"
    quality_gates: 16  # INCREASED from 7
  dependencies_crates:
    - "walkdir = \"2.5\""
    - "rayon = \"1.10\""
    - "glob = \"0.3\""
    - "num_cpus = \"1.16\""
    - "grep = \"0.3\""
    - "regex = \"1.10\""
    - "loom = \"0.7\""      # NEW: Systematic concurrency testing
  theoretical_foundations:
    - "Blumofe & Leiserson (1999): Work-stealing scheduler"
    - "Aho & Corasick (1975): Multi-pattern string matching"
    - "Thompson (1968): NFA-based regex matching"
    - "Flanagan & Godefroid (2005): DPOR for model checking"
  api:
    - "walk(path) -> Array<FileEntry> - Basic recursive walk"
    - "walk_parallel(path, callback) -> Array<Any> - Parallel processing"
    - "walk_with_options(path, options) -> Array<FileEntry> - Advanced options"
    - "glob(pattern) -> Array<String> - Glob pattern matching"
    - "find(path, predicate) -> Array<FileEntry> - Find with predicate"
    - "search(pattern, path, options?) -> Array<SearchMatch> - Fast text search"
  cli:
    - "ruchy find - Smart file finder (simpler than GNU find)"
    - "ruchy tree - Visual directory tree with stats"
    - "ruchy du - Disk usage with visual charts"
    - "ruchy count - File statistics with language detection"
    - "ruchy rg - Fast parallel text search (like ripgrep)"
  implementation_phases:
    - phase: "RED"
      tasks:
        - "Create tests/stdlib_dir_walk_test.rs with 70 unit tests"
        - "Add 3 concurrency tests (loom, thread sanitizer, stress)"
        - "Add 5 security tests (traversal, symlinks, unicode, injection, TOCTOU)"
        - "Add 2 performance benchmarks (abstraction overhead, parallel speedup)"
        - "All tests fail initially (no implementation)"
        - "Property tests defined (4 tests × 10K cases)"
    - phase: "GREEN"
      tasks:
        - "Implement walk() - basic recursive traversal"
        - "Implement walk_parallel() - rayon parallel processing (with memory defect documentation)"
        - "Implement walk_with_options() - advanced configuration"
        - "Implement glob() and find() utilities"
        - "Implement search() - fast text search with grep crate"
        - "All 70/70 unit tests passing"
        - "All 3 concurrency tests passing (loom + thread sanitizer clean)"
        - "All 5 security tests passing (attacks blocked)"
    - phase: "REFACTOR"
      tasks:
        - "Verify complexity ≤10 for all functions"
        - "Run mutation tests (target ≥90%)"
        - "Performance benchmarks: abstraction overhead <1µs, parallel speedup ≥2x"
        - "Security audit: penetration testing against documented attack vectors"
        - "Code review with algorithm justification (theoretical foundations)"
  use_cases:
    - "ETL pipelines: Process thousands of CSV files in parallel"
    - "Log analysis: Search errors across directory trees"
    - "Data science: Build training datasets from image directories"
    - "Code analysis: Count lines of code, find patterns"
    - "Security audits: Find sensitive data patterns in codebases"
  impact: "Enables high-performance data processing + text search for data engineering and sysadmin workflows"
```

---

## Success Criteria (Updated per Critical Review)

### Functional
- ✅ All 70 unit tests passing (100%) (INCREASED from 60)
- ✅ Property tests validate 40K random scenarios
- ✅ Real-world examples work (ETL, log analysis, dataset building, security audit)
- ✅ Parallel processing utilizes all CPU cores
- ✅ Text search matches ripgrep performance
- ✅ All 5 CLI tools functional and production-ready

### Concurrency (NEW)
- ✅ All 3 concurrency tests passing:
  - Loom: Systematic exploration of thread interleavings (no data races)
  - Thread Sanitizer: Dynamic race detection (zero violations)
  - Stress test: 100 iterations with high contention (no panics/deadlocks)

### Security (NEW)
- ✅ All 5 security tests passing:
  - Directory traversal: `../` escapes blocked/sanitized
  - Symlink bombs: Circular symlinks detected and terminated
  - Unicode attacks: Homograph attacks prevented via normalization
  - Path injection: Malicious filenames don't execute commands
  - TOCTOU races: Filesystem changes handled gracefully

### Quality
- ✅ Mutation coverage ≥90% (INCREASED from 80% per critical review)
- ✅ Complexity ≤10 for all functions
- ✅ Zero SATD (no TODO/FIXME/HACK)
- ✅ TDG grade A- minimum

### Performance (Updated)
- ✅ Parallel ≥2x faster than serial (4+ core systems)
- ✅ Abstraction overhead <1µs per item (NEW benchmark - was "zero-cost" claim)
- ✅ Memory efficient (iterator-based API planned for v2.0)
- ✅ Theoretical foundation validated (work-stealing, Aho-Corasick, Thompson NFA)

### Usability
- ✅ Simpler than Python's os.walk (fewer lines of code)
- ✅ More powerful than Python (built-in parallelism + text search)
- ✅ CLI tools require zero coding (like Python's -m modules)
- ✅ Examples in documentation (10+ real-world scenarios)
- ✅ Text search simpler than ripgrep (sensible defaults)

---

## Future Enhancements (Not in v1.0)

1. **Async I/O Support** (v2.0)
   - `walk_async()` for I/O-bound workloads
   - Tokio integration for network filesystems

2. **Progress Reporting** (v2.0)
   - Callback for progress updates
   - Estimated time remaining

3. **Caching** (v2.0)
   - Cache directory structure for repeated walks
   - Invalidation on filesystem changes

4. **Filter DSL** (v3.0)
   - SQL-like syntax: `walk("/data").where("size > 1MB AND ext = '.csv'")`

---

## References

- Rust `walkdir` crate: https://docs.rs/walkdir
- Rust `rayon` crate: https://docs.rs/rayon
- Python `os.walk`: https://docs.python.org/3/library/os.html#os.walk
- Ruchy stdlib quality gates: `docs/execution/roadmap.yaml`
