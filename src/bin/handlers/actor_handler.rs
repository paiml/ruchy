//! Actor Observatory Command Handler
//!
//! Handles observing and debugging actor systems with message tracing.

use anyhow::{bail, Context, Result};
use std::path::Path;

/// Handle actor observe command
pub fn handle_actor_observe_command(
    _config: Option<&Path>,
    refresh_interval: u64,
    max_traces: usize,
    max_actors: usize,
    enable_deadlock_detection: bool,
    deadlock_interval: u64,
    start_mode: &str,
    use_color: bool,
    format: &str,
    export: Option<&Path>,
    _duration: u64,
    verbose: bool,
    filter_actor: Option<&str>,
    filter_failed: bool,
    filter_slow: Option<u64>,
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
        "overview" | "actors" | "messages" | "metrics" | "deadlocks"
    ) {
        bail!(
            "Invalid start mode '{}'. Supported: overview, actors, messages, metrics, deadlocks",
            start_mode
        );
    }

    if verbose {
        let msg = format!("→ Starting Actor Observatory ({})", start_mode);
        println!(
            "{}",
            if use_color {
                msg.bright_blue().to_string()
            } else {
                msg
            }
        );
    }

    // Generate observatory output based on format
    let content = match format {
        "text" => generate_actor_observe_text(
            refresh_interval,
            max_traces,
            max_actors,
            enable_deadlock_detection,
            deadlock_interval,
            start_mode,
            use_color,
            filter_actor,
            filter_failed,
            filter_slow,
        ),
        "json" => generate_actor_observe_json(
            refresh_interval,
            max_traces,
            max_actors,
            enable_deadlock_detection,
            deadlock_interval,
            start_mode,
            filter_actor,
            filter_failed,
            filter_slow,
        )?,
        "interactive" => generate_actor_observe_interactive(
            refresh_interval,
            max_traces,
            max_actors,
            enable_deadlock_detection,
            deadlock_interval,
            start_mode,
            use_color,
            filter_actor,
            filter_failed,
            filter_slow,
        ),
        _ => unreachable!(),
    };

    // Write or print output
    if let Some(output_path) = export {
        fs::write(output_path, &content)
            .with_context(|| format!("Failed to write output: {}", output_path.display()))?;
        let msg = format!("✓ Actor observatory data saved: {}", output_path.display());
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

/// Generate text format actor observatory output
fn generate_actor_observe_text(
    refresh_interval: u64,
    max_traces: usize,
    max_actors: usize,
    enable_deadlock_detection: bool,
    deadlock_interval: u64,
    start_mode: &str,
    use_color: bool,
    filter_actor: Option<&str>,
    filter_failed: bool,
    filter_slow: Option<u64>,
) -> String {
    use colored::Colorize;

    let mut output = String::new();
    if use_color {
        output.push_str(&"=== Actor Observatory ===".bright_cyan().to_string());
    } else {
        output.push_str("=== Actor Observatory ===");
    }
    output.push('\n');

    output.push_str(&format!("Mode: {}\n", start_mode));
    output.push_str(&format!("Refresh Interval: {}ms\n", refresh_interval));
    output.push_str(&format!("Max Traces: {}\n", max_traces));
    output.push_str(&format!("Max Actors: {}\n\n", max_actors));

    if let Some(filter) = filter_actor {
        output.push_str(&format!("Filter (Actor): {}\n", filter));
    }
    if filter_failed {
        output.push_str("Filter (Failed Messages Only): enabled\n");
    }
    if let Some(slow_threshold) = filter_slow {
        output.push_str(&format!("Filter (Slow Messages): >{}μs\n", slow_threshold));
    }
    if filter_actor.is_some() || filter_failed || filter_slow.is_some() {
        output.push('\n');
    }

    if enable_deadlock_detection {
        output.push_str(&format!(
            "Deadlock Detection: enabled (interval: {}ms)\n\n",
            deadlock_interval
        ));
    }

    // Stub: No actors currently running
    output.push_str("Status: No active actor system detected\n");
    output.push_str("To observe actors, start a Ruchy program with actor system support.\n\n");

    output.push_str("Example:\n");
    output.push_str("  ruchy run actor_program.ruchy &\n");
    output.push_str("  ruchy actor:observe --refresh-interval 500\n");

    output
}

/// Generate JSON format actor observatory output
fn generate_actor_observe_json(
    refresh_interval: u64,
    max_traces: usize,
    max_actors: usize,
    enable_deadlock_detection: bool,
    deadlock_interval: u64,
    start_mode: &str,
    filter_actor: Option<&str>,
    filter_failed: bool,
    filter_slow: Option<u64>,
) -> Result<String> {
    use serde_json::json;

    let data = json!({
        "observatory": {
            "mode": start_mode,
            "refresh_interval_ms": refresh_interval,
            "max_traces": max_traces,
            "max_actors": max_actors,
            "deadlock_detection": {
                "enabled": enable_deadlock_detection,
                "interval_ms": deadlock_interval
            },
            "filters": {
                "actor_pattern": filter_actor,
                "failed_only": filter_failed,
                "slow_threshold_us": filter_slow
            },
            "status": "no_active_actors",
            "actors": [],
            "message_traces": [],
            "metrics": {
                "total_actors": 0,
                "active_actors": 0,
                "idle_actors": 0,
                "crashed_actors": 0,
                "total_messages": 0,
                "failed_messages": 0
            }
        }
    });

    Ok(serde_json::to_string_pretty(&data)?)
}

/// Generate interactive format actor observatory output
fn generate_actor_observe_interactive(
    refresh_interval: u64,
    max_traces: usize,
    max_actors: usize,
    enable_deadlock_detection: bool,
    deadlock_interval: u64,
    start_mode: &str,
    use_color: bool,
    filter_actor: Option<&str>,
    filter_failed: bool,
    filter_slow: Option<u64>,
) -> String {
    use colored::Colorize;

    // Interactive mode would normally use a TUI library like crossterm/tui-rs
    // For now, we provide a static snapshot similar to text mode
    let mut output = String::new();
    let header = "╔════════════════════════════════════════════════════════════════╗\n\
                  ║          🎭 Ruchy Actor Observatory (Interactive)           ║\n\
                  ╚════════════════════════════════════════════════════════════════╝";
    if use_color {
        output.push_str(&header.bright_cyan().to_string());
    } else {
        output.push_str(header);
    }
    output.push('\n');
    output.push('\n');

    output.push_str(&format!(
        "Mode: {} | Refresh: {}ms | Max Traces: {} | Max Actors: {}\n",
        start_mode, refresh_interval, max_traces, max_actors
    ));

    if enable_deadlock_detection {
        output.push_str(&format!(
            "Deadlock Detection: ✓ ({}ms)\n",
            deadlock_interval
        ));
    }

    if filter_actor.is_some() || filter_failed || filter_slow.is_some() {
        output.push_str("\nFilters: ");
        if let Some(f) = filter_actor {
            output.push_str(&format!("actor={} ", f));
        }
        if filter_failed {
            output.push_str("failed ");
        }
        if let Some(s) = filter_slow {
            output.push_str(&format!("slow>{}μs ", s));
        }
        output.push('\n');
    }

    output.push('\n');
    output.push_str("════════════════════════════════════════════════════════════════\n");
    output.push_str("Status: No active actor system detected\n\n");
    output.push_str("To observe actors, start a Ruchy program with actor system support.\n");
    output.push_str("Press Ctrl+C to exit.\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_actor_observe_text() {
        let output = generate_actor_observe_text(
            1000, 100, 50, true, 5000, "overview", false, None, false, None,
        );
        assert!(output.contains("Actor Observatory"));
        assert!(output.contains("Mode: overview"));
    }

    #[test]
    fn test_generate_actor_observe_json() {
        let output = generate_actor_observe_json(
            1000, 100, 50, true, 5000, "overview", None, false, None,
        )
        .unwrap();
        assert!(output.contains("\"mode\": \"overview\""));
    }
}
