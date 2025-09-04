//! Unit tests for the history module
//! Target: 80% coverage of history management functionality

#[cfg(test)]
mod history_tests {
    use ruchy::runtime::repl::history::{HistoryManager, HistoryConfig, SearchOptions};
    use ruchy::runtime::repl::Value;
    use std::collections::VecDeque;
    
    fn create_test_config() -> HistoryConfig {
        HistoryConfig {
            max_size: 100,
            save_results: true,
            deduplicate: true,
            persist_to_file: false,
            file_path: None,
        }
    }
    
    #[test]
    fn test_history_creation() {
        let config = create_test_config();
        let history = HistoryManager::new(config);
        
        assert_eq!(history.command_count(), 0);
        assert_eq!(history.result_count(), 0);
        assert!(!history.has_previous());
        assert!(!history.has_next());
    }
    
    #[test]
    fn test_add_command() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("test1".to_string());
        assert_eq!(history.command_count(), 1);
        
        history.add_command("test2".to_string());
        assert_eq!(history.command_count(), 2);
        
        history.add_command("test3".to_string());
        assert_eq!(history.command_count(), 3);
    }
    
    #[test]
    fn test_add_result() {
        let mut config = create_test_config();
        config.save_results = true;
        let mut history = HistoryManager::new(config);
        
        history.add_result(Value::Int(42));
        assert_eq!(history.result_count(), 1);
        
        history.add_result(Value::String("hello".to_string()));
        assert_eq!(history.result_count(), 2);
    }
    
    #[test]
    fn test_deduplication() {
        let mut config = create_test_config();
        config.deduplicate = true;
        let mut history = HistoryManager::new(config);
        
        history.add_command("test".to_string());
        history.add_command("test".to_string());
        history.add_command("test".to_string());
        
        // With deduplication, consecutive duplicates should be removed
        assert_eq!(history.command_count(), 1);
    }
    
    #[test]
    fn test_no_deduplication() {
        let mut config = create_test_config();
        config.deduplicate = false;
        let mut history = HistoryManager::new(config);
        
        history.add_command("test".to_string());
        history.add_command("test".to_string());
        history.add_command("test".to_string());
        
        // Without deduplication, all commands are kept
        assert_eq!(history.command_count(), 3);
    }
    
    #[test]
    fn test_max_size_limit() {
        let mut config = create_test_config();
        config.max_size = 3;
        let mut history = HistoryManager::new(config);
        
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_command("cmd3".to_string());
        history.add_command("cmd4".to_string());
        
        // Should only keep last 3 commands
        assert_eq!(history.command_count(), 3);
        let recent = history.get_recent_commands(10);
        assert!(!recent.contains(&"cmd1".to_string()));
        assert!(recent.contains(&"cmd4".to_string()));
    }
    
    #[test]
    fn test_navigation_previous() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("first".to_string());
        history.add_command("second".to_string());
        history.add_command("third".to_string());
        
        assert_eq!(history.get_previous(), Some("third".to_string()));
        assert_eq!(history.get_previous(), Some("second".to_string()));
        assert_eq!(history.get_previous(), Some("first".to_string()));
        assert_eq!(history.get_previous(), None); // At beginning
    }
    
    #[test]
    fn test_navigation_next() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("first".to_string());
        history.add_command("second".to_string());
        history.add_command("third".to_string());
        
        // Navigate to beginning
        history.get_previous();
        history.get_previous();
        history.get_previous();
        
        assert_eq!(history.get_next(), Some("second".to_string()));
        assert_eq!(history.get_next(), Some("third".to_string()));
        assert_eq!(history.get_next(), None); // At end
    }
    
    #[test]
    fn test_navigation_reset() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("first".to_string());
        history.add_command("second".to_string());
        
        history.get_previous();
        history.get_previous();
        
        history.reset_position();
        
        // After reset, should be at end
        assert!(!history.has_next());
        assert!(history.has_previous());
    }
    
    #[test]
    fn test_search_basic() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("test command".to_string());
        history.add_command("another test".to_string());
        history.add_command("final command".to_string());
        
        let options = SearchOptions {
            case_sensitive: false,
            regex: false,
            limit: None,
        };
        
        let results = history.search("test", &options);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"test command".to_string()));
        assert!(results.contains(&"another test".to_string()));
    }
    
    #[test]
    fn test_search_case_sensitive() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("Test Command".to_string());
        history.add_command("test command".to_string());
        
        let options = SearchOptions {
            case_sensitive: true,
            regex: false,
            limit: None,
        };
        
        let results = history.search("test", &options);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"test command".to_string()));
    }
    
    #[test]
    fn test_search_with_limit() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        for i in 0..10 {
            history.add_command(format!("test {}", i));
        }
        
        let options = SearchOptions {
            case_sensitive: false,
            regex: false,
            limit: Some(3),
        };
        
        let results = history.search("test", &options);
        assert_eq!(results.len(), 3);
    }
    
    #[test]
    fn test_get_recent_commands() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_command("cmd3".to_string());
        
        let recent = history.get_recent_commands(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], "cmd3");
        assert_eq!(recent[1], "cmd2");
    }
    
    #[test]
    fn test_get_recent_results() {
        let mut config = create_test_config();
        config.save_results = true;
        let mut history = HistoryManager::new(config);
        
        history.add_result(Value::Int(1));
        history.add_result(Value::Int(2));
        history.add_result(Value::Int(3));
        
        let recent = history.get_recent_results(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], Value::Int(3));
        assert_eq!(recent[1], Value::Int(2));
    }
    
    #[test]
    fn test_clear() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_result(Value::Int(42));
        
        history.clear();
        
        assert_eq!(history.command_count(), 0);
        assert_eq!(history.result_count(), 0);
        assert!(!history.has_previous());
    }
    
    #[test]
    fn test_format_display() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("first".to_string());
        history.add_command("second".to_string());
        
        let display = history.format_display();
        assert!(display.contains("Command History"));
        assert!(display.contains("first"));
        assert!(display.contains("second"));
    }
    
    #[test]
    fn test_get_command_at_index() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("cmd0".to_string());
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        
        assert_eq!(history.get_command_at(0), Some("cmd0".to_string()));
        assert_eq!(history.get_command_at(1), Some("cmd1".to_string()));
        assert_eq!(history.get_command_at(2), Some("cmd2".to_string()));
        assert_eq!(history.get_command_at(3), None);
    }
    
    #[test]
    fn test_get_result_at_index() {
        let mut config = create_test_config();
        config.save_results = true;
        let mut history = HistoryManager::new(config);
        
        history.add_result(Value::Int(10));
        history.add_result(Value::Int(20));
        history.add_result(Value::Int(30));
        
        assert_eq!(history.get_result_at(0), Some(Value::Int(10)));
        assert_eq!(history.get_result_at(1), Some(Value::Int(20)));
        assert_eq!(history.get_result_at(2), Some(Value::Int(30)));
        assert_eq!(history.get_result_at(3), None);
    }
    
    #[test]
    fn test_persistence_disabled() {
        let mut config = create_test_config();
        config.persist_to_file = false;
        let history = HistoryManager::new(config);
        
        // Should not attempt to load from file
        assert_eq!(history.command_count(), 0);
    }
    
    #[test]
    fn test_export_to_json() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_result(Value::Int(42));
        
        let json = history.export_json();
        assert!(json.is_ok());
        
        let json_str = json.unwrap();
        assert!(json_str.contains("\"commands\""));
        assert!(json_str.contains("cmd1"));
        assert!(json_str.contains("cmd2"));
    }
    
    #[test]
    fn test_import_from_json() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        let json = r#"{
            "commands": ["imported1", "imported2"],
            "results": []
        }"#;
        
        let result = history.import_json(json);
        assert!(result.is_ok());
        
        assert_eq!(history.command_count(), 2);
        assert_eq!(history.get_command_at(0), Some("imported1".to_string()));
        assert_eq!(history.get_command_at(1), Some("imported2".to_string()));
    }
    
    #[test]
    fn test_command_frequency() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_command("cmd1".to_string());
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        
        let freq = history.get_command_frequency();
        assert_eq!(freq.get("cmd1"), Some(&3));
        assert_eq!(freq.get("cmd2"), Some(&2));
    }
    
    #[test]
    fn test_trim_to_size() {
        let mut config = create_test_config();
        config.max_size = 5;
        let mut history = HistoryManager::new(config);
        
        for i in 0..10 {
            history.add_command(format!("cmd{}", i));
        }
        
        assert_eq!(history.command_count(), 5);
        
        // Should keep most recent commands
        let recent = history.get_recent_commands(10);
        assert!(recent.contains(&"cmd9".to_string()));
        assert!(!recent.contains(&"cmd0".to_string()));
    }
    
    #[test]
    fn test_results_not_saved_when_disabled() {
        let mut config = create_test_config();
        config.save_results = false;
        let mut history = HistoryManager::new(config);
        
        history.add_result(Value::Int(42));
        history.add_result(Value::String("test".to_string()));
        
        assert_eq!(history.result_count(), 0);
    }
    
    #[test]
    fn test_complex_navigation_scenario() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("a".to_string());
        history.add_command("b".to_string());
        history.add_command("c".to_string());
        
        // Navigate back
        assert_eq!(history.get_previous(), Some("c".to_string()));
        assert_eq!(history.get_previous(), Some("b".to_string()));
        
        // Navigate forward
        assert_eq!(history.get_next(), Some("c".to_string()));
        
        // Add new command resets position
        history.add_command("d".to_string());
        
        // Should now be at end
        assert!(!history.has_next());
        assert_eq!(history.get_previous(), Some("d".to_string()));
    }
    
    #[test]
    fn test_search_regex() {
        let config = create_test_config();
        let mut history = HistoryManager::new(config);
        
        history.add_command("test123".to_string());
        history.add_command("test456".to_string());
        history.add_command("other789".to_string());
        
        let options = SearchOptions {
            case_sensitive: false,
            regex: true,
            limit: None,
        };
        
        // Search for pattern "test[0-9]+"
        let results = history.search(r"test\d+", &options);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"test123".to_string()));
        assert!(results.contains(&"test456".to_string()));
    }
    
    #[test]
    fn test_stats() {
        let mut config = create_test_config();
        config.save_results = true;
        let mut history = HistoryManager::new(config);
        
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_result(Value::Int(42));
        
        let stats = history.get_stats();
        assert_eq!(stats.total_commands, 2);
        assert_eq!(stats.total_results, 1);
        assert_eq!(stats.unique_commands, 2);
    }
}

