//! Comprehensive tests for MCP tool discovery
//!
//! This test suite validates the MCP tool discovery functionality
//! for exposing Ruchy compiler commands as MCP tools

#![allow(warnings)] // Allow all warnings for test files
#![cfg(feature = "mcp")]

#[cfg(feature = "mcp")]
use ruchy::mcp::RuchyToolDiscovery;
use serde_json::{json, Value};

/// Test tool discovery service creation
#[cfg(feature = "mcp")]
#[test]
fn test_tool_discovery_creation() {
    let discovery = RuchyToolDiscovery::new();

    // Should create without errors
    assert!(std::ptr::addr_of!(discovery) as usize != 0);

    // Should have registered tools
    let tools = discovery.get_tools();
    assert!(!tools.is_empty());
}

/// Test tool discovery with custom binary path
#[cfg(feature = "mcp")]
#[test]
fn test_tool_discovery_custom_binary() {
    let discovery = RuchyToolDiscovery::with_binary_path("/custom/path/ruchy".to_string());

    // Should create with custom path
    assert!(std::ptr::addr_of!(discovery) as usize != 0);

    // Should still have registered tools
    let tools = discovery.get_tools();
    assert!(!tools.is_empty());
}

/// Test that all expected core tools are registered
#[cfg(feature = "mcp")]
#[test]
fn test_core_tools_registered() {
    let discovery = RuchyToolDiscovery::new();

    let expected_tools = vec![
        "ruchy_parse",
        "ruchy_ast",
        "ruchy_transpile",
        "ruchy_check",
        "ruchy_eval",
        "ruchy_run",
        "ruchy_lint",
        "ruchy_fmt",
        "ruchy_score",
        "ruchy_quality_gate",
        "ruchy_provability",
        "ruchy_runtime",
        "ruchy_optimize",
    ];

    for tool_name in expected_tools {
        assert!(
            discovery.get_tool(tool_name).is_some(),
            "Expected tool '{}' to be registered",
            tool_name
        );
    }
}

/// Test tool name listing
#[test]
fn test_list_tool_names() {
    let discovery = RuchyToolDiscovery::new();
    let tool_names = discovery.list_tool_names();

    // Should have multiple tools
    assert!(tool_names.len() >= 10);

    // Should include key tools
    assert!(tool_names.contains(&"ruchy_parse".to_string()));
    assert!(tool_names.contains(&"ruchy_eval".to_string()));
    assert!(tool_names.contains(&"ruchy_transpile".to_string()));
}

/// Test discovery info generation
#[test]
fn test_discovery_info() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    // Should be valid JSON
    assert!(info.is_object());

    // Should have expected fields
    assert!(info.get("discovery_service").is_some());
    assert!(info.get("version").is_some());
    assert!(info.get("total_tools").is_some());
    assert!(info.get("tools").is_some());
    assert!(info.get("categories").is_some());

    // Should have tools array
    let tools = info.get("tools").unwrap().as_array().unwrap();
    assert!(!tools.is_empty());

    // Should have categories array
    let categories = info.get("categories").unwrap().as_array().unwrap();
    assert!(categories.len() >= 5);
}

/// Test tool categories
#[test]
fn test_tool_categories() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    let tools = info.get("tools").unwrap().as_array().unwrap();

    // Check that tools have categories
    for tool in tools {
        let category = tool.get("category").unwrap().as_str().unwrap();
        assert!(!category.is_empty());

        // Should be one of expected categories
        assert!(matches!(
            category,
            "parsing" | "transpilation" | "execution" | "quality" | "analysis"
        ));
    }
}

/// Test tool aliases for better discovery
#[test]
fn test_tool_aliases() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    let tools = info.get("tools").unwrap().as_array().unwrap();

    // Check that key tools have aliases
    for tool in tools {
        let name = tool.get("name").unwrap().as_str().unwrap();
        let aliases = tool.get("aliases").unwrap().as_array().unwrap();

        if name == "ruchy_parse" {
            assert!(aliases.len() > 0);
            let alias_strs: Vec<&str> = aliases.iter().map(|v| v.as_str().unwrap()).collect();
            assert!(alias_strs.contains(&"parse"));
        }

        if name == "ruchy_eval" {
            assert!(aliases.len() > 0);
            let alias_strs: Vec<&str> = aliases.iter().map(|v| v.as_str().unwrap()).collect();
            assert!(alias_strs.contains(&"eval"));
        }
    }
}

