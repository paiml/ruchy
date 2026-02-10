//! Semantic Tagging & Corpus Filtering [7]
//!
//! Tags files by semantic category (async, generics, closures, etc.)
//! for targeted regression testing and corpus filtering.
//!
//! # Usage
//! ```text
//! ruchy corpus tag examples/    # Auto-tag all files
//! ruchy corpus filter --tag=async --tag=generics  # Filter corpus
//! ruchy test --tags=async       # Test only async-tagged files
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Semantic tag for categorizing code patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticTag {
    /// Async/await patterns
    Async,
    /// Generic type parameters
    Generics,
    /// Closures and lambdas
    Closures,
    /// Trait implementations
    Traits,
    /// Lifetimes and borrowing
    Lifetimes,
    /// Error handling (Result, Option)
    ErrorHandling,
    /// Macros usage
    Macros,
    /// Collections (`Vec`, `HashMap`, etc.)
    Collections,
    /// Iterators and combinators
    Iterators,
    /// Pattern matching
    PatternMatch,
    /// Concurrency (threads, channels)
    Concurrency,
    /// FFI and unsafe blocks
    Ffi,
    /// Standard library usage
    StdLib,
    /// I/O operations
    Io,
    /// String manipulation
    Strings,
    /// Numeric operations
    Numerics,
    /// Control flow (loops, conditionals)
    ControlFlow,
    /// Structs and enums
    DataTypes,
    /// Module system
    Modules,
    /// Testing code
    Testing,
}

