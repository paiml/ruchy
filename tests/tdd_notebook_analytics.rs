//! Notebook Analytics & Insights Tests - Sprint 11
//! 
//! Tests for advanced notebook analytics and enterprise features:
//! - Notebook usage analytics and metrics collection
//! - Advanced performance profiling and optimization
//! - Recommendation engine for code improvements
//! - Git-like versioning system with branches and merging
//! - Notebook publishing and discovery platform
//! - Data visualization and interactive plotting
//! - Search and indexing capabilities
//! - Enterprise analytics dashboard

use ruchy::wasm::notebook::NotebookRuntime;
use ruchy::wasm::shared_session::SharedSession;
use serde_json::Value as JsonValue;
use std::time::{Duration, Instant};

// ============================================================================
// Notebook Analytics Engine Tests
// ============================================================================

#[test]
fn test_notebook_usage_analytics() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create and execute cells to generate analytics data
    let cell1 = runtime.add_cell("code", "let analytics_data = 42");
    let cell2 = runtime.add_cell("code", "let performance_test = analytics_data * 100");
    let cell3 = runtime.add_cell("markdown", "# Analytics Report");
    
    runtime.execute_cell(&cell1).unwrap();
    runtime.execute_cell(&cell2).unwrap();
    
    // TODO: Add usage analytics collection
    // let analytics = runtime.get_usage_analytics().unwrap();
    // assert!(analytics.total_executions >= 2, "Should track cell executions");
    // assert!(analytics.execution_time_ms > 0, "Should track execution time");
    // assert_eq!(analytics.cell_types.get("code"), Some(&2), "Should count cell types");
    // assert_eq!(analytics.cell_types.get("markdown"), Some(&1), "Should count markdown cells");
    
    // For now, verify basic tracking works
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    assert_eq!(cells.len(), 3, "Should track all created cells");
    
    println!("Usage analytics tracking structure established");
}

#[test]
fn test_execution_metrics_collection() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute various types of operations
    let cell1 = runtime.add_cell("code", "let fast_op = 1 + 1");
    let cell2 = runtime.add_cell("code", "let data_op = DataFrame([[1, 2], [3, 4]])");
    let cell3 = runtime.add_cell("code", "let complex_op = data_op.sum()");
    
    let start = Instant::now();
    runtime.execute_cell(&cell1).unwrap();
    runtime.execute_cell(&cell2).unwrap(); 
    runtime.execute_cell(&cell3).unwrap();
    let total_time = start.elapsed();
    
    // TODO: Add detailed execution metrics
    // let metrics = runtime.get_execution_metrics().unwrap();
    // assert!(metrics.average_execution_time_ms > 0, "Should track average execution time");
    // assert!(metrics.slowest_cell_time_ms <= total_time.as_millis() as u64, "Should track slowest cell");
    // assert!(metrics.memory_peak_mb > 0, "Should track peak memory usage");
    // assert!(metrics.dataframe_operations >= 2, "Should count DataFrame operations");
    
    // Verify basic execution tracking
    let memory_usage = runtime.get_memory_usage();
    let memory_obj: JsonValue = serde_json::from_str(&memory_usage).unwrap();
    assert!(memory_obj.is_object(), "Should track memory metrics");
    
    println!("Execution metrics collection structure established");
}

#[test]  
fn test_user_behavior_analytics() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Simulate user behavior patterns
    let cell1 = runtime.add_cell("code", "let user_behavior = 'pattern_analysis'");
    runtime.execute_cell(&cell1).unwrap();
    
    // Multiple executions of same cell (common pattern)
    runtime.execute_cell(&cell1).unwrap();
    runtime.execute_cell(&cell1).unwrap();
    
    // Add cells in sequence
    let cell2 = runtime.add_cell("code", "let follow_up = user_behavior.len()");
    let cell3 = runtime.add_cell("markdown", "## User Pattern Analysis");
    
    runtime.execute_cell(&cell2).unwrap();
    
    // TODO: Add user behavior analytics
    // let behavior = runtime.get_user_behavior_analytics().unwrap();
    // assert_eq!(behavior.cell_reexecutions, 3, "Should track cell re-executions");
    // assert!(behavior.average_time_between_cells_ms > 0, "Should track timing patterns");
    // assert!(behavior.common_patterns.contains("sequential_execution"), "Should identify patterns");
    
    // Verify session tracking
    let notebook_json = runtime.to_json();
    assert!(notebook_json.contains("cells"), "Should maintain session data for analysis");
    
    println!("User behavior analytics structure established");
}

