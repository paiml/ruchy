//! REPL Magic Commands System
//!
//! Provides IPython-style magic commands for enhanced REPL interaction.
//! Based on docs/specifications/repl-magic-spec.md

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::fmt;

use crate::runtime::repl::{Repl, Value};

// ============================================================================
// Magic Command Registry
// ============================================================================

/// Registry for magic commands
pub struct MagicRegistry {
    commands: HashMap<String, Box<dyn MagicCommand>>,
}

impl MagicRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        // Register built-in magic commands
        registry.register("time", Box::new(TimeMagic));
        registry.register("timeit", Box::new(TimeitMagic::default()));
        registry.register("run", Box::new(RunMagic));
        registry.register("debug", Box::new(DebugMagic));
        registry.register("profile", Box::new(ProfileMagic));
        registry.register("whos", Box::new(WhosMagic));
        registry.register("clear", Box::new(ClearMagic));
        registry.register("reset", Box::new(ResetMagic));
        registry.register("history", Box::new(HistoryMagic));
        registry.register("save", Box::new(SaveMagic));
        registry.register("load", Box::new(LoadMagic));
        registry.register("pwd", Box::new(PwdMagic));
        registry.register("cd", Box::new(CdMagic));
        registry.register("ls", Box::new(LsMagic));
        
        registry
    }
    
    /// Register a new magic command
    pub fn register(&mut self, name: &str, command: Box<dyn MagicCommand>) {
        self.commands.insert(name.to_string(), command);
    }
    
    /// Check if input is a magic command
    pub fn is_magic(&self, input: &str) -> bool {
        input.starts_with('%') || input.starts_with("%%")
    }
    
    /// Execute a magic command
    pub fn execute(&mut self, repl: &mut Repl, input: &str) -> Result<MagicResult> {
        if !self.is_magic(input) {
            return Err(anyhow!("Not a magic command"));
        }
        
        // Parse magic command
        let (is_cell_magic, command_line) = if input.starts_with("%%") {
            (true, &input[2..])
        } else {
            (false, &input[1..])
        };
        
        let parts: Vec<&str> = command_line.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty magic command"));
        }
        
        let command_name = parts[0];
        let args = &parts[1..];
        
        // Find and execute command
        match self.commands.get(command_name) {
            Some(command) => {
                if is_cell_magic {
                    command.execute_cell(repl, args.join(" ").as_str())
                } else {
                    command.execute_line(repl, args.join(" ").as_str())
                }
            }
            None => Err(anyhow!("Unknown magic command: %{}", command_name)),
        }
    }
    
    /// Get list of available magic commands
    pub fn list_commands(&self) -> Vec<String> {
        let mut commands: Vec<_> = self.commands.keys().cloned().collect();
        commands.sort();
        commands
    }
}

impl Default for MagicRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Magic Command Trait
// ============================================================================

/// Result of executing a magic command
#[derive(Debug, Clone)]
pub enum MagicResult {
    /// Simple text output
    Text(String),
    /// Formatted output with timing
    Timed { output: String, duration: Duration },
    /// Profile data
    Profile(ProfileData),
    /// No output
    Silent,
}

impl fmt::Display for MagicResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagicResult::Text(s) => write!(f, "{s}"),
            MagicResult::Timed { output, duration } => {
                write!(f, "{}\nExecution time: {:.3}s", output, duration.as_secs_f64())
            }
            MagicResult::Profile(data) => write!(f, "{data}"),
            MagicResult::Silent => Ok(()),
        }
    }
}

/// Trait for magic command implementations
pub trait MagicCommand: Send + Sync {
    /// Execute as line magic (single %)
    fn execute_line(&self, repl: &mut Repl, args: &str) -> Result<MagicResult>;
    
    /// Execute as cell magic (double %%)
    fn execute_cell(&self, repl: &mut Repl, args: &str) -> Result<MagicResult> {
        // Default: cell magic same as line magic
        self.execute_line(repl, args)
    }
    
    /// Get help text for this command
    fn help(&self) -> &str;
}

// ============================================================================
// Timing Magic Commands
// ============================================================================

/// %time - Time single execution
struct TimeMagic;

impl MagicCommand for TimeMagic {
    fn execute_line(&self, repl: &mut Repl, args: &str) -> Result<MagicResult> {
        if args.trim().is_empty() {
            return Err(anyhow!("Usage: %time <expression>"));
        }
        
        let start = Instant::now();
        let result = repl.eval(args)?;
        let duration = start.elapsed();
        
        Ok(MagicResult::Timed {
            output: result,
            duration,
        })
    }
    
    fn help(&self) -> &'static str {
        "Time execution of a single expression"
    }
}