impl SemanticTag {
    /// Get all semantic tags
    #[must_use]
    pub fn all() -> &'static [SemanticTag] {
        &[
            Self::Async,
            Self::Generics,
            Self::Closures,
            Self::Traits,
            Self::Lifetimes,
            Self::ErrorHandling,
            Self::Macros,
            Self::Collections,
            Self::Iterators,
            Self::PatternMatch,
            Self::Concurrency,
            Self::Ffi,
            Self::StdLib,
            Self::Io,
            Self::Strings,
            Self::Numerics,
            Self::ControlFlow,
            Self::DataTypes,
            Self::Modules,
            Self::Testing,
        ]
    }

    /// Parse tag from string
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "async" => Some(Self::Async),
            "generics" => Some(Self::Generics),
            "closures" | "closure" | "lambda" => Some(Self::Closures),
            "traits" | "trait" => Some(Self::Traits),
            "lifetimes" | "lifetime" | "borrow" => Some(Self::Lifetimes),
            "error" | "errors" | "error_handling" | "result" | "option" => {
                Some(Self::ErrorHandling)
            }
            "macros" | "macro" => Some(Self::Macros),
            "collections" | "collection" | "vec" | "hashmap" => Some(Self::Collections),
            "iterators" | "iterator" | "iter" => Some(Self::Iterators),
            "pattern" | "match" | "pattern_match" => Some(Self::PatternMatch),
            "concurrency" | "concurrent" | "thread" | "threads" => Some(Self::Concurrency),
            "ffi" | "unsafe" => Some(Self::Ffi),
            "stdlib" | "std" | "std_lib" => Some(Self::StdLib),
            "io" | "input" | "output" | "file" => Some(Self::Io),
            "strings" | "string" | "str" => Some(Self::Strings),
            "numerics" | "numeric" | "math" | "numbers" => Some(Self::Numerics),
            "control" | "control_flow" | "loop" | "if" => Some(Self::ControlFlow),
            "data" | "data_types" | "struct" | "enum" => Some(Self::DataTypes),
            "modules" | "module" | "mod" => Some(Self::Modules),
            "testing" | "test" | "tests" => Some(Self::Testing),
            _ => None,
        }
    }

    /// Get tag name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Async => "async",
            Self::Generics => "generics",
            Self::Closures => "closures",
            Self::Traits => "traits",
            Self::Lifetimes => "lifetimes",
            Self::ErrorHandling => "error_handling",
            Self::Macros => "macros",
            Self::Collections => "collections",
            Self::Iterators => "iterators",
            Self::PatternMatch => "pattern_match",
            Self::Concurrency => "concurrency",
            Self::Ffi => "ffi",
            Self::StdLib => "stdlib",
            Self::Io => "io",
            Self::Strings => "strings",
            Self::Numerics => "numerics",
            Self::ControlFlow => "control_flow",
            Self::DataTypes => "data_types",
            Self::Modules => "modules",
            Self::Testing => "testing",
        }
    }

    /// Get patterns to detect this tag in source code
    #[must_use]
    pub fn patterns(&self) -> &'static [&'static str] {
        match self {
            Self::Async => &["async", "await", ".await"],
            Self::Generics => &["<T>", "<T,", "<T:", "impl<", "where T", "-> T"],
            Self::Closures => &["|", "move |", "|| {", "fn("],
            Self::Traits => &["trait ", "impl ", "dyn ", "Box<dyn"],
            Self::Lifetimes => &["'a", "'static", "&'", "lifetime"],
            Self::ErrorHandling => &[
                "Result<", "Option<", "?", "unwrap(", "expect(", "Ok(", "Err(", "Some(", "None",
            ],
            Self::Macros => &["macro_rules!", "!", "#["],
            Self::Collections => &[
                "Vec<",
                "HashMap<",
                "HashSet<",
                "BTreeMap<",
                "VecDeque<",
                "vec![",
                "hashmap!",
            ],
            Self::Iterators => &[
                ".iter()",
                ".into_iter()",
                ".map(",
                ".filter(",
                ".collect(",
                ".fold(",
            ],
            Self::PatternMatch => &["match ", "if let ", "while let ", "=>"],
            Self::Concurrency => &["thread::", "spawn(", "Mutex<", "RwLock<", "Arc<", "mpsc::"],
            Self::Ffi => &["unsafe ", "extern ", "#[no_mangle]", "*const", "*mut"],
            Self::StdLib => &["std::", "use std::", "core::"],
            Self::Io => &["File::", "Read", "Write", "BufReader", "stdin(", "stdout("],
            Self::Strings => &["String::", "str::", "format!", "to_string(", "&str"],
            Self::Numerics => &["i32", "i64", "u32", "f64", "usize", "+", "-", "*", "/"],
            Self::ControlFlow => &[
                "if ", "else ", "for ", "while ", "loop ", "break", "continue", "return",
            ],
            Self::DataTypes => &["struct ", "enum ", "type "],
            Self::Modules => &["mod ", "pub mod", "use ", "crate::"],
            Self::Testing => &["#[test]", "#[cfg(test)]", "assert!", "assert_eq!", "test_"],
        }
    }

    /// Get tag complexity weight (higher = more complex feature)
    #[must_use]
    pub fn complexity_weight(&self) -> u8 {
        match self {
            Self::Async => 9,
            Self::Generics => 8,
            Self::Closures => 6,
            Self::Traits => 8,
            Self::Lifetimes => 10,
            Self::ErrorHandling => 5,
            Self::Macros => 9,
            Self::Collections => 4,
            Self::Iterators => 6,
            Self::PatternMatch => 5,
            Self::Concurrency => 9,
            Self::Ffi => 10,
            Self::StdLib => 3,
            Self::Io => 4,
            Self::Strings => 2,
            Self::Numerics => 2,
            Self::ControlFlow => 3,
            Self::DataTypes => 4,
            Self::Modules => 3,
            Self::Testing => 3,
        }
    }
}

impl fmt::Display for SemanticTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Tagged file entry in corpus
#[derive(Debug, Clone)]
pub struct TaggedFile {
    /// File path
    pub path: String,
    /// Detected tags
    pub tags: HashSet<SemanticTag>,
    /// Confidence score for each tag (0.0-1.0)
    pub confidence: HashMap<SemanticTag, f64>,
    /// Total complexity score
    pub complexity_score: u32,
}

