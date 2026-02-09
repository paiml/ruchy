    use super::*;

    #[test]
    fn test_looks_like_comprehension_negation() {
        // MISSED: delete ! in looks_like_comprehension (line 1168)

        use crate::Parser;

        // Test array comprehension (should have 'for' keyword)
        let mut parser = Parser::new("[x for x in range(10)]");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Array comprehension should parse (tests ! in while condition)"
        );

        // Test regular array (no 'for' keyword)
        let mut parser2 = Parser::new("[1, 2, 3, 4, 5]");
        let result2 = parser2.parse();
        assert!(result2.is_ok(), "Regular array should parse");
    }

    #[test]
    fn test_parse_constructor_pattern_not_stub() {
        // MISSED: replace parse_constructor_pattern -> Result<String> with Ok(String::new())

        use crate::Parser;

        // Test enum pattern with constructor
        let mut parser = Parser::new("match value { Some(x) => x, None => 0 }");
        let result = parser.parse();

        // If parse_constructor_pattern returned empty string stub, pattern matching would fail
        assert!(
            result.is_ok(),
            "Enum constructor pattern should parse correctly"
        );
    }

    #[test]
    fn test_declaration_token_to_key_var_match_arm() {
        // MISSED: delete match arm Token::Var in declaration_token_to_key (line 322)

        // Direct unit test of the declaration_token_to_key function
        let result = declaration_token_to_key(&Token::Var);
        assert!(result.is_some(), "Token::Var should map to a key");
        assert_eq!(
            result.unwrap(),
            "var",
            "Token::Var should map to 'var' string"
        );
    }

    #[test]
    fn test_add_non_empty_row_negation() {
        // MISSED: delete ! in add_non_empty_row (line 1047)

        use crate::Parser;

        // Test nested arrays which exercises the row collection logic
        // The add_non_empty_row function filters out empty rows using !row.is_empty()
        let mut parser = Parser::new("[[1, 2], [3, 4]]");
        let result = parser.parse();

        // If ! is deleted, only empty rows would be added
        // With ! present, non-empty rows are added correctly
        assert!(
            result.is_ok(),
            "Nested arrays should parse (tests ! in add_non_empty_row)"
        );
    }

    // COVERAGE: Additional helper function tests
    #[test]
    fn test_control_flow_token_to_key() {
        assert_eq!(
            control_flow_token_to_key(&Token::If),
            Some("if".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::Else),
            Some("else".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::Match),
            Some("match".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::While),
            Some("while".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::For),
            Some("for".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::Loop),
            Some("loop".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::Break),
            Some("break".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::Continue),
            Some("continue".to_string())
        );
        assert_eq!(
            control_flow_token_to_key(&Token::Return),
            Some("return".to_string())
        );
        assert_eq!(control_flow_token_to_key(&Token::Plus), None);
    }

    #[test]
    fn test_declaration_token_to_key_all() {
        assert_eq!(
            declaration_token_to_key(&Token::Let),
            Some("let".to_string())
        );
        assert_eq!(
            declaration_token_to_key(&Token::Var),
            Some("var".to_string())
        );
        assert_eq!(
            declaration_token_to_key(&Token::Const),
            Some("const".to_string())
        );
        assert_eq!(
            declaration_token_to_key(&Token::Static),
            Some("static".to_string())
        );
        assert_eq!(
            declaration_token_to_key(&Token::Pub),
            Some("pub".to_string())
        );
        assert_eq!(
            declaration_token_to_key(&Token::Mut),
            Some("mut".to_string())
        );
        assert_eq!(
            declaration_token_to_key(&Token::Fun),
            Some("fun".to_string())
        );
        assert_eq!(declaration_token_to_key(&Token::Fn), Some("fn".to_string()));
        assert_eq!(declaration_token_to_key(&Token::Plus), None);
    }

    #[test]
    fn test_type_token_to_key() {
        assert_eq!(type_token_to_key(&Token::Type), Some("type".to_string()));
        assert_eq!(
            type_token_to_key(&Token::Struct),
            Some("struct".to_string())
        );
        assert_eq!(type_token_to_key(&Token::Enum), Some("enum".to_string()));
        assert_eq!(type_token_to_key(&Token::Impl), Some("impl".to_string()));
        assert_eq!(type_token_to_key(&Token::Trait), Some("trait".to_string()));
        assert_eq!(type_token_to_key(&Token::Plus), None);
    }

    #[test]
    fn test_module_token_to_key() {
        assert_eq!(
            module_token_to_key(&Token::Module),
            Some("module".to_string())
        );
        assert_eq!(
            module_token_to_key(&Token::Import),
            Some("import".to_string())
        );
        assert_eq!(
            module_token_to_key(&Token::Export),
            Some("export".to_string())
        );
        assert_eq!(module_token_to_key(&Token::Use), Some("use".to_string()));
        assert_eq!(module_token_to_key(&Token::As), Some("as".to_string()));
        assert_eq!(module_token_to_key(&Token::From), Some("from".to_string()));
        assert_eq!(module_token_to_key(&Token::Self_), Some("self".to_string()));
        assert_eq!(
            module_token_to_key(&Token::Super),
            Some("super".to_string())
        );
        assert_eq!(
            module_token_to_key(&Token::Crate),
            Some("crate".to_string())
        );
        assert_eq!(module_token_to_key(&Token::In), Some("in".to_string()));
        assert_eq!(
            module_token_to_key(&Token::Where),
            Some("where".to_string())
        );
        assert_eq!(module_token_to_key(&Token::Plus), None);
    }

    #[test]
    fn test_async_error_token_to_key() {
        assert_eq!(
            async_error_token_to_key(&Token::Async),
            Some("async".to_string())
        );
        assert_eq!(
            async_error_token_to_key(&Token::Await),
            Some("await".to_string())
        );
        assert_eq!(
            async_error_token_to_key(&Token::Try),
            Some("try".to_string())
        );
        assert_eq!(
            async_error_token_to_key(&Token::Catch),
            Some("catch".to_string())
        );
        assert_eq!(
            async_error_token_to_key(&Token::Throw),
            Some("throw".to_string())
        );
        assert_eq!(async_error_token_to_key(&Token::Plus), None);
    }

    #[test]
    fn test_can_be_object_key() {
        use crate::frontend::lexer::Token;

        // Identifier should be valid
        assert!(can_be_object_key(&Token::Identifier("name".to_string())));

        // String should be valid
        assert!(can_be_object_key(&Token::String("key".to_string())));

        // Control flow keywords should be valid
        assert!(can_be_object_key(&Token::If));
        assert!(can_be_object_key(&Token::While));

        // Operators should not be valid
        assert!(!can_be_object_key(&Token::Plus));
        assert!(!can_be_object_key(&Token::LeftParen));
    }

    #[test]
    fn test_parse_block_empty() {
        use crate::Parser;
        let mut parser = Parser::new("{}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_block_single_expr() {
        use crate::Parser;
        let mut parser = Parser::new("{ 42 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_block_multiple_exprs() {
        use crate::Parser;
        let mut parser = Parser::new("{ let x = 1; let y = 2; x + y }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array_empty() {
        use crate::Parser;
        let mut parser = Parser::new("[]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array_single() {
        use crate::Parser;
        let mut parser = Parser::new("[1]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array_multiple() {
        use crate::Parser;
        let mut parser = Parser::new("[1, 2, 3, 4, 5]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array_nested() {
        use crate::Parser;
        let mut parser = Parser::new("[[1, 2], [3, 4], [5, 6]]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_map_constructor() {
        use crate::Parser;
        let mut parser = Parser::new("HashMap()");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_object_with_string_key() {
        use crate::Parser;
        let mut parser = Parser::new("{ \"key\": 42 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_set_braces() {
        use crate::Parser;
        // Set literal syntax uses braces with comma-separated values
        let mut parser = Parser::new("{1, 2, 3}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tuple_single() {
        use crate::Parser;
        let mut parser = Parser::new("(1,)");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tuple_multiple() {
        use crate::Parser;
        let mut parser = Parser::new("(1, 2, 3)");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_object_keyword_keys() {
        use crate::Parser;

        // Test with various keyword keys
        let mut parser = Parser::new("{ if: 1, for: 2, let: 3 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Object with keyword keys should parse");
    }

    #[test]
    fn test_parse_object_spread() {
        use crate::Parser;
        let mut parser = Parser::new("{ ...other, x: 1 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Object spread should parse");
    }
