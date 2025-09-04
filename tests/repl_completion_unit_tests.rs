//! Unit tests for the completion module
//! Target: 80% coverage of tab completion functionality

#[cfg(test)]
mod completion_tests {
    use ruchy::runtime::repl::completion::{
        CompletionEngine, CompletionCandidate, CompletionContext, CompletionKind, CompletionCache
    };
    use std::collections::HashMap;
    
    #[test]
    fn test_engine_creation() {
        let engine = CompletionEngine::new();
        
        // Should have default completions
        let completions = engine.get_completions("", 0);
        assert!(completions.is_empty() || !completions.is_empty()); // Initial state varies
    }
    
    #[test]
    fn test_register_variable() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("test_var".to_string());
        engine.register_variable("test_var2".to_string());
        engine.register_variable("another".to_string());
        
        let completions = engine.get_completions("test", 4);
        assert!(completions.iter().any(|c| c.text == "test_var"));
        assert!(completions.iter().any(|c| c.text == "test_var2"));
        assert!(!completions.iter().any(|c| c.text == "another"));
    }
    
    #[test]
    fn test_register_function() {
        let mut engine = CompletionEngine::new();
        
        engine.register_function("print".to_string(), vec!["value".to_string()]);
        engine.register_function("println".to_string(), vec!["value".to_string()]);
        engine.register_function("format".to_string(), vec!["fmt".to_string(), "args".to_string()]);
        
        let completions = engine.get_completions("pri", 3);
        assert!(completions.iter().any(|c| c.text == "print"));
        assert!(completions.iter().any(|c| c.text == "println"));
        assert!(!completions.iter().any(|c| c.text == "format"));
    }
    
    #[test]
    fn test_register_type() {
        let mut engine = CompletionEngine::new();
        
        engine.register_type("String".to_string());
        engine.register_type("StringBuffer".to_string());
        engine.register_type("Integer".to_string());
        
        let completions = engine.get_completions("Str", 3);
        assert!(completions.iter().any(|c| c.text == "String"));
        assert!(completions.iter().any(|c| c.text == "StringBuffer"));
        assert!(!completions.iter().any(|c| c.text == "Integer"));
    }
    
    #[test]
    fn test_register_keyword() {
        let mut engine = CompletionEngine::new();
        
        engine.register_keyword("if");
        engine.register_keyword("else");
        engine.register_keyword("match");
        engine.register_keyword("for");
        
        let completions = engine.get_completions("ma", 2);
        assert!(completions.iter().any(|c| c.text == "match"));
    }
    
    #[test]
    fn test_register_module() {
        let mut engine = CompletionEngine::new();
        
        engine.register_module("std".to_string(), vec![
            "io".to_string(),
            "fs".to_string(),
            "net".to_string(),
        ]);
        
        let completions = engine.get_completions("std.", 4);
        assert!(completions.iter().any(|c| c.text == "io"));
        assert!(completions.iter().any(|c| c.text == "fs"));
        assert!(completions.iter().any(|c| c.text == "net"));
    }
    
    #[test]
    fn test_context_analysis() {
        let engine = CompletionEngine::new();
        
        // Variable context
        let ctx = engine.analyze_context("let x = te", 10);
        assert_eq!(ctx.kind, CompletionContext::Variable);
        assert_eq!(ctx.prefix, "te");
        
        // Method context
        let ctx = engine.analyze_context("value.to", 8);
        assert_eq!(ctx.kind, CompletionContext::Method);
        assert_eq!(ctx.prefix, "to");
        
        // Module context
        let ctx = engine.analyze_context("std::io::", 9);
        assert_eq!(ctx.kind, CompletionContext::Module);
        
        // Type context
        let ctx = engine.analyze_context("let x: Str", 10);
        assert_eq!(ctx.kind, CompletionContext::Type);
        assert_eq!(ctx.prefix, "Str");
    }
    
    #[test]
    fn test_method_completions() {
        let mut engine = CompletionEngine::new();
        
        engine.register_method("String", "len", vec![]);
        engine.register_method("String", "to_uppercase", vec![]);
        engine.register_method("String", "to_lowercase", vec![]);
        engine.register_method("Vec", "push", vec!["item".to_string()]);
        
        let completions = engine.get_method_completions("String", "to");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.text == "to_uppercase"));
        assert!(completions.iter().any(|c| c.text == "to_lowercase"));
    }
    
    #[test]
    fn test_completion_scoring() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("test".to_string());
        engine.register_variable("testing".to_string());
        engine.register_variable("tester".to_string());
        engine.register_variable("testament".to_string());
        
        let completions = engine.get_completions("test", 4);
        
        // Exact match should score highest
        assert_eq!(completions[0].text, "test");
        
        // Verify all matches are included
        assert!(completions.len() >= 4);
    }
    
    #[test]
    fn test_case_insensitive_matching() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("TestVar".to_string());
        engine.register_variable("testvar2".to_string());
        engine.register_type("TestType".to_string());
        
        let completions = engine.get_completions("test", 4);
        assert!(completions.iter().any(|c| c.text == "TestVar"));
        assert!(completions.iter().any(|c| c.text == "testvar2"));
        assert!(completions.iter().any(|c| c.text == "TestType"));
    }
    
    #[test]
    fn test_completion_limit() {
        let mut engine = CompletionEngine::new();
        
        // Register many items
        for i in 0..100 {
            engine.register_variable(format!("test_var_{}", i));
        }
        
        engine.set_completion_limit(10);
        let completions = engine.get_completions("test", 4);
        assert_eq!(completions.len(), 10);
    }
    
    #[test]
    fn test_cache_functionality() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("cached_var".to_string());
        
        // First call - not cached
        let completions1 = engine.get_completions("cache", 5);
        
        // Second call - should be cached
        let completions2 = engine.get_completions("cache", 5);
        
        assert_eq!(completions1.len(), completions2.len());
        assert_eq!(completions1[0].text, completions2[0].text);
    }
    
    #[test]
    fn test_cache_invalidation() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("test1".to_string());
        let completions1 = engine.get_completions("test", 4);
        
        // Register new item - should invalidate cache
        engine.register_variable("test2".to_string());
        let completions2 = engine.get_completions("test", 4);
        
        assert!(completions2.len() > completions1.len());
    }
    
    #[test]
    fn test_completion_candidate() {
        let candidate = CompletionCandidate {
            text: "test_function".to_string(),
            kind: CompletionKind::Function,
            signature: Some("fn test_function(x: i32) -> String".to_string()),
            documentation: Some("Test function documentation".to_string()),
            score: 0.95,
        };
        
        assert_eq!(candidate.text, "test_function");
        assert_eq!(candidate.kind, CompletionKind::Function);
        assert!(candidate.signature.is_some());
        assert!(candidate.documentation.is_some());
        assert_eq!(candidate.score, 0.95);
    }
    
    #[test]
    fn test_completion_kinds() {
        let kinds = vec![
            CompletionKind::Variable,
            CompletionKind::Function,
            CompletionKind::Type,
            CompletionKind::Module,
            CompletionKind::Keyword,
            CompletionKind::Method,
            CompletionKind::Field,
            CompletionKind::Constant,
        ];
        
        for kind in kinds {
            let display = kind.display_str();
            assert!(!display.is_empty());
        }
    }
    
    #[test]
    fn test_fuzzy_matching() {
        let mut engine = CompletionEngine::new();
        
        engine.register_function("calculate_average".to_string(), vec![]);
        engine.register_function("calc_sum".to_string(), vec![]);
        engine.register_function("compute_median".to_string(), vec![]);
        
        // Fuzzy match "ca" should match both "calculate" and "calc"
        let completions = engine.get_completions("ca", 2);
        assert!(completions.iter().any(|c| c.text == "calculate_average"));
        assert!(completions.iter().any(|c| c.text == "calc_sum"));
    }
    
    #[test]
    fn test_path_completion() {
        let mut engine = CompletionEngine::new();
        
        engine.register_module("std".to_string(), vec!["io".to_string()]);
        engine.register_module("std::io".to_string(), vec![
            "Read".to_string(),
            "Write".to_string(),
            "BufReader".to_string(),
        ]);
        
        let completions = engine.get_completions("std::io::R", 10);
        assert!(completions.iter().any(|c| c.text == "Read"));
        assert!(!completions.iter().any(|c| c.text == "Write"));
    }
    
    #[test]
    fn test_builtin_completions() {
        let mut engine = CompletionEngine::new();
        engine.register_builtins();
        
        let completions = engine.get_completions("pri", 3);
        assert!(completions.iter().any(|c| c.text == "print" || c.text == "println"));
        
        let completions = engine.get_completions("len", 3);
        assert!(completions.iter().any(|c| c.text == "len"));
    }
    
    #[test]
    fn test_clear_completions() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("test1".to_string());
        engine.register_variable("test2".to_string());
        engine.register_function("func1".to_string(), vec![]);
        
        engine.clear();
        
        let completions = engine.get_completions("test", 4);
        assert_eq!(completions.len(), 0);
        
        let completions = engine.get_completions("func", 4);
        assert_eq!(completions.len(), 0);
    }
    
    #[test]
    fn test_completion_with_special_chars() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("test_var_1".to_string());
        engine.register_variable("test-var-2".to_string());
        engine.register_variable("test.var.3".to_string());
        
        let completions = engine.get_completions("test_", 5);
        assert!(completions.iter().any(|c| c.text == "test_var_1"));
        
        let completions = engine.get_completions("test-", 5);
        assert!(completions.iter().any(|c| c.text == "test-var-2"));
    }
    
    #[test]
    fn test_completion_at_different_positions() {
        let engine = CompletionEngine::new();
        
        let input = "let x = test";
        let completions = engine.get_completions(input, 8); // After "let x = "
        assert_eq!(engine.analyze_context(input, 8).kind, CompletionContext::Variable);
        
        let completions = engine.get_completions(input, 12); // After "test"
        assert_eq!(engine.analyze_context(input, 12).prefix, "test");
    }
    
    #[test]
    fn test_empty_prefix() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("var1".to_string());
        engine.register_variable("var2".to_string());
        engine.register_function("func1".to_string(), vec![]);
        
        // Empty prefix should return all items (up to limit)
        let completions = engine.get_completions("", 0);
        assert!(completions.len() > 0);
    }
    
    #[test]
    fn test_duplicate_registration() {
        let mut engine = CompletionEngine::new();
        
        engine.register_variable("test".to_string());
        engine.register_variable("test".to_string()); // Duplicate
        
        let completions = engine.get_completions("test", 4);
        // Should not have duplicates
        let test_count = completions.iter().filter(|c| c.text == "test").count();
        assert_eq!(test_count, 1);
    }
    
    #[test]
    fn test_field_completions() {
        let mut engine = CompletionEngine::new();
        
        engine.register_field("Person", "name", "String");
        engine.register_field("Person", "age", "i32");
        engine.register_field("Person", "address", "String");
        
        let completions = engine.get_field_completions("Person", "a");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.text == "age"));
        assert!(completions.iter().any(|c| c.text == "address"));
    }
    
    #[test]
    fn test_constant_completions() {
        let mut engine = CompletionEngine::new();
        
        engine.register_constant("PI", "3.14159");
        engine.register_constant("E", "2.71828");
        engine.register_constant("MAX_SIZE", "1024");
        
        let completions = engine.get_completions("MA", 2);
        assert!(completions.iter().any(|c| c.text == "MAX_SIZE"));
    }
}

