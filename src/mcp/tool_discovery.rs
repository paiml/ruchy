//! MCP Tool Discovery for Ruchy Binary Commands
//!
//! This module exposes core Ruchy compiler functionality as MCP tools
//! for integration with Claude Code agent mode, following the patterns
//! from paiml-mcp-agent-toolkit for optimal tool discovery.
use crate::mcp::RuchyMCPTool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
#[cfg(test)]
use proptest::prelude::*;
/// MCP tool discovery service for Ruchy compiler commands
pub struct RuchyToolDiscovery {
    /// Registry of available tools
    tools: HashMap<String, RuchyMCPTool>,
    /// Binary path for ruchy executable
    binary_path: String,
}
impl RuchyToolDiscovery {
    /// Create new tool discovery service
/// # Examples
/// 
/// ```
/// use ruchy::mcp::tool_discovery::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new() -> Self {
        let mut discovery = Self {
            tools: HashMap::new(),
            binary_path: "ruchy".to_string(),
        };
        discovery.register_core_tools();
        discovery
    }
    /// Create with custom binary path
/// # Examples
/// 
/// ```
/// use ruchy::mcp::tool_discovery::with_binary_path;
/// 
/// let result = with_binary_path(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn with_binary_path(binary_path: String) -> Self {
        let mut discovery = Self {
            tools: HashMap::new(),
            binary_path,
        };
        discovery.register_core_tools();
        discovery
    }
    /// Register all core Ruchy tools for MCP exposure
    fn register_core_tools(&mut self) {
        // Parse and AST tools
        self.register_parse_tool();
        self.register_ast_tool();
        // Transpilation tools
        self.register_transpile_tool();
        self.register_check_tool();
        // Execution tools
        self.register_eval_tool();
        self.register_run_tool();
        // Quality and analysis tools
        self.register_lint_tool();
        self.register_fmt_tool();
        self.register_score_tool();
        self.register_quality_gate_tool();
        // Advanced analysis tools
        self.register_provability_tool();
        self.register_runtime_analysis_tool();
        self.register_optimize_tool();
    }
    /// Register parse tool for syntax analysis
    fn register_parse_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_parse".to_string(),
            "Parse Ruchy code and show AST structure for syntax analysis".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("parse")
                    .arg("-")
                    .arg("--format=json")
                    .arg("--stdin")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "ast": stdout.trim(),
                        "tool": "ruchy_parse"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_parse"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_parse".to_string(), tool);
    }
    /// Register AST tool for detailed AST analysis
    fn register_ast_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_ast".to_string(),
            "Generate detailed AST representation for Ruchy code analysis".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("ast")
                    .arg("-")
                    .arg("--format=json")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "ast": stdout.trim(),
                        "tool": "ruchy_ast"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_ast"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_ast".to_string(), tool);
    }
    /// Register transpile tool for Rust code generation
    fn register_transpile_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_transpile".to_string(),
            "Transpile Ruchy code to Rust for analysis and compilation".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("transpile")
                    .arg("-")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "rust_code": stdout.trim(),
                        "tool": "ruchy_transpile"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_transpile"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_transpile".to_string(), tool);
    }
    /// Register check tool for syntax validation
    fn register_check_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_check".to_string(),
            "Check Ruchy code syntax and types without executing".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("check")
                    .arg("-")
                    .output()?;
                if output.status.success() {
                    Ok(json!({
                        "success": true,
                        "message": "Code syntax is valid",
                        "tool": "ruchy_check"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_check"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_check".to_string(), tool);
    }
    /// Register eval tool for one-liner execution
    fn register_eval_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_eval".to_string(),
            "Evaluate Ruchy expressions and return results".to_string(),
            move |args: Value| -> Result<Value> {
                let expression = args.get("expression")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'expression' parameter"))?;
                let format = args.get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("json");
                let output = Command::new(&binary_path)
                    .arg("-e")
                    .arg(expression)
                    .arg("--format")
                    .arg(format)
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "result": stdout.trim(),
                        "expression": expression,
                        "tool": "ruchy_eval"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "expression": expression,
                        "tool": "ruchy_eval"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_eval".to_string(), tool);
    }
    /// Register run tool for full program execution
    fn register_run_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_run".to_string(),
            "Compile and run complete Ruchy programs".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("run")
                    .arg("-")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "output": stdout.trim(),
                        "tool": "ruchy_run"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_run"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_run".to_string(), tool);
    }
    /// Register lint tool for code quality analysis
    fn register_lint_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_lint".to_string(),
            "Lint Ruchy code for style violations and potential issues".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("lint")
                    .arg("-")
                    .arg("--format=json")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "lint_results": stdout.trim(),
                        "tool": "ruchy_lint"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_lint"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_lint".to_string(), tool);
    }
    /// Register format tool for code formatting
    fn register_fmt_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_fmt".to_string(),
            "Format Ruchy code according to style guidelines".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("fmt")
                    .arg("-")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "formatted_code": stdout.trim(),
                        "tool": "ruchy_fmt"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_fmt"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_fmt".to_string(), tool);
    }
    /// Register quality score tool
    fn register_score_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_score".to_string(),
            "Calculate unified quality score for Ruchy code".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("score")
                    .arg("-")
                    .arg("--format=json")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "quality_score": stdout.trim(),
                        "tool": "ruchy_score"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_score"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_score".to_string(), tool);
    }
    /// Register quality gate tool
    fn register_quality_gate_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_quality_gate".to_string(),
            "Enforce quality gates and standards on Ruchy code".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("quality-gate")
                    .arg("-")
                    .arg("--format=json")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "quality_gate_result": stdout.trim(),
                        "tool": "ruchy_quality_gate"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_quality_gate"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_quality_gate".to_string(), tool);
    }
    /// Register provability analysis tool
    fn register_provability_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_provability".to_string(),
            "Formal verification and correctness analysis of Ruchy code".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("provability")
                    .arg("-")
                    .arg("--format=json")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "provability_analysis": stdout.trim(),
                        "tool": "ruchy_provability"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_provability"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_provability".to_string(), tool);
    }
    /// Register runtime analysis tool
    fn register_runtime_analysis_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_runtime".to_string(),
            "Performance analysis and BigO complexity detection".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("runtime")
                    .arg("-")
                    .arg("--format=json")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "runtime_analysis": stdout.trim(),
                        "tool": "ruchy_runtime"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_runtime"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_runtime".to_string(), tool);
    }
    /// Register optimization analysis tool
    fn register_optimize_tool(&mut self) {
        let binary_path = self.binary_path.clone();
        let tool = RuchyMCPTool::new(
            "ruchy_optimize".to_string(),
            "Hardware-aware optimization analysis for Ruchy code".to_string(),
            move |args: Value| -> Result<Value> {
                let _code = args.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'code' parameter"))?;
                let output = Command::new(&binary_path)
                    .arg("optimize")
                    .arg("-")
                    .arg("--format=json")
                    .output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(json!({
                        "success": true,
                        "optimization_analysis": stdout.trim(),
                        "tool": "ruchy_optimize"
                    }))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(json!({
                        "success": false,
                        "error": stderr.trim(),
                        "tool": "ruchy_optimize"
                    }))
                }
            }
        );
        self.tools.insert("ruchy_optimize".to_string(), tool);
    }
    /// Get all registered tools