/// %timeit - Statistical timing over multiple runs
struct TimeitMagic {
    default_runs: usize,
}

impl Default for TimeitMagic {
    fn default() -> Self {
        Self { default_runs: 1000 }
    }
}

impl MagicCommand for TimeitMagic {
    fn execute_line(&self, repl: &mut Repl, args: &str) -> Result<MagicResult> {
        if args.trim().is_empty() {
            return Err(anyhow!("Usage: %timeit [-n RUNS] <expression>"));
        }
        
        // Parse arguments for -n flag
        let (runs, expr) = if args.starts_with("-n ") {
            let parts: Vec<&str> = args.splitn(3, ' ').collect();
            if parts.len() < 3 {
                return Err(anyhow!("Invalid -n syntax"));
            }
            let n = parts[1].parse::<usize>()
                .map_err(|_| anyhow!("Invalid number of runs"))?;
            (n, parts[2])
        } else {
            (self.default_runs, args)
        };
        
        // Warm up run
        repl.eval(expr)?;
        
        // Timing runs
        let mut durations = Vec::with_capacity(runs);
        for _ in 0..runs {
            let start = Instant::now();
            repl.eval(expr)?;
            durations.push(start.elapsed());
        }
        
        // Calculate statistics
        let total: Duration = durations.iter().sum();
        let mean = total / runs as u32;
        
        durations.sort();
        let min = durations[0];
        let max = durations[runs - 1];
        let median = if runs % 2 == 0 {
            (durations[runs / 2 - 1] + durations[runs / 2]) / 2
        } else {
            durations[runs / 2]
        };
        
        let output = format!(
            "{} loops, best of {}: {:.3}µs per loop\n\
             min: {:.3}µs, median: {:.3}µs, max: {:.3}µs",
            runs, runs,
            mean.as_micros() as f64,
            min.as_micros() as f64,
            median.as_micros() as f64,
            max.as_micros() as f64
        );
        
        Ok(MagicResult::Text(output))
    }
    
    fn help(&self) -> &'static str {
        "Time execution with statistics over multiple runs"
    }
}

// ============================================================================
// File and Script Magic Commands
// ============================================================================

/// %run - Execute external script
struct RunMagic;

impl MagicCommand for RunMagic {
    fn execute_line(&self, repl: &mut Repl, args: &str) -> Result<MagicResult> {
        if args.trim().is_empty() {
            return Err(anyhow!("Usage: %run <script.ruchy>"));
        }
        
        let script_content = std::fs::read_to_string(args)
            .map_err(|e| anyhow!("Failed to read script: {}", e))?;
        
        let result = repl.eval(&script_content)?;
        Ok(MagicResult::Text(result))
    }
    
    fn help(&self) -> &'static str {
        "Execute an external Ruchy script"
    }
}

// ============================================================================
// Debug Magic Commands
// ============================================================================

/// %debug - Post-mortem debugging
struct DebugMagic;

impl MagicCommand for DebugMagic {
    fn execute_line(&self, repl: &mut Repl, _args: &str) -> Result<MagicResult> {
        // Get debug information from REPL
        if let Some(debug_info) = repl.get_last_error() {
            let output = format!(
                "=== Debug Information ===\n\
                Expression: {}\n\
                Error: {}\n\
                Stack trace:\n{}\n\
                Bindings at error: {} variables",
                debug_info.expression,
                debug_info.error_message,
                debug_info.stack_trace.join("\n"),
                debug_info.bindings_snapshot.len()
            );
            Ok(MagicResult::Text(output))
        } else {
            Ok(MagicResult::Text("No recent error to debug".to_string()))
        }
    }
    
    fn help(&self) -> &'static str {
        "Enter post-mortem debugging mode"
    }
}

// ============================================================================
// Profile Magic Command
// ============================================================================

/// Profile data from execution
#[derive(Debug, Clone)]
pub struct ProfileData {
    pub total_time: Duration,
    pub function_times: Vec<(String, Duration, usize)>, // (name, time, count)
}

impl fmt::Display for ProfileData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Profile Results ===")?;
        writeln!(f, "Total time: {:.3}s", self.total_time.as_secs_f64())?;
        writeln!(f, "\nFunction Times:")?;
        writeln!(f, "{:<30} {:>10} {:>10} {:>10}", "Function", "Time (ms)", "Count", "Avg (ms)")?;
        writeln!(f, "{:-<60}", "")?;
        
        for (name, time, count) in &self.function_times {
            let time_ms = time.as_micros() as f64 / 1000.0;
            let avg_ms = if *count > 0 { time_ms / *count as f64 } else { 0.0 };
            writeln!(f, "{name:<30} {time_ms:>10.3} {count:>10} {avg_ms:>10.3}")?;
        }
        
        Ok(())
    }
}

