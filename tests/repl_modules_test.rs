//! Comprehensive tests for refactored REPL modules
//! Validates modularity and maintains coverage

#[cfg(test)]
mod value_tests {
    use ruchy::runtime::repl_modules::value::{Value, DataFrameColumn};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_value_equality() {
        assert_eq!(Value::Int(42), Value::Int(42));
        assert_ne!(Value::Int(42), Value::Int(43));
        assert_eq!(Value::String("hello".to_string()), Value::String("hello".to_string()));
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_value_type_names() {
        assert_eq!(Value::Int(42).type_name(), "int");
        assert_eq!(Value::Float(3.14).type_name(), "float");
        assert_eq!(Value::String("test".to_string()).type_name(), "str");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::List(vec![]).type_name(), "list");
        assert_eq!(Value::Unit.type_name(), "unit");
        assert_eq!(Value::Nil.type_name(), "nil");
    }

    #[test]
    fn test_value_truthiness() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
        assert!(!Value::String("".to_string()).is_truthy());
        assert!(!Value::Nil.is_truthy());
        assert!(!Value::Unit.is_truthy());
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Unit), "()");
        assert_eq!(format!("{}", Value::Nil), "None");
    }

    #[test]
    fn test_list_display() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(format!("{}", list), "[1, 2, 3]");
    }

    #[test]
    fn test_tuple_display() {
        let tuple = Value::Tuple(vec![Value::Int(1), Value::String("a".to_string())]);
        assert_eq!(format!("{}", tuple), "(1, a)");
        
        let single = Value::Tuple(vec![Value::Int(42)]);
        assert_eq!(format!("{}", single), "(42,)");
    }

    #[test]
    fn test_range_display() {
        let range = Value::Range { start: 0, end: 10, inclusive: false };
        assert_eq!(format!("{}", range), "0..10");
        
        let inclusive_range = Value::Range { start: 0, end: 10, inclusive: true };
        assert_eq!(format!("{}", inclusive_range), "0..=10");
    }

    #[test]
    fn test_value_hash() {
        let mut set = HashSet::new();
        set.insert(Value::Int(42));
        set.insert(Value::String("hello".to_string()));
        set.insert(Value::Bool(true));
        
        assert!(set.contains(&Value::Int(42)));
        assert!(set.contains(&Value::String("hello".to_string())));
        assert!(!set.contains(&Value::Int(43)));
    }
}

#[cfg(test)]
mod config_tests {
    use ruchy::runtime::repl_modules::config::ReplConfig;

    #[test]
    fn test_config_default() {
        let config = ReplConfig::default();
        assert_eq!(config.history_file, ".ruchy_history");
        assert_eq!(config.max_history, 1000);
        assert_eq!(config.prompt, "ruchy> ");
        assert!(config.colored_output);
    }

    #[test]
    fn test_config_builder() {
        let config = ReplConfig::new()
            .with_history_file("custom_history".to_string())
            .with_max_history(500)
            .with_prompt(">>> ".to_string())
            .with_vi_mode(true);
        
        assert_eq!(config.history_file, "custom_history");
        assert_eq!(config.max_history, 500);
        assert_eq!(config.prompt, ">>> ");
        assert!(config.vi_mode);
    }
}

#[cfg(test)]
mod memory_tests {
    use ruchy::runtime::repl_modules::memory::MemoryTracker;

    #[test]
    fn test_memory_allocation() {
        let tracker = MemoryTracker::new(1024);
        
        assert!(tracker.allocate(512).is_ok());
        assert_eq!(tracker.current_usage(), 512);
        assert_eq!(tracker.remaining(), 512);
        
        assert!(tracker.allocate(256).is_ok());
        assert_eq!(tracker.current_usage(), 768);
        assert_eq!(tracker.remaining(), 256);
    }

    #[test]
    fn test_memory_limit_exceeded() {
        let tracker = MemoryTracker::new(1024);
        
        assert!(tracker.allocate(1024).is_ok());
        assert!(tracker.allocate(1).is_err());
    }

