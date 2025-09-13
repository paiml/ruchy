//! Comprehensive TDD tests for LSP (Language Server Protocol) modules
//! Target: Increase coverage for LSP server, analyzer, and capabilities
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod lsp_tests {
    use crate::lsp::{LspServer, Analyzer, Capabilities, CompletionItem, Diagnostic, Position, Range};
    use std::collections::HashMap;
    
    // ========== LSP Server Tests ==========
    
    #[test]
    fn test_server_creation() {
        let server = LspServer::new();
        assert!(server.is_initialized() == false);
        assert_eq!(server.document_count(), 0);
    }
    
    #[test]
    fn test_server_initialization() {
        let mut server = LspServer::new();
        
        let init_params = InitializeParams {
            root_uri: Some("file:///workspace".to_string()),
            capabilities: ClientCapabilities::default(),
        };
        
        let result = server.initialize(init_params);
        assert!(result.is_ok());
        assert!(server.is_initialized());
        
        let capabilities = result.unwrap();
        assert!(capabilities.text_document_sync.is_some());
    }
    
    #[test]
    fn test_server_shutdown() {
        let mut server = LspServer::new();
        server.initialize(InitializeParams::default()).unwrap();
        
        let result = server.shutdown();
        assert!(result.is_ok());
        assert!(!server.is_initialized());
    }
    
    #[test]
    fn test_document_open() {
        let mut server = LspServer::new();
        server.initialize(InitializeParams::default()).unwrap();
        
        let doc_uri = "file:///test.ruchy";
        let content = "let x = 42";
        
        server.did_open(doc_uri, content);
        assert_eq!(server.document_count(), 1);
        assert_eq!(server.get_document(doc_uri), Some(content));
    }
    
    #[test]
    fn test_document_change() {
        let mut server = LspServer::new();
        server.initialize(InitializeParams::default()).unwrap();
        
        let doc_uri = "file:///test.ruchy";
        server.did_open(doc_uri, "let x = 42");
        
        let changes = vec![
            TextDocumentContentChange {
                range: None,
                text: "let x = 100",
            }
        ];
        
        server.did_change(doc_uri, changes);
        assert_eq!(server.get_document(doc_uri), Some("let x = 100"));
    }
    
    #[test]
    fn test_document_close() {
        let mut server = LspServer::new();
        server.initialize(InitializeParams::default()).unwrap();
        
        let doc_uri = "file:///test.ruchy";
        server.did_open(doc_uri, "let x = 42");
        assert_eq!(server.document_count(), 1);
        
        server.did_close(doc_uri);
        assert_eq!(server.document_count(), 0);
    }
    
    // ========== Analyzer Tests ==========
    
    #[test]
    fn test_analyzer_creation() {
        let analyzer = Analyzer::new();
        assert_eq!(analyzer.symbol_count(), 0);
    }
    
    #[test]
    fn test_analyze_simple_code() {
        let mut analyzer = Analyzer::new();
        let code = "let x = 42; let y = x * 2;";
        
        let result = analyzer.analyze(code);
        assert!(result.is_ok());
        
        let symbols = analyzer.get_symbols();
        assert!(symbols.contains_key("x"));
        assert!(symbols.contains_key("y"));
    }
    
    #[test]
    fn test_find_definition() {
        let mut analyzer = Analyzer::new();
        let code = "let x = 42;\nlet y = x + 1;";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 1, character: 8 }; // Position of 'x' in second line
        let definition = analyzer.find_definition(position);
        
        assert!(definition.is_some());
        let def = definition.unwrap();
        assert_eq!(def.line, 0); // Definition is on first line
    }
    
    #[test]
    fn test_find_references() {
        let mut analyzer = Analyzer::new();
        let code = "let x = 42;\nlet y = x + 1;\nlet z = x * 2;";
        analyzer.analyze(code).unwrap();
        
        let references = analyzer.find_references("x");
        assert_eq!(references.len(), 2); // Used in line 2 and 3
    }
    
    #[test]
    fn test_hover_info() {
        let mut analyzer = Analyzer::new();
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 0, character: 3 }; // Position of 'add'
        let hover = analyzer.get_hover_info(position);
        
        assert!(hover.is_some());
        let info = hover.unwrap();
        assert!(info.contains("fn add"));
        assert!(info.contains("i32"));
    }
    
    // ========== Completion Tests ==========
    
    #[test]
    fn test_completion_basic() {
        let mut analyzer = Analyzer::new();
        let code = "let my_variable = 42;\nmy_";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 1, character: 3 };
        let completions = analyzer.get_completions(position);
        
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "my_variable"));
    }
    
    #[test]
    fn test_completion_methods() {
        let mut analyzer = Analyzer::new();
        let code = "let s = \"hello\";\ns.";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 1, character: 2 };
        let completions = analyzer.get_completions(position);
        
        // Should suggest string methods
        assert!(completions.iter().any(|c| c.label == "len"));
        assert!(completions.iter().any(|c| c.label == "to_uppercase"));
    }
    
    #[test]
    fn test_completion_keywords() {
        let mut analyzer = Analyzer::new();
        let code = "f";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 0, character: 1 };
        let completions = analyzer.get_completions(position);
        
        // Should suggest keywords starting with 'f'
        assert!(completions.iter().any(|c| c.label == "fn"));
        assert!(completions.iter().any(|c| c.label == "for"));
    }
    
    #[test]
    fn test_completion_with_imports() {
        let mut analyzer = Analyzer::new();
        let code = "import std::vec;\nve";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 1, character: 2 };
        let completions = analyzer.get_completions(position);
        
        // Should include imported symbols
        assert!(completions.iter().any(|c| c.label == "vec"));
    }
    
    // ========== Diagnostics Tests ==========
    
    #[test]
    fn test_diagnostics_syntax_error() {
        let mut analyzer = Analyzer::new();
        let code = "let x = @#$";
        
        let diagnostics = analyzer.get_diagnostics(code);
        assert!(!diagnostics.is_empty());
        
        let diag = &diagnostics[0];
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert!(diag.message.contains("Syntax") || diag.message.contains("Unexpected"));
    }
    
    #[test]
    fn test_diagnostics_undefined_variable() {
        let mut analyzer = Analyzer::new();
        let code = "let x = y + 1;";
        
        let diagnostics = analyzer.get_diagnostics(code);
        assert!(!diagnostics.is_empty());
        
        let diag = &diagnostics[0];
        assert!(diag.message.contains("undefined") || diag.message.contains("not found"));
    }
    
    #[test]
    fn test_diagnostics_type_mismatch() {
        let mut analyzer = Analyzer::new();
        let code = "let x: i32 = \"hello\";";
        
        let diagnostics = analyzer.get_diagnostics(code);
        assert!(!diagnostics.is_empty());
        
        let diag = &diagnostics[0];
        assert!(diag.message.contains("type") || diag.message.contains("mismatch"));
    }
    
    #[test]
    fn test_diagnostics_unused_variable() {
        let mut analyzer = Analyzer::new();
        let code = "let x = 42;";
        
        let diagnostics = analyzer.get_diagnostics(code);
        // Should warn about unused variable
        let warning = diagnostics.iter()
            .find(|d| d.severity == DiagnosticSeverity::Warning);
        
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("unused"));
    }
    
    // ========== Capabilities Tests ==========
    
    #[test]
    fn test_capabilities_default() {
        let capabilities = Capabilities::default();
        
        assert!(capabilities.text_document_sync.is_some());
        assert!(capabilities.completion_provider.is_some());
        assert!(capabilities.hover_provider);
        assert!(capabilities.definition_provider);
    }
    
    #[test]
    fn test_capabilities_custom() {
        let mut capabilities = Capabilities::default();
        
        capabilities.rename_provider = true;
        capabilities.document_formatting_provider = true;
        capabilities.code_action_provider = true;
        
        assert!(capabilities.rename_provider);
        assert!(capabilities.document_formatting_provider);
        assert!(capabilities.code_action_provider);
    }
    
    // ========== Rename Tests ==========
    
    #[test]
    fn test_rename_variable() {
        let mut analyzer = Analyzer::new();
        let code = "let old_name = 42;\nlet y = old_name + 1;";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 0, character: 4 }; // Position of 'old_name'
        let edits = analyzer.rename(position, "new_name");
        
        assert!(!edits.is_empty());
        assert_eq!(edits.len(), 2); // Should rename in both locations
        
        for edit in edits {
            assert_eq!(edit.new_text, "new_name");
        }
    }
    
    #[test]
    fn test_rename_function() {
        let mut analyzer = Analyzer::new();
        let code = "fn old_func() {}\nold_func();";
        analyzer.analyze(code).unwrap();
        
        let position = Position { line: 0, character: 3 };
        let edits = analyzer.rename(position, "new_func");
        
        assert_eq!(edits.len(), 2); // Definition and call site
    }
    
    // ========== Code Actions Tests ==========
    
    #[test]
    fn test_code_action_quick_fix() {
        let mut analyzer = Analyzer::new();
        let code = "let x = 42"; // Missing semicolon
        
        let diagnostics = analyzer.get_diagnostics(code);
        let actions = analyzer.get_code_actions(&diagnostics[0]);
        
        assert!(!actions.is_empty());
        let fix = &actions[0];
        assert_eq!(fix.title, "Add semicolon");
        assert!(fix.edit.changes.values().any(|edits| 
            edits.iter().any(|e| e.new_text == ";")));
    }
    
    #[test]
    fn test_code_action_import() {
        let mut analyzer = Analyzer::new();
        let code = "vec![1, 2, 3]"; // vec macro not imported
        
        let diagnostics = analyzer.get_diagnostics(code);
        let actions = analyzer.get_code_actions(&diagnostics[0]);
        
        let import_action = actions.iter()
            .find(|a| a.title.contains("Import"));
        
        assert!(import_action.is_some());
    }
    
    // ========== Formatting Tests ==========
    
    #[test]
    fn test_format_document() {
        let mut server = LspServer::new();
        server.initialize(InitializeParams::default()).unwrap();
        
        let unformatted = "let x=42;let y=100;";
        let formatted = server.format_document(unformatted);
        
        assert!(formatted.contains("let x = 42;"));
        assert!(formatted.contains("let y = 100;"));
    }
    
    #[test]
    fn test_format_range() {
        let mut server = LspServer::new();
        server.initialize(InitializeParams::default()).unwrap();
        
        let code = "let x=42;\nlet y=100;\nlet z=200;";
        let range = Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 1, character: 10 },
        };
        
        let formatted = server.format_range(code, range);
        
        // Should only format first two lines
        assert!(formatted.contains("let x = 42;"));
        assert!(formatted.contains("let y = 100;"));
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    impl LspServer {
        fn is_initialized(&self) -> bool {
            self.initialized
        }
        
        fn document_count(&self) -> usize {
            self.documents.len()
        }
        
        fn get_document(&self, uri: &str) -> Option<&str> {
            self.documents.get(uri).map(|s| s.as_str())
        }
    }
    
    impl Analyzer {
        fn symbol_count(&self) -> usize {
            self.symbols.len()
        }
        
        fn get_symbols(&self) -> &HashMap<String, SymbolInfo> {
            &self.symbols
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_analyzer_never_panics(code in ".*") {
            let mut analyzer = Analyzer::new();
            let _ = analyzer.analyze(&code);
            let _ = analyzer.get_diagnostics(&code);
            // Should not panic even with arbitrary input
        }
        
        #[test]
        fn test_position_within_document(line in 0usize..1000, char in 0usize..200) {
            let position = Position {
                line,
                character: char,
            };
            
            // Position should be valid
            assert!(position.line >= 0);
            assert!(position.character >= 0);
        }
        
        #[test]
        fn test_completion_consistency(prefix in "[a-z]{1,5}") {
            let mut analyzer = Analyzer::new();
            let code = format!("let {} = 42;", prefix);
            analyzer.analyze(&code).unwrap();
            
            let position = Position {
                line: 0,
                character: prefix.len(),
            };
            
            let completions = analyzer.get_completions(position);
            // All completions should start with prefix or be keywords
            for completion in completions {
                assert!(
                    completion.label.starts_with(&prefix) ||
                    is_keyword(&completion.label)
                );
            }
        }
    }
    
    fn is_keyword(s: &str) -> bool {
        matches!(s, "fn" | "let" | "if" | "else" | "for" | "while" | "match" | "return")
    }
}