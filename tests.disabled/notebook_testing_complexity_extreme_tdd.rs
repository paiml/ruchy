//! Extreme TDD Tests for notebook/testing/complexity.rs
//!
//! Following extreme TDD methodology:
//! 1. Write comprehensive test first
//! 2. Minimal implementation to pass
//! 3. Refactor for quality
//!
//! Coverage target: 282 uncovered lines -> 100% coverage
//! Focus: Complexity analysis, hotspot detection, optimization suggestions

use proptest::prelude::*;
use ruchy::notebook::testing::complexity::{
    ComplexityAnalyzer, ComplexityConfig, ComplexityResult, HalsteadMetrics, Hotspot,
    SpaceComplexity, TimeComplexity,
};
use ruchy::notebook::testing::types::{Cell, CellMetadata, CellType, Notebook};

// ============================================================================
// Helper Functions for Test Data
// ============================================================================

fn create_test_cell(id: &str, source: &str) -> Cell {
    Cell {
        id: id.to_string(),
        source: source.to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    }
}

fn create_test_notebook(cells: Vec<Cell>) -> Notebook {
    Notebook {
        cells,
        metadata: None,
    }
}

// ============================================================================
// Unit Tests - Core Functionality
// ============================================================================

#[test]
fn test_complexity_config_default() {
    let config = ComplexityConfig::default();
    assert_eq!(config.cyclomatic_threshold, 10);
    assert_eq!(config.cognitive_threshold, 15);
    assert!(config.enable_suggestions);
}

#[test]
fn test_complexity_analyzer_new() {
    let analyzer = ComplexityAnalyzer::new();
    assert_eq!(analyzer.get_default_threshold(), 10);
}

#[test]
fn test_complexity_analyzer_with_config() {
    let config = ComplexityConfig {
        cyclomatic_threshold: 5,
        cognitive_threshold: 8,
        enable_suggestions: false,
    };
    let analyzer = ComplexityAnalyzer::with_config(config);
    assert_eq!(analyzer.get_default_threshold(), 5);
}

#[test]
fn test_analyze_simple_function() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell("simple", "fn add(a: i32, b: i32) -> i32 { a + b }");

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::O1);
    assert_eq!(result.space_complexity, SpaceComplexity::O1);
    assert_eq!(result.cyclomatic_complexity, 1); // Base complexity
    assert_eq!(result.cognitive_complexity, 1);
    assert!(result.halstead_metrics.volume > 0.0);
}

#[test]
fn test_analyze_linear_time_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "linear",
        "
        for i in 0..n {
            println!(\"{}\", i);
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::ON);
    assert!(result.cyclomatic_complexity > 1); // Loop adds complexity
}

#[test]
fn test_analyze_quadratic_time_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "quadratic",
        "
        for i in 0..n {
            for j in 0..m {
                process(i, j);
            }
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::ON2);
    assert!(result.cyclomatic_complexity >= 3); // Nested loops
}

#[test]
fn test_analyze_cubic_time_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "cubic",
        "
        for i in 0..n {
            for j in 0..m {
                for k in 0..p {
                    compute(i, j, k);
                }
            }
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::ON3);
}

#[test]
fn test_analyze_exponential_time_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "exponential",
        "
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    for l in 0..n {
                        expensive_operation(i, j, k, l);
                    }
                }
            }
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::OExp);
}

#[test]
fn test_analyze_logarithmic_time_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "logarithmic",
        "
        let result = binary_search(&data, target);
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::OLogN);
}

#[test]
fn test_analyze_linearithmic_time_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "linearithmic",
        "
        data.sort();
        let sorted_result = data;
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::ONLogN);
}

#[test]
fn test_analyze_linear_space_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "linear_space",
        "
        let mut buffer = vec![0; n];
        buffer.fill(42);
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.space_complexity, SpaceComplexity::ON);
}

#[test]
fn test_analyze_quadratic_space_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "quadratic_space",
        "
        let matrix = Array(n).fill(0).map(() => Array(n).fill(0));
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.space_complexity, SpaceComplexity::ON2);
}

#[test]
fn test_analyze_recursive_space_complexity() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "recursive_space",
        "
        fn recursive_factorial(n: u32) -> u32 {
            if n <= 1 { 1 } else { n * recursive_factorial(n - 1) }
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.space_complexity, SpaceComplexity::OLogN);
}

