//! Dataflow Debug Command Handler
//!
//! Handles debugging of DataFrame pipelines with profiling and memory tracking.

use anyhow::{bail, Context, Result};
use std::path::Path;

/// Handle dataflow debug command
pub fn handle_dataflow_debug_command(
    _config: Option<&Path>,
    max_rows: usize,
    auto_materialize: bool,
    enable_profiling: bool,
    timeout: u64,
    track_memory: bool,
    compute_diffs: bool,
    sample_rate: f64,
    refresh_interval: u64,
    use_color: bool,
    format: &str,
    export: Option<&Path>,
    verbose: bool,
    breakpoints: &[String],
    start_mode: &str,
) -> Result<()> {
    use colored::Colorize;
    use std::fs;

    // Validate format
    if !matches!(format, "interactive" | "json" | "text") {
        bail!(
            "Invalid format '{}'. Supported formats: interactive, json, text",
            format
        );
    }

    // Validate start_mode
    if !matches!(
        start_mode,
        "overview" | "stages" | "data" | "metrics" | "history"
    ) {
        bail!(
            "Invalid start mode '{}'. Supported: overview, stages, data, metrics, history",
            start_mode
        );
    }

    // Validate sample_rate
    if !(0.0..=1.0).contains(&sample_rate) {
        bail!(
            "Invalid sample rate '{}'. Must be between 0.0 and 1.0",
            sample_rate
        );
    }

    if verbose {
        let msg = format!("→ Starting Dataflow Debugger ({})", start_mode);
        println!(
            "{}",
            if use_color {
                msg.bright_blue().to_string()
            } else {
                msg
            }
        );
    }

    // Generate debug output based on format
    let content = match format {
        "text" => generate_dataflow_debug_text(
            max_rows,
            auto_materialize,
            enable_profiling,
            timeout,
            track_memory,
            compute_diffs,
            sample_rate,
            refresh_interval,
            start_mode,
            use_color,
            breakpoints,
        ),
        "json" => generate_dataflow_debug_json(
            max_rows,
            auto_materialize,
            enable_profiling,
            timeout,
            track_memory,
            compute_diffs,
            sample_rate,
            refresh_interval,
            start_mode,
            breakpoints,
        )?,
        "interactive" => generate_dataflow_debug_interactive(
            max_rows,
            auto_materialize,
            enable_profiling,
            timeout,
            track_memory,
            compute_diffs,
            sample_rate,
            refresh_interval,
            start_mode,
            use_color,
            breakpoints,
        ),
        _ => unreachable!(),
    };

    // Write or print output
    if let Some(output_path) = export {
        fs::write(output_path, &content)
            .with_context(|| format!("Failed to write output: {}", output_path.display()))?;
        let msg = format!("✓ Dataflow debug data saved: {}", output_path.display());
        println!(
            "{}",
            if use_color {
                msg.bright_green().to_string()
            } else {
                msg
            }
        );
    } else {
        print!("{}", content);
    }

    Ok(())
}

/// Generate text format dataflow debug output
fn generate_dataflow_debug_text(
    max_rows: usize,
    auto_materialize: bool,
    enable_profiling: bool,
    timeout: u64,
    track_memory: bool,
    compute_diffs: bool,
    sample_rate: f64,
    refresh_interval: u64,
    start_mode: &str,
    use_color: bool,
    breakpoints: &[String],
) -> String {
    use colored::Colorize;

    let mut output = String::new();
    if use_color {
        output.push_str(&"=== Dataflow Debugger ===".bright_cyan().to_string());
    } else {
        output.push_str("=== Dataflow Debugger ===");
    }
    output.push('\n');

    output.push_str(&format!("Mode: {}\n", start_mode));
    output.push_str(&format!("Max Rows: {}\n", max_rows));
    output.push_str(&format!("Timeout: {}ms\n", timeout));
    output.push_str(&format!("Sample Rate: {:.1}%\n", sample_rate * 100.0));
    output.push_str(&format!("Refresh Interval: {}ms\n\n", refresh_interval));

    if auto_materialize {
        output.push_str("Auto-Materialize: enabled\n");
    }
    if enable_profiling {
        output.push_str("Performance Profiling: enabled\n");
    }
    if track_memory {
        output.push_str("Memory Tracking: enabled\n");
    }
    if compute_diffs {
        output.push_str("Stage Diffs: enabled\n");
    }
    if !breakpoints.is_empty() {
        output.push_str(&format!("Breakpoints: {:?}\n", breakpoints));
    }
    if auto_materialize
        || enable_profiling
        || track_memory
        || compute_diffs
        || !breakpoints.is_empty()
    {
        output.push('\n');
    }

    // Stub: No pipeline currently running
    output.push_str("Status: No active DataFrame pipeline detected\n");
    output.push_str("To debug pipelines, start a Ruchy program with DataFrame operations.\n\n");

    output.push_str("Example:\n");
    output.push_str("  ruchy run pipeline.ruchy &\n");
    output.push_str("  ruchy dataflow:debug --enable-profiling --track-memory\n");

    output
}

