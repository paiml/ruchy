// SPRINT3-002: Complexity analysis implementation
// PMAT Complexity: <10 per function

use crate::notebook::testing::types::*;

/// Complexity analysis for notebook cells
pub struct ComplexityAnalyzer {
    config: ComplexityConfig,
}

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
    O1,      // Constant
    OLogN,   // Logarithmic  
    ON,      // Linear
    ONLogN,  // Linearithmic
    ON2,     // Quadratic
    ON3,     // Cubic
    OExp,    // Exponential
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpaceComplexity {
    O1,      // Constant
    OLogN,   // Logarithmic
    ON,      // Linear
    ON2,     // Quadratic
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

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            config: ComplexityConfig::default(),
        }
    }
    
    pub fn with_config(config: ComplexityConfig) -> Self {
        Self { config }
    }
    
    pub fn get_default_threshold(&self) -> usize {
        self.config.cyclomatic_threshold
    }
    
    /// Analyze complexity of a cell
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
        
        if complexity == 0 { complexity = 1; }
        complexity
    }
    
    fn calculate_halstead(&self, source: &str) -> HalsteadMetrics {
        // Simplified Halstead metrics
        let operators = source.matches(|c: char| "+-*/%=<>!&|".contains(c)).count();
        let operands = source.split_whitespace()
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
                if current_depth > 0 {
                    current_depth -= 1;
                }
            }
        }
        
        max_depth
    }
    
    /// Find performance hotspots in a notebook
    pub fn find_hotspots(&self, notebook: &Notebook) -> Vec<Hotspot> {
        let mut hotspots = Vec::new();
        
        for cell in &notebook.cells {
            if matches!(cell.cell_type, CellType::Code) {
                let result = self.analyze(cell);
                
                // Check if it's a hotspot
                let is_hotspot = matches!(
                    result.time_complexity,
                    TimeComplexity::ON2 | TimeComplexity::ON3 | TimeComplexity::OExp
                ) || result.cyclomatic_complexity > self.config.cyclomatic_threshold;
                
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
                    suggestions.push("If data is partially sorted, consider using insertion sort or TimSort".to_string());
                }
            }
            TimeComplexity::OExp => {
                suggestions.push("Exponential complexity detected - consider dynamic programming or memoization".to_string());
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
            suggestions.push("High cognitive complexity - reduce nesting and simplify control flow".to_string());
        }
        
        suggestions
    }
}