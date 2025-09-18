//! Tab Completion Engine - EXTREME Quality
//!
//! Provides intelligent tab completion with <10 complexity

use std::collections::HashSet;

/// Tab completion provider
pub struct CompletionEngine {
    keywords: HashSet<String>,
    commands: HashSet<String>,
}

impl CompletionEngine {
    /// Create new completion engine (complexity: 3)
    pub fn new() -> Self {
        let mut engine = Self {
            keywords: HashSet::new(),
            commands: HashSet::new(),
        };
        engine.init_keywords();
        engine.init_commands();
        engine
    }

    /// Initialize language keywords (complexity: 5)
    fn init_keywords(&mut self) {
        let keywords = [
            "fn", "let", "if", "else", "for", "while", "loop",
            "match", "return", "break", "continue", "true", "false",
            "struct", "enum", "impl", "trait", "type", "const",
        ];
        for kw in keywords {
            self.keywords.insert(kw.to_string());
        }
    }

    /// Initialize REPL commands (complexity: 4)
    fn init_commands(&mut self) {
        let commands = [
            ":help", ":quit", ":history", ":clear", ":ast",
            ":tokens", ":type", ":save", ":load", ":debug",
        ];
        for cmd in commands {
            self.commands.insert(cmd.to_string());
        }
    }

    /// Get completions for input (complexity: 7)
    pub fn complete(&self, input: &str) -> Vec<String> {
        let mut results = Vec::new();

        // Command completions
        if input.starts_with(':') {
            for cmd in &self.commands {
                if cmd.starts_with(input) {
                    results.push(cmd.clone());
                }
            }
        } else {
            // Keyword completions
            for kw in &self.keywords {
                if kw.starts_with(input) {
                    results.push(kw.clone());
                }
            }
        }

        results.sort();
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_completion() {
        let engine = CompletionEngine::new();
        let completions = engine.complete("fn");
        assert!(completions.contains(&"fn".to_string()));
    }

    #[test]
    fn test_command_completion() {
        let engine = CompletionEngine::new();
        let completions = engine.complete(":he");
        assert!(completions.contains(&":help".to_string()));
    }
}