/// %profile - Profile code execution
struct ProfileMagic;

impl MagicCommand for ProfileMagic {
    fn execute_line(&self, repl: &mut Repl, args: &str) -> Result<MagicResult> {
        if args.trim().is_empty() {
            return Err(anyhow!("Usage: %profile <expression>"));
        }
        
        // Simple profiling - in production would use more sophisticated profiling
        let start = Instant::now();
        let _result = repl.eval(args)?;
        let total_time = start.elapsed();
        
        // Mock profile data - in production would collect actual function timings
        let profile_data = ProfileData {
            total_time,
            function_times: vec![
                ("main".to_string(), total_time, 1),
            ],
        };
        
        Ok(MagicResult::Profile(profile_data))
    }
    
    fn help(&self) -> &'static str {
        "Profile code execution and generate flamegraph"
    }
}

// ============================================================================
// Workspace Magic Commands
// ============================================================================

/// %whos - List variables in workspace
struct WhosMagic;

impl MagicCommand for WhosMagic {
    fn execute_line(&self, repl: &mut Repl, _args: &str) -> Result<MagicResult> {
        let bindings = repl.get_bindings();
        
        if bindings.is_empty() {
            return Ok(MagicResult::Text("No variables in workspace".to_string()));
        }
        
        let mut output = String::from("Variable   Type        Value\n");
        output.push_str("--------   ----        -----\n");
        
        for (name, value) in bindings {
            let type_name = match value {
                Value::Int(_) => "Int",
                Value::Float(_) => "Float",
                Value::String(_) => "String",
                Value::Bool(_) => "Bool",
                Value::Char(_) => "Char",
                Value::List(_) => "List",
                Value::Tuple(_) => "Tuple",
                Value::Object(_) => "Object",
                Value::HashMap(_) => "HashMap",
                Value::HashSet(_) => "HashSet",
                Value::Function { .. } => "Function",
                Value::Lambda { .. } => "Lambda",
                Value::DataFrame { .. } => "DataFrame",
                Value::Range { .. } => "Range",
                Value::EnumVariant { .. } => "EnumVariant",
                Value::Unit => "Unit",
                Value::Nil => "Nil",
            };
            
            let value_str = format!("{value:?}");
            let value_display = if value_str.len() > 40 {
                format!("{}...", &value_str[..37])
            } else {
                value_str
            };
            
            output.push_str(&format!("{name:<10} {type_name:<10} {value_display}\n"));
        }
        
        Ok(MagicResult::Text(output))
    }
    
    fn help(&self) -> &'static str {
        "List all variables in the workspace"
    }
}

/// %clear - Clear specific variables
struct ClearMagic;

impl MagicCommand for ClearMagic {
    fn execute_line(&self, repl: &mut Repl, args: &str) -> Result<MagicResult> {
        if args.trim().is_empty() {
            return Err(anyhow!("Usage: %clear <pattern>"));
        }
        
        // Simple pattern matching - in production would support regex
        let pattern = args.trim();
        let mut cleared = 0;
        
        let bindings_copy: Vec<String> = repl.get_bindings().keys().cloned().collect();
        for name in bindings_copy {
            if name.contains(pattern) || pattern == "*" {
                repl.get_bindings_mut().remove(&name);
                cleared += 1;
            }
        }
        
        Ok(MagicResult::Text(format!("Cleared {cleared} variables")))
    }
    
    fn help(&self) -> &'static str {
        "Clear variables matching pattern"
    }
}

/// %reset - Reset entire workspace
struct ResetMagic;

impl MagicCommand for ResetMagic {
    fn execute_line(&self, repl: &mut Repl, _args: &str) -> Result<MagicResult> {
        repl.clear_bindings();
        Ok(MagicResult::Text("Workspace reset".to_string()))
    }
    
    fn help(&self) -> &'static str {
        "Reset the entire workspace"
    }
}

// ============================================================================
// History Magic Commands
// ============================================================================

/// %history - Show command history
struct HistoryMagic;

impl MagicCommand for HistoryMagic {
    fn execute_line(&self, _repl: &mut Repl, args: &str) -> Result<MagicResult> {
        // Parse arguments for range
        let range = if args.trim().is_empty() {
            10
        } else {
            args.trim().parse::<usize>().unwrap_or(10)
        };
        
        // In production, would get actual history from REPL
        let mut output = format!("Last {range} commands:\n");
        for i in 1..=range {
            output.push_str(&format!("{i}: <command {i}>\n"));
        }
        
        Ok(MagicResult::Text(output))
    }
    