impl TaggedFile {
    /// Create new tagged file
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            tags: HashSet::new(),
            confidence: HashMap::new(),
            complexity_score: 0,
        }
    }

    /// Add tag with confidence
    pub fn add_tag(&mut self, tag: SemanticTag, confidence: f64) {
        self.tags.insert(tag);
        self.confidence.insert(tag, confidence);
        self.complexity_score += u32::from(tag.complexity_weight());
    }

    /// Check if file has tag
    #[must_use]
    pub fn has_tag(&self, tag: SemanticTag) -> bool {
        self.tags.contains(&tag)
    }

    /// Check if file has all specified tags
    #[must_use]
    pub fn has_all_tags(&self, tags: &[SemanticTag]) -> bool {
        tags.iter().all(|t| self.tags.contains(t))
    }

    /// Check if file has any of specified tags
    #[must_use]
    pub fn has_any_tag(&self, tags: &[SemanticTag]) -> bool {
        tags.iter().any(|t| self.tags.contains(t))
    }

    /// Get primary tag (highest confidence)
    #[must_use]
    pub fn primary_tag(&self) -> Option<SemanticTag> {
        self.confidence
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&tag, _)| tag)
    }

    /// Get tag summary string
    #[must_use]
    pub fn tag_summary(&self) -> String {
        let mut tags: Vec<_> = self.tags.iter().collect();
        tags.sort_by_key(|t| t.name());
        tags.iter().map(|t| t.name()).collect::<Vec<_>>().join(", ")
    }
}

/// Semantic tagger for analyzing source files
#[derive(Debug, Default)]
pub struct SemanticTagger {
    /// Minimum pattern matches for tag detection
    pub min_matches: usize,
    /// Confidence threshold
    pub confidence_threshold: f64,
}

impl SemanticTagger {
    /// Create new tagger
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_matches: 1,
            confidence_threshold: 0.5,
        }
    }

    /// Set minimum matches required
    pub fn with_min_matches(mut self, min: usize) -> Self {
        self.min_matches = min;
        self
    }

    /// Set confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Analyze source code and return tags
    #[must_use]
    pub fn analyze(&self, source: &str) -> TaggedFile {
        let mut file = TaggedFile::new("");

        for tag in SemanticTag::all() {
            let patterns = tag.patterns();
            let mut matches = 0;

            for pattern in patterns {
                matches += source.matches(pattern).count();
            }

            if matches >= self.min_matches {
                // Confidence based on match density
                let source_len = source.len().max(1);
                let match_density = matches as f64 / (source_len as f64 / 100.0);
                let confidence = (match_density / 10.0).min(1.0);

                if confidence >= self.confidence_threshold {
                    file.add_tag(*tag, confidence);
                }
            }
        }

        file
    }

    /// Analyze and tag a file by path
    #[must_use]
    pub fn tag_file(&self, path: &str, source: &str) -> TaggedFile {
        let mut file = self.analyze(source);
        file.path = path.to_string();
        file
    }
}

/// Corpus filter for selecting files by tags
#[derive(Debug, Default)]
pub struct CorpusFilter {
    /// Required tags (AND logic)
    pub required: Vec<SemanticTag>,
    /// Optional tags (OR logic)
    pub optional: Vec<SemanticTag>,
    /// Excluded tags (NOT logic)
    pub excluded: Vec<SemanticTag>,
    /// Minimum complexity score
    pub min_complexity: Option<u32>,
    /// Maximum complexity score
    pub max_complexity: Option<u32>,
}

impl CorpusFilter {
    /// Create new filter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Require tag (AND)
    pub fn require(mut self, tag: SemanticTag) -> Self {
        self.required.push(tag);
        self
    }

    /// Add optional tag (OR)
    pub fn optional(mut self, tag: SemanticTag) -> Self {
        self.optional.push(tag);
        self
    }

    /// Exclude tag (NOT)
    pub fn exclude(mut self, tag: SemanticTag) -> Self {
        self.excluded.push(tag);
        self
    }

    /// Set minimum complexity
    pub fn with_min_complexity(mut self, min: u32) -> Self {
        self.min_complexity = Some(min);
        self
    }

    /// Set maximum complexity
    pub fn with_max_complexity(mut self, max: u32) -> Self {
        self.max_complexity = Some(max);
        self
    }