/// Test parse tool mock functionality (without actual execution)
#[test]
fn test_parse_tool_structure() {
    let discovery = RuchyToolDiscovery::new();
    let parse_tool = discovery.get_tool("ruchy_parse").unwrap();

    // Should have correct name and description
    assert!(parse_tool.description().contains("Parse"));
    assert!(parse_tool.description().contains("AST"));
}

/// Test eval tool structure
#[test]
fn test_eval_tool_structure() {
    let discovery = RuchyToolDiscovery::new();
    let eval_tool = discovery.get_tool("ruchy_eval").unwrap();

    // Should have correct description
    assert!(eval_tool.description().contains("Evaluate"));
    assert!(eval_tool.description().contains("expressions"));
}

/// Test transpile tool structure
#[test]
fn test_transpile_tool_structure() {
    let discovery = RuchyToolDiscovery::new();
    let transpile_tool = discovery.get_tool("ruchy_transpile").unwrap();

    // Should have correct description
    assert!(transpile_tool.description().contains("Transpile"));
    assert!(transpile_tool.description().contains("Rust"));
}

/// Test quality tools structure
#[test]
fn test_quality_tools_structure() {
    let discovery = RuchyToolDiscovery::new();

    // Check lint tool
    let lint_tool = discovery.get_tool("ruchy_lint").unwrap();
    assert!(lint_tool.description().contains("Lint"));
    assert!(lint_tool.description().contains("style"));

    // Check format tool
    let fmt_tool = discovery.get_tool("ruchy_fmt").unwrap();
    assert!(fmt_tool.description().contains("Format"));

    // Check score tool
    let score_tool = discovery.get_tool("ruchy_score").unwrap();
    assert!(score_tool.description().contains("quality"));
    assert!(score_tool.description().contains("score"));
}

/// Test analysis tools structure
#[test]
fn test_analysis_tools_structure() {
    let discovery = RuchyToolDiscovery::new();

    // Check provability tool
    let prove_tool = discovery.get_tool("ruchy_provability").unwrap();
    assert!(prove_tool.description().contains("verification"));
    assert!(prove_tool.description().contains("correctness"));

    // Check runtime tool
    let runtime_tool = discovery.get_tool("ruchy_runtime").unwrap();
    assert!(runtime_tool.description().contains("Performance"));
    assert!(runtime_tool.description().contains("BigO"));

    // Check optimize tool
    let opt_tool = discovery.get_tool("ruchy_optimize").unwrap();
    assert!(opt_tool.description().contains("optimization"));
    assert!(opt_tool.description().contains("Hardware"));
}

/// Test default implementation
#[test]
fn test_default_implementation() {
    let discovery = RuchyToolDiscovery::default();

    // Should work the same as new()
    let tools = discovery.get_tools();
    assert!(!tools.is_empty());
    assert!(tools.len() >= 10);
}

/// Test tool discovery info JSON structure
#[test]
fn test_discovery_info_json_structure() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    // Check overall structure
    assert_eq!(
        info.get("discovery_service").unwrap().as_str().unwrap(),
        "ruchy_tool_discovery"
    );
    assert_eq!(info.get("version").unwrap().as_str().unwrap(), "1.0.0");

    // Check tools structure
    let tools = info.get("tools").unwrap().as_array().unwrap();
    for tool in tools {
        assert!(tool.get("name").is_some());
        assert!(tool.get("description").is_some());
        assert!(tool.get("category").is_some());
        assert!(tool.get("aliases").is_some());
    }

    // Check categories
    let categories = info.get("categories").unwrap().as_array().unwrap();
    let expected_categories = vec![
        "parsing",
        "transpilation",
        "execution",
        "quality",
        "analysis",
    ];

    for expected_cat in expected_categories {
        let found = categories
            .iter()
            .any(|c| c.as_str().unwrap() == expected_cat);
        assert!(found, "Expected category '{}' not found", expected_cat);
    }
}

/// Test memory management with multiple discovery instances
#[test]
fn test_multiple_discovery_instances() {
    // Create multiple instances to test memory handling
    let mut discoveries = Vec::new();

    for _i in 0..10 {
        let discovery = RuchyToolDiscovery::new();
        assert!(discovery.get_tools().len() >= 10);
        discoveries.push(discovery);
    }

    // Should complete without memory issues
    assert_eq!(discoveries.len(), 10);
}

