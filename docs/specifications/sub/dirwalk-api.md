# Sub-spec: Directory Walking — API Design & Use Cases

**Parent:** [multi-threaded-dir-walk-spec.md](../multi-threaded-dir-walk-spec.md) API Design Section

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