    /// Check if file matches filter
    #[must_use]
    pub fn matches(&self, file: &TaggedFile) -> bool {
        // Check required tags (all must be present)
        if !self.required.is_empty() && !file.has_all_tags(&self.required) {
            return false;
        }

        // Check optional tags (at least one must be present if any specified)
        if !self.optional.is_empty() && !file.has_any_tag(&self.optional) {
            return false;
        }

        // Check excluded tags (none should be present)
        if file.has_any_tag(&self.excluded) {
            return false;
        }

        // Check complexity bounds
        if let Some(min) = self.min_complexity {
            if file.complexity_score < min {
                return false;
            }
        }
        if let Some(max) = self.max_complexity {
            if file.complexity_score > max {
                return false;
            }
        }

        true
    }

    /// Filter corpus by this filter
    #[must_use]
    pub fn filter<'a>(&self, corpus: &'a [TaggedFile]) -> Vec<&'a TaggedFile> {
        corpus.iter().filter(|f| self.matches(f)).collect()
    }
}

/// Corpus statistics by tag
#[derive(Debug, Clone)]
pub struct TagStatistics {
    /// Tag being tracked
    pub tag: SemanticTag,
    /// Number of files with this tag
    pub file_count: usize,
    /// Total occurrences across all files
    pub total_occurrences: usize,
    /// Average confidence across files
    pub avg_confidence: f64,
    /// Files with this tag (paths)
    pub files: Vec<String>,
}

impl TagStatistics {
    /// Create new statistics entry
    #[must_use]
    pub fn new(tag: SemanticTag) -> Self {
        Self {
            tag,
            file_count: 0,
            total_occurrences: 0,
            avg_confidence: 0.0,
            files: Vec::new(),
        }
    }

    /// Add file to statistics
    pub fn add_file(&mut self, path: &str, confidence: f64) {
        self.files.push(path.to_string());
        self.total_occurrences += 1;
        self.file_count = self.files.len();

        // Update average confidence
        let total_confidence: f64 = self
            .files
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i == self.files.len() - 1 {
                    confidence
                } else {
                    self.avg_confidence
                }
            })
            .sum();
        self.avg_confidence = total_confidence / self.file_count as f64;
    }
}

/// Generate tag statistics for a corpus
#[must_use]
pub fn corpus_statistics(corpus: &[TaggedFile]) -> Vec<TagStatistics> {
    let mut stats: HashMap<SemanticTag, TagStatistics> = HashMap::new();

    for file in corpus {
        for &tag in &file.tags {
            let confidence = file.confidence.get(&tag).copied().unwrap_or(0.0);
            stats
                .entry(tag)
                .or_insert_with(|| TagStatistics::new(tag))
                .add_file(&file.path, confidence);
        }
    }

    let mut result: Vec<_> = stats.into_values().collect();
    result.sort_by(|a, b| b.file_count.cmp(&a.file_count));
    result
}