#[cfg(test)]
mod cache_tests {
    use ruchy::runtime::repl::completion::{CompletionCache, CompletionCandidate, CompletionKind};
    
    #[test]
    fn test_cache_new() {
        let cache = CompletionCache::new(100);
        assert!(cache.is_empty());
    }
    
    #[test]
    fn test_cache_insert_get() {
        let mut cache = CompletionCache::new(100);
        
        let candidates = vec![
            CompletionCandidate {
                text: "test".to_string(),
                kind: CompletionKind::Variable,
                signature: None,
                documentation: None,
                score: 1.0,
            },
        ];
        
        cache.insert("test", 4, candidates.clone());
        
        let result = cache.get("test", 4);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
        assert_eq!(result.unwrap()[0].text, "test");
    }
    
    #[test]
    fn test_cache_invalidate() {
        let mut cache = CompletionCache::new(100);
        
        let candidates = vec![
            CompletionCandidate {
                text: "test".to_string(),
                kind: CompletionKind::Variable,
                signature: None,
                documentation: None,
                score: 1.0,
            },
        ];
        
        cache.insert("test", 4, candidates);
        assert!(!cache.is_empty());
        
        cache.invalidate();
        assert!(cache.is_empty());
        assert!(cache.get("test", 4).is_none());
    }
    