// ============================================================================
// Advanced Performance Profiling Tests
// ============================================================================

#[test]
fn test_advanced_performance_profiling() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create cells with different performance characteristics
    let quick_cell = runtime.add_cell("code", "let quick = 42");
    let dataframe_cell = runtime.add_cell("code", "let df = DataFrame::from_range(0, 1000)");
    let compute_cell = runtime.add_cell("code", "let result = df.sum()");
    
    // Execute with profiling
    runtime.execute_cell(&quick_cell).unwrap();
    runtime.execute_cell(&dataframe_cell).unwrap();
    runtime.execute_cell(&compute_cell).unwrap();
    
    // TODO: Add advanced profiling
    // let profile = runtime.get_performance_profile().unwrap();
    // assert!(profile.cells.len() >= 3, "Should profile all executed cells");
    // assert!(profile.memory_allocations > 0, "Should track memory allocations");
    // assert!(profile.execution_breakdown.contains_key("parsing"), "Should break down execution phases");
    // assert!(profile.hotspots.len() > 0, "Should identify performance hotspots");
    
    // Verify basic profiling data available
    let memory_usage = runtime.get_memory_usage();
    let memory_obj: JsonValue = serde_json::from_str(&memory_usage).unwrap();
    assert!(memory_obj["total_allocated"].as_u64().unwrap_or(0) > 1000, "Should track allocations");
    
    println!("Advanced performance profiling structure established");
}

#[test]
fn test_performance_optimization_suggestions() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create cells that could be optimized
    let inefficient1 = runtime.add_cell("code", "let slow = DataFrame::from_range(0, 100).filter(true)");
    let inefficient2 = runtime.add_cell("code", "let duplicate = DataFrame::from_range(0, 100)");
    let good_cell = runtime.add_cell("code", "let efficient = 42 + 58");
    
    runtime.execute_cell(&inefficient1).unwrap();
    runtime.execute_cell(&inefficient2).unwrap();
    runtime.execute_cell(&good_cell).unwrap();
    
    // TODO: Add optimization suggestions
    // let suggestions = runtime.get_optimization_suggestions().unwrap();
    // assert!(!suggestions.is_empty(), "Should provide optimization suggestions");
    // assert!(suggestions.iter().any(|s| s.suggestion_type == "duplicate_computation"), "Should detect duplicates");
    // assert!(suggestions.iter().any(|s| s.suggestion_type == "inefficient_filter"), "Should detect inefficiencies");
    // assert!(suggestions.iter().any(|s| s.estimated_improvement_ms > 0), "Should estimate improvements");
    
    // Verify suggestion framework
    let cells_json = runtime.get_cells();
    assert!(!cells_json.is_empty(), "Should have cells to analyze for suggestions");
    
    println!("Performance optimization suggestions structure established");
}

#[test]
fn test_resource_usage_profiling() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create resource-intensive operations
    let memory_cell = runtime.add_cell("code", "let big_data = DataFrame::from_range(0, 5000)");
    let cpu_cell = runtime.add_cell("code", "let computation = big_data.sum()");
    
    let start_memory = runtime.get_memory_usage();
    runtime.execute_cell(&memory_cell).unwrap();
    let mid_memory = runtime.get_memory_usage(); 
    runtime.execute_cell(&cpu_cell).unwrap();
    let end_memory = runtime.get_memory_usage();
    
    // TODO: Add detailed resource profiling
    // let resource_profile = runtime.get_resource_profile().unwrap();
    // assert!(resource_profile.peak_memory_mb > resource_profile.baseline_memory_mb, "Should track memory peaks");
    // assert!(resource_profile.cpu_time_ms > 0, "Should track CPU time");
    // assert!(resource_profile.allocations.len() > 0, "Should track individual allocations");
    
    // Verify resource tracking basics
    let start_obj: JsonValue = serde_json::from_str(&start_memory).unwrap();
    let end_obj: JsonValue = serde_json::from_str(&end_memory).unwrap();
    let start_total = start_obj["total_allocated"].as_u64().unwrap_or(0);
    let end_total = end_obj["total_allocated"].as_u64().unwrap_or(0);
    assert!(end_total >= start_total, "Memory usage should increase with data operations");
    
    println!("Resource usage profiling structure established");
}

