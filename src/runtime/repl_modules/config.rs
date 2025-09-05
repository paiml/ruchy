//! REPL configuration module
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

/// REPL configuration
#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub history_file: String,
    pub max_history: usize,
    pub prompt: String,
    pub multiline_prompt: String,
    pub auto_indent: bool,
    pub colored_output: bool,
    pub vi_mode: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            history_file: ".ruchy_history".to_string(),
            max_history: 1000,
            prompt: "ruchy> ".to_string(),
            multiline_prompt: "....> ".to_string(),
            auto_indent: true,
            colored_output: true,
            vi_mode: false,
        }
    }
}

impl ReplConfig {
    /// Create a new REPL configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the history file path
    pub fn with_history_file(mut self, path: String) -> Self {
        self.history_file = path;
        self
    }

    /// Set the maximum history size
    pub fn with_max_history(mut self, size: usize) -> Self {
        self.max_history = size;
        self
    }

    /// Set the main prompt
    pub fn with_prompt(mut self, prompt: String) -> Self {
        self.prompt = prompt;
        self
    }

    /// Set the multiline prompt
    pub fn with_multiline_prompt(mut self, prompt: String) -> Self {
        self.multiline_prompt = prompt;
        self
    }

    /// Enable or disable auto-indent
    pub fn with_auto_indent(mut self, enabled: bool) -> Self {
        self.auto_indent = enabled;
        self
    }

    /// Enable or disable colored output
    pub fn with_colored_output(mut self, enabled: bool) -> Self {
        self.colored_output = enabled;
        self
    }

    /// Enable or disable vi mode
    pub fn with_vi_mode(mut self, enabled: bool) -> Self {
        self.vi_mode = enabled;
        self
    }
}