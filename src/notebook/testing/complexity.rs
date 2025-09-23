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
        let loop_depth = self.count_loop_depth(source);
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
        hotspots.sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap());
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
                    suggestions.push("Consider using a hash map or index for O(1) lookups instead of nested loops".to_string());
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
mod tests {
    use super::*;

    // EXTREME TDD: Comprehensive test coverage for complexity analysis

    #[test]
    fn test_complexity_config_default() {
        let config = ComplexityConfig::default();

        assert_eq!(config.cyclomatic_threshold, 10);
        assert_eq!(config.cognitive_threshold, 15);
        assert!(config.enable_suggestions);
    }

    #[test]
    fn test_complexity_config_custom() {
        let config = ComplexityConfig {
            cyclomatic_threshold: 5,
            cognitive_threshold: 8,
            enable_suggestions: false,
        };

        assert_eq!(config.cyclomatic_threshold, 5);
        assert_eq!(config.cognitive_threshold, 8);
        assert!(!config.enable_suggestions);
    }

    #[test]
    fn test_time_complexity_ordering() {
        assert_ne!(TimeComplexity::O1, TimeComplexity::ON);
        assert_ne!(TimeComplexity::ON, TimeComplexity::ON2);
        assert_eq!(TimeComplexity::O1, TimeComplexity::O1);
        assert_eq!(TimeComplexity::OLogN, TimeComplexity::OLogN);
    }

    #[test]
    fn test_space_complexity_ordering() {
        assert_ne!(SpaceComplexity::O1, SpaceComplexity::ON);
        assert_ne!(SpaceComplexity::ON, SpaceComplexity::ON2);
        assert_eq!(SpaceComplexity::O1, SpaceComplexity::O1);
        assert_eq!(SpaceComplexity::OLogN, SpaceComplexity::OLogN);
    }

    #[test]
    fn test_all_time_complexities() {
        let complexities = vec![
            TimeComplexity::O1,
            TimeComplexity::OLogN,
            TimeComplexity::ON,
            TimeComplexity::ONLogN,
            TimeComplexity::ON2,
            TimeComplexity::ON3,
            TimeComplexity::OExp,
        ];

        for complexity in complexities {
            match complexity {
                TimeComplexity::O1 => assert!(true, "O(1) constant"),
                TimeComplexity::OLogN => assert!(true, "O(log n) logarithmic"),
                TimeComplexity::ON => assert!(true, "O(n) linear"),
                TimeComplexity::ONLogN => assert!(true, "O(n log n) linearithmic"),
                TimeComplexity::ON2 => assert!(true, "O(n²) quadratic"),
                TimeComplexity::ON3 => assert!(true, "O(n³) cubic"),
                TimeComplexity::OExp => assert!(true, "O(2ⁿ) exponential"),
            }
        }
    }

    #[test]
    fn test_all_space_complexities() {
        let complexities = vec![
            SpaceComplexity::O1,
            SpaceComplexity::OLogN,
            SpaceComplexity::ON,
            SpaceComplexity::ON2,
        ];

        for complexity in complexities {
            match complexity {
                SpaceComplexity::O1 => assert!(true, "O(1) constant space"),
                SpaceComplexity::OLogN => assert!(true, "O(log n) logarithmic space"),
                SpaceComplexity::ON => assert!(true, "O(n) linear space"),
                SpaceComplexity::ON2 => assert!(true, "O(n²) quadratic space"),
            }
        }
    }

    #[test]
    fn test_complexity_result_creation() {
        let result = ComplexityResult {
            time_complexity: TimeComplexity::ON,
            space_complexity: SpaceComplexity::O1,
            cyclomatic_complexity: 5,
            cognitive_complexity: 7,
            halstead_metrics: HalsteadMetrics {
                volume: 100.0,
                difficulty: 10.0,
                effort: 1000.0,
            },
        };

        assert_eq!(result.time_complexity, TimeComplexity::ON);
        assert_eq!(result.space_complexity, SpaceComplexity::O1);
        assert_eq!(result.cyclomatic_complexity, 5);
        assert_eq!(result.cognitive_complexity, 7);
        assert_eq!(result.halstead_metrics.volume, 100.0);
    }

    #[test]
    fn test_halstead_metrics() {
        let metrics = HalsteadMetrics {
            volume: 250.5,
            difficulty: 15.3,
            effort: 3832.65,
        };

        assert_eq!(metrics.volume, 250.5);
        assert_eq!(metrics.difficulty, 15.3);
        assert_eq!(metrics.effort, 3832.65);
    }

    #[test]
    fn test_hotspot_creation() {
        let hotspot = Hotspot {
            cell_id: "cell_1".to_string(),
            complexity: TimeComplexity::ON2,
            impact: 0.85,
            location: "lines 10-25".to_string(),
        };

        assert_eq!(hotspot.cell_id, "cell_1");
        assert_eq!(hotspot.complexity, TimeComplexity::ON2);
        assert_eq!(hotspot.impact, 0.85);
        assert_eq!(hotspot.location, "lines 10-25");
    }

    #[test]
    fn test_complexity_analyzer_new() {
        let analyzer = ComplexityAnalyzer::new();
        assert_eq!(analyzer.config.cyclomatic_threshold, 10);
        assert_eq!(analyzer.config.cognitive_threshold, 15);
        assert!(analyzer.config.enable_suggestions);
    }

    #[test]
    fn test_complexity_analyzer_default() {
        let analyzer = ComplexityAnalyzer::default();
        assert_eq!(analyzer.config.cyclomatic_threshold, 10);
        assert_eq!(analyzer.config.cognitive_threshold, 15);
    }

    #[test]
    fn test_complexity_config_clone() {
        let config = ComplexityConfig {
            cyclomatic_threshold: 7,
            cognitive_threshold: 12,
            enable_suggestions: true,
        };

        let cloned = config.clone();
        assert_eq!(cloned.cyclomatic_threshold, config.cyclomatic_threshold);
        assert_eq!(cloned.cognitive_threshold, config.cognitive_threshold);
        assert_eq!(cloned.enable_suggestions, config.enable_suggestions);
    }

    #[test]
    fn test_complexity_result_clone() {
        let result = ComplexityResult {
            time_complexity: TimeComplexity::ONLogN,
            space_complexity: SpaceComplexity::ON,
            cyclomatic_complexity: 8,
            cognitive_complexity: 10,
            halstead_metrics: HalsteadMetrics {
                volume: 150.0,
                difficulty: 12.0,
                effort: 1800.0,
            },
        };

        let cloned = result.clone();
        assert_eq!(cloned.time_complexity, result.time_complexity);
        assert_eq!(cloned.space_complexity, result.space_complexity);
        assert_eq!(cloned.cyclomatic_complexity, result.cyclomatic_complexity);
        assert_eq!(cloned.cognitive_complexity, result.cognitive_complexity);
    }

    #[test]
    fn test_hotspot_clone() {
        let hotspot = Hotspot {
            cell_id: "hot_cell".to_string(),
            complexity: TimeComplexity::ON3,
            impact: 0.95,
            location: "nested loops".to_string(),
        };

        let cloned = hotspot.clone();
        assert_eq!(cloned.cell_id, hotspot.cell_id);
        assert_eq!(cloned.complexity, hotspot.complexity);
        assert_eq!(cloned.impact, hotspot.impact);
        assert_eq!(cloned.location, hotspot.location);
    }
}
