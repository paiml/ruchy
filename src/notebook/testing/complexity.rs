// SPRINT3-002: Complexity analysis implementation
// PMAT Complexity: <10 per function
use crate::notebook::testing::types::{Cell, CellType, Notebook};
#[derive(Debug, Clone)]
pub struct ComplexityConfig {
    pub cyclomatic_threshold: usize,
    pub cognitive_threshold: usize,
    pub enable_suggestions: bool,
}
impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            cyclomatic_threshold: 10,
            cognitive_threshold: 15,
            enable_suggestions: true,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TimeComplexity {
    O1,     // Constant
    OLogN,  // Logarithmic
    ON,     // Linear
    ONLogN, // Linearithmic
    ON2,    // Quadratic
    ON3,    // Cubic
    OExp,   // Exponential
}
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceComplexity {
    O1,    // Constant
    OLogN, // Logarithmic
    ON,    // Linear
    ON2,   // Quadratic
}
#[derive(Debug, Clone)]
pub struct ComplexityResult {
    pub time_complexity: TimeComplexity,
    pub space_complexity: SpaceComplexity,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub halstead_metrics: HalsteadMetrics,
}
#[derive(Debug, Clone)]
pub struct HalsteadMetrics {
    pub volume: f64,
    pub difficulty: f64,
    pub effort: f64,
}
#[derive(Debug, Clone)]
pub struct Hotspot {
    pub cell_id: String,
    pub complexity: TimeComplexity,
    pub impact: f64,
    pub location: String,
}