/// Generate JSON format dataflow debug output
fn generate_dataflow_debug_json(
    max_rows: usize,
    auto_materialize: bool,
    enable_profiling: bool,
    timeout: u64,
    track_memory: bool,
    compute_diffs: bool,
    sample_rate: f64,
    refresh_interval: u64,
    start_mode: &str,
    breakpoints: &[String],
) -> Result<String> {
    use serde_json::json;

    let data = json!({
        "debugger": {
            "mode": start_mode,
            "max_rows": max_rows,
            "timeout_ms": timeout,
            "sample_rate": sample_rate,
            "refresh_interval_ms": refresh_interval,
            "options": {
                "auto_materialize": auto_materialize,
                "enable_profiling": enable_profiling,
                "track_memory": track_memory,
                "compute_diffs": compute_diffs
            },
            "breakpoints": breakpoints,
            "status": "no_active_pipeline",
            "stages": [],
            "current_stage": null,
            "metrics": {
                "total_stages": 0,
                "completed_stages": 0,
                "failed_stages": 0,
                "total_rows_processed": 0,
                "memory_usage_mb": 0.0,
                "execution_time_ms": 0
            }
        }
    });

    Ok(serde_json::to_string_pretty(&data)?)
}

/// Generate interactive format dataflow debug output
fn generate_dataflow_debug_interactive(
    max_rows: usize,
    auto_materialize: bool,
    enable_profiling: bool,
    timeout: u64,
    track_memory: bool,
    compute_diffs: bool,
    sample_rate: f64,
    _refresh_interval: u64,
    start_mode: &str,
    use_color: bool,
    breakpoints: &[String],
) -> String {
    use colored::Colorize;

    // Interactive mode would normally use a TUI library like crossterm/tui-rs
    // For now, we provide a static snapshot similar to text mode
    let mut output = String::new();
    let header = "╔════════════════════════════════════════════════════════════════╗\n\
                  ║          🔍 Ruchy Dataflow Debugger (Interactive)          ║\n\
                  ╚════════════════════════════════════════════════════════════════╝";
    if use_color {
        output.push_str(&header.bright_cyan().to_string());
    } else {
        output.push_str(header);
    }
    output.push('\n');
    output.push('\n');

    output.push_str(&format!(
        "Mode: {} | Max Rows: {} | Timeout: {}ms | Sample: {:.0}%\n",
        start_mode,
        max_rows,
        timeout,
        sample_rate * 100.0
    ));

    let mut features = Vec::new();
    if auto_materialize {
        features.push("auto-materialize");
    }
    if enable_profiling {
        features.push("profiling");
    }
    if track_memory {
        features.push("memory-tracking");
    }
    if compute_diffs {
        features.push("diffs");
    }
    if !features.is_empty() {
        output.push_str(&format!("Features: {}\n", features.join(", ")));
    }

    if !breakpoints.is_empty() {
        output.push_str(&format!("Breakpoints: {:?}\n", breakpoints));
    }

    output.push('\n');
    output.push_str("════════════════════════════════════════════════════════════════\n");
    output.push_str("Status: No active DataFrame pipeline detected\n\n");
    output.push_str("To debug pipelines, start a Ruchy program with DataFrame operations.\n");
    output.push_str("Press Ctrl+C to exit.\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_dataflow_debug_text() {
        let output = generate_dataflow_debug_text(
            100,
            true,
            true,
            5000,
            true,
            true,
            0.5,
            1000,
            "overview",
            false,
            &[],
        );
        assert!(output.contains("Dataflow Debugger"));
        assert!(output.contains("Mode: overview"));
    }

    #[test]
    fn test_generate_dataflow_debug_json() {
        let output = generate_dataflow_debug_json(
            100,
            true,
            true,
            5000,
            true,
            true,
            0.5,
            1000,
            "overview",
            &[],
        )
        .unwrap();
        assert!(output.contains("\"mode\": \"overview\""));
    }

    // ===== EXTREME TDD Round 145 - Dataflow Handler Tests =====

    #[test]
    fn test_generate_dataflow_debug_text_with_color() {
        let output = generate_dataflow_debug_text(
            100,
            true,
            true,
            5000,
            true,
            true,
            0.5,
            1000,
            "overview",
            true,
            &[],
        );
        assert!(output.contains("Dataflow Debugger"));
    }

    #[test]
    fn test_generate_dataflow_debug_text_stages_mode() {
        let output = generate_dataflow_debug_text(
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            "stages",
            false,
            &[],
        );
        assert!(output.contains("Mode: stages"));
    }

    #[test]
    fn test_generate_dataflow_debug_text_data_mode() {
        let output = generate_dataflow_debug_text(
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            "data",
            false,
            &[],
        );
        assert!(output.contains("Mode: data"));
    }

    #[test]
    fn test_generate_dataflow_debug_text_metrics_mode() {
        let output = generate_dataflow_debug_text(
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            "metrics",
            false,
            &[],
        );
        assert!(output.contains("Mode: metrics"));
    }

    #[test]
    fn test_generate_dataflow_debug_text_history_mode() {
        let output = generate_dataflow_debug_text(
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            "history",
            false,
            &[],
        );
        assert!(output.contains("Mode: history"));
    }

    #[test]
    fn test_generate_dataflow_debug_text_with_breakpoints() {
        let breakpoints = vec!["stage1".to_string(), "stage2".to_string()];
        let output = generate_dataflow_debug_text(
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            "overview",
            false,
            &breakpoints,
        );
        assert!(output.contains("Breakpoints"));
    }

    #[test]
    fn test_generate_dataflow_debug_text_all_features() {
        let output = generate_dataflow_debug_text(
            100,
            true,
            true,
            5000,
            true,
            true,
            0.5,
            1000,
            "overview",
            false,
            &[],
        );
        assert!(output.contains("Auto-Materialize"));
        assert!(output.contains("Performance Profiling"));
        assert!(output.contains("Memory Tracking"));
        assert!(output.contains("Stage Diffs"));
    }

    #[test]
    fn test_generate_dataflow_debug_json_stages_mode() {
        let output = generate_dataflow_debug_json(
            50,
            false,
            false,
            3000,
            false,
            false,
            0.25,
            500,
            "stages",
            &[],
        )
        .unwrap();
        assert!(output.contains("\"mode\": \"stages\""));
    }

    #[test]
    fn test_generate_dataflow_debug_json_with_breakpoints() {
        let breakpoints = vec!["bp1".to_string()];
        let output = generate_dataflow_debug_json(
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            "overview",
            &breakpoints,
        )
        .unwrap();
        assert!(output.contains("\"breakpoints\""));
        assert!(output.contains("bp1"));
    }

    #[test]
    fn test_generate_dataflow_debug_interactive_basic() {
        let output = generate_dataflow_debug_interactive(
            100,
            true,
            true,
            5000,
            true,
            true,
            0.5,
            1000,
            "overview",
            false,
            &[],
        );
        assert!(output.contains("Dataflow Debugger"));
        assert!(output.contains("Interactive"));
    }

    #[test]
    fn test_generate_dataflow_debug_interactive_with_color() {
        let output = generate_dataflow_debug_interactive(
            100,
            true,
            true,
            5000,
            true,
            true,
            0.5,
            1000,
            "overview",
            true,
            &[],
        );
        assert!(output.contains("Dataflow Debugger"));
    }

    #[test]
    fn test_generate_dataflow_debug_interactive_with_breakpoints() {
        let breakpoints = vec!["step1".to_string(), "step2".to_string()];
        let output = generate_dataflow_debug_interactive(
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            "stages",
            false,
            &breakpoints,
        );
        assert!(output.contains("Breakpoints"));
    }

    #[test]
    fn test_generate_dataflow_debug_interactive_features() {
        let output = generate_dataflow_debug_interactive(
            100,
            true,
            true,
            5000,
            true,
            true,
            0.5,
            1000,
            "overview",
            false,
            &[],
        );
        assert!(output.contains("auto-materialize"));
        assert!(output.contains("profiling"));
        assert!(output.contains("memory-tracking"));
        assert!(output.contains("diffs"));
    }

    #[test]
    fn test_handle_dataflow_debug_text_format() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            false,
            "text",
            None,
            false,
            &[],
            "overview",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_dataflow_debug_json_format() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            false,
            "json",
            None,
            false,
            &[],
            "overview",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_dataflow_debug_interactive_format() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            false,
            "interactive",
            None,
            false,
            &[],
            "overview",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_dataflow_debug_invalid_format() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            false,
            "invalid",
            None,
            false,
            &[],
            "overview",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_dataflow_debug_invalid_mode() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            false,
            "text",
            None,
            false,
            &[],
            "invalid_mode",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_dataflow_debug_invalid_sample_rate_high() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            1.5,
            1000,
            false,
            "text",
            None,
            false,
            &[],
            "overview",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_dataflow_debug_invalid_sample_rate_negative() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            -0.1,
            1000,
            false,
            "text",
            None,
            false,
            &[],
            "overview",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_dataflow_debug_with_export() {
        let temp = NamedTempFile::new().unwrap();
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            false,
            "text",
            Some(temp.path()),
            false,
            &[],
            "overview",
        );
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("Dataflow Debugger"));
    }

    #[test]
    fn test_handle_dataflow_debug_verbose() {
        let result = handle_dataflow_debug_command(
            None,
            100,
            false,
            false,
            5000,
            false,
            false,
            0.5,
            1000,
            false,
            "text",
            None,
            true,
            &[],
            "overview",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_dataflow_debug_with_all_options() {
        let breakpoints = vec!["bp1".to_string()];
        let result = handle_dataflow_debug_command(
            None,
            200,
            true,
            true,
            10000,
            true,
            true,
            1.0,
            2000,
            true,
            "text",
            None,
            true,
            &breakpoints,
            "stages",
        );
        assert!(result.is_ok());
    }
}