    #[test]
    fn test_memory_deallocation() {
        let tracker = MemoryTracker::new(1024);
        
        tracker.allocate(512).unwrap();
        tracker.deallocate(256);
        assert_eq!(tracker.current_usage(), 256);
        assert_eq!(tracker.remaining(), 768);
    }

    #[test]
    fn test_memory_reset() {
        let tracker = MemoryTracker::new(1024);
        
        tracker.allocate(512).unwrap();
        tracker.reset();
        assert_eq!(tracker.current_usage(), 0);
        assert_eq!(tracker.remaining(), 1024);
    }

    #[test]
    fn test_memory_percentage() {
        let tracker = MemoryTracker::new(1000);
        
        tracker.allocate(500).unwrap();
        assert_eq!(tracker.usage_percentage(), 50.0);
        
        tracker.allocate(400).unwrap();
        assert_eq!(tracker.usage_percentage(), 90.0);
        assert!(!tracker.is_nearly_exhausted());
        
        tracker.allocate(10).unwrap();
        assert!(tracker.is_nearly_exhausted());
    }
}

#[cfg(test)]
mod history_tests {
    use ruchy::runtime::repl_modules::history::HistoryManager;

    #[test]
    fn test_history_add() {
        let mut history = HistoryManager::new(10);
        
        history.add("command1".to_string(), true);
        history.add("command2".to_string(), false);
        
        assert_eq!(history.len(), 2);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_history_last() {
        let mut history = HistoryManager::new(10);
        
        history.add("first".to_string(), true);
        history.add("second".to_string(), true);
        history.add("third".to_string(), true);
        
        let last = history.last(2);
        assert_eq!(last, vec!["third", "second"]);
    }

    #[test]
    fn test_history_search() {
        let mut history = HistoryManager::new(10);
        
        history.add("print 1".to_string(), true);
        history.add("x = 42".to_string(), true);
        history.add("print 2".to_string(), true);
        
        let results = history.search("print");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"print 1".to_string()));
        assert!(results.contains(&"print 2".to_string()));
    }

    #[test]
    fn test_history_max_size() {
        let mut history = HistoryManager::new(3);
        
        history.add("cmd1".to_string(), true);
        history.add("cmd2".to_string(), true);
        history.add("cmd3".to_string(), true);
        history.add("cmd4".to_string(), true);
        
        assert_eq!(history.len(), 3);
        let all = history.last(10);
        assert_eq!(all, vec!["cmd4", "cmd3", "cmd2"]);
    }

    #[test]
    fn test_history_stats() {
        let mut history = HistoryManager::new(10);
        
        history.add_with_timing("cmd1".to_string(), std::time::Duration::from_millis(100), true);
        history.add_with_timing("cmd2".to_string(), std::time::Duration::from_millis(200), false);
        history.add_with_timing("cmd3".to_string(), std::time::Duration::from_millis(150), true);
        
        let stats = history.stats();
        assert_eq!(stats.total_commands, 3);
        assert_eq!(stats.successful_commands, 2);
        assert_eq!(stats.failed_commands, 1);
        assert_eq!(stats.success_rate() as i32, 66); // ~66.67%
    }

    #[test]
    fn test_history_clear() {
        let mut history = HistoryManager::new(10);
        
        history.add("cmd".to_string(), true);
        assert_eq!(history.len(), 1);
        
        history.clear();
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }
}

#[cfg(test)]
mod state_tests {
    use ruchy::runtime::repl_modules::state::{ReplMode, ReplState, Checkpoint};
    use ruchy::runtime::repl_modules::value::Value;
    use std::collections::HashMap;

    #[test]
    fn test_repl_modes() {
        assert_eq!(ReplMode::Normal.display_name(), "Normal");
        assert_eq!(ReplMode::Debug.display_name(), "Debug");
        
        assert!(ReplMode::Normal.is_interactive());
        assert!(ReplMode::Debug.is_interactive());
        assert!(!ReplMode::Script.is_interactive());
    }

