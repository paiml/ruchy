//! REPL Tab Completion Engine
//!
//! Provides intelligent tab completion for commands, keywords, and variables.

/// Tab completion engine for the REPL
#[derive(Debug)]
pub struct CompletionEngine {
    /// Built-in keywords to complete
    keywords: Vec<String>,
    /// Built-in commands to complete
    commands: Vec<String>,
}

impl CompletionEngine {
    /// Create a new completion engine (complexity: 3)
    pub fn new() -> Self {
        let keywords = vec![
            "fn", "let", "if", "else", "for", "while", "match", "true", "false", "nil", "return",
            "break", "continue", "in", "mut", "struct", "enum", "impl", "trait", "pub", "use",
            "mod", "type", "const", "static",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let commands = vec![
            ":help",
            ":quit",
            ":exit",
            ":clear",
            ":history",
            ":reset",
            ":mode",
            ":debug",
            ":ast",
            ":transpile",
            ":bench",
            ":load",
            ":save",
            ":vars",
            ":funcs",
            ":types",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        Self { keywords, commands }
    }

    /// Get completion suggestions for input (complexity: 6)
    pub fn complete(&self, input: &str) -> Vec<String> {
        let input = input.trim();

        if input.is_empty() {
            return Vec::new();
        }

        let mut completions = Vec::new();

        // Command completion
        if input.starts_with(':') {
            for cmd in &self.commands {
                if cmd.starts_with(input) {
                    completions.push(cmd.clone());
                }
            }
        } else {
            // Keyword completion
            for keyword in &self.keywords {
                if keyword.starts_with(input) {
                    completions.push(keyword.clone());
                }
            }
        }

        completions.sort();
        completions
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_engine_creation() {
        let engine = CompletionEngine::new();
        assert!(!engine.keywords.is_empty());
        assert!(!engine.commands.is_empty());
    }

    #[test]
    fn test_command_completion() {
        let engine = CompletionEngine::new();
        let completions = engine.complete(":he");
        assert!(completions.contains(&":help".to_string()));
    }

    #[test]
    fn test_keyword_completion() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("fn");
        assert!(completions.contains(&"fn".to_string()));
    }

    #[test]
    fn test_empty_input() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("");
        assert!(completions.is_empty());
    }

    // === Additional tests for improved coverage ===

    #[test]
    fn test_completion_engine_default() {
        let engine = CompletionEngine::default();
        assert!(!engine.keywords.is_empty());
        assert!(!engine.commands.is_empty());
    }

    #[test]
    fn test_completion_engine_debug() {
        let engine = CompletionEngine::new();
        let debug_str = format!("{:?}", engine);
        assert!(debug_str.contains("CompletionEngine"));
        assert!(debug_str.contains("keywords"));
        assert!(debug_str.contains("commands"));
    }

    #[test]
    fn test_keyword_completion_let() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("le");
        assert!(completions.contains(&"let".to_string()));
    }

    #[test]
    fn test_keyword_completion_if_else() {
        let engine = CompletionEngine::new();
        let if_completions = engine.complete("if");
        assert!(if_completions.contains(&"if".to_string()));

        let else_completions = engine.complete("el");
        assert!(else_completions.contains(&"else".to_string()));
    }

    #[test]
    fn test_keyword_completion_for_while() {
        let engine = CompletionEngine::new();

        let for_completions = engine.complete("fo");
        assert!(for_completions.contains(&"for".to_string()));

        let while_completions = engine.complete("wh");
        assert!(while_completions.contains(&"while".to_string()));
    }