#[cfg(test)]
mod history_config_tests {
    use ruchy::runtime::repl::history::HistoryConfig;
    use std::path::PathBuf;
    
    #[test]
    fn test_default_config() {
        let config = HistoryConfig::default();
        assert_eq!(config.max_size, 1000);
        assert!(config.save_results);
        assert!(config.deduplicate);
        assert!(!config.persist_to_file);
        assert!(config.file_path.is_none());
    }
    
    #[test]
    fn test_custom_config() {
        let config = HistoryConfig {
            max_size: 500,
            save_results: false,
            deduplicate: false,
            persist_to_file: true,
            file_path: Some(PathBuf::from("/tmp/history.json")),
        };
        
        assert_eq!(config.max_size, 500);
        assert!(!config.save_results);
        assert!(!config.deduplicate);
        assert!(config.persist_to_file);
        assert_eq!(config.file_path, Some(PathBuf::from("/tmp/history.json")));
    }
}

#[cfg(test)]
mod search_options_tests {
    use ruchy::runtime::repl::history::SearchOptions;
    
    #[test]
    fn test_default_search_options() {
        let options = SearchOptions::default();
        assert!(!options.case_sensitive);
        assert!(!options.regex);
        assert!(options.limit.is_none());
    }
    
    #[test]
    fn test_custom_search_options() {
        let options = SearchOptions {
            case_sensitive: true,
            regex: true,
            limit: Some(10),
        };
        
        assert!(options.case_sensitive);
        assert!(options.regex);
        assert_eq!(options.limit, Some(10));
    }
}