    #[test]
    fn test_checkpoint_creation() {
        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), Value::Int(42));
        
        let checkpoint = Checkpoint::new(1, bindings.clone(), "test".to_string(), None);
        
        assert_eq!(checkpoint.id, 1);
        assert_eq!(checkpoint.description, "test");
        assert!(!checkpoint.has_parent());
        assert_eq!(checkpoint.bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_checkpoint_hierarchy() {
        let parent = Checkpoint::new(1, HashMap::new(), "parent".to_string(), None);
        let child = parent.create_child(2, "child".to_string());
        
        assert_eq!(child.id, 2);
        assert_eq!(child.parent_id, Some(1));
        assert!(child.has_parent());
    }

    #[test]
    fn test_repl_state_transitions() {
        let state = ReplState::start_evaluation("x = 42".to_string());
        assert!(state.is_evaluating());
        assert!(!state.is_ready());
        
        if let ReplState::Evaluating { expression, start_time } = state {
            let completed = ReplState::complete_evaluation(Value::Int(42), start_time);
            assert!(!completed.is_evaluating());
        }
        
        let error_state = ReplState::error("Division by zero".to_string(), None);
        assert!(error_state.is_error());
        
        let recovery = ReplState::start_recovery(1);
        assert!(recovery.is_recovering());
    }

    #[test]
    fn test_repl_state_descriptions() {
        assert_eq!(ReplState::Ready.description(), "Ready");
        
        let eval = ReplState::start_evaluation("print(42)".to_string());
        assert!(eval.description().contains("Evaluating: print(42)"));
        
        let error = ReplState::error("Syntax error".to_string(), None);
        assert!(error.description().contains("Error: Syntax error"));
    }
}

#[cfg(test)]
mod command_tests {
    use ruchy::runtime::repl_modules::commands::{CommandHandler, CommandResult};
    use ruchy::runtime::repl_modules::state::ReplMode;

    #[test]
    fn test_command_handler_creation() {
        let handler = CommandHandler::new();
        let commands = handler.list_commands();
        
        assert!(commands.contains(&"help".to_string()));
        assert!(commands.contains(&"exit".to_string()));
        assert!(commands.contains(&"clear".to_string()));
        assert!(commands.contains(&"mode".to_string()));
    }

    #[test]
    fn test_command_processing() {
        let handler = CommandHandler::new();
        
        assert!(handler.process("not a command").is_none());
        assert!(handler.process(":help").is_some());
        
        match handler.process(":unknown") {
            Some(CommandResult::Error(msg)) => assert!(msg.contains("Unknown command")),
            _ => panic!("Expected error for unknown command"),
        }
    }

    #[test]
    fn test_mode_command() {
        let handler = CommandHandler::new();
        
        match handler.process(":mode debug") {
            Some(CommandResult::SwitchMode(mode)) => assert_eq!(mode, ReplMode::Debug),
            _ => panic!("Expected mode switch"),
        }
        
        match handler.process(":mode invalid") {
            Some(CommandResult::Error(msg)) => assert!(msg.contains("Unknown mode")),
            _ => panic!("Expected error for invalid mode"),
        }
    }

    #[test]
    fn test_help_command() {
        let handler = CommandHandler::new();
        
        let help = handler.get_help(None);
        assert!(help.contains("Available commands"));
        assert!(help.contains("help"));
        assert!(help.contains("exit"));
        
        let specific = handler.get_help(Some("exit"));
        assert!(specific.contains("Exit the REPL"));
    }
}