/// Complexity analyzer for notebook code
pub struct ComplexityAnalyzer {
    config: ComplexityConfig,
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplexityAnalyzer {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::complexity::ComplexityAnalyzer;
    ///
    /// let instance = ComplexityAnalyzer::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            config: ComplexityConfig::default(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::complexity::ComplexityAnalyzer;
    ///
    /// let mut instance = ComplexityAnalyzer::new();
    /// let result = instance.with_config();
    /// // Verify behavior
    /// ```
    pub fn with_config(config: ComplexityConfig) -> Self {
        Self { config }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::complexity::ComplexityAnalyzer;
    ///
    /// let mut instance = ComplexityAnalyzer::new();
    /// let result = instance.get_default_threshold();
    /// // Verify behavior
    /// ```
    pub fn get_default_threshold(&self) -> usize {
        self.config.cyclomatic_threshold
    }
    /// Analyze complexity of a cell
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::complexity::ComplexityAnalyzer;
    ///
    /// let mut instance = ComplexityAnalyzer::new();
    /// let result = instance.analyze();
    /// // Verify behavior
    /// ```
    pub fn analyze(&self, cell: &Cell) -> ComplexityResult {
        let source = &cell.source;
        // Analyze time complexity
        let time_complexity = self.analyze_time_complexity(source);
        // Analyze space complexity
        let space_complexity = self.analyze_space_complexity(source);
        // Calculate cyclomatic complexity
        let cyclomatic = self.calculate_cyclomatic(source);
        // Calculate cognitive complexity
        let cognitive = self.calculate_cognitive(source);
        // Calculate Halstead metrics
        let halstead = self.calculate_halstead(source);
        ComplexityResult {
            time_complexity,
            space_complexity,
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity: cognitive,
            halstead_metrics: halstead,
        }
    }
    fn analyze_time_complexity(&self, source: &str) -> TimeComplexity {
        // ISSUE-142 FIX: Check for recursion FIRST before loop analysis
        let recursive_calls = self.count_recursive_calls(source);
        let loop_depth = self.count_loop_depth(source);

        // Recursion detection (Issue #142)
        if recursive_calls >= 2 {
            // Multiple recursive calls = exponential (fibonacci, tribonacci, etc.)
            TimeComplexity::OExp
        } else if recursive_calls == 1 && loop_depth >= 1 {
            // Single recursion + loop = exponential or worse
            TimeComplexity::OExp
        } else if recursive_calls == 1 {
            // Single recursive call = linear (factorial, sum, etc.)
            TimeComplexity::ON
        } else {
            // No recursion - analyze loops only
            self.analyze_loop_complexity(source, loop_depth)
        }
    }

    /// Analyze complexity based on loop depth only (no recursion)
    /// Extracted from `analyze_time_complexity` to reduce complexity
    /// Complexity: 6 (≤10 ✓)
    fn analyze_loop_complexity(&self, source: &str, loop_depth: usize) -> TimeComplexity {
        match loop_depth {
            0 => {
                if source.contains("sort") {
                    TimeComplexity::ONLogN
                } else if source.contains("binary_search") {
                    TimeComplexity::OLogN
                } else {
                    TimeComplexity::O1
                }
            }
            1 => TimeComplexity::ON,
            2 => TimeComplexity::ON2,
            3 => TimeComplexity::ON3,
            _ => TimeComplexity::OExp,
        }
    }
    fn analyze_space_complexity(&self, source: &str) -> SpaceComplexity {
        if source.contains("Array(n).fill(0).map(() => Array(n)") {
            SpaceComplexity::ON2
        } else if source.contains("Array(n)") || source.contains("vec![") {
            SpaceComplexity::ON
        } else if source.contains("recursive") || source.contains("fn ") {
            SpaceComplexity::OLogN // Stack space for recursion
        } else {
            SpaceComplexity::O1
        }
    }
    fn calculate_cyclomatic(&self, source: &str) -> usize {
        let mut complexity = 1; // Base complexity
                                // Decision points
        complexity += source.matches("if ").count();
        complexity += source.matches("else if").count();
        complexity += source.matches("for ").count();
        complexity += source.matches("while ").count();
        complexity += source.matches("match ").count();
        complexity += source.matches("&&").count();
        complexity += source.matches("||").count();
        complexity
    }
    fn calculate_cognitive(&self, source: &str) -> usize {
        let mut complexity = 0;
        let mut nesting_level = 0;
        for line in source.lines() {
            let trimmed = line.trim();
            // Increase nesting for blocks
            if trimmed.contains('{') {
                nesting_level += 1;
            }
            // Add complexity for control flow at current nesting
            if trimmed.starts_with("if ") || trimmed.starts_with("else if") {
                complexity += 1 + nesting_level;
            }
            if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                complexity += 1 + nesting_level;
            }
            // Decrease nesting
            if trimmed.contains('}') && nesting_level > 0 {
                nesting_level -= 1;
            }
        }
        if complexity == 0 {
            complexity = 1;
        }
        complexity
    }
    fn calculate_halstead(&self, source: &str) -> HalsteadMetrics {
        // Simplified Halstead metrics
        let operators = source.matches(|c: char| "+-*/%=<>!&|".contains(c)).count();
        let operands = source
            .split_whitespace()
            .filter(|w| w.parse::<f64>().is_ok() || w.starts_with('"'))
            .count();
        let n1 = operators.max(1) as f64;
        let n2 = operands.max(1) as f64;
        let n = n1 + n2;
        let volume = n * n.log2();
        let difficulty = (n1 / 2.0) * (n2 / n2.max(1.0));
        let effort = volume * difficulty;
        HalsteadMetrics {
            volume,
            difficulty,
            effort,
        }
    }
    fn count_loop_depth(&self, source: &str) -> usize {
        let mut max_depth: usize = 0;
        let mut current_depth: usize = 0;
        for line in source.lines() {
            if line.trim().starts_with("for ") || line.trim().starts_with("while ") {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            if line.contains('}') {
                current_depth = current_depth.saturating_sub(1);
            }
        }
        max_depth
    }

    /// Count recursive function calls (Issue #142)
    /// Detects branching recursion like fibonacci(n-1) + fibonacci(n-2)
    /// Complexity: 6 (simple string analysis with helper)
    fn count_recursive_calls(&self, source: &str) -> usize {
        // Extract function name from source
        let func_name = self.extract_function_name(source);

        if func_name.is_empty() {
            return 0; // No function found
        }

        // Count how many times function calls itself
        // Look for patterns like: func_name( or func_name (
        let pattern1 = format!("{func_name}(");
        let pattern2 = format!("{func_name} (");

        let count =
            source.matches(pattern1.as_str()).count() + source.matches(pattern2.as_str()).count();

        // Subtract 1 for the function definition itself
        count.saturating_sub(1)
    }

    /// Extract function name from source code (helper for recursion detection)
    /// Looks for patterns: "fun name(" or "fn name("
    /// Complexity: 4 (simple string parsing)
    fn extract_function_name(&self, source: &str) -> String {
        for line in source.lines() {
            let trimmed = line.trim();
            // Match Ruchy syntax: "fun function_name(" or "pub fun function_name("
            if let Some(start) = trimmed.find("fun ") {
                let after_fun = &trimmed[start + 4..];
                if let Some(paren) = after_fun.find('(') {
                    return after_fun[..paren].trim().to_string();
                }
            }
            // Match Rust syntax: "fn function_name("
            if let Some(start) = trimmed.find("fn ") {
                let after_fn = &trimmed[start + 3..];
                if let Some(paren) = after_fn.find('(') {
                    return after_fn[..paren].trim().to_string();
                }
            }
        }
        String::new() // No function found
    }

    /// Find performance hotspots in a notebook
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::complexity::find_hotspots;
    ///
    /// let result = find_hotspots(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn find_hotspots(&self, notebook: &Notebook) -> Vec<Hotspot> {
        let mut hotspots = Vec::new();
        for cell in &notebook.cells {
            if matches!(cell.cell_type, CellType::Code) {
                let result = self.analyze(cell);
                // Check if it's a hotspot
                let is_hotspot = matches!(
                    result.time_complexity,
                    TimeComplexity::ON2 | TimeComplexity::ON3 | TimeComplexity::OExp
                ) || result.cyclomatic_complexity
                    > self.config.cyclomatic_threshold;
                if is_hotspot {
                    let impact = self.calculate_impact(&result);
                    hotspots.push(Hotspot {
                        cell_id: cell.id.clone(),
                        complexity: result.time_complexity,
                        impact,
                        location: format!("Cell {}", cell.id),
                    });
                }
            }
        }
        // Sort by impact
        hotspots.sort_by(|a, b| {
            b.impact
                .partial_cmp(&a.impact)
                .expect("Impact values must be valid f64 (not NaN) for sorting")
        });
        hotspots
    }
    fn calculate_impact(&self, result: &ComplexityResult) -> f64 {
        let time_weight = match result.time_complexity {
            TimeComplexity::O1 => 0.1,
            TimeComplexity::OLogN => 0.2,
            TimeComplexity::ON => 0.3,
            TimeComplexity::ONLogN => 0.5,
            TimeComplexity::ON2 => 0.7,
            TimeComplexity::ON3 => 0.9,
            TimeComplexity::OExp => 1.0,
        };
        let cyclo_weight = (result.cyclomatic_complexity as f64 / 20.0).min(1.0);
        time_weight * 0.7 + cyclo_weight * 0.3
    }
    /// Suggest optimizations for a cell
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::complexity::suggest_optimizations;
    ///
    /// let result = suggest_optimizations(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn suggest_optimizations(&self, cell: &Cell) -> Vec<String> {
        let mut suggestions = Vec::new();
        if !self.config.enable_suggestions {
            return suggestions;
        }
        let result = self.analyze(cell);
        // Time complexity suggestions
        match result.time_complexity {
            TimeComplexity::ON2 | TimeComplexity::ON3 => {
                if cell.source.contains("for") && cell.source.contains("==") {
                    suggestions.push(
                        "Consider using a hash map or index for O(1) lookups instead of nested loops".to_string(),
                    );
                }
                if cell.source.contains("sort") {
                    suggestions.push(
                        "If data is partially sorted, consider using insertion sort or TimSort"
                            .to_string(),
                    );
                }
            }
            TimeComplexity::OExp => {
                suggestions.push(
                    "Exponential complexity detected - consider dynamic programming or memoization"
                        .to_string(),
                );
            }
            _ => {}
        }
        // Cyclomatic complexity suggestions
        if result.cyclomatic_complexity > self.config.cyclomatic_threshold {
            suggestions.push(format!(
                "High cyclomatic complexity ({}) - consider breaking into smaller functions",
                result.cyclomatic_complexity
            ));
        }
        // Cognitive complexity suggestions
        if result.cognitive_complexity > self.config.cognitive_threshold {
            suggestions.push(
                "High cognitive complexity - reduce nesting and simplify control flow".to_string(),
            );
        }
        suggestions
    }
}

