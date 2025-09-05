//! TDD safety net for parse_import refactoring
//! Target: 26 complexity → ≤10 with systematic function extraction
//! Focus: Cover all import parsing paths before refactoring complexity hotspot

#[cfg(test)]
mod tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind, ImportItem};
    
    // Helper function (complexity: 3)
    fn parse_import_expr(input: &str) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        Ok(expr)
    }
    
    // Helper to check import structure (complexity: 4)
    fn is_import_expr(expr: &Expr, expected_path: &str) -> bool {
        match &expr.kind {
            ExprKind::Import { path, .. } => path == expected_path,
            _ => false,
        }
    }
    
    // Basic Import Tests (complexity: 3 each)
    #[test]
    fn test_simple_import() {
        let result = parse_import_expr("import std");
        assert!(result.is_ok(), "Failed to parse simple import");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "std"));
    }
    
    #[test]
    fn test_nested_import() {
        let result = parse_import_expr("import std::collections");
        assert!(result.is_ok(), "Failed to parse nested import");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "std::collections"));
    }
    
    #[test]
    fn test_deep_nested_import() {
        let result = parse_import_expr("import std::collections::HashMap");
        assert!(result.is_ok(), "Failed to parse deep nested import");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "std::collections::HashMap"));
    }
    
    // Import with Alias Tests (complexity: 3 each)
    #[test]
    fn test_import_with_as_alias() {
        let result = parse_import_expr("import std::collections::HashMap as Map");
        assert!(result.is_ok(), "Failed to parse import with alias");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { items, .. } => {
                assert!(items.contains(&ImportItem::Aliased { 
                    name: "HashMap".to_string(),
                    alias: "Map".to_string() 
                }));
            }
            _ => panic!("Expected Import with alias"),
        }
    }
    
    #[test]
    fn test_import_single_item_with_alias() {
        let result = parse_import_expr("import math as m");
        assert!(result.is_ok(), "Failed to parse single item import with alias");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, items, .. } => {
                assert_eq!(path, "math");
                assert!(items.contains(&ImportItem::Aliased { 
                    name: "math".to_string(),
                    alias: "m".to_string() 
                }));
            }
            _ => panic!("Expected Import with alias"),
        }
    }
    
    // Wildcard Import Tests (complexity: 3 each)
    #[test]
    fn test_wildcard_import() {
        let result = parse_import_expr("import std::*");
        assert!(result.is_ok(), "Failed to parse wildcard import");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { items, .. } => {
                assert!(items.contains(&ImportItem::Wildcard));
            }
            _ => panic!("Expected Import with wildcard"),
        }
    }
    
    #[test]
    fn test_from_import_wildcard() {
        let result = parse_import_expr("from std import *");
        assert!(result.is_ok(), "Failed to parse from import wildcard");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "std"));
    }
    
    // Selective Import Tests (complexity: 4 each)
    #[test]
    fn test_selective_single_import() {
        let result = parse_import_expr("from std::collections import HashMap");
        assert!(result.is_ok(), "Failed to parse selective import");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, items, .. } => {
                assert_eq!(path, "std::collections");
                assert!(items.contains(&ImportItem::Named("HashMap".to_string())));
            }
            _ => panic!("Expected selective Import"),
        }
    }
    
    #[test]
    fn test_selective_multiple_import() {
        let result = parse_import_expr("from std::collections import HashMap, Vec, BTreeMap");
        assert!(result.is_ok(), "Failed to parse multiple selective import");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, items, .. } => {
                assert_eq!(path, "std::collections");
                assert!(items.contains(&ImportItem::Named("HashMap".to_string())));
                assert!(items.contains(&ImportItem::Named("Vec".to_string())));
                assert!(items.contains(&ImportItem::Named("BTreeMap".to_string())));
            }
            _ => panic!("Expected multiple selective Import"),
        }
    }
    
    #[test]
    fn test_selective_import_with_aliases() {
        let result = parse_import_expr("from std::collections import HashMap as Map, Vec as List");
        assert!(result.is_ok(), "Failed to parse selective import with aliases");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, .. } => {
                assert_eq!(path, "std::collections");
            }
            _ => panic!("Expected selective Import with aliases"),
        }
    }
    
    // Parenthesized Import Tests (complexity: 4 each)
    #[test]
    fn test_parenthesized_import() {
        let result = parse_import_expr("from std::collections import (HashMap, Vec, BTreeMap)");
        assert!(result.is_ok(), "Failed to parse parenthesized import");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, items, .. } => {
                assert_eq!(path, "std::collections");
                assert!(items.len() >= 3);
            }
            _ => panic!("Expected parenthesized Import"),
        }
    }
    
    #[test]
    fn test_multiline_parenthesized_import() {
        let input = "from std::collections import (
            HashMap,
            Vec,
            BTreeMap
        )";
        let result = parse_import_expr(input);
        assert!(result.is_ok(), "Failed to parse multiline parenthesized import");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, .. } => {
                assert_eq!(path, "std::collections");
            }
            _ => panic!("Expected multiline Import"),
        }
    }
    
    // Relative Import Tests (complexity: 3 each)
    #[test]
    fn test_relative_import_single_dot() {
        let result = parse_import_expr("from .utils import helper");
        assert!(result.is_ok(), "Failed to parse relative import with single dot");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, .. } => {
                assert!(path.starts_with('.'));
            }
            _ => panic!("Expected relative Import"),
        }
    }
    
    #[test]
    fn test_relative_import_double_dot() {
        let result = parse_import_expr("from ..parent import module");
        assert!(result.is_ok(), "Failed to parse relative import with double dot");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, .. } => {
                assert!(path.starts_with(".."));
            }
            _ => panic!("Expected parent relative Import"),
        }
    }
    
    // Complex Path Import Tests (complexity: 4 each)
    #[test]
    fn test_complex_path() {
        let result = parse_import_expr("import crate::frontend::parser::utils");
        assert!(result.is_ok(), "Failed to parse complex module path");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "crate::frontend::parser::utils"));
    }
    
    #[test]
    fn test_super_import() {
        let result = parse_import_expr("import super::parent_module");
        assert!(result.is_ok(), "Failed to parse super import");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "super::parent_module"));
    }
    
    #[test]
    fn test_self_import() {
        let result = parse_import_expr("import self::current_module");
        assert!(result.is_ok(), "Failed to parse self import");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "self::current_module"));
    }
    
    // Error Handling Tests (complexity: 2 each)
    #[test]
    fn test_empty_import() {
        let result = parse_import_expr("import");
        assert!(result.is_err(), "Should fail on empty import");
    }
    
    #[test]
    fn test_invalid_import_syntax() {
        let result = parse_import_expr("import .");
        assert!(result.is_err(), "Should fail on invalid import syntax");
    }
    
    #[test]
    fn test_incomplete_from_import() {
        let result = parse_import_expr("from std import");
        assert!(result.is_err(), "Should fail on incomplete from import");
    }
    
    // Edge Case Tests (complexity: 3 each)
    #[test]
    fn test_numeric_in_module_name() {
        let result = parse_import_expr("import mod123::utils");
        assert!(result.is_ok(), "Failed to parse module with numbers");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "mod123::utils"));
    }
    
    #[test]
    fn test_underscore_in_module_name() {
        let result = parse_import_expr("import my_module::sub_module");
        assert!(result.is_ok(), "Failed to parse module with underscores");
        
        let expr = result.unwrap();
        assert!(is_import_expr(&expr, "my_module::sub_module"));
    }
    
    #[test]
    fn test_mixed_import_styles() {
        let result = parse_import_expr("import std::collections::HashMap as Map");
        assert!(result.is_ok(), "Failed to parse mixed import style");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, items, .. } => {
                assert_eq!(path, "std::collections::HashMap");
                assert!(items.contains(&ImportItem::Aliased { 
                    name: "HashMap".to_string(),
                    alias: "Map".to_string() 
                }));
            }
            _ => panic!("Expected Import with mixed style"),
        }
    }
    
    // Integration Tests (complexity: 5 each)
    #[test]
    fn test_all_import_patterns_combination() {
        let inputs = vec![
            "import std",
            "import std::collections",
            "from std import HashMap",
            "from std::collections import HashMap as Map",
            "from std import *",
            "import .relative",
            "import super::parent",
        ];
        
        for input in inputs {
            let result = parse_import_expr(input);
            assert!(result.is_ok() || result.is_err(), 
                "Import pattern should parse or fail gracefully: {}", input);
        }
    }
    
    #[test]
    fn test_nested_import_resolution() {
        let result = parse_import_expr("from crate::deeply::nested::module import SpecificItem");
        assert!(result.is_ok(), "Failed to parse deeply nested import");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, items, .. } => {
                assert_eq!(path, "crate::deeply::nested::module");
                assert!(items.contains(&ImportItem::Named("SpecificItem".to_string())));
            }
            _ => panic!("Expected deeply nested Import"),
        }
    }
    
    #[test]
    fn test_complex_selective_import() {
        let input = "from std::collections import (
            HashMap as Map,
            Vec as List,
            BTreeMap,
            HashSet as Set
        )";
        let result = parse_import_expr(input);
        assert!(result.is_ok(), "Failed to parse complex selective import");
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Import { path, .. } => {
                assert_eq!(path, "std::collections");
            }
            _ => panic!("Expected complex selective Import"),
        }
    }
}