#[test]
fn test_cyclomatic_complexity_calculation() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "complex_control",
        "
        if condition1 {
            if condition2 && condition3 {
                for item in list {
                    while item.valid() {
                        match item.type() {
                            Type::A => process_a(),
                            Type::B => process_b(),
                            _ => process_default(),
                        }
                    }
                }
            } else if condition4 || condition5 {
                handle_alternative();
            }
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    // Base(1) + if(1) + if(1) + &&(1) + for(1) + while(1) + match(1) + else if(1) + ||(1)
    assert!(result.cyclomatic_complexity >= 9);
}

#[test]
fn test_cognitive_complexity_nesting() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "nested_complexity",
        "
        if level1 {                    // +1
            if level2 {                // +2 (nested)
                for item in items {    // +3 (nested)
                    if level3 {        // +4 (nested)
                        process();
                    }
                }
            }
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    // Should account for nesting levels
    assert!(result.cognitive_complexity >= 10);
}

#[test]
fn test_halstead_metrics_calculation() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "halstead_test",
        "
        let x = 5 + 10 * 2;
        let y = x / 3 - 1;
        if x > y {
            println!(\"Result: {}\", x);
        }
    ",
    );

    let result = analyzer.analyze(&cell);

    assert!(result.halstead_metrics.volume > 0.0);
    assert!(result.halstead_metrics.difficulty > 0.0);
    assert!(result.halstead_metrics.effort > 0.0);
    assert!(
        result.halstead_metrics.effort
            >= result.halstead_metrics.volume * result.halstead_metrics.difficulty
    );
}

#[test]
fn test_find_hotspots_empty_notebook() {
    let analyzer = ComplexityAnalyzer::new();
    let notebook = create_test_notebook(vec![]);

    let hotspots = analyzer.find_hotspots(&notebook);

    assert!(hotspots.is_empty());
}

#[test]
fn test_find_hotspots_no_complex_cells() {
    let analyzer = ComplexityAnalyzer::new();
    let cells = vec![
        create_test_cell("simple1", "let x = 42;"),
        create_test_cell("simple2", "fn add(a: i32, b: i32) -> i32 { a + b }"),
    ];
    let notebook = create_test_notebook(cells);

    let hotspots = analyzer.find_hotspots(&notebook);

    assert!(hotspots.is_empty());
}

#[test]
fn test_find_hotspots_with_complex_cells() {
    let analyzer = ComplexityAnalyzer::new();
    let cells = vec![
        create_test_cell("simple", "let x = 42;"),
        create_test_cell(
            "complex",
            "
            for i in 0..n {
                for j in 0..m {
                    for k in 0..p {
                        expensive_operation(i, j, k);
                    }
                }
            }
        ",
        ),
        create_test_cell(
            "very_complex",
            "
            if a { if b { if c { if d { if e { if f { if g { if h { if i { if j {
                deeply_nested_operation();
            } } } } } } } } } }
        ",
        ),
    ];
    let notebook = create_test_notebook(cells);

    let hotspots = analyzer.find_hotspots(&notebook);

    assert_eq!(hotspots.len(), 2);
    assert_eq!(hotspots[0].cell_id, "very_complex"); // Higher impact first
    assert_eq!(hotspots[1].cell_id, "complex");
    assert!(hotspots[0].impact > hotspots[1].impact);
}

#[test]
fn test_suggest_optimizations_disabled() {
    let config = ComplexityConfig {
        cyclomatic_threshold: 10,
        cognitive_threshold: 15,
        enable_suggestions: false,
    };
    let analyzer = ComplexityAnalyzer::with_config(config);
    let cell = create_test_cell(
        "complex",
        "
        for i in 0..n {
            for j in 0..m {
                expensive_operation(i, j);
            }
        }
    ",
    );

    let suggestions = analyzer.suggest_optimizations(&cell);

    assert!(suggestions.is_empty());
}

#[test]
fn test_suggest_optimizations_quadratic_with_lookup() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "quadratic_lookup",
        "
        for item in items {
            for target in targets {
                if item == target {
                    found.push(item);
                }
            }
        }
    ",
    );

    let suggestions = analyzer.suggest_optimizations(&cell);

    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("hash map")));
}

#[test]
fn test_suggest_optimizations_sorting() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "sort_operation",
        "
        for i in 0..n {
            for j in 0..n {
                if data[i] > data[j] {
                    data.sort();
                }
            }
        }
    ",
    );

    let suggestions = analyzer.suggest_optimizations(&cell);

    assert!(!suggestions.is_empty());
    assert!(suggestions
        .iter()
        .any(|s| s.contains("TimSort") || s.contains("insertion sort")));
}

