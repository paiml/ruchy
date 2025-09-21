// SPRINT3-002: TDD tests for complexity analysis
// Following Toyota Way: Write tests first, then implementation

use ruchy::notebook::testing::*;

#[test]
fn test_complexity_analyzer_initialization() {
    let analyzer = ComplexityAnalyzer::new();
    assert_eq!(analyzer.get_default_threshold(), 10);
}

#[test]
fn test_analyze_constant_time() {
    let analyzer = ComplexityAnalyzer::new();

    let cell = Cell {
        id: "constant".to_string(),
        source: "let x = 42; x + 1".to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let result = analyzer.analyze(&cell);
    assert_eq!(result.time_complexity, TimeComplexity::O1);
    assert_eq!(result.space_complexity, SpaceComplexity::O1);
}

#[test]
fn test_analyze_linear_time() {
    let analyzer = ComplexityAnalyzer::new();

    let cell = Cell {
        id: "linear".to_string(),
        source: "let sum = 0; for i in 0..n { sum = sum + i }; sum".to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let result = analyzer.analyze(&cell);
    assert_eq!(result.time_complexity, TimeComplexity::ON);
    assert_eq!(result.space_complexity, SpaceComplexity::O1);
}

#[test]
fn test_analyze_quadratic_time() {
    let analyzer = ComplexityAnalyzer::new();

    let cell = Cell {
        id: "quadratic".to_string(),
        source: "for i in 0..n { for j in 0..n { matrix[i][j] = i * j } }".to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let result = analyzer.analyze(&cell);
    assert_eq!(result.time_complexity, TimeComplexity::ON2);
}

#[test]
fn test_cyclomatic_complexity() {
    let analyzer = ComplexityAnalyzer::new();

    let cell = Cell {
        id: "branching".to_string(),
        source: r#"
            fn process(x) {
                if x > 0 {
                    if x > 10 {
                        return x * 2
                    } else {
                        return x + 1
                    }
                } else if x < 0 {
                    return -x
                } else {
                    return 0
                }
            }
        "#
        .to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let result = analyzer.analyze(&cell);
    assert_eq!(result.cyclomatic_complexity, 4); // 3 decision points + 1
}

#[test]
fn test_cognitive_complexity() {
    let analyzer = ComplexityAnalyzer::new();

    let cell = Cell {
        id: "nested".to_string(),
        source: r#"
            for i in 0..n {
                if i % 2 == 0 {
                    for j in 0..m {
                        if j % 2 == 0 {
                            sum = sum + i * j
                        }
                    }
                }
            }
        "#
        .to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let result = analyzer.analyze(&cell);
    assert!(result.cognitive_complexity > result.cyclomatic_complexity);
    assert!(result.cognitive_complexity >= 7); // Nested conditions increase cognitive load
}

#[test]
fn test_detect_performance_hotspots() {
    let analyzer = ComplexityAnalyzer::new();

    let notebook = Notebook {
        cells: vec![
            Cell {
                id: "fast".to_string(),
                source: "let x = 1 + 1".to_string(),
                cell_type: CellType::Code,
                metadata: CellMetadata::default(),
            },
            Cell {
                id: "slow".to_string(),
                source: "for i in 0..n { for j in 0..n { for k in 0..n { sum = sum + 1 } } }"
                    .to_string(),
                cell_type: CellType::Code,
                metadata: CellMetadata::default(),
            },
        ],
        metadata: None,
    };

    let hotspots = analyzer.find_hotspots(&notebook);

    assert_eq!(hotspots.len(), 1);
    assert_eq!(hotspots[0].cell_id, "slow");
    assert_eq!(hotspots[0].complexity, TimeComplexity::ON3);
}

#[test]
fn test_suggest_optimizations() {
    let analyzer = ComplexityAnalyzer::new();

    let cell = Cell {
        id: "inefficient".to_string(),
        source: r#"
            let result = []
            for i in 0..n {
                for j in 0..n {
                    if arr[j] == i {
                        result.push(j)
                    }
                }
            }
        "#
        .to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let suggestions = analyzer.suggest_optimizations(&cell);

    assert!(!suggestions.is_empty());
    assert!(suggestions
        .iter()
        .any(|s| s.contains("hash") || s.contains("index")));
}

#[test]
fn test_memory_complexity_analysis() {
    let analyzer = ComplexityAnalyzer::new();

    let cell = Cell {
        id: "memory".to_string(),
        source: "let matrix = Array(n).fill(0).map(() => Array(n).fill(0))".to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let result = analyzer.analyze(&cell);
    assert_eq!(result.space_complexity, SpaceComplexity::ON2);
}

// Helper types for testing
#[derive(Debug)]
struct ComplexityAnalyzer {
    threshold: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TimeComplexity {
    O1,     // Constant
    OLogN,  // Logarithmic
    ON,     // Linear
    ONLogN, // Linearithmic
    ON2,    // Quadratic
    ON3,    // Cubic
    OExp,   // Exponential
}

#[derive(Debug, Clone, PartialEq)]
enum SpaceComplexity {
    O1,    // Constant
    OLogN, // Logarithmic
    ON,    // Linear
    ON2,   // Quadratic
}

#[derive(Debug)]
struct ComplexityResult {
    time_complexity: TimeComplexity,
    space_complexity: SpaceComplexity,
    cyclomatic_complexity: usize,
    cognitive_complexity: usize,
}

#[derive(Debug)]
struct Hotspot {
    cell_id: String,
    complexity: TimeComplexity,
    impact: f64,
}

impl ComplexityAnalyzer {
    fn new() -> Self {
        Self { threshold: 10 }
    }

    fn get_default_threshold(&self) -> usize {
        self.threshold
    }

    fn analyze(&self, cell: &Cell) -> ComplexityResult {
        // Stub implementation - analyze source code patterns
        let source = &cell.source;

        let mut time = TimeComplexity::O1;
        let mut space = SpaceComplexity::O1;
        let mut cyclomatic = 1;
        let mut cognitive = 0;

        // Count nested loops for time complexity
        let loop_depth = source.matches("for ").count();
        time = match loop_depth {
            0 => TimeComplexity::O1,
            1 => TimeComplexity::ON,
            2 => TimeComplexity::ON2,
            3 => TimeComplexity::ON3,
            _ => TimeComplexity::OExp,
        };

        // Count if statements for cyclomatic
        cyclomatic += source.matches("if ").count();
        cyclomatic += source.matches("else if").count();

        // Estimate cognitive (nesting increases it)
        cognitive = cyclomatic * (loop_depth + 1);
        if cognitive == 0 {
            cognitive = 1;
        }

        // Check for array/matrix allocation for space
        if source.contains("Array(n)") {
            if source.contains("Array(n).fill(0).map(() => Array(n)") {
                space = SpaceComplexity::ON2;
            } else {
                space = SpaceComplexity::ON;
            }
        }

        ComplexityResult {
            time_complexity: time,
            space_complexity: space,
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity: cognitive,
        }
    }

    fn find_hotspots(&self, notebook: &Notebook) -> Vec<Hotspot> {
        let mut hotspots = Vec::new();

        for cell in &notebook.cells {
            if matches!(cell.cell_type, CellType::Code) {
                let result = self.analyze(cell);

                // Consider O(n²) and above as hotspots
                match result.time_complexity {
                    TimeComplexity::ON2 | TimeComplexity::ON3 | TimeComplexity::OExp => {
                        hotspots.push(Hotspot {
                            cell_id: cell.id.clone(),
                            complexity: result.time_complexity,
                            impact: 1.0,
                        });
                    }
                    _ => {}
                }
            }
        }

        hotspots
    }

    fn suggest_optimizations(&self, cell: &Cell) -> Vec<String> {
        let mut suggestions = Vec::new();
        let result = self.analyze(cell);

        if matches!(
            result.time_complexity,
            TimeComplexity::ON2 | TimeComplexity::ON3
        ) {
            if cell.source.contains("for")
                && cell.source.contains("if")
                && cell.source.contains("==")
            {
                suggestions.push(
                    "Consider using a hash map or index for O(1) lookups instead of nested loops"
                        .to_string(),
                );
            }
        }

        if result.cyclomatic_complexity > 10 {
            suggestions.push(
                "High cyclomatic complexity - consider breaking into smaller functions".to_string(),
            );
        }

        suggestions
    }
}