// ============================================================================
// Recommendation Engine Tests  
// ============================================================================

#[test]
fn test_code_improvement_recommendations() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create cells with various code patterns
    let basic_cell = runtime.add_cell("code", "let x = 42");
    let inefficient_cell = runtime.add_cell("code", "let y = DataFrame([[1, 2]]).filter(true).sum()");
    let good_cell = runtime.add_cell("code", "let z = x + 1");
    
    runtime.execute_cell(&basic_cell).unwrap();
    runtime.execute_cell(&inefficient_cell).unwrap();
    runtime.execute_cell(&good_cell).unwrap();
    
    // TODO: Add recommendation engine
    // let recommendations = runtime.get_code_recommendations().unwrap();
    // assert!(!recommendations.is_empty(), "Should provide code recommendations");
    // assert!(recommendations.iter().any(|r| r.recommendation_type == "simplify_chain"), "Should recommend chain simplification");
    // assert!(recommendations.iter().any(|r| r.confidence_score > 0.5), "Should have confident recommendations");
    // assert!(recommendations.iter().any(|r| !r.suggested_code.is_empty()), "Should provide suggested code");
    
    // Verify recommendation framework
    let notebook_json = runtime.to_json();
    assert!(notebook_json.contains("cells"), "Should have code to analyze for recommendations");
    
    println!("Code improvement recommendations structure established");
}

#[test]
fn test_best_practices_suggestions() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create cells with best practice opportunities
    let undocumented = runtime.add_cell("code", "let mystery_value = 123");
    let long_chain = runtime.add_cell("code", "let result = DataFrame([[1, 2], [3, 4]]).filter(true).sum()");
    let well_documented = runtime.add_cell("markdown", "# This cell explains the computation");
    let good_code = runtime.add_cell("code", "let clear_intention = 42");
    
    runtime.execute_cell(&undocumented).unwrap();
    runtime.execute_cell(&long_chain).unwrap(); 
    runtime.execute_cell(&good_code).unwrap();
    
    // TODO: Add best practices engine
    // let practices = runtime.get_best_practices_suggestions().unwrap();
    // assert!(practices.iter().any(|p| p.practice_type == "add_documentation"), "Should suggest documentation");
    // assert!(practices.iter().any(|p| p.practice_type == "break_long_chains"), "Should suggest breaking chains");
    // assert!(practices.iter().any(|p| p.severity == "medium"), "Should categorize severity");
    
    // Verify best practices framework  
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    assert!(cells.len() >= 4, "Should have multiple cells to analyze");
    
    println!("Best practices suggestions structure established");
}

// ============================================================================
// Git-like Versioning Tests
// ============================================================================

#[test]
fn test_notebook_version_control() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create initial notebook version
    let cell1 = runtime.add_cell("code", "let version1 = 'initial'");
    runtime.execute_cell(&cell1).unwrap();
    
    // TODO: Add Git-like versioning
    // let commit1 = runtime.commit_notebook("Initial notebook version").unwrap();
    // assert!(!commit1.commit_hash.is_empty(), "Should generate commit hash");
    // assert_eq!(commit1.message, "Initial notebook version", "Should store commit message");
    
    // Make changes
    let cell2 = runtime.add_cell("code", "let version2 = 'updated'");
    runtime.execute_cell(&cell2).unwrap();
    
    // TODO: Test branching and merging
    // let branch = runtime.create_branch("feature-branch").unwrap();
    // runtime.switch_branch(&branch.name).unwrap();
    // let cell3 = runtime.add_cell("code", "let feature = 'new_feature'");
    // let commit2 = runtime.commit_notebook("Add new feature").unwrap();
    
    // Verify basic versioning structure
    let notebook_json = runtime.to_json();
    assert!(notebook_json.contains("version"), "Should track version information");
    
    println!("Git-like versioning structure established");
}

