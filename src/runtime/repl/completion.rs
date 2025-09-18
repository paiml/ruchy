//! REPL Tab Completion Engine
//!
//! Provides intelligent tab completion for commands, keywords, and variables.

/// Tab completion engine for the REPL
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
            "fn", "let", "if", "else", "for", "while", "match", "true", "false",
            "nil", "return", "break", "continue", "in", "mut", "struct", "enum",
            "impl", "trait", "pub", "use", "mod", "type", "const", "static",
        ].into_iter().map(String::from).collect();

        let commands = vec![
            ":help", ":quit", ":exit", ":clear", ":history", ":reset",
            ":mode", ":debug", ":ast", ":transpile", ":bench", ":load",
            ":save", ":vars", ":funcs", ":types",
        ].into_iter().map(String::from).collect();

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
}