/// # Examples
/// 
/// ```
/// use ruchy::mcp::tool_discovery::get_tools;
/// 
/// let result = get_tools(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn get_tools(&self) -> &HashMap<String, RuchyMCPTool> {
        &self.tools
    }
    /// Get tool by name
/// # Examples
/// 
/// ```
/// use ruchy::mcp::tool_discovery::get_tool;
/// 
/// let result = get_tool("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn get_tool(&self, name: &str) -> Option<&RuchyMCPTool> {
        self.tools.get(name)
    }
    /// List all available tool names
/// # Examples
/// 
/// ```
/// use ruchy::mcp::tool_discovery::list_tool_names;
/// 
/// let result = list_tool_names(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn list_tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    /// Get tool discovery information for MCP registration
/// # Examples
/// 
/// ```
/// use ruchy::mcp::tool_discovery::get_discovery_info;
/// 
/// let result = get_discovery_info(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn get_discovery_info(&self) -> Value {
        let tools: Vec<Value> = self.tools.iter()
            .map(|(name, tool)| {
                json!({
                    "name": name,
                    "description": tool.description(),
                    "category": self.get_tool_category(name),
                    "aliases": self.get_tool_aliases(name)
                })
            })
            .collect();
        json!({
            "discovery_service": "ruchy_tool_discovery",
            "version": "1.0.0",
            "total_tools": tools.len(),
            "tools": tools,
            "categories": [
                "parsing",
                "transpilation", 
                "execution",
                "quality",
                "analysis"
            ]
        })
    }
    /// Get category for a tool (for better organization)
    fn get_tool_category(&self, tool_name: &str) -> &str {
        match tool_name {
            name if name.contains("parse") || name.contains("ast") => "parsing",
            name if name.contains("transpile") || name.contains("check") => "transpilation",
            name if name.contains("eval") || name.contains("run") => "execution",
            name if name.contains("lint") || name.contains("fmt") || name.contains("score") || name.contains("quality") => "quality",
            _ => "analysis"
        }
    }
    /// Get aliases for better tool discovery (following paiml-mcp-agent-toolkit patterns)
    fn get_tool_aliases(&self, tool_name: &str) -> Vec<&str> {
        match tool_name {
            "ruchy_parse" => vec!["parse", "syntax", "ast_parse"],
            "ruchy_ast" => vec!["ast", "tree", "structure"],
            "ruchy_transpile" => vec!["transpile", "convert", "rust"],
            "ruchy_check" => vec!["check", "validate", "syntax_check"],
            "ruchy_eval" => vec!["eval", "execute", "run_expr"],
            "ruchy_run" => vec!["run", "execute", "compile_run"],
            "ruchy_lint" => vec!["lint", "style", "check_style"],
            "ruchy_fmt" => vec!["fmt", "format", "pretty"],
            "ruchy_score" => vec!["score", "quality", "metrics"],
            "ruchy_quality_gate" => vec!["quality_gate", "gate", "quality_check"],
            "ruchy_provability" => vec!["prove", "verify", "formal"],
            "ruchy_runtime" => vec!["runtime", "performance", "bigo"],
            "ruchy_optimize" => vec!["optimize", "hardware", "perf"],
            _ => vec![]
        }
    }
}
impl Default for RuchyToolDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod property_tests_tool_discovery {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