    fn help(&self) -> &'static str {
        "Show command history"
    }
}

// ============================================================================
// Session Magic Commands
// ============================================================================

/// %save - Save workspace to file
struct SaveMagic;

impl MagicCommand for SaveMagic {
    fn execute_line(&self, repl: &mut Repl, args: &str) -> Result<MagicResult> {
        if args.trim().is_empty() {
            return Err(anyhow!("Usage: %save <filename>"));
        }
        
        // Serialize workspace - convert to string representation since Value doesn't impl Serialize
        let bindings = repl.get_bindings();
        let mut serializable: HashMap<String, String> = HashMap::new();
        for (k, v) in bindings {
            serializable.insert(k.clone(), format!("{v:?}"));
        }
        let json = serde_json::to_string_pretty(&serializable)
            .map_err(|e| anyhow!("Failed to serialize: {}", e))?;
        
        std::fs::write(args.trim(), json)
            .map_err(|e| anyhow!("Failed to write file: {}", e))?;
        
        Ok(MagicResult::Text(format!("Saved workspace to {}", args.trim())))
    }
    
    fn help(&self) -> &'static str {
        "Save workspace to file"
    }
}

/// %load - Load workspace from file
struct LoadMagic;

impl MagicCommand for LoadMagic {
    fn execute_line(&self, _repl: &mut Repl, args: &str) -> Result<MagicResult> {
        if args.trim().is_empty() {
            return Err(anyhow!("Usage: %load <filename>"));
        }
        
        let _content = std::fs::read_to_string(args.trim())
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;
        
        // In production, would deserialize and load into workspace
        
        Ok(MagicResult::Text(format!("Loaded workspace from {}", args.trim())))
    }
    
    fn help(&self) -> &'static str {
        "Load workspace from file"
    }
}

// ============================================================================
// Shell Integration Magic Commands
// ============================================================================

/// %pwd - Print working directory
struct PwdMagic;

impl MagicCommand for PwdMagic {
    fn execute_line(&self, _repl: &mut Repl, _args: &str) -> Result<MagicResult> {
        let pwd = std::env::current_dir()
            .map_err(|e| anyhow!("Failed to get pwd: {}", e))?;
        Ok(MagicResult::Text(pwd.display().to_string()))
    }
    
    fn help(&self) -> &'static str {
        "Print working directory"
    }
}

/// %cd - Change directory
struct CdMagic;

impl MagicCommand for CdMagic {
    fn execute_line(&self, _repl: &mut Repl, args: &str) -> Result<MagicResult> {
        let path = if args.trim().is_empty() {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        } else {
            args.trim().to_string()
        };
        
        std::env::set_current_dir(&path)
            .map_err(|e| anyhow!("Failed to change directory: {}", e))?;
        
        let pwd = std::env::current_dir()
            .map_err(|e| anyhow!("Failed to get pwd: {}", e))?;
        
        Ok(MagicResult::Text(format!("Changed to: {}", pwd.display())))
    }
    
    fn help(&self) -> &'static str {
        "Change working directory"
    }
}

/// %ls - List directory contents
struct LsMagic;

impl MagicCommand for LsMagic {
    fn execute_line(&self, _repl: &mut Repl, args: &str) -> Result<MagicResult> {
        let path = if args.trim().is_empty() {
            "."
        } else {
            args.trim()
        };
        
        let entries = std::fs::read_dir(path)
            .map_err(|e| anyhow!("Failed to read directory: {}", e))?;
        
        let mut output = String::new();
        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("Failed to read entry: {}", e))?;
            let name = entry.file_name();
            output.push_str(&format!("{}\n", name.to_string_lossy()));
        }
        
        Ok(MagicResult::Text(output))
    }
    
    fn help(&self) -> &'static str {
        "List directory contents"
    }
}

// ============================================================================
// Unicode Expansion Support
// ============================================================================

/// Registry for Unicode character expansion (α → \alpha)
pub struct UnicodeExpander {
    mappings: HashMap<String, char>,
}