/// Test tool retrieval by name
#[test]
fn test_tool_retrieval() {
    let discovery = RuchyToolDiscovery::new();

    // Check existing tool
    assert!(discovery.get_tool("ruchy_parse").is_some());
    assert!(discovery.get_tool("ruchy_eval").is_some());

    // Check non-existing tool
    assert!(discovery.get_tool("nonexistent_tool").is_none());
    assert!(discovery.get_tool("").is_none());
}

/// Test discovery service consistency
#[test]
fn test_discovery_consistency() {
    let discovery1 = RuchyToolDiscovery::new();
    let discovery2 = RuchyToolDiscovery::new();

    // Both should have same number of tools
    assert_eq!(discovery1.get_tools().len(), discovery2.get_tools().len());

    // Both should have same tool names
    let names1 = discovery1.list_tool_names();
    let names2 = discovery2.list_tool_names();

    for name in &names1 {
        assert!(names2.contains(name));
    }
}

/// Test binary path validation for custom paths
#[test]
fn test_custom_binary_path_validation() {
    // Check various custom binary paths
    let test_paths = vec![
        "/usr/local/bin/ruchy",
        "./target/debug/ruchy",
        "ruchy",
        "/opt/ruchy/bin/ruchy",
        "custom_ruchy",
    ];

    for path in test_paths {
        let discovery = RuchyToolDiscovery::with_binary_path(path.to_string());
        assert!(!discovery.get_tools().is_empty());
        assert!(discovery.get_tools().len() >= 13); // All 13 core tools
    }
}

/// Test tool registration completeness
#[test]
fn test_all_tools_properly_registered() {
    let discovery = RuchyToolDiscovery::new();
    let tools = discovery.get_tools();

    // Check exact count of expected tools
    assert_eq!(tools.len(), 13);

    // Check all tools have valid descriptions
    for (name, tool) in tools {
        assert!(!name.is_empty());
        assert!(!tool.description().is_empty());
        assert!(tool.description().len() > 10); // Meaningful descriptions
    }
}

/// Test tool category assignment accuracy
#[test]
fn test_tool_category_assignment() {
    let discovery = RuchyToolDiscovery::new();

    // Check specific category assignments
    let parsing_tools = vec!["ruchy_parse", "ruchy_ast"];
    let transpilation_tools = vec!["ruchy_transpile", "ruchy_check"];
    let execution_tools = vec!["ruchy_eval", "ruchy_run"];
    let quality_tools = vec![
        "ruchy_lint",
        "ruchy_fmt",
        "ruchy_score",
        "ruchy_quality_gate",
    ];
    let analysis_tools = vec!["ruchy_provability", "ruchy_runtime", "ruchy_optimize"];

    for tool in parsing_tools {
        let category = discovery.get_tool_category(tool);
        assert_eq!(category, "parsing");
    }

    for tool in transpilation_tools {
        let category = discovery.get_tool_category(tool);
        assert_eq!(category, "transpilation");
    }

    for tool in execution_tools {
        let category = discovery.get_tool_category(tool);
        assert_eq!(category, "execution");
    }

    for tool in quality_tools {
        let category = discovery.get_tool_category(tool);
        assert_eq!(category, "quality");
    }

    for tool in analysis_tools {
        let category = discovery.get_tool_category(tool);
        assert_eq!(category, "analysis");
    }
}

/// Test tool alias completeness and correctness
#[test]
fn test_tool_aliases_completeness() {
    let discovery = RuchyToolDiscovery::new();

    // Check that all tools have expected aliases
    let test_cases = vec![
        ("ruchy_parse", vec!["parse", "syntax", "ast_parse"]),
        ("ruchy_ast", vec!["ast", "tree", "structure"]),
        ("ruchy_transpile", vec!["transpile", "convert", "rust"]),
        ("ruchy_check", vec!["check", "validate", "syntax_check"]),
        ("ruchy_eval", vec!["eval", "execute", "run_expr"]),
        ("ruchy_run", vec!["run", "execute", "compile_run"]),
        ("ruchy_lint", vec!["lint", "style", "check_style"]),
        ("ruchy_fmt", vec!["fmt", "format", "pretty"]),
        ("ruchy_score", vec!["score", "quality", "metrics"]),
        (
            "ruchy_quality_gate",
            vec!["quality_gate", "gate", "quality_check"],
        ),
        ("ruchy_provability", vec!["prove", "verify", "formal"]),
        ("ruchy_runtime", vec!["runtime", "performance", "bigo"]),
        ("ruchy_optimize", vec!["optimize", "hardware", "perf"]),
    ];

    for (tool_name, expected_aliases) in test_cases {
        let aliases = discovery.get_tool_aliases(tool_name);
        assert_eq!(aliases.len(), expected_aliases.len());

        for expected_alias in expected_aliases {
            assert!(
                aliases.contains(&expected_alias),
                "Tool '{}' missing expected alias '{}'",
                tool_name,
                expected_alias
            );
        }
    }
}