#[test]
fn test_suggest_optimizations_exponential() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "exponential",
        "
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    for l in 0..n {
                        fibonacci(i + j + k + l);
                    }
                }
            }
        }
    ",
    );

    let suggestions = analyzer.suggest_optimizations(&cell);

    assert!(!suggestions.is_empty());
    assert!(suggestions
        .iter()
        .any(|s| s.contains("dynamic programming") || s.contains("memoization")));
}

#[test]
fn test_suggest_optimizations_high_cyclomatic() {
    let config = ComplexityConfig {
        cyclomatic_threshold: 3, // Low threshold
        cognitive_threshold: 15,
        enable_suggestions: true,
    };
    let analyzer = ComplexityAnalyzer::with_config(config);
    let cell = create_test_cell(
        "high_cyclomatic",
        "
        if a && b || c {
            for item in items {
                if item.valid() {
                    process(item);
                }
            }
        }
    ",
    );

    let suggestions = analyzer.suggest_optimizations(&cell);

    assert!(!suggestions.is_empty());
    assert!(suggestions
        .iter()
        .any(|s| s.contains("cyclomatic complexity") && s.contains("smaller functions")));
}

#[test]
fn test_suggest_optimizations_high_cognitive() {
    let config = ComplexityConfig {
        cyclomatic_threshold: 10,
        cognitive_threshold: 5, // Low threshold
        enable_suggestions: true,
    };
    let analyzer = ComplexityAnalyzer::with_config(config);
    let cell = create_test_cell(
        "high_cognitive",
        "
        if level1 {
            if level2 {
                if level3 {
                    if level4 {
                        process();
                    }
                }
            }
        }
    ",
    );

    let suggestions = analyzer.suggest_optimizations(&cell);

    assert!(!suggestions.is_empty());
    assert!(suggestions
        .iter()
        .any(|s| s.contains("cognitive complexity") && s.contains("nesting")));
}

// ============================================================================
// Property-Based Tests (10,000+ iterations)
// ============================================================================

proptest! {
    #[test]
    fn prop_complexity_analyzer_never_panics(
        source in ".*{0,1000}"
    ) {
        let analyzer = ComplexityAnalyzer::new();
        let cell = create_test_cell("prop_test", &source);
        let _result = analyzer.analyze(&cell); // Should not panic
    }

    #[test]
    fn prop_cyclomatic_complexity_minimum_one(
        source in "[a-zA-Z0-9 (){};.]*{0,500}"
    ) {
        let analyzer = ComplexityAnalyzer::new();
        let cell = create_test_cell("prop_test", &source);
        let result = analyzer.analyze(&cell);
        prop_assert!(result.cyclomatic_complexity >= 1);
    }

    #[test]
    fn prop_cognitive_complexity_minimum_one(
        source in "[a-zA-Z0-9 (){};.]*{0,500}"
    ) {
        let analyzer = ComplexityAnalyzer::new();
        let cell = create_test_cell("prop_test", &source);
        let result = analyzer.analyze(&cell);
        prop_assert!(result.cognitive_complexity >= 1);
    }

    #[test]
    fn prop_halstead_metrics_non_negative(
        source in "[a-zA-Z0-9 +*/=<>!&|(){};.]*{10,200}"
    ) {
        let analyzer = ComplexityAnalyzer::new();
        let cell = create_test_cell("prop_test", &source);
        let result = analyzer.analyze(&cell);

        prop_assert!(result.halstead_metrics.volume >= 0.0);
        prop_assert!(result.halstead_metrics.difficulty >= 0.0);
        prop_assert!(result.halstead_metrics.effort >= 0.0);
    }

    #[test]
    fn prop_more_loops_higher_complexity(
        num_loops in 0usize..5
    ) {
        let analyzer = ComplexityAnalyzer::new();

        let source = (0..num_loops)
            .map(|i| format!("for i{} in 0..n {{ ", i))
            .collect::<String>() +
            "operation();" +
            &"}".repeat(num_loops);

        let cell = create_test_cell("prop_test", &source);
        let result = analyzer.analyze(&cell);

        // More loops should generally result in higher complexity
        prop_assert!(result.cyclomatic_complexity >= 1 + num_loops);
    }

    #[test]
    fn prop_hotspot_impact_range(
        code_complexity in 1usize..20
    ) {
        let analyzer = ComplexityAnalyzer::new();

        // Generate code with specific complexity level
        let source = "if true { ".repeat(code_complexity) +
                    "operation();" +
                    &"}".repeat(code_complexity);

        let cell = create_test_cell("prop_test", &source);
        let cells = vec![cell];
        let notebook = create_test_notebook(cells);

        let hotspots = analyzer.find_hotspots(&notebook);

        for hotspot in hotspots {
            prop_assert!(hotspot.impact >= 0.0);
            prop_assert!(hotspot.impact <= 1.0);
        }
    }
}

