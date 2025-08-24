//! Comprehensive tests for MCP tool discovery
//!
//! This test suite validates the MCP tool discovery functionality
//! for exposing Ruchy compiler commands as MCP tools

#![allow(warnings)]  // Allow all warnings for test files

use ruchy::mcp::RuchyToolDiscovery;
use serde_json::{json, Value};

/// Test tool discovery service creation
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
        assert!(discovery.get_tool(tool_name).is_some(), 
                "Expected tool '{}' to be registered", tool_name);
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
        assert!(matches!(category, 
            "parsing" | "transpilation" | "execution" | "quality" | "analysis"));
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
            let alias_strs: Vec<&str> = aliases.iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            assert!(alias_strs.contains(&"parse"));
        }
        
        if name == "ruchy_eval" {
            assert!(aliases.len() > 0);
            let alias_strs: Vec<&str> = aliases.iter()
                .map(|v| v.as_str().unwrap())
                .collect();
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
    
    // Test lint tool
    let lint_tool = discovery.get_tool("ruchy_lint").unwrap();
    assert!(lint_tool.description().contains("Lint"));
    assert!(lint_tool.description().contains("style"));
    
    // Test format tool
    let fmt_tool = discovery.get_tool("ruchy_fmt").unwrap();
    assert!(fmt_tool.description().contains("Format"));
    
    // Test score tool
    let score_tool = discovery.get_tool("ruchy_score").unwrap();
    assert!(score_tool.description().contains("quality"));
    assert!(score_tool.description().contains("score"));
}

/// Test analysis tools structure
#[test]
fn test_analysis_tools_structure() {
    let discovery = RuchyToolDiscovery::new();
    
    // Test provability tool
    let prove_tool = discovery.get_tool("ruchy_provability").unwrap();
    assert!(prove_tool.description().contains("verification"));
    assert!(prove_tool.description().contains("correctness"));
    
    // Test runtime tool
    let runtime_tool = discovery.get_tool("ruchy_runtime").unwrap();
    assert!(runtime_tool.description().contains("Performance"));
    assert!(runtime_tool.description().contains("BigO"));
    
    // Test optimize tool
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
    
    // Test overall structure
    assert_eq!(info.get("discovery_service").unwrap().as_str().unwrap(), "ruchy_tool_discovery");
    assert_eq!(info.get("version").unwrap().as_str().unwrap(), "1.0.0");
    
    // Test tools structure
    let tools = info.get("tools").unwrap().as_array().unwrap();
    for tool in tools {
        assert!(tool.get("name").is_some());
        assert!(tool.get("description").is_some());
        assert!(tool.get("category").is_some());
        assert!(tool.get("aliases").is_some());
    }
    
    // Test categories
    let categories = info.get("categories").unwrap().as_array().unwrap();
    let expected_categories = vec!["parsing", "transpilation", "execution", "quality", "analysis"];
    
    for expected_cat in expected_categories {
        let found = categories.iter().any(|c| c.as_str().unwrap() == expected_cat);
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
    
    // Test existing tool
    assert!(discovery.get_tool("ruchy_parse").is_some());
    assert!(discovery.get_tool("ruchy_eval").is_some());
    
    // Test non-existing tool
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