#[cfg(test)]
mod test_issue_142_bigo_recursion {
    use crate::notebook::testing::complexity::{ComplexityAnalyzer, TimeComplexity};
    use crate::notebook::testing::types::{Cell, CellMetadata, CellType};

    fn make_test_cell(id: &str, code: &str) -> Cell {
        Cell {
            id: id.to_string(),
            source: code.to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata { test: None },
        }
    }

    #[test]
    fn test_issue_142_recursive_fibonacci_should_be_exponential() {
        // Issue #142: fibonacci(n-1) + fibonacci(n-2) is O(2^n)
        let analyzer = ComplexityAnalyzer::new();
        let cell = make_test_cell(
            "fib",
            r"
pub fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
",
        );

        let result = analyzer.analyze(&cell);
        assert_eq!(
            result.time_complexity,
            TimeComplexity::OExp,
            "Fibonacci with 2 recursive calls should be O(2^n) exponential"
        );
    }

    #[test]
    fn test_single_recursion_factorial_should_be_linear() {
        // Single recursive call is O(n)
        let analyzer = ComplexityAnalyzer::new();
        let cell = make_test_cell(
            "fact",
            r"
pub fun factorial(n: i32) -> i32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
",
        );

        let result = analyzer.analyze(&cell);
        assert_eq!(
            result.time_complexity,
            TimeComplexity::ON,
            "Factorial with 1 recursive call should be O(n) linear"
        );
    }