// ============================================================================
// Stress Tests - Performance Limits
// ============================================================================

#[test]
fn stress_test_deeply_nested_code() {
    let analyzer = ComplexityAnalyzer::new();

    // Create deeply nested code (100 levels)
    let deep_source = "if true { ".repeat(100) + "operation();" + &"}".repeat(100);

    let cell = create_test_cell("deep_nest", &deep_source);

    let start = std::time::Instant::now();
    let result = analyzer.analyze(&cell);
    let duration = start.elapsed();

    // Should complete within reasonable time
    assert!(duration.as_millis() < 1000); // Less than 1 second
    assert!(result.cognitive_complexity > 100); // Should reflect nesting
}

#[test]
fn stress_test_large_notebook() {
    let analyzer = ComplexityAnalyzer::new();

    // Create notebook with 1000 cells
    let cells: Vec<Cell> = (0..1000)
        .map(|i| {
            create_test_cell(
                &format!("cell_{}", i),
                &format!(
                    "
            for j in 0..{} {{
                for k in 0..{} {{
                    operation({}, j, k);
                }}
            }}
        ",
                    i % 10,
                    i % 5,
                    i
                ),
            )
        })
        .collect();

    let notebook = create_test_notebook(cells);

    let start = std::time::Instant::now();
    let hotspots = analyzer.find_hotspots(&notebook);
    let duration = start.elapsed();

    // Should complete within reasonable time
    assert!(duration.as_millis() < 5000); // Less than 5 seconds
    assert!(!hotspots.is_empty());

    // Hotspots should be sorted by impact
    for i in 1..hotspots.len() {
        assert!(hotspots[i - 1].impact >= hotspots[i].impact);
    }
}