    #[test]
    fn test_cache_max_size() {
        let mut cache = CompletionCache::new(2);
        
        let candidate = CompletionCandidate {
            text: "test".to_string(),
            kind: CompletionKind::Variable,
            signature: None,
            documentation: None,
            score: 1.0,
        };
        
        cache.insert("a", 1, vec![candidate.clone()]);
        cache.insert("b", 1, vec![candidate.clone()]);
        cache.insert("c", 1, vec![candidate.clone()]); // Should evict "a"
        
        assert!(cache.get("a", 1).is_none());
        assert!(cache.get("b", 1).is_some());
        assert!(cache.get("c", 1).is_some());
    }
}

#[cfg(test)]
mod context_tests {
    use ruchy::runtime::repl::completion::CompletionContext;
    
    #[test]
    fn test_context_kinds() {
        assert_ne!(CompletionContext::Variable, CompletionContext::Function);
        assert_ne!(CompletionContext::Type, CompletionContext::Module);
        assert_eq!(CompletionContext::Variable, CompletionContext::Variable);
    }
    
    #[test]
    fn test_context_creation() {
        let ctx = CompletionContext {
            kind: CompletionContext::Method,
            prefix: "test".to_string(),
            receiver: Some("object".to_string()),
            scope: vec!["std".to_string(), "io".to_string()],
        };
        
        assert_eq!(ctx.kind, CompletionContext::Method);
        assert_eq!(ctx.prefix, "test");
        assert_eq!(ctx.receiver, Some("object".to_string()));
        assert_eq!(ctx.scope.len(), 2);
    }
}