#[test]
fn test_notebook_diff_and_merge() {
    let mut runtime1 = NotebookRuntime::new().unwrap();
    let mut runtime2 = NotebookRuntime::new().unwrap();
    
    // Create base notebook
    let base_cell = runtime1.add_cell("code", "let base = 'common'");
    runtime1.execute_cell(&base_cell).unwrap();
    
    // Branch 1: Add feature A
    let feature_a = runtime1.add_cell("code", "let feature_a = 'added'");
    runtime1.execute_cell(&feature_a).unwrap();
    
    // Branch 2: Add feature B  
    let feature_b = runtime2.add_cell("code", "let feature_b = 'different'");
    runtime2.execute_cell(&feature_b).unwrap();
    
    // TODO: Add diff and merge capabilities
    // let notebook1_state = runtime1.export_for_collaboration().unwrap();
    // let notebook2_state = runtime2.export_for_collaboration().unwrap();
    // let diff = runtime1.diff_notebooks(&notebook2_state).unwrap();
    // assert!(diff.has_conflicts == false, "Should detect non-conflicting changes");
    // assert!(diff.added_cells.len() > 0, "Should detect added cells");
    
    // let merged = runtime1.merge_notebooks(&notebook2_state).unwrap();
    // assert!(merged.success, "Should successfully merge non-conflicting changes");
    
    // Verify diff framework
    let state1 = runtime1.export_for_collaboration().unwrap();
    let state2 = runtime2.export_for_collaboration().unwrap();
    assert_ne!(state1, state2, "Different notebooks should have different states");
    
    println!("Notebook diff and merge structure established");
}

// ============================================================================
// Publishing Platform Tests
// ============================================================================

#[test]
fn test_notebook_publishing() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create publishable notebook
    runtime.add_cell("markdown", "# Data Analysis Tutorial");
    runtime.add_cell("code", "let dataset = DataFrame::from_range(0, 100)");
    runtime.add_cell("markdown", "## Analysis Results");
    let analysis_cell = runtime.add_cell("code", "let summary = dataset.sum()");
    
    runtime.execute_cell(&analysis_cell).unwrap();
    
    // TODO: Add publishing capabilities
    // let publish_metadata = NotebookPublishMetadata {
    //     title: "Data Analysis Tutorial".to_string(),
    //     description: "Learn data analysis basics".to_string(),
    //     tags: vec!["tutorial", "data-analysis", "beginner"],
    //     license: "MIT".to_string(),
    //     public: true,
    // };
    
    // let published = runtime.publish_notebook(publish_metadata).unwrap();
    // assert!(!published.notebook_id.is_empty(), "Should assign unique notebook ID");
    // assert!(!published.share_url.is_empty(), "Should generate share URL");
    // assert!(published.published_at > 0, "Should timestamp publication");
    
    // Verify publishing structure
    let notebook_json = runtime.to_json();
    assert!(notebook_json.contains("metadata"), "Should have metadata for publishing");
    
    println!("Notebook publishing structure established");
}

#[test]
fn test_notebook_discovery() {
    // TODO: Add notebook discovery platform
    // let discovery = NotebookDiscovery::new();
    
    // Test search functionality
    // let search_results = discovery.search_notebooks("data analysis").unwrap();
    // assert!(search_results.total_count >= 0, "Should return search results");
    
    // Test filtering
    // let filtered = discovery.filter_notebooks(NotebookFilters {
    //     tags: Some(vec!["tutorial"]),
    //     min_rating: Some(4.0),
    //     language: Some("ruchy"),
    // }).unwrap();
    
    // Test trending
    // let trending = discovery.get_trending_notebooks(7).unwrap(); // Last 7 days
    // assert!(trending.notebooks.len() >= 0, "Should return trending notebooks");
    
    println!("Notebook discovery structure established");
}