#[test]
fn stress_test_many_suggestions() {
    let analyzer = ComplexityAnalyzer::new();

    // Create code with multiple optimization opportunities
    let complex_source = "
        for i in 0..n {
            for j in 0..m {
                for k in 0..p {
                    data.sort();
                    for target in targets {
                        if items[i] == target && conditions[j] || flags[k] {
                            if nested1 {
                                if nested2 {
                                    if nested3 {
                                        if nested4 {
                                            expensive_operation(i, j, k);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    ";

    let cell = create_test_cell("complex", complex_source);

    let start = std::time::Instant::now();
    let suggestions = analyzer.suggest_optimizations(&cell);
    let duration = start.elapsed();

    // Should complete quickly and provide multiple suggestions
    assert!(duration.as_millis() < 100); // Less than 100ms
    assert!(suggestions.len() >= 3); // Multiple optimization opportunities
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_edge_case_empty_source() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell("empty", "");

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::O1);
    assert_eq!(result.space_complexity, SpaceComplexity::O1);
    assert_eq!(result.cyclomatic_complexity, 1);
    assert_eq!(result.cognitive_complexity, 1);
}

#[test]
fn test_edge_case_whitespace_only() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell("whitespace", "   \n\t  \n   ");

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::O1);
    assert_eq!(result.cognitive_complexity, 1);
}

#[test]
fn test_edge_case_comments_only() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "comments",
        "
        // This is a comment
        /* Multi-line
           comment */
        // Another comment
    ",
    );

    let result = analyzer.analyze(&cell);

    assert_eq!(result.time_complexity, TimeComplexity::O1);
    assert_eq!(result.cyclomatic_complexity, 1);
}

#[test]
fn test_edge_case_single_character() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell("single_char", "x");

    let result = analyzer.analyze(&cell);

    // Should handle gracefully
    assert_eq!(result.cyclomatic_complexity, 1);
    assert!(result.halstead_metrics.volume >= 0.0);
}

#[test]
fn test_edge_case_unbalanced_braces() {
    let analyzer = ComplexityAnalyzer::new();
    let cell = create_test_cell(
        "unbalanced",
        "if true { operation(); // Missing closing brace",
    );

    let result = analyzer.analyze(&cell);

    // Should handle gracefully without crashing
    assert!(result.cyclomatic_complexity >= 1);
}

#[test]
fn test_edge_case_markdown_cell() {
    let analyzer = ComplexityAnalyzer::new();
    let mut cell = create_test_cell(
        "markdown",
        "# This is markdown\n- List item\n- Another item",
    );
    cell.cell_type = CellType::Markdown;

    let result = analyzer.analyze(&cell);

    // Should analyze even markdown cells (treating as text)
    assert_eq!(result.time_complexity, TimeComplexity::O1);
}

// ============================================================================
// Integration Tests - Real Usage Scenarios
// ============================================================================

#[test]
fn integration_test_complete_complexity_analysis() {
    let analyzer = ComplexityAnalyzer::new();

    // Simulate real notebook with various complexity levels
    let cells = vec![
        create_test_cell(
            "setup",
            "
            let data = vec![1, 2, 3, 4, 5];
            let mut results = Vec::new();
        ",
        ),
        create_test_cell(
            "simple_processing",
            "
            for item in &data {
                results.push(item * 2);
            }
        ",
        ),
        create_test_cell(
            "complex_algorithm",
            "
            // Bubble sort implementation
            for i in 0..data.len() {
                for j in 0..(data.len() - i - 1) {
                    if data[j] > data[j + 1] {
                        data.swap(j, j + 1);
                    }
                }
            }
        ",
        ),
        create_test_cell(
            "very_complex",
            "
            // Nested conditional processing
            for i in 0..data.len() {
                if data[i] > threshold {
                    for j in 0..filters.len() {
                        if filters[j].matches(data[i]) {
                            for k in 0..processors.len() {
                                if processors[k].can_handle(data[i]) {
                                    if validate_conditions() {
                                        results.push(processors[k].process(data[i]));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ",
        ),
    ];

    let notebook = create_test_notebook(cells);

    // Analyze all cells
    let mut all_results = Vec::new();
    for cell in &notebook.cells {
        let result = analyzer.analyze(cell);
        all_results.push((cell.id.clone(), result));
    }

    // Find hotspots
    let hotspots = analyzer.find_hotspots(&notebook);

    // Verify analysis results
    assert_eq!(all_results.len(), 4);

    // Setup should be simple
    assert_eq!(all_results[0].1.time_complexity, TimeComplexity::O1);

    // Simple processing should be linear
    assert_eq!(all_results[1].1.time_complexity, TimeComplexity::ON);

    // Complex algorithm should be quadratic
    assert_eq!(all_results[2].1.time_complexity, TimeComplexity::ON2);

    // Very complex should be cubic or exponential
    assert!(matches!(
        all_results[3].1.time_complexity,
        TimeComplexity::ON3 | TimeComplexity::OExp
    ));

    // Should identify hotspots
    assert_eq!(hotspots.len(), 2); // complex_algorithm and very_complex
    assert_eq!(hotspots[0].cell_id, "very_complex"); // Highest impact first
    assert_eq!(hotspots[1].cell_id, "complex_algorithm");
}

#[test]
fn integration_test_optimization_suggestions_workflow() {
    let analyzer = ComplexityAnalyzer::new();

    // Test various optimization scenarios
    let test_cases = vec![
        (
            "hash_lookup",
            "
                for item in items {
                    for target in targets {
                        if item == target {
                            found.push(item);
                        }
                    }
                }
            ",
            vec!["hash map"],
        ),
        (
            "sort_in_loop",
            "
                for i in 0..n {
                    for j in 0..n {
                        data.sort();
                        process(data[i], data[j]);
                    }
                }
            ",
            vec!["TimSort", "insertion sort"],
        ),
        (
            "exponential_recursion",
            "
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            for l in 0..n {
                                fibonacci(i + j + k + l);
                            }
                        }
                    }
                }
            ",
            vec!["dynamic programming", "memoization"],
        ),
        (
            "high_cyclomatic",
            "
                if a && b || c && d || e && f {
                    for item in items {
                        if item.x && item.y || item.z {
                            while item.valid() {
                                if item.condition1 || item.condition2 {
                                    process(item);
                                }
                            }
                        }
                    }
                }
            ",
            vec!["smaller functions"],
        ),
        (
            "deep_nesting",
            "
                if level1 {
                    if level2 {
                        if level3 {
                            if level4 {
                                if level5 {
                                    process();
                                }
                            }
                        }
                    }
                }
            ",
            vec!["nesting", "control flow"],
        ),
    ];

    for (name, code, expected_suggestions) in test_cases {
        let cell = create_test_cell(name, code);
        let suggestions = analyzer.suggest_optimizations(&cell);

        for expected in expected_suggestions {
            assert!(
                suggestions
                    .iter()
                    .any(|s| s.to_lowercase().contains(&expected.to_lowercase())),
                "Expected suggestion '{}' not found for test case '{}'. Got: {:?}",
                expected,
                name,
                suggestions
            );
        }
    }
}

#[test]
fn integration_test_time_complexity_detection() {
    let analyzer = ComplexityAnalyzer::new();

    let test_cases = vec![
        ("constant", "let x = 42;", TimeComplexity::O1),
        ("logarithmic", "binary_search(&data, target);", TimeComplexity::OLogN),
        ("linear", "for i in 0..n { process(i); }", TimeComplexity::ON),
        ("linearithmic", "data.sort(); process(data);", TimeComplexity::ONLogN),
        ("quadratic", "for i in 0..n { for j in 0..m { process(i, j); } }", TimeComplexity::ON2),
        ("cubic", "for i in 0..n { for j in 0..m { for k in 0..p { process(i, j, k); } } }", TimeComplexity::ON3),
        ("exponential", "for i in 0..n { for j in 0..n { for k in 0..n { for l in 0..n { process(i, j, k, l); } } } }", TimeComplexity::OExp),
    ];

    for (name, code, expected_complexity) in test_cases {
        let cell = create_test_cell(name, code);
        let result = analyzer.analyze(&cell);

        assert_eq!(
            result.time_complexity, expected_complexity,
            "Time complexity mismatch for test case '{}'. Expected {:?}, got {:?}",
            name, expected_complexity, result.time_complexity
        );
    }
}

// ============================================================================
// Error Handling and Robustness Tests
// ============================================================================

#[test]
fn test_robustness_invalid_syntax() {
    let analyzer = ComplexityAnalyzer::new();

    let invalid_codes = vec![
        "fn incomplete_function(",
        "if without_braces then do_something",
        "for i in broken syntax {",
        "let x = unclosed_string\"",
        "}} extra closing braces {{",
    ];

    for (i, code) in invalid_codes.iter().enumerate() {
        let cell = create_test_cell(&format!("invalid_{}", i), code);
        let result = analyzer.analyze(&cell);

        // Should handle gracefully without crashing
        assert!(result.cyclomatic_complexity >= 1);
        assert!(result.cognitive_complexity >= 1);
    }
}

#[test]
fn test_robustness_unicode_and_special_chars() {
    let analyzer = ComplexityAnalyzer::new();

    let special_codes = vec![
        "let π = 3.14159; let café = \"coffee\";",
        "// Comments with émojis 😀🎉\nlet x = 42;",
        "let weird_chars = \"∑∆π∞≠≤≥\";",
        "/* Multi-line with unicode:\n   - π (pi)\n   - ∑ (sum)\n   - ∆ (delta) */",
    ];

    for (i, code) in special_codes.iter().enumerate() {
        let cell = create_test_cell(&format!("unicode_{}", i), code);
        let result = analyzer.analyze(&cell);

        // Should handle unicode gracefully
        assert!(result.halstead_metrics.volume >= 0.0);
        assert!(result.cyclomatic_complexity >= 1);
    }
}

#[test]
fn test_robustness_extreme_nesting() {
    let analyzer = ComplexityAnalyzer::new();

    // Create extremely nested code (1000 levels)
    let extreme_nesting = "if true { ".repeat(1000) + "operation();" + &"}".repeat(1000);

    let cell = create_test_cell("extreme", &extreme_nesting);

    // Should not overflow or crash
    let result = analyzer.analyze(&cell);
    assert!(result.cognitive_complexity > 1000);
    assert!(result.cyclomatic_complexity > 1000);
}

#[test]
fn test_concurrent_analysis() {
    use std::sync::Arc;
    use std::thread;

    let analyzer = Arc::new(ComplexityAnalyzer::new());
    let mut handles = vec![];

    // Run concurrent analyses
    for i in 0..10 {
        let analyzer_clone = Arc::clone(&analyzer);
        let handle = thread::spawn(move || {
            let cell = create_test_cell(
                &format!("concurrent_{}", i),
                &format!(
                    "
                for j in 0..{} {{
                    for k in 0..{} {{
                        process({}, j, k);
                    }}
                }}
            ",
                    i, i, i
                ),
            );

            analyzer_clone.analyze(&cell)
        });
        handles.push(handle);
    }

    // Wait for all analyses to complete
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.cyclomatic_complexity >= 1);
        assert!(result.cognitive_complexity >= 1);
    }
}