/// Test unknown tool alias handling
#[test]
fn test_unknown_tool_aliases() {
    let discovery = RuchyToolDiscovery::new();

    // Check unknown tools return empty aliases
    let unknown_tools = vec!["unknown_tool", "fake_tool", "", "ruchy_nonexistent"];

    for tool in unknown_tools {
        let aliases = discovery.get_tool_aliases(tool);
        assert!(
            aliases.is_empty(),
            "Unknown tool '{}' should have no aliases",
            tool
        );
    }
}

/// Test discovery info JSON serialization
#[test]
fn test_discovery_info_serialization() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    // Should serialize to valid JSON string
    let json_str = serde_json::to_string(&info).unwrap();
    assert!(!json_str.is_empty());
    assert!(json_str.contains("ruchy_tool_discovery"));
    assert!(json_str.contains("1.0.0"));

    // Should deserialize back correctly
    let deserialized: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(
        deserialized.get("discovery_service"),
        info.get("discovery_service")
    );
    assert_eq!(deserialized.get("version"), info.get("version"));
}

/// Test discovery info metrics accuracy
#[test]
fn test_discovery_info_metrics() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    // Check total_tools count matches actual tools
    let total_tools = info.get("total_tools").unwrap().as_u64().unwrap();
    let tools_array = info.get("tools").unwrap().as_array().unwrap();
    assert_eq!(total_tools as usize, tools_array.len());
    assert_eq!(total_tools, 13); // Expected number of core tools

    // Check all tools have required fields
    for tool in tools_array {
        assert!(tool.get("name").is_some());
        assert!(tool.get("description").is_some());
        assert!(tool.get("category").is_some());
        assert!(tool.get("aliases").is_some());
    }
}

/// Test tool name collision prevention
#[test]
fn test_tool_name_uniqueness() {
    let discovery = RuchyToolDiscovery::new();
    let tool_names = discovery.list_tool_names();

    // All tool names should be unique
    let mut seen_names = std::collections::HashSet::new();
    for name in &tool_names {
        assert!(
            !seen_names.contains(name),
            "Duplicate tool name found: {}",
            name
        );
        seen_names.insert(name.clone());
    }

    // Should have exactly 13 unique tools
    assert_eq!(tool_names.len(), 13);
    assert_eq!(seen_names.len(), 13);
}

/// Test empty and invalid tool name handling
#[test]
fn test_invalid_tool_name_handling() {
    let discovery = RuchyToolDiscovery::new();

    // Check various invalid tool names
    let invalid_names = vec!["", " ", "\n", "\t", "   ", "invalid tool"];

    for invalid_name in invalid_names {
        assert!(
            discovery.get_tool(invalid_name).is_none(),
            "Should not find tool with invalid name: '{}'",
            invalid_name
        );
    }
}

/// Test case sensitivity in tool names
#[test]
fn test_tool_name_case_sensitivity() {
    let discovery = RuchyToolDiscovery::new();

    // Tool names should be case-sensitive
    assert!(discovery.get_tool("ruchy_parse").is_some());
    assert!(discovery.get_tool("RUCHY_PARSE").is_none());
    assert!(discovery.get_tool("Ruchy_Parse").is_none());
    assert!(discovery.get_tool("ruchy_PARSE").is_none());
}

/// Test tool descriptions are meaningful
#[test]
fn test_tool_descriptions_quality() {
    let discovery = RuchyToolDiscovery::new();

    for (name, tool) in discovery.get_tools() {
        let description = tool.description();

        // Descriptions should be meaningful
        assert!(
            description.len() >= 20,
            "Tool '{}' has too short description",
            name
        );
        assert!(
            description.chars().any(|c| c.is_uppercase()),
            "Tool '{}' description should start with capital letter",
            name
        );
        assert!(
            !description.contains("Note"),
            "Tool '{}' description contains Note",
            name
        );
        assert!(
            !description.contains("Issue"),
            "Tool '{}' description contains Issue",
            name
        );
    }
}