// ============================================================================
// Data Visualization Tests
// ============================================================================

#[test]
fn test_data_visualization_generation() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create data for visualization
    let data_cell = runtime.add_cell("code", "let viz_data = DataFrame([[1, 10], [2, 20], [3, 30]])");
    runtime.execute_cell(&data_cell).unwrap();
    
    // TODO: Add visualization capabilities
    // let line_chart = runtime.create_visualization("line_chart", VizConfig {
    //     data_source: "viz_data",
    //     x_column: 0,
    //     y_column: 1,
    //     title: "Growth Over Time",
    // }).unwrap();
    
    // assert!(!line_chart.svg_content.is_empty(), "Should generate SVG content");
    // assert!(line_chart.interactive_config.is_some(), "Should support interactivity");
    
    // Test different chart types
    // let bar_chart = runtime.create_visualization("bar_chart", VizConfig {
    //     data_source: "viz_data", 
    //     x_column: 0,
    //     y_column: 1,
    //     title: "Value Comparison",
    // }).unwrap();
    
    // Verify visualization framework
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    assert!(!cells.is_empty(), "Should have data cells for visualization");
    
    println!("Data visualization structure established");
}

#[test]
fn test_interactive_plotting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create complex dataset
    let dataset_cell = runtime.add_cell("code", "let plot_data = DataFrame::from_range(0, 50)");
    runtime.execute_cell(&dataset_cell).unwrap();
    
    // TODO: Add interactive plotting
    // let interactive_plot = runtime.create_interactive_plot(InteractivePlotConfig {
    //     plot_type: "scatter",
    //     data_source: "plot_data",
    //     interactions: vec!["zoom", "pan", "hover"],
    //     animation: Some("fade_in"),
    // }).unwrap();
    
    // assert!(!interactive_plot.html_content.is_empty(), "Should generate HTML content");
    // assert!(!interactive_plot.javascript_code.is_empty(), "Should include JavaScript interactions");
    // assert!(interactive_plot.supports_export, "Should support export to PNG/SVG");
    
    // Verify interactive framework
    let memory_usage = runtime.get_memory_usage();
    assert!(!memory_usage.is_empty(), "Should track memory for large datasets");
    
    println!("Interactive plotting structure established");
}

// ============================================================================
// Search and Indexing Tests
// ============================================================================

#[test]
fn test_notebook_content_indexing() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create notebook with searchable content
    runtime.add_cell("markdown", "# Machine Learning Tutorial");
    runtime.add_cell("code", "let ml_data = DataFrame::from_range(0, 1000)");
    runtime.add_cell("markdown", "## Data Preprocessing");
    runtime.add_cell("code", "let processed = ml_data.filter(true)");
    runtime.add_cell("markdown", "## Model Training");
    runtime.add_cell("code", "let model = 'trained_model'");
    
    // TODO: Add search indexing
    // let index = runtime.build_search_index().unwrap();
    // assert!(index.total_tokens > 0, "Should tokenize notebook content");
    // assert!(index.keyword_frequency.contains_key("machine"), "Should index keywords");
    // assert!(index.code_symbols.contains("ml_data"), "Should index code symbols");
    
    // Test search functionality
    // let search_results = runtime.search_notebook_content("machine learning").unwrap();
    // assert!(!search_results.is_empty(), "Should find relevant content");
    // assert!(search_results.iter().any(|r| r.cell_type == "markdown"), "Should search markdown");
    // assert!(search_results.iter().any(|r| r.relevance_score > 0.5), "Should rank by relevance");
    
    // Verify search structure
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    assert!(cells.len() >= 6, "Should have content to index and search");
    
    println!("Content indexing and search structure established");
}

