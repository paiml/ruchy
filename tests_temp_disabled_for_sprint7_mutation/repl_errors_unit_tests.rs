//! Unit tests for the errors module
//! Target: 80% coverage of error handling and recovery functionality

#[cfg(test)]
mod error_tests {
    use ruchy::runtime::repl::errors::{
        ErrorCategory, ErrorContext, ErrorHandler, ErrorSeverity, ErrorStatistics,
        RecoveryStrategy, ReplError, SuggestionProvider,
    };
    use std::collections::HashMap;

    #[test]
    fn test_error_creation() {
        let error = ReplError::ParseError {
            message: "Unexpected token".to_string(),
            position: 10,
        };

        match error {
            ReplError::ParseError { message, position } => {
                assert_eq!(message, "Unexpected token");
                assert_eq!(position, 10);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_variants() {
        let errors = vec![
            ReplError::ParseError {
                message: "Parse error".to_string(),
                position: 0,
            },
            ReplError::TypeError {
                expected: "Int".to_string(),
                found: "String".to_string(),
            },
            ReplError::NameError {
                name: "undefined".to_string(),
            },
            ReplError::RuntimeError {
                message: "Runtime error".to_string(),
            },
            ReplError::DivisionByZero,
            ReplError::IndexOutOfBounds { index: 10, len: 5 },
            ReplError::StackOverflow { depth: 1000 },
            ReplError::Timeout { duration_ms: 5000 },
        ];

        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(
            ReplError::ParseError {
                message: "".to_string(),
                position: 0
            }
            .severity(),
            ErrorSeverity::Error
        );

        assert_eq!(ReplError::DivisionByZero.severity(), ErrorSeverity::Error);

        assert_eq!(
            ReplError::StackOverflow { depth: 1000 }.severity(),
            ErrorSeverity::Fatal
        );

        assert_eq!(
            ReplError::Timeout { duration_ms: 5000 }.severity(),
            ErrorSeverity::Fatal
        );
    }

    #[test]
    fn test_error_category() {
        assert_eq!(
            ReplError::ParseError {
                message: "".to_string(),
                position: 0
            }
            .category(),
            ErrorCategory::Syntax
        );

        assert_eq!(
            ReplError::TypeError {
                expected: "".to_string(),
                found: "".to_string()
            }
            .category(),
            ErrorCategory::Type
        );

        assert_eq!(
            ReplError::NameError {
                name: "".to_string()
            }
            .category(),
            ErrorCategory::Name
        );

        assert_eq!(
            ReplError::RuntimeError {
                message: "".to_string()
            }
            .category(),
            ErrorCategory::Runtime
        );
    }

    #[test]
    fn test_error_context() {
        let mut context = ErrorContext::new();

        context = context.with_source("let x = 10".to_string());
        context = context.with_position(8);
        context = context.with_line(5);
        context = context.with_file("test.ruchy".to_string());

        assert_eq!(context.source, Some("let x = 10".to_string()));
        assert_eq!(context.position, Some(8));
        assert_eq!(context.line, Some(5));
        assert_eq!(context.file, Some("test.ruchy".to_string()));
    }

    #[test]
    fn test_error_context_variables() {
        let mut context = ErrorContext::new();

        context.add_variable("x", "10");
        context.add_variable("y", "20");

        assert_eq!(context.variables.len(), 2);
        assert_eq!(context.variables.get("x"), Some(&"10".to_string()));
        assert_eq!(context.variables.get("y"), Some(&"20".to_string()));
    }

    #[test]
    fn test_error_handler_new() {
        let handler = ErrorHandler::new();

        let stats = handler.get_statistics();
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.errors_by_category.len(), 0);
    }

    #[test]
    fn test_error_handling() {
        let mut handler = ErrorHandler::new();

        let error = ReplError::NameError {
            name: "undefined_var".to_string(),
        };
        let context = ErrorContext::new();

        let strategy = handler.handle_error(error, context).unwrap();

        match strategy {
            RecoveryStrategy::Skip
            | RecoveryStrategy::UseDefault(_)
            | RecoveryStrategy::AskUser { .. } => {
                // Valid strategies
            }
            _ => panic!("Unexpected recovery strategy"),
        }

        let stats = handler.get_statistics();
        assert_eq!(stats.total_errors, 1);
    }

    #[test]
    fn test_recovery_strategies() {
        let strategies = vec![
            RecoveryStrategy::Skip,
            RecoveryStrategy::UseDefault("nil".to_string()),
            RecoveryStrategy::Retry {
                modification: "fixed code".to_string(),
            },
            RecoveryStrategy::AskUser {
                prompt: "Please provide value".to_string(),
            },
            RecoveryStrategy::Abort,
        ];

        for strategy in strategies {
            assert!(!strategy.to_string().is_empty());
        }
    }

    #[test]
    fn test_suggestion_provider() {
        let provider = SuggestionProvider::new();

        // Test name suggestions
        let suggestions = provider.suggest_for_name("pritn");
        assert!(suggestions.iter().any(|s| s == "print"));

        // Test type suggestions
        let suggestions = provider.suggest_for_type("Stirng");
        assert!(suggestions.iter().any(|s| s == "String"));
    }

    #[test]
    fn test_error_formatting() {
        let error = ReplError::TypeError {
            expected: "Int".to_string(),
            found: "String".to_string(),
        };

        let mut context = ErrorContext::new()
            .with_source("let x: Int = \"hello\"".to_string())
            .with_position(13)
            .with_line(1);

        let handler = ErrorHandler::new();
        let formatted = handler.format_error(&error, &context);

        assert!(formatted.contains("Type Error"));
        assert!(formatted.contains("expected Int"));
        assert!(formatted.contains("found String"));
    }

    #[test]
    fn test_error_statistics() {
        let mut handler = ErrorHandler::new();

        // Handle various errors
        handler.handle_error(
            ReplError::ParseError {
                message: "test".to_string(),
                position: 0,
            },
            ErrorContext::new(),
        );
        handler.handle_error(
            ReplError::NameError {
                name: "x".to_string(),
            },
            ErrorContext::new(),
        );
        handler.handle_error(
            ReplError::TypeError {
                expected: "Int".to_string(),
                found: "String".to_string(),
            },
            ErrorContext::new(),
        );
        handler.handle_error(
            ReplError::ParseError {
                message: "test2".to_string(),
                position: 10,
            },
            ErrorContext::new(),
        );

        let stats = handler.get_statistics();
        assert_eq!(stats.total_errors, 4);
        assert_eq!(
            stats.errors_by_category.get(&ErrorCategory::Syntax),
            Some(&2)
        );
        assert_eq!(stats.errors_by_category.get(&ErrorCategory::Name), Some(&1));
        assert_eq!(stats.errors_by_category.get(&ErrorCategory::Type), Some(&1));
    }

    #[test]
    fn test_error_recovery_parse_error() {
        let mut handler = ErrorHandler::new();

        let error = ReplError::ParseError {
            message: "Missing closing parenthesis".to_string(),
            position: 10,
        };

        let context = ErrorContext::new().with_source("print(hello".to_string());

        let strategy = handler.handle_error(error, context).unwrap();

        // Should suggest retry with fix
        match strategy {
            RecoveryStrategy::Retry { modification } => {
                assert!(modification.contains(")"));
            }
            _ => panic!("Expected Retry strategy"),
        }
    }

    #[test]
    fn test_error_recovery_name_error() {
        let mut handler = ErrorHandler::new();

        let error = ReplError::NameError {
            name: "prnt".to_string(),
        };

        let context = ErrorContext::new();
        let strategy = handler.handle_error(error, context).unwrap();

        // Should suggest similar names
        match strategy {
            RecoveryStrategy::AskUser { prompt } => {
                assert!(prompt.contains("Did you mean"));
                assert!(prompt.contains("print"));
            }
            _ => panic!("Expected AskUser strategy"),
        }
    }

    #[test]
    fn test_error_recovery_division_by_zero() {
        let mut handler = ErrorHandler::new();

        let error = ReplError::DivisionByZero;
        let context = ErrorContext::new();

        let strategy = handler.handle_error(error, context).unwrap();

        // Should use default value
        match strategy {
            RecoveryStrategy::UseDefault(val) => {
                assert!(val == "inf" || val == "0" || val == "nil");
            }
            _ => panic!("Expected UseDefault strategy"),
        }
    }

    #[test]
    fn test_error_recovery_stack_overflow() {
        let mut handler = ErrorHandler::new();

        let error = ReplError::StackOverflow { depth: 10000 };
        let context = ErrorContext::new();

        let strategy = handler.handle_error(error, context).unwrap();

        // Should abort for fatal errors
        assert_eq!(strategy, RecoveryStrategy::Abort);
    }

    #[test]
    fn test_error_history() {
        let mut handler = ErrorHandler::new();

        for i in 0..5 {
            handler.handle_error(
                ReplError::RuntimeError {
                    message: format!("Error {}", i),
                },
                ErrorContext::new(),
            );
        }

        let history = handler.get_error_history(10);
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_error_patterns() {
        let mut handler = ErrorHandler::new();

        // Create a pattern of similar errors
        for _ in 0..10 {
            handler.handle_error(
                ReplError::NameError {
                    name: "undefined".to_string(),
                },
                ErrorContext::new(),
            );
        }

        let patterns = handler.detect_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns[0].contains("NameError"));
        assert!(patterns[0].contains("10 times"));
    }

    #[test]
    fn test_clear_error_history() {
        let mut handler = ErrorHandler::new();

        handler.handle_error(
            ReplError::RuntimeError {
                message: "test".to_string(),
            },
            ErrorContext::new(),
        );

        let stats = handler.get_statistics();
        assert_eq!(stats.total_errors, 1);

        handler.clear_history();

        let stats = handler.get_statistics();
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_error_message_similarity() {
        let provider = SuggestionProvider::new();

        let similar = provider.find_similar("pritn", &["print", "println", "printf", "test"]);
        assert!(similar.contains(&"print".to_string()));

        let similar = provider.find_similar("Strng", &["String", "Int", "Bool", "Float"]);
        assert!(similar.contains(&"String".to_string()));
    }

    #[test]
    fn test_contextual_suggestions() {
        let mut handler = ErrorHandler::new();

        let error = ReplError::TypeError {
            expected: "Int".to_string(),
            found: "String".to_string(),
        };

        let mut context = ErrorContext::new().with_source("let x: Int = \"42\"".to_string());

        let suggestions = handler.get_contextual_suggestions(&error, &context);
        assert!(!suggestions.is_empty());
        assert!(suggestions
            .iter()
            .any(|s| s.contains("parse") || s.contains("to_int")));
    }

    #[test]
    fn test_error_suppression() {
        let mut handler = ErrorHandler::new();
        handler.set_suppression_level(ErrorSeverity::Warning);

        // Warning should be suppressed
        let error = ReplError::RuntimeError {
            message: "Minor issue".to_string(),
        };

        // This would normally handle the error but with suppression might skip
        let _ = handler.handle_error(error, ErrorContext::new());

        // Errors should not be suppressed
        let error = ReplError::ParseError {
            message: "Syntax error".to_string(),
            position: 0,
        };

        let result = handler.handle_error(error, ErrorContext::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_code_generation() {
        let error = ReplError::TypeError {
            expected: "Int".to_string(),
            found: "String".to_string(),
        };

        let code = error.error_code();
        assert!(code.starts_with("E"));
        assert!(code.len() == 5); // E + 4 digits
    }

    #[test]
    fn test_error_serialization() {
        let error = ReplError::RuntimeError {
            message: "Test error".to_string(),
        };

        let context = ErrorContext::new()
            .with_source("test code".to_string())
            .with_line(10);

        let handler = ErrorHandler::new();
        let json = handler.serialize_error(&error, &context);

        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"error_type\":"));
        assert!(json_str.contains("\"message\":"));
        assert!(json_str.contains("Test error"));
    }
}

#[cfg(test)]
mod recovery_strategy_tests {
    use ruchy::runtime::repl::errors::RecoveryStrategy;

    #[test]
    fn test_strategy_display() {
        assert_eq!(RecoveryStrategy::Skip.to_string(), "Skip execution");
        assert_eq!(
            RecoveryStrategy::UseDefault("nil".to_string()).to_string(),
            "Use default value: nil"
        );
        assert_eq!(
            RecoveryStrategy::Retry {
                modification: "fix".to_string()
            }
            .to_string(),
            "Retry with: fix"
        );
        assert_eq!(
            RecoveryStrategy::AskUser {
                prompt: "Enter value".to_string()
            }
            .to_string(),
            "Ask user: Enter value"
        );
        assert_eq!(RecoveryStrategy::Abort.to_string(), "Abort execution");
    }

    #[test]
    fn test_strategy_equality() {
        assert_eq!(RecoveryStrategy::Skip, RecoveryStrategy::Skip);
        assert_ne!(RecoveryStrategy::Skip, RecoveryStrategy::Abort);

        assert_eq!(
            RecoveryStrategy::UseDefault("nil".to_string()),
            RecoveryStrategy::UseDefault("nil".to_string())
        );

        assert_ne!(
            RecoveryStrategy::UseDefault("nil".to_string()),
            RecoveryStrategy::UseDefault("0".to_string())
        );
    }
}

#[cfg(test)]
mod error_statistics_tests {
    use ruchy::runtime::repl::errors::{ErrorCategory, ErrorStatistics};
    use std::collections::HashMap;

    #[test]
    fn test_statistics_new() {
        let stats = ErrorStatistics::new();

        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.recovered_errors, 0);
        assert_eq!(stats.fatal_errors, 0);
        assert!(stats.errors_by_category.is_empty());
    }

    #[test]
    fn test_statistics_update() {
        let mut stats = ErrorStatistics::new();

        stats.record_error(ErrorCategory::Syntax, true);
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.recovered_errors, 1);

        stats.record_error(ErrorCategory::Runtime, false);
        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.recovered_errors, 1);
        assert_eq!(stats.fatal_errors, 1);

        assert_eq!(
            stats.errors_by_category.get(&ErrorCategory::Syntax),
            Some(&1)
        );
        assert_eq!(
            stats.errors_by_category.get(&ErrorCategory::Runtime),
            Some(&1)
        );
    }

    #[test]
    fn test_statistics_recovery_rate() {
        let mut stats = ErrorStatistics::new();

        assert_eq!(stats.recovery_rate(), 0.0);

        stats.record_error(ErrorCategory::Syntax, true);
        stats.record_error(ErrorCategory::Name, true);
        stats.record_error(ErrorCategory::Type, false);
        stats.record_error(ErrorCategory::Runtime, true);

        assert_eq!(stats.recovery_rate(), 75.0);
    }

    #[test]
    fn test_statistics_most_common_category() {
        let mut stats = ErrorStatistics::new();

        stats.record_error(ErrorCategory::Syntax, true);
        stats.record_error(ErrorCategory::Syntax, true);
        stats.record_error(ErrorCategory::Name, true);
        stats.record_error(ErrorCategory::Syntax, false);

        assert_eq!(stats.most_common_category(), Some(ErrorCategory::Syntax));
    }
}
