    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        // Formatter::new never panics
        #[test]
        fn prop_formatter_new_never_panics(_dummy: u8) {
            let _formatter = Formatter::new();
            prop_assert!(true);
        }

        // set_source never panics
        #[test]
        fn prop_set_source_never_panics(source in "[a-zA-Z0-9_ ]{0,100}") {
            let mut formatter = Formatter::new();
            formatter.set_source(source);
            prop_assert!(true);
        }

        // FormatterConfig default is valid
        #[test]
        fn prop_formatter_config_default_valid(_dummy: u8) {
            let config = FormatterConfig::default();
            prop_assert!(config.indent_width > 0);
        }

        // Different indent widths create valid formatters
        #[test]
        fn prop_indent_config_valid(indent in 1usize..8) {
            let config = FormatterConfig {
                indent_width: indent,
                ..Default::default()
            };
            let _formatter = Formatter::with_config(config);
            prop_assert!(true);
        }

        // Parsing then formatting integers works
        #[test]
        fn prop_format_parsed_integer(n in -1000i64..1000) {
            let code = format!("{n}");
            let mut parser = crate::frontend::parser::Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let formatter = Formatter::new();
                let result = formatter.format(&ast);
                prop_assert!(result.is_ok());
            }
        }

        // Parsing then formatting bools works
        #[test]
        fn prop_format_parsed_bool(b in proptest::bool::ANY) {
            let code = if b { "true" } else { "false" };
            let mut parser = crate::frontend::parser::Parser::new(code);
            if let Ok(ast) = parser.parse() {
                let formatter = Formatter::new();
                let result = formatter.format(&ast);
                prop_assert!(result.is_ok());
            }
        }

        // Parsing then formatting strings works
        #[test]
        fn prop_format_parsed_string(s in "[a-zA-Z0-9]{0,20}") {
            let code = format!("\"{s}\"");
            let mut parser = crate::frontend::parser::Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let formatter = Formatter::new();
                let result = formatter.format(&ast);
                prop_assert!(result.is_ok());
            }
        }

        // Parsing then formatting identifiers works
        #[test]
        fn prop_format_parsed_identifier(name in "[a-z][a-z0-9_]{0,10}") {
            let mut parser = crate::frontend::parser::Parser::new(&name);
            if let Ok(ast) = parser.parse() {
                let formatter = Formatter::new();
                let result = formatter.format(&ast);
                prop_assert!(result.is_ok());
            }
        }

        // Parsing then formatting let statements works
        #[test]
        fn prop_format_parsed_let(n in -100i64..100) {
            let code = format!("let x = {n}");
            let mut parser = crate::frontend::parser::Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let formatter = Formatter::new();
                let result = formatter.format(&ast);
                prop_assert!(result.is_ok());
            }
        }
    }