#[cfg(test)]
mod evaluator_tests {
    use ruchy::runtime::repl_modules::evaluator::Evaluator;
    use ruchy::runtime::repl_modules::value::Value;
    use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp};
    use ruchy::frontend::Span;

    fn make_literal(val: Literal) -> Expr {
        Expr {
            kind: ExprKind::Literal(val),
            span: Span::new(0, 0),
            attributes: vec![],
        }
    }

    fn make_binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span: Span::new(0, 0),
            attributes: vec![],
        }
    }

    #[test]
    fn test_evaluate_literals() {
        let mut eval = Evaluator::new();
        
        let int_expr = make_literal(Literal::Integer(42));
        assert_eq!(eval.evaluate(&int_expr), Ok(Value::Int(42)));
        
        let float_expr = make_literal(Literal::Float(3.14));
        assert_eq!(eval.evaluate(&float_expr), Ok(Value::Float(3.14)));
        
        let str_expr = make_literal(Literal::String("hello".to_string()));
        assert_eq!(eval.evaluate(&str_expr), Ok(Value::String("hello".to_string())));
        
        let bool_expr = make_literal(Literal::Bool(true));
        assert_eq!(eval.evaluate(&bool_expr), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_evaluate_arithmetic() {
        let mut eval = Evaluator::new();
        
        let add = make_binary(
            BinaryOp::Add,
            make_literal(Literal::Integer(2)),
            make_literal(Literal::Integer(3)),
        );
        assert_eq!(eval.evaluate(&add), Ok(Value::Int(5)));
        
        let sub = make_binary(
            BinaryOp::Sub,
            make_literal(Literal::Integer(10)),
            make_literal(Literal::Integer(3)),
        );
        assert_eq!(eval.evaluate(&sub), Ok(Value::Int(7)));
        
        let mul = make_binary(
            BinaryOp::Mul,
            make_literal(Literal::Integer(4)),
            make_literal(Literal::Integer(5)),
        );
        assert_eq!(eval.evaluate(&mul), Ok(Value::Int(20)));
        
        let div = make_binary(
            BinaryOp::Div,
            make_literal(Literal::Integer(15)),
            make_literal(Literal::Integer(3)),
        );
        assert_eq!(eval.evaluate(&div), Ok(Value::Int(5)));
    }

    #[test]
    fn test_evaluate_comparison() {
        let mut eval = Evaluator::new();
        
        let eq = make_binary(
            BinaryOp::Eq,
            make_literal(Literal::Integer(5)),
            make_literal(Literal::Integer(5)),
        );
        assert_eq!(eval.evaluate(&eq), Ok(Value::Bool(true)));
        
        let ne = make_binary(
            BinaryOp::Ne,
            make_literal(Literal::Integer(5)),
            make_literal(Literal::Integer(3)),
        );
        assert_eq!(eval.evaluate(&ne), Ok(Value::Bool(true)));
        
        let lt = make_binary(
            BinaryOp::Lt,
            make_literal(Literal::Integer(3)),
            make_literal(Literal::Integer(5)),
        );
        assert_eq!(eval.evaluate(&lt), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_evaluate_bindings() {
        let mut eval = Evaluator::new();
        
        eval.set_binding("x".to_string(), Value::Int(42));
        assert_eq!(eval.get_binding("x"), Some(&Value::Int(42)));
        
        let id_expr = Expr {
            kind: ExprKind::Identifier("x".to_string()),
            span: Span::new(0, 0),
            attributes: vec![],
        };
        assert_eq!(eval.evaluate(&id_expr), Ok(Value::Int(42)));
        
        eval.clear_bindings();
        assert_eq!(eval.get_binding("x"), None);
    }

    #[test]
    fn test_division_by_zero() {
        let mut eval = Evaluator::new();
        
        let div = make_binary(
            BinaryOp::Div,
            make_literal(Literal::Integer(10)),
            make_literal(Literal::Integer(0)),
        );
        assert!(eval.evaluate(&div).is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let mut eval = Evaluator::new();
        
        let id_expr = Expr {
            kind: ExprKind::Identifier("undefined".to_string()),
            span: Span::new(0, 0),
            attributes: vec![],
        };
        
        let result = eval.evaluate(&id_expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }
}