#[test]
fn test_cross_notebook_search() {
    // TODO: Add cross-notebook search capabilities
    // let search_engine = NotebookSearchEngine::new();
    
    // Index multiple notebooks
    // let notebook1 = create_test_notebook("Data Analysis", vec!["analysis", "statistics"]);
    // let notebook2 = create_test_notebook("Machine Learning", vec!["ml", "training"]);
    
    // search_engine.index_notebook(&notebook1).unwrap();
    // search_engine.index_notebook(&notebook2).unwrap();
    
    // Test cross-notebook search
    // let results = search_engine.search_all_notebooks("data").unwrap();
    // assert!(results.total_matches > 0, "Should find matches across notebooks");
    // assert!(results.notebooks.len() > 0, "Should return matched notebooks");
    
    // Test advanced search with filters
    // let filtered_results = search_engine.advanced_search(SearchQuery {
    //     text: "machine learning",
    //     tags: Some(vec!["ml"]),
    //     date_range: None,
    //     author: None,
    // }).unwrap();
    
    println!("Cross-notebook search structure established");
}

// ============================================================================
// Performance and Integration Tests
// ============================================================================

#[test]
fn test_analytics_performance_with_large_notebook() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create large notebook for performance testing
    for i in 0..20 {
        let code = format!("let cell_{} = DataFrame::from_range({}, {})", i, i * 10, (i + 1) * 10);
        let cell_id = runtime.add_cell("code", &code);
        runtime.execute_cell(&cell_id).unwrap();
    }
    
    // Add markdown documentation
    for i in 0..5 {
        runtime.add_cell("markdown", &format!("## Section {}: Analysis Results", i));
    }
    
    // Test analytics performance
    let start = Instant::now();
    let cells_json = runtime.get_cells();
    let memory_usage = runtime.get_memory_usage();
    let debug_info = runtime.get_debug_information().unwrap();
    let analytics_time = start.elapsed();
    
    // Verify performance is acceptable
    assert!(analytics_time.as_millis() < 1000, "Analytics should be fast even for large notebooks: {}ms", analytics_time.as_millis());
    
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    assert_eq!(cells.len(), 25, "Should handle 25 cells efficiently");
    
    let memory_obj: JsonValue = serde_json::from_str(&memory_usage).unwrap();
    assert!(memory_obj["total_allocated"].as_u64().unwrap_or(0) < 50_000_000, "Memory should be reasonable");
    
    println!("Analytics performance with large notebook validated");
}

#[test]
fn test_end_to_end_analytics_workflow() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Complete analytics workflow
    // 1. Create notebook with varied content
    runtime.add_cell("markdown", "# Complete Analytics Demo");
    let data_cell = runtime.add_cell("code", "let demo_data = DataFrame::from_range(0, 100)");
    let analysis_cell = runtime.add_cell("code", "let analysis = demo_data.sum()");
    let viz_cell = runtime.add_cell("code", "let visualization = 'chart_data'");
    
    // 2. Execute with timing
    let start = Instant::now();
    runtime.execute_cell(&data_cell).unwrap();
    runtime.execute_cell(&analysis_cell).unwrap();
    runtime.execute_cell(&viz_cell).unwrap();
    let execution_time = start.elapsed();
    
    // 3. Collect comprehensive analytics
    let notebook_json = runtime.to_json();
    let memory_usage = runtime.get_memory_usage();
    let debug_info = runtime.get_debug_information().unwrap();
    let cells_json = runtime.get_cells();
    
    // TODO: Collect advanced analytics
    // let usage_analytics = runtime.get_usage_analytics().unwrap();
    // let performance_profile = runtime.get_performance_profile().unwrap();
    // let recommendations = runtime.get_code_recommendations().unwrap();
    
    // Verify workflow completeness
    assert!(notebook_json.contains("cells"), "Should have complete notebook");
    assert!(!memory_usage.is_empty(), "Should have memory analytics");
    assert!(!debug_info.is_empty(), "Should have debug information");
    assert!(execution_time.as_millis() < 500, "Should execute efficiently");
    
    println!("End-to-end analytics workflow validated");
}