    #[test]
    fn test_keyword_completion_match() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("ma");
        assert!(completions.contains(&"match".to_string()));
    }

    #[test]
    fn test_keyword_completion_true_false() {
        let engine = CompletionEngine::new();

        let true_completions = engine.complete("tr");
        assert!(true_completions.contains(&"true".to_string()));
        assert!(true_completions.contains(&"trait".to_string()));

        let false_completions = engine.complete("fa");
        assert!(false_completions.contains(&"false".to_string()));
    }

    #[test]
    fn test_keyword_completion_return_break_continue() {
        let engine = CompletionEngine::new();

        let return_completions = engine.complete("re");
        assert!(return_completions.contains(&"return".to_string()));

        let break_completions = engine.complete("br");
        assert!(break_completions.contains(&"break".to_string()));

        let continue_completions = engine.complete("co");
        assert!(continue_completions.contains(&"continue".to_string()));
        assert!(continue_completions.contains(&"const".to_string()));
    }

    #[test]
    fn test_keyword_completion_mut_pub_use_mod() {
        let engine = CompletionEngine::new();

        let mut_completions = engine.complete("mu");
        assert!(mut_completions.contains(&"mut".to_string()));

        let pub_completions = engine.complete("pu");
        assert!(pub_completions.contains(&"pub".to_string()));

        let use_completions = engine.complete("us");
        assert!(use_completions.contains(&"use".to_string()));

        let mod_completions = engine.complete("mo");
        assert!(mod_completions.contains(&"mod".to_string()));
    }

    #[test]
    fn test_keyword_completion_struct_enum_impl() {
        let engine = CompletionEngine::new();

        let struct_completions = engine.complete("st");
        assert!(struct_completions.contains(&"struct".to_string()));
        assert!(struct_completions.contains(&"static".to_string()));

        let enum_completions = engine.complete("en");
        assert!(enum_completions.contains(&"enum".to_string()));

        let impl_completions = engine.complete("im");
        assert!(impl_completions.contains(&"impl".to_string()));
    }

    #[test]
    fn test_keyword_completion_type_const_static() {
        let engine = CompletionEngine::new();

        let type_completions = engine.complete("ty");
        assert!(type_completions.contains(&"type".to_string()));

        let const_completions = engine.complete("cons");
        assert!(const_completions.contains(&"const".to_string()));

        let static_completions = engine.complete("stat");
        assert!(static_completions.contains(&"static".to_string()));
    }

    #[test]
    fn test_command_completion_quit_exit() {
        let engine = CompletionEngine::new();

        let quit_completions = engine.complete(":qu");
        assert!(quit_completions.contains(&":quit".to_string()));

        let exit_completions = engine.complete(":ex");
        assert!(exit_completions.contains(&":exit".to_string()));
    }

    #[test]
    fn test_command_completion_clear_history_reset() {
        let engine = CompletionEngine::new();

        let clear_completions = engine.complete(":cl");
        assert!(clear_completions.contains(&":clear".to_string()));

        let history_completions = engine.complete(":hi");
        assert!(history_completions.contains(&":history".to_string()));

        let reset_completions = engine.complete(":re");
        assert!(reset_completions.contains(&":reset".to_string()));
    }

    #[test]
    fn test_command_completion_mode_debug_ast() {
        let engine = CompletionEngine::new();

        let mode_completions = engine.complete(":mo");
        assert!(mode_completions.contains(&":mode".to_string()));

        let debug_completions = engine.complete(":de");
        assert!(debug_completions.contains(&":debug".to_string()));

        let ast_completions = engine.complete(":as");
        assert!(ast_completions.contains(&":ast".to_string()));
    }

    #[test]
    fn test_command_completion_transpile_bench() {
        let engine = CompletionEngine::new();

        let transpile_completions = engine.complete(":tr");
        assert!(transpile_completions.contains(&":transpile".to_string()));

        let bench_completions = engine.complete(":be");
        assert!(bench_completions.contains(&":bench".to_string()));
    }

    #[test]
    fn test_command_completion_load_save() {
        let engine = CompletionEngine::new();

        let load_completions = engine.complete(":lo");
        assert!(load_completions.contains(&":load".to_string()));

        let save_completions = engine.complete(":sa");
        assert!(save_completions.contains(&":save".to_string()));
    }

    #[test]
    fn test_command_completion_vars_funcs_types() {
        let engine = CompletionEngine::new();

        let vars_completions = engine.complete(":va");
        assert!(vars_completions.contains(&":vars".to_string()));

        let funcs_completions = engine.complete(":fu");
        assert!(funcs_completions.contains(&":funcs".to_string()));

        let types_completions = engine.complete(":ty");
        assert!(types_completions.contains(&":types".to_string()));
    }

    #[test]
    fn test_completion_no_matches() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("xyz");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_command_completion_no_matches() {
        let engine = CompletionEngine::new();
        let completions = engine.complete(":xyz");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_completion_whitespace_only() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("   ");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_completion_sorted_results() {
        let engine = CompletionEngine::new();
        let completions = engine.complete(":");
        // Results should be sorted alphabetically
        let mut sorted = completions.clone();
        sorted.sort();
        assert_eq!(completions, sorted);
    }

    #[test]
    fn test_completion_exact_match() {
        let engine = CompletionEngine::new();

        // Exact keyword match should return itself
        let completions = engine.complete("nil");
        assert!(completions.contains(&"nil".to_string()));

        // Exact command match should return itself
        let completions = engine.complete(":help");
        assert!(completions.contains(&":help".to_string()));
    }

    #[test]
    fn test_completion_colon_only() {
        let engine = CompletionEngine::new();
        let completions = engine.complete(":");
        // Should return all commands
        assert!(!completions.is_empty());
        assert!(completions.len() >= 10); // We have many commands
    }

    #[test]
    fn test_keyword_in() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("in");
        assert!(completions.contains(&"in".to_string()));
    }

    #[test]
    fn test_keyword_impl_completion() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("im");
        assert!(completions.contains(&"impl".to_string()));
    }

    #[test]
    fn test_all_keywords_present() {
        let engine = CompletionEngine::new();
        let expected_keywords = vec![
            "fn", "let", "if", "else", "for", "while", "match", "true", "false",
            "nil", "return", "break", "continue", "in", "mut", "struct", "enum",
            "impl", "trait", "pub", "use", "mod", "type", "const", "static",
        ];
        for keyword in expected_keywords {
            assert!(
                engine.keywords.contains(&keyword.to_string()),
                "Missing keyword: {}",
                keyword
            );
        }
    }

    #[test]
    fn test_all_commands_present() {
        let engine = CompletionEngine::new();
        let expected_commands = vec![
            ":help", ":quit", ":exit", ":clear", ":history", ":reset",
            ":mode", ":debug", ":ast", ":transpile", ":bench",
            ":load", ":save", ":vars", ":funcs", ":types",
        ];
        for command in expected_commands {
            assert!(
                engine.commands.contains(&command.to_string()),
                "Missing command: {}",
                command
            );
        }
    }
}