/// Render tag distribution as ASCII
#[must_use]
pub fn render_tag_distribution(stats: &[TagStatistics], width: usize) -> String {
    let mut lines = vec![format!(
        "{}╭{}╮",
        " ".repeat(2),
        "─".repeat(width.saturating_sub(4))
    )];

    lines.push(format!(
        "{}│ {:^width$} │",
        " ".repeat(2),
        "SEMANTIC TAG DISTRIBUTION",
        width = width.saturating_sub(6)
    ));

    lines.push(format!(
        "{}├{}┤",
        " ".repeat(2),
        "─".repeat(width.saturating_sub(4))
    ));

    let max_count = stats.iter().map(|s| s.file_count).max().unwrap_or(1);
    let bar_width = width.saturating_sub(30);

    for stat in stats.iter().take(10) {
        let bar_len = if max_count > 0 {
            (stat.file_count * bar_width) / max_count
        } else {
            0
        };
        let bar = "█".repeat(bar_len);
        let padding = " ".repeat(bar_width.saturating_sub(bar_len));

        lines.push(format!(
            "{}│ {:15} {:3} {}{} │",
            " ".repeat(2),
            stat.tag.name(),
            stat.file_count,
            bar,
            padding
        ));
    }

    lines.push(format!(
        "{}╰{}╯",
        " ".repeat(2),
        "─".repeat(width.saturating_sub(4))
    ));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: SemanticTag Tests
    // ============================================================

    #[test]
    fn test_semantic_tag_all() {
        let all = SemanticTag::all();
        assert_eq!(all.len(), 20);
    }

    #[test]
    fn test_semantic_tag_from_str() {
        assert_eq!(SemanticTag::from_str("async"), Some(SemanticTag::Async));
        assert_eq!(SemanticTag::from_str("ASYNC"), Some(SemanticTag::Async));
        assert_eq!(
            SemanticTag::from_str("generics"),
            Some(SemanticTag::Generics)
        );
        assert_eq!(
            SemanticTag::from_str("closure"),
            Some(SemanticTag::Closures)
        );
        assert_eq!(SemanticTag::from_str("unknown"), None);
    }

    #[test]
    fn test_semantic_tag_name() {
        assert_eq!(SemanticTag::Async.name(), "async");
        assert_eq!(SemanticTag::Generics.name(), "generics");
        assert_eq!(SemanticTag::ErrorHandling.name(), "error_handling");
    }

    #[test]
    fn test_semantic_tag_patterns() {
        let patterns = SemanticTag::Async.patterns();
        assert!(patterns.contains(&"async"));
        assert!(patterns.contains(&"await"));
    }

    #[test]
    fn test_semantic_tag_complexity() {
        assert!(
            SemanticTag::Lifetimes.complexity_weight() > SemanticTag::Strings.complexity_weight()
        );
        assert!(SemanticTag::Ffi.complexity_weight() >= 10);
        assert!(SemanticTag::Numerics.complexity_weight() <= 3);
    }

    #[test]
    fn test_semantic_tag_display() {
        assert_eq!(format!("{}", SemanticTag::Async), "async");
    }

    // ============================================================
    // EXTREME TDD: TaggedFile Tests
    // ============================================================

    #[test]
    fn test_tagged_file_new() {
        let file = TaggedFile::new("test.ruchy");
        assert_eq!(file.path, "test.ruchy");
        assert!(file.tags.is_empty());
        assert_eq!(file.complexity_score, 0);
    }

    #[test]
    fn test_tagged_file_add_tag() {
        let mut file = TaggedFile::new("test.ruchy");
        file.add_tag(SemanticTag::Async, 0.85);

        assert!(file.has_tag(SemanticTag::Async));
        assert_eq!(file.confidence.get(&SemanticTag::Async), Some(&0.85));
        assert!(file.complexity_score > 0);
    }

    #[test]
    fn test_tagged_file_has_all_tags() {
        let mut file = TaggedFile::new("test.ruchy");
        file.add_tag(SemanticTag::Async, 0.9);
        file.add_tag(SemanticTag::Generics, 0.8);

        assert!(file.has_all_tags(&[SemanticTag::Async, SemanticTag::Generics]));
        assert!(!file.has_all_tags(&[SemanticTag::Async, SemanticTag::Ffi]));
    }

    #[test]
    fn test_tagged_file_has_any_tag() {
        let mut file = TaggedFile::new("test.ruchy");
        file.add_tag(SemanticTag::Async, 0.9);

        assert!(file.has_any_tag(&[SemanticTag::Async, SemanticTag::Ffi]));
        assert!(!file.has_any_tag(&[SemanticTag::Ffi, SemanticTag::Generics]));
    }

    #[test]
    fn test_tagged_file_primary_tag() {
        let mut file = TaggedFile::new("test.ruchy");
        file.add_tag(SemanticTag::Async, 0.5);
        file.add_tag(SemanticTag::Generics, 0.9);

        assert_eq!(file.primary_tag(), Some(SemanticTag::Generics));
    }

    #[test]
    fn test_tagged_file_tag_summary() {
        let mut file = TaggedFile::new("test.ruchy");
        file.add_tag(SemanticTag::Async, 0.9);
        file.add_tag(SemanticTag::Closures, 0.8);

        let summary = file.tag_summary();
        assert!(summary.contains("async"));
        assert!(summary.contains("closures"));
    }

    // ============================================================
    // EXTREME TDD: SemanticTagger Tests
    // ============================================================

    #[test]
    fn test_semantic_tagger_new() {
        let tagger = SemanticTagger::new();
        assert_eq!(tagger.min_matches, 1);
        assert!((tagger.confidence_threshold - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_semantic_tagger_analyze_async() {
        let tagger = SemanticTagger::new();
        let source = "async fn main() { foo().await; }";
        let file = tagger.analyze(source);

        assert!(file.has_tag(SemanticTag::Async));
    }

    #[test]
    fn test_semantic_tagger_analyze_generics() {
        let tagger = SemanticTagger::new();
        let source = "fn process<T>(item: T) -> T { item }";
        let file = tagger.analyze(source);

        assert!(file.has_tag(SemanticTag::Generics));
    }

    #[test]
    fn test_semantic_tagger_analyze_error_handling() {
        let tagger = SemanticTagger::new();
        let source = "fn read() -> Result<String, Error> { Ok(data?) }";
        let file = tagger.analyze(source);

        assert!(file.has_tag(SemanticTag::ErrorHandling));
    }

    #[test]
    fn test_semantic_tagger_tag_file() {
        let tagger = SemanticTagger::new().with_confidence_threshold(0.1);
        let source = "fn main() { let x = vec![1, 2, 3]; let y = vec![4, 5]; }";
        let file = tagger.tag_file("test.ruchy", source);

        assert_eq!(file.path, "test.ruchy");
        assert!(file.has_tag(SemanticTag::Collections));
    }

    #[test]
    fn test_semantic_tagger_with_min_matches() {
        let tagger = SemanticTagger::new().with_min_matches(3);
        let source = "async fn a() {}"; // Only 1 async match
        let file = tagger.analyze(source);

        assert!(!file.has_tag(SemanticTag::Async)); // Not enough matches
    }

    // ============================================================
    // EXTREME TDD: CorpusFilter Tests
    // ============================================================

    #[test]
    fn test_corpus_filter_new() {
        let filter = CorpusFilter::new();
        assert!(filter.required.is_empty());
        assert!(filter.optional.is_empty());
        assert!(filter.excluded.is_empty());
    }

    #[test]
    fn test_corpus_filter_require() {
        let filter = CorpusFilter::new().require(SemanticTag::Async);

        let mut file1 = TaggedFile::new("a.ruchy");
        file1.add_tag(SemanticTag::Async, 0.9);

        let file2 = TaggedFile::new("b.ruchy");

        assert!(filter.matches(&file1));
        assert!(!filter.matches(&file2));
    }

    #[test]
    fn test_corpus_filter_optional() {
        let filter = CorpusFilter::new()
            .optional(SemanticTag::Async)
            .optional(SemanticTag::Generics);

        let mut file1 = TaggedFile::new("a.ruchy");
        file1.add_tag(SemanticTag::Async, 0.9);

        let mut file2 = TaggedFile::new("b.ruchy");
        file2.add_tag(SemanticTag::Closures, 0.9);

        assert!(filter.matches(&file1)); // Has async
        assert!(!filter.matches(&file2)); // Has neither
    }

    #[test]
    fn test_corpus_filter_exclude() {
        let filter = CorpusFilter::new().exclude(SemanticTag::Ffi);

        let mut file1 = TaggedFile::new("a.ruchy");
        file1.add_tag(SemanticTag::Async, 0.9);

        let mut file2 = TaggedFile::new("b.ruchy");
        file2.add_tag(SemanticTag::Ffi, 0.9);

        assert!(filter.matches(&file1)); // No FFI
        assert!(!filter.matches(&file2)); // Has FFI
    }

    #[test]
    fn test_corpus_filter_complexity() {
        let filter = CorpusFilter::new()
            .with_min_complexity(5)
            .with_max_complexity(20);

        let mut file1 = TaggedFile::new("a.ruchy");
        file1.add_tag(SemanticTag::Async, 0.9); // Weight 9

        let mut file2 = TaggedFile::new("b.ruchy");
        file2.add_tag(SemanticTag::Strings, 0.9); // Weight 2

        assert!(filter.matches(&file1)); // Complexity 9
        assert!(!filter.matches(&file2)); // Complexity 2 < min 5
    }

    #[test]
    fn test_corpus_filter_filter() {
        let filter = CorpusFilter::new().require(SemanticTag::Async);

        let mut file1 = TaggedFile::new("a.ruchy");
        file1.add_tag(SemanticTag::Async, 0.9);

        let file2 = TaggedFile::new("b.ruchy");

        let corpus = vec![file1, file2];
        let filtered = filter.filter(&corpus);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].path, "a.ruchy");
    }

    // ============================================================
    // EXTREME TDD: Statistics Tests
    // ============================================================

    #[test]
    fn test_tag_statistics_new() {
        let stats = TagStatistics::new(SemanticTag::Async);
        assert_eq!(stats.tag, SemanticTag::Async);
        assert_eq!(stats.file_count, 0);
    }

    #[test]
    fn test_tag_statistics_add_file() {
        let mut stats = TagStatistics::new(SemanticTag::Async);
        stats.add_file("a.ruchy", 0.9);
        stats.add_file("b.ruchy", 0.8);

        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.total_occurrences, 2);
    }

    #[test]
    fn test_corpus_statistics() {
        let mut file1 = TaggedFile::new("a.ruchy");
        file1.add_tag(SemanticTag::Async, 0.9);
        file1.add_tag(SemanticTag::Generics, 0.8);

        let mut file2 = TaggedFile::new("b.ruchy");
        file2.add_tag(SemanticTag::Async, 0.7);

        let corpus = vec![file1, file2];
        let stats = corpus_statistics(&corpus);

        // Async should be first (2 files)
        assert_eq!(stats[0].tag, SemanticTag::Async);
        assert_eq!(stats[0].file_count, 2);
    }

    #[test]
    fn test_render_tag_distribution() {
        let mut file = TaggedFile::new("a.ruchy");
        file.add_tag(SemanticTag::Async, 0.9);

        let corpus = vec![file];
        let stats = corpus_statistics(&corpus);
        let output = render_tag_distribution(&stats, 50);

        assert!(output.contains("SEMANTIC TAG"));
        assert!(output.contains("async"));
    }

    // ========================================================================
    // Coverage: SemanticTag::name — all variants (16 uncov, 30.4% cov)
    // ========================================================================

    #[test]
    fn test_semantic_tag_name_all_variants() {
        assert_eq!(SemanticTag::Async.name(), "async");
        assert_eq!(SemanticTag::Generics.name(), "generics");
        assert_eq!(SemanticTag::Closures.name(), "closures");
        assert_eq!(SemanticTag::Traits.name(), "traits");
        assert_eq!(SemanticTag::Lifetimes.name(), "lifetimes");
        assert_eq!(SemanticTag::ErrorHandling.name(), "error_handling");
        assert_eq!(SemanticTag::Macros.name(), "macros");
        assert_eq!(SemanticTag::Collections.name(), "collections");
        assert_eq!(SemanticTag::Iterators.name(), "iterators");
        assert_eq!(SemanticTag::PatternMatch.name(), "pattern_match");
        assert_eq!(SemanticTag::Concurrency.name(), "concurrency");
        assert_eq!(SemanticTag::Ffi.name(), "ffi");
        assert_eq!(SemanticTag::StdLib.name(), "stdlib");
        assert_eq!(SemanticTag::Io.name(), "io");
        assert_eq!(SemanticTag::Strings.name(), "strings");
        assert_eq!(SemanticTag::Numerics.name(), "numerics");
        assert_eq!(SemanticTag::ControlFlow.name(), "control_flow");
        assert_eq!(SemanticTag::DataTypes.name(), "data_types");
        assert_eq!(SemanticTag::Modules.name(), "modules");
        assert_eq!(SemanticTag::Testing.name(), "testing");
    }
}