    #[test]
    fn test_triple_recursion_should_be_exponential() {
        // Triple recursion is O(3^n)
        let analyzer = ComplexityAnalyzer::new();
        let cell = make_test_cell(
            "trib",
            r"
pub fun tribonacci(n: i32) -> i32 {
    if n <= 1 { n } 
    else { tribonacci(n-1) + tribonacci(n-2) + tribonacci(n-3) }
}
",
        );

        let result = analyzer.analyze(&cell);
        assert_eq!(
            result.time_complexity,
            TimeComplexity::OExp,
            "Tribonacci with 3 recursive calls should be O(3^n) exponential"
        );
    }

    #[test]
    fn test_non_recursive_linear_search() {
        // Simple loop, no recursion
        let analyzer = ComplexityAnalyzer::new();
        let cell = make_test_cell(
            "search",
            r"
pub fun search(arr: Vec<i32>, target: i32) -> bool {
    for x in arr {
        if x == target { return true; }
    }
    false
}
",
        );

        let result = analyzer.analyze(&cell);
        assert_eq!(
            result.time_complexity,
            TimeComplexity::ON,
            "Single loop should be O(n) linear"
        );
    }

    #[test]
    fn test_no_recursion_no_loops_should_be_constant() {
        let analyzer = ComplexityAnalyzer::new();
        let cell = make_test_cell(
            "add",
            r"
pub fun add(a: i32, b: i32) -> i32 {
    a + b
}
",
        );

        let result = analyzer.analyze(&cell);
        assert_eq!(
            result.time_complexity,
            TimeComplexity::O1,
            "Simple arithmetic should be O(1) constant"
        );
    }

    #[test]
    fn test_recursion_with_loop_should_be_exponential_or_worse() {
        // Recursion + loop combination
        let analyzer = ComplexityAnalyzer::new();
        let cell = make_test_cell(
            "weird",
            r"
pub fun weird_recursion(n: i32) -> i32 {
    if n <= 0 { return 0; }
    let mut sum = 0;
    for i in 0..n {
        sum += weird_recursion(n - 1);
    }
    sum
}
",
        );

        let result = analyzer.analyze(&cell);
        // Should be at least exponential (recursion in a loop)
        assert!(
            matches!(result.time_complexity, TimeComplexity::OExp),
            "Recursion inside loop should be exponential or worse"
        );
    }
}