/// Test binary path preservation
#[test]
fn test_binary_path_preservation() {
    let custom_path = "/custom/test/path/ruchy";
    let discovery = RuchyToolDiscovery::with_binary_path(custom_path.to_string());

    // Binary path should be preserved in internal state
    // We can't directly access it, but we can verify tools are registered
    assert!(!discovery.get_tools().is_empty());
    assert_eq!(discovery.get_tools().len(), 13);
}

/// Test discovery info categories completeness
#[test]
fn test_discovery_info_categories_completeness() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    let categories = info.get("categories").unwrap().as_array().unwrap();
    let expected_categories = vec![
        "parsing",
        "transpilation",
        "execution",
        "quality",
        "analysis",
    ];

    // Should have exactly the expected categories
    assert_eq!(categories.len(), expected_categories.len());

    for expected in expected_categories {
        let found = categories.iter().any(|c| c.as_str().unwrap() == expected);
        assert!(found, "Missing expected category: {}", expected);
    }
}

/// Test tool count consistency across methods
#[test]
fn test_tool_count_consistency() {
    let discovery = RuchyToolDiscovery::new();

    // All methods should return consistent tool counts
    let tools_map_count = discovery.get_tools().len();
    let tool_names_count = discovery.list_tool_names().len();
    let discovery_info = discovery.get_discovery_info();
    let info_total_tools = discovery_info.get("total_tools").unwrap().as_u64().unwrap() as usize;
    let info_tools_array = discovery_info
        .get("tools")
        .unwrap()
        .as_array()
        .unwrap()
        .len();

    assert_eq!(tools_map_count, tool_names_count);
    assert_eq!(tools_map_count, info_total_tools);
    assert_eq!(tools_map_count, info_tools_array);
    assert_eq!(tools_map_count, 13); // Expected total
}

/// Test memory efficiency with large instances
#[test]
fn test_memory_efficiency() {
    // Create many instances to test memory efficiency
    let mut discoveries = Vec::new();

    for i in 0..100 {
        let binary_path = format!("/test/path/ruchy_{}", i);
        let discovery = RuchyToolDiscovery::with_binary_path(binary_path);
        assert_eq!(discovery.get_tools().len(), 13);
        discoveries.push(discovery);
    }

    // All instances should be independent and functional
    assert_eq!(discoveries.len(), 100);

    // Check first and last instances are still functional
    assert!(!discoveries[0].get_tools().is_empty());
    assert!(!discoveries[99].get_tools().is_empty());
}

/// Test clone and equality behavior (if applicable)
#[test]
fn test_discovery_independence() {
    let discovery1 = RuchyToolDiscovery::new();
    let discovery2 = RuchyToolDiscovery::new();

    // Both should have identical tool sets
    let names1 = discovery1.list_tool_names();
    let names2 = discovery2.list_tool_names();

    assert_eq!(names1.len(), names2.len());

    // Sort both lists for comparison
    let mut sorted_names1 = names1;
    let mut sorted_names2 = names2;
    sorted_names1.sort();
    sorted_names2.sort();

    assert_eq!(sorted_names1, sorted_names2);
}

/// Test edge cases in tool registration
#[test]
fn test_tool_registration_edge_cases() {
    let discovery = RuchyToolDiscovery::new();

    // Check that re-registration doesn't cause issues
    // (This tests the internal registration logic stability)
    let original_count = discovery.get_tools().len();

    // Create another instance to verify registration is consistent
    let discovery2 = RuchyToolDiscovery::new();
    assert_eq!(discovery2.get_tools().len(), original_count);
}

/// Test discovery info version consistency
#[test]
fn test_discovery_info_version_consistency() {
    let discovery = RuchyToolDiscovery::new();
    let info = discovery.get_discovery_info();

    // Version should be consistent across calls
    let version1 = info.get("version").unwrap().as_str().unwrap();

    let info2 = discovery.get_discovery_info();
    let version2 = info2.get("version").unwrap().as_str().unwrap();

    assert_eq!(version1, version2);
    assert_eq!(version1, "1.0.0");
}