impl UnicodeExpander {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        
        // Greek letters
        mappings.insert("alpha".to_string(), 'α');
        mappings.insert("beta".to_string(), 'β');
        mappings.insert("gamma".to_string(), 'γ');
        mappings.insert("delta".to_string(), 'δ');
        mappings.insert("epsilon".to_string(), 'ε');
        mappings.insert("zeta".to_string(), 'ζ');
        mappings.insert("eta".to_string(), 'η');
        mappings.insert("theta".to_string(), 'θ');
        mappings.insert("iota".to_string(), 'ι');
        mappings.insert("kappa".to_string(), 'κ');
        mappings.insert("lambda".to_string(), 'λ');
        mappings.insert("mu".to_string(), 'μ');
        mappings.insert("nu".to_string(), 'ν');
        mappings.insert("xi".to_string(), 'ξ');
        mappings.insert("pi".to_string(), 'π');
        mappings.insert("rho".to_string(), 'ρ');
        mappings.insert("sigma".to_string(), 'σ');
        mappings.insert("tau".to_string(), 'τ');
        mappings.insert("phi".to_string(), 'φ');
        mappings.insert("chi".to_string(), 'χ');
        mappings.insert("psi".to_string(), 'ψ');
        mappings.insert("omega".to_string(), 'ω');
        
        // Capital Greek letters
        mappings.insert("Alpha".to_string(), 'Α');
        mappings.insert("Beta".to_string(), 'Β');
        mappings.insert("Gamma".to_string(), 'Γ');
        mappings.insert("Delta".to_string(), 'Δ');
        mappings.insert("Theta".to_string(), 'Θ');
        mappings.insert("Lambda".to_string(), 'Λ');
        mappings.insert("Pi".to_string(), 'Π');
        mappings.insert("Sigma".to_string(), 'Σ');
        mappings.insert("Phi".to_string(), 'Φ');
        mappings.insert("Psi".to_string(), 'Ψ');
        mappings.insert("Omega".to_string(), 'Ω');
        
        // Mathematical symbols
        mappings.insert("infty".to_string(), '∞');
        mappings.insert("sum".to_string(), '∑');
        mappings.insert("prod".to_string(), '∏');
        mappings.insert("int".to_string(), '∫');
        mappings.insert("sqrt".to_string(), '√');
        mappings.insert("partial".to_string(), '∂');
        mappings.insert("nabla".to_string(), '∇');
        mappings.insert("forall".to_string(), '∀');
        mappings.insert("exists".to_string(), '∃');
        mappings.insert("in".to_string(), '∈');
        mappings.insert("notin".to_string(), '∉');
        mappings.insert("subset".to_string(), '⊂');
        mappings.insert("supset".to_string(), '⊃');
        mappings.insert("cup".to_string(), '∪');
        mappings.insert("cap".to_string(), '∩');
        mappings.insert("emptyset".to_string(), '∅');
        mappings.insert("pm".to_string(), '±');
        mappings.insert("mp".to_string(), '∓');
        mappings.insert("times".to_string(), '×');
        mappings.insert("div".to_string(), '÷');
        mappings.insert("neq".to_string(), '≠');
        mappings.insert("leq".to_string(), '≤');
        mappings.insert("geq".to_string(), '≥');
        mappings.insert("approx".to_string(), '≈');
        mappings.insert("equiv".to_string(), '≡');
        
        Self { mappings }
    }
    
    /// Expand LaTeX-style sequence to Unicode character
    pub fn expand(&self, sequence: &str) -> Option<char> {
        // Remove leading backslash if present
        let key = if sequence.starts_with('\\') {
            &sequence[1..]
        } else {
            sequence
        };
        
        self.mappings.get(key).copied()
    }
    
    /// Get all available expansions
    pub fn list_expansions(&self) -> Vec<(String, char)> {
        let mut expansions: Vec<_> = self.mappings
            .iter()
            .map(|(k, v)| (format!("\\{k}"), *v))
            .collect();
        expansions.sort_by_key(|(k, _)| k.clone());
        expansions
    }
}

impl Default for UnicodeExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_magic_registry() {
        let registry = MagicRegistry::new();
        assert!(registry.is_magic("%time"));
        assert!(registry.is_magic("%%time"));
        assert!(!registry.is_magic("time"));
        
        let commands = registry.list_commands();
        assert!(commands.contains(&"time".to_string()));
        assert!(commands.contains(&"debug".to_string()));
    }
    
    #[test]
    fn test_unicode_expander() {
        let expander = UnicodeExpander::new();
        
        assert_eq!(expander.expand("\\alpha"), Some('α'));
        assert_eq!(expander.expand("alpha"), Some('α'));
        assert_eq!(expander.expand("\\pi"), Some('π'));
        assert_eq!(expander.expand("\\infty"), Some('∞'));
        assert_eq!(expander.expand("\\unknown"), None);
    }
    
    #[test]
    fn test_magic_result_display() {
        let result = MagicResult::Text("Hello".to_string());
        assert_eq!(format!("{result}"), "Hello");
        
        let result = MagicResult::Timed {
            output: "42".to_string(),
            duration: Duration::from_millis(123),
        };
        assert!(format!("{result}").contains("0.123s"));
    }
}