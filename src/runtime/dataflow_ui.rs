//! Terminal UI for dataflow debugger (RUCHY-0818)
//!
//! Provides an interactive terminal interface for debugging `DataFrame` pipelines,
//! displaying breakpoints, materialized data, and execution flow.
use crate::runtime::dataflow_debugger::{
    DataflowDebugger, PipelineStage, SessionState, StageStatus,
    MaterializedFrame,
};
use anyhow::Result;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
/// Terminal UI for dataflow debugging
pub struct DataflowUI {
    /// Reference to the dataflow debugger
    debugger: Arc<Mutex<DataflowDebugger>>,
    /// UI configuration
    config: UIConfig,
    /// Current display mode
    display_mode: DisplayMode,
    /// Terminal dimensions
    #[allow(dead_code)] // Future feature for responsive UI
    terminal_size: (u16, u16), // (width, height)
    /// UI refresh rate
    refresh_interval: Duration,
    /// Last refresh time
    last_refresh: Instant,
    /// Color support enabled
    colors_enabled: bool,
}
/// Configuration for the dataflow UI
#[derive(Debug, Clone)]
pub struct UIConfig {
    /// Maximum number of rows to display in data preview
    pub max_preview_rows: usize,
    /// Maximum number of events to show in history
    pub max_history_events: usize,
    /// Enable real-time refresh
    pub auto_refresh: bool,
    /// Refresh interval in milliseconds
    pub refresh_interval_ms: u64,
    /// Show performance metrics
    pub show_metrics: bool,
    /// Enable color output
    pub enable_colors: bool,
    /// Compact display mode
    pub compact_mode: bool,
}
/// Display modes for the UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    /// Pipeline overview with all stages
    Overview,
    /// Detailed stage information
    StageDetail(String),
    /// Breakpoint management
    Breakpoints,
    /// Materialized data viewer
    DataViewer(String),
    /// Performance metrics
    Metrics,
    /// Execution history
    History,
    /// Stage diff comparison
    Diff(String, String),
    /// Help screen
    Help,
}
impl Default for UIConfig {
    fn default() -> Self {
        Self {
            max_preview_rows: 20,
            max_history_events: 100,
            auto_refresh: true,
            refresh_interval_ms: 1000,
            show_metrics: true,
            enable_colors: true,
            compact_mode: false,
        }
    }
}
impl DataflowUI {
    /// Create a new dataflow UI
    pub fn new(debugger: Arc<Mutex<DataflowDebugger>>, config: UIConfig) -> Self {
        Self {
            debugger,
            config: config.clone(),
            display_mode: DisplayMode::Overview,
            terminal_size: Self::get_terminal_size(),
            refresh_interval: Duration::from_millis(config.refresh_interval_ms),
            last_refresh: Instant::now(),
            colors_enabled: config.enable_colors,
        }
    }
    /// Start the interactive UI loop
    pub fn run_interactive(&mut self) -> Result<()> {
        self.print_header()?;
        self.print_help_hint()?;
        loop {
            self.refresh_display()?;
            if let Some(command) = self.get_user_input()? {
                match self.handle_command(&command)? {
                    UIAction::Continue => continue,
                    UIAction::Exit => break,
                }
            }
            // Auto-refresh if enabled
            if self.config.auto_refresh && self.last_refresh.elapsed() >= self.refresh_interval {
                self.refresh_display()?;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }
    /// Refresh the current display
    pub fn refresh_display(&mut self) -> Result<()> {
        self.clear_screen()?;
        match &self.display_mode {
            DisplayMode::Overview => self.render_overview()?,
            DisplayMode::StageDetail(stage_id) => self.render_stage_detail(stage_id)?,
            DisplayMode::Breakpoints => self.render_breakpoints()?,
            DisplayMode::DataViewer(stage_id) => self.render_data_viewer(stage_id)?,
            DisplayMode::Metrics => self.render_metrics()?,
            DisplayMode::History => self.render_history()?,
            DisplayMode::Diff(stage1, stage2) => self.render_diff(stage1, stage2)?,
            DisplayMode::Help => self.render_help()?,
        }
        self.print_status_bar()?;
        self.print_command_prompt()?;
        self.last_refresh = Instant::now();
        Ok(())
    }
    /// Render pipeline overview
    fn render_overview(&self) -> Result<()> {
        self.print_title("📊 Dataflow Pipeline Overview")?;
        let debugger = self.debugger
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire debugger lock"))?;
        let session_status = debugger.get_session_status()?;
        self.render_session_info(&session_status)?;
        // In a real implementation, we would get actual pipeline stages
        let sample_stages = self.get_sample_stages();
        println!();
        self.print_section_header("Pipeline Stages")?;
        self.print_separator()?;
        if self.colors_enabled {
            println!("{:<4} {:<20} {:<12} {:<10} {:<15} {:<10}", 
                     "#", "Stage Name", "Type", "Status", "Rows", "Time");
        } else {
            println!("{:<4} {:<20} {:<12} {:<10} {:<15} {:<10}", 
                     "#", "Stage Name", "Type", "Status", "Rows", "Time");
        }
        self.print_separator()?;
        for (i, stage) in sample_stages.iter().enumerate() {
            let status_color = if self.colors_enabled {
                match stage.status {
                    StageStatus::Completed => "\x1b[32m", // Green
                    StageStatus::Running => "\x1b[33m",   // Yellow
                    StageStatus::Failed(_) => "\x1b[31m", // Red
                    StageStatus::Paused => "\x1b[36m",    // Cyan
                    _ => "\x1b[37m",                      // White
                }
            } else {
                ""
            };
            let reset_color = if self.colors_enabled { "\x1b[0m" } else { "" };
            let time_str = stage.execution_time.map_or_else(|| "-".to_string(), |t| format!("{}ms", t.as_millis()));
            let rows_str = stage.rows_processed.map_or_else(|| "-".to_string(), |r| format!("{r}"));
            println!("{:<4} {:<20} {:<12} {}{:<10}{} {:<15} {:<10}", 
                     i + 1,
                     stage.stage_name,
                     stage.stage_type,
                     status_color,
                     stage.status,
                     reset_color,
                     rows_str,
                     time_str);
        }
        Ok(())
    }
    /// Render detailed stage information
    fn render_stage_detail(&self, stage_id: &str) -> Result<()> {
        self.print_title(&format!("🔍 Stage Detail: {stage_id}"))?;
        // In a real implementation, we would get actual stage details
        let stage = self.get_sample_stage(stage_id);
        self.print_section_header("Stage Information")?;
        println!("ID: {}", stage.stage_id);
        println!("Name: {}", stage.stage_name);
        println!("Type: {}", stage.stage_type);
        println!("Status: {}", stage.status);
        if let Some(time) = stage.execution_time {
            println!("Execution Time: {}ms", time.as_millis());
        }
        if let Some(rows) = stage.rows_processed {
            println!("Rows Processed: {rows}");
        }
        if let Some(memory) = stage.memory_usage {
            println!("Memory Usage: {}", self.format_bytes(memory));
        }
        // Show schema information if available
        if let Some(input_schema) = &stage.input_schema {
            println!();
            self.print_section_header("Input Schema")?;
            for col in &input_schema.columns {
                println!("  {}: {} (nullable: {})", col.name, col.data_type, col.nullable);
            }
        }
        if let Some(output_schema) = &stage.output_schema {
            println!();
            self.print_section_header("Output Schema")?;
            for col in &output_schema.columns {
                println!("  {}: {} (nullable: {})", col.name, col.data_type, col.nullable);
            }
        }
        // Show metadata
        if !stage.metadata.is_empty() {
            println!();
            self.print_section_header("Metadata")?;
            for (key, value) in &stage.metadata {
                println!("  {key}: {value}");
            }
        }
        Ok(())
    }
    /// Render breakpoints view
    fn render_breakpoints(&self) -> Result<()> {
        self.print_title("🔴 Breakpoint Management")?;
        println!("Active Breakpoints:");
        self.print_separator()?;
        println!("{:<20} {:<10} {:<15} {:<8}", "Stage ID", "Condition", "Actions", "Hit Count");
        self.print_separator()?;
        // In a real implementation, we would get actual breakpoints
        println!("{:<20} {:<10} {:<15} {:<8}", "load_data", "Always", "Pause,Print", "3");
        println!("{:<20} {:<10} {:<15} {:<8}", "filter_stage", "RowCount>1000", "Materialize", "1");
        println!();
        println!("Commands:");
        println!("  add <stage_id> [condition] - Add breakpoint");
        println!("  remove <stage_id>          - Remove breakpoint");
        println!("  toggle <stage_id>          - Toggle breakpoint");
        println!("  clear                      - Clear all breakpoints");
        Ok(())
    }
    /// Render data viewer
    fn render_data_viewer(&self, stage_id: &str) -> Result<()> {
        self.print_title(&format!("📋 Data Viewer: {stage_id}"))?;
        let _debugger = self.debugger
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire debugger lock"))?;
        // In a real implementation, we would get actual materialized data
        let sample_data = self.get_sample_materialized_data(stage_id);
        self.print_section_header(&format!("Sample Data ({} rows)", sample_data.total_rows))?;
        // Display schema
        println!("Schema:");
        for col in &sample_data.schema.columns {
            println!("  {}: {}", col.name, col.data_type);
        }
        println!();
        println!("Data Preview (showing {} of {} rows):", 
                 sample_data.sample_data.len(), sample_data.total_rows);
        self.print_separator()?;
        // Print column headers
        for col in &sample_data.schema.columns {
            print!("{:<15} ", col.name);
        }
        println!();
        self.print_separator()?;
        // Print data rows
        for row in &sample_data.sample_data {
            for value in &row.values {
                let value_str = format!("{value}");
                let truncated = if value_str.len() > 14 {
                    format!("{}...", &value_str[..11])
                } else {
                    value_str
                };
                print!("{truncated:<15} ");
            }
            println!();
        }
        println!();
        println!("Memory Size: {}", self.format_bytes(sample_data.memory_size));
        println!("Materialized: {}", self.format_timestamp(sample_data.timestamp));
        Ok(())
    }
    /// Render performance metrics
    fn render_metrics(&self) -> Result<()> {
        self.print_title("⚡ Performance Metrics")?;
        let debugger = self.debugger
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire debugger lock"))?;
        let metrics = debugger.get_stage_metrics()?;
        if metrics.is_empty() {
            println!("No performance metrics available yet.");
            return Ok(());
        }
        self.print_section_header("Stage Performance")?;
        self.print_separator()?;
        println!("{:<20} {:<10} {:<12} {:<12} {:<10} {:<10}", 
                 "Stage", "Time (ms)", "Memory (MB)", "Input Rows", "Output Rows", "Cache Hit");
        self.print_separator()?;
        for (stage_id, metric) in &metrics {
            let cache_hit = metric.cache_hit_ratio.map_or_else(|| "-".to_string(), |r| format!("{:.1}%", r * 100.0));
            println!("{:<20} {:<10} {:<12} {:<12} {:<10} {:<10}", 
                     stage_id,
                     metric.execution_time.as_millis(),
                     metric.peak_memory / (1024 * 1024),
                     metric.input_rows,
                     metric.output_rows,
                     cache_hit);
        }
        // Summary metrics
        let total_time: Duration = metrics.values().map(|m| m.execution_time).sum();
        let total_memory: usize = metrics.values().map(|m| m.peak_memory).sum();
        let total_rows: usize = metrics.values().map(|m| m.output_rows).sum();
        println!();
        self.print_section_header("Summary")?;
        println!("Total Execution Time: {}ms", total_time.as_millis());
        println!("Peak Memory Usage: {}", self.format_bytes(total_memory));
        println!("Total Rows Processed: {total_rows}");
        Ok(())
    }
    /// Render execution history
    fn render_history(&self) -> Result<()> {
        self.print_title("📜 Execution History")?;
        let debugger = self.debugger
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire debugger lock"))?;
        let history = debugger.get_execution_history()?;
        let recent_events = history.iter().rev().take(self.config.max_history_events);
        self.print_separator()?;
        println!("{:<20} {:<15} {:<20} {:<30}", "Timestamp", "Event Type", "Stage", "Details");
        self.print_separator()?;
        for event in recent_events {
            let timestamp_str = self.format_timestamp(event.timestamp);
            let details = event.data.iter()
                .map(|(k, v)| format!("{k}:{v}"))
                .collect::<Vec<_>>()
                .join(", ");
            println!("{:<20} {:<15} {:<20} {:<30}", 
                     timestamp_str,
                     format!("{:?}", event.event_type),
                     event.stage_id,
                     details);
        }
        Ok(())
    }
    /// Render stage diff comparison
    fn render_diff(&self, stage1: &str, stage2: &str) -> Result<()> {
        self.print_title(&format!("🔄 Stage Diff: {stage1} → {stage2}"))?;
        let debugger = self.debugger
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire debugger lock"))?;
        // In a real implementation, we would compute actual diff
        match debugger.compute_stage_diff(stage1, stage2) {
            Ok(diff) => {
                self.print_section_header("Diff Summary")?;
                println!("Row Count Change: {}", diff.row_count_diff);
                println!("Schema Changed: {}", diff.schema_changed);
                if !diff.column_changes.is_empty() {
                    println!();
                    self.print_section_header("Column Changes")?;
                    for change in &diff.column_changes {
                        println!("  {change:?}");
                    }
                }
                if !diff.data_changes.is_empty() {
                    println!();
                    self.print_section_header("Data Changes")?;
                    for change in &diff.data_changes {
                        println!("  {change:?}");
                    }
                }
            }
            Err(e) => {
                println!("Error computing diff: {e}");
                println!("Make sure both stages have materialized data.");
            }
        }
        Ok(())
    }
    /// Render help screen
    fn render_help(&self) -> Result<()> {
        self.print_title("❓ Dataflow Debugger Help")?;
        println!("Navigation Commands:");
        println!("  overview                    - Show pipeline overview");
        println!("  stage <id>                  - Show stage details");
        println!("  breakpoints                 - Manage breakpoints");
        println!("  data <stage_id>             - View materialized data");
        println!("  metrics                     - Show performance metrics");
        println!("  history                     - Show execution history");
        println!("  diff <stage1> <stage2>      - Compare stages");
        println!("  help                        - Show this help");
        println!("  quit/exit                   - Exit debugger");
        println!();
        println!("Debugging Commands:");
        println!("  materialize <stage_id>      - Materialize stage data");
        println!("  break <stage_id>            - Add breakpoint");
        println!("  continue                    - Continue execution");
        println!("  step                        - Execute next stage");
        println!("  export <format> <path>      - Export debug data");
        println!();
        println!("Display Commands:");
        println!("  refresh                     - Refresh current view");
        println!("  colors on/off               - Toggle color output");
        println!("  compact on/off              - Toggle compact mode");
        Ok(())
    }
    /// Handle user commands
    fn handle_command(&mut self, command: &str) -> Result<UIAction> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(UIAction::Continue);
        }
        match parts[0].to_lowercase().as_str() {
            "quit" | "exit" | "q" => Ok(UIAction::Exit),
            "help" | "h" => {
                self.display_mode = DisplayMode::Help;
                Ok(UIAction::Continue)
            }
            "overview" | "o" => {
                self.display_mode = DisplayMode::Overview;
                Ok(UIAction::Continue)
            }
            "stage" | "s" => {
                if parts.len() > 1 {
                    self.display_mode = DisplayMode::StageDetail(parts[1].to_string());
                } else {
                    println!("Usage: stage <stage_id>");
                }
                Ok(UIAction::Continue)
            }
            "breakpoints" | "bp" => {
                self.display_mode = DisplayMode::Breakpoints;
                Ok(UIAction::Continue)
            }
            "data" | "d" => {
                if parts.len() > 1 {
                    self.display_mode = DisplayMode::DataViewer(parts[1].to_string());
                } else {
                    println!("Usage: data <stage_id>");
                }
                Ok(UIAction::Continue)
            }
            "metrics" | "m" => {
                self.display_mode = DisplayMode::Metrics;
                Ok(UIAction::Continue)
            }
            "history" | "hist" => {
                self.display_mode = DisplayMode::History;
                Ok(UIAction::Continue)
            }
            "diff" => {
                if parts.len() > 2 {
                    self.display_mode = DisplayMode::Diff(parts[1].to_string(), parts[2].to_string());
                } else {
                    println!("Usage: diff <stage1> <stage2>");
                }
                Ok(UIAction::Continue)
            }
            "refresh" | "r" => {
                // Force refresh on next loop
                self.last_refresh = Instant::now().checked_sub(self.refresh_interval).expect("Subtraction underflow");
                Ok(UIAction::Continue)
            }
            "colors" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "on" => self.colors_enabled = true,
                        "off" => self.colors_enabled = false,
                        _ => println!("Usage: colors on/off"),
                    }
                }
                Ok(UIAction::Continue)
            }
            "materialize" => {
                if parts.len() > 1 {
                    let debugger = self.debugger
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Failed to acquire debugger lock"))?;
                    match debugger.materialize_stage(parts[1]) {
                        Ok(_) => println!("Materialized data for stage: {}", parts[1]),
                        Err(e) => println!("Failed to materialize: {e}"),
                    }
                } else {
                    println!("Usage: materialize <stage_id>");
                }
                Ok(UIAction::Continue)
            }
            _ => {
                println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
                Ok(UIAction::Continue)
            }
        }
    }
    /// Get user input
    fn get_user_input(&self) -> Result<Option<String>> {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => Ok(Some(input.trim().to_string())),
            Err(e) => Err(anyhow::anyhow!("Failed to read input: {e}")),
        }
    }
    // Helper methods for UI rendering
    fn clear_screen(&self) -> Result<()> {
        print!("\x1b[2J\x1b[H");
        io::stdout().flush()?;
        Ok(())
    }
    fn print_header(&self) -> Result<()> {
        if self.colors_enabled {
            println!("\x1b[1;34m╔══════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
            println!("\x1b[1;34m║                        RUCHY DATAFLOW DEBUGGER                              ║\x1b[0m");
            println!("\x1b[1;34m╚══════════════════════════════════════════════════════════════════════════════╝\x1b[0m");
        } else {
            println!("===============================================================================");
            println!("                        RUCHY DATAFLOW DEBUGGER                              ");
            println!("===============================================================================");
        }
        Ok(())
    }
    fn print_title(&self, title: &str) -> Result<()> {
        println!();
        if self.colors_enabled {
            println!("\x1b[1;36m{title}\x1b[0m");
        } else {
            println!("{title}");
        }
        println!();
        Ok(())
    }
    fn print_section_header(&self, header: &str) -> Result<()> {
        if self.colors_enabled {
            println!("\x1b[1;33m{header}:\x1b[0m");
        } else {
            println!("{header}:");
        }
        Ok(())
    }
    fn print_separator(&self) -> Result<()> {
        println!("{}", "-".repeat(80));
        Ok(())
    }
    fn print_help_hint(&self) -> Result<()> {
        println!("Type 'help' for commands, 'quit' to exit");
        println!();
        Ok(())
    }
    fn print_status_bar(&self) -> Result<()> {
        let status = format!("Mode: {:?} | Auto-refresh: {} | Colors: {}", 
                           self.display_mode, 
                           self.config.auto_refresh,
                           self.colors_enabled);
        if self.colors_enabled {
            println!("\n\x1b[7m{status:<80}\x1b[0m");
        } else {
            println!("\n{status}");
        }
        Ok(())
    }
    fn print_command_prompt(&self) -> Result<()> {
        println!();
        Ok(())
    }
    fn get_terminal_size() -> (u16, u16) {
        // In a real implementation, would detect actual terminal size
        (80, 24)
    }
    fn format_bytes(&self, bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        if size >= 100.0 {
            format!("{:.0}{}", size, UNITS[unit_index])
        } else if size >= 10.0 {
            format!("{:.1}{}", size, UNITS[unit_index])
        } else {
            format!("{:.2}{}", size, UNITS[unit_index])
        }
    }
    fn format_timestamp(&self, timestamp: std::time::SystemTime) -> String {
        // Simplified timestamp formatting
        format!("{:?}", timestamp.duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0)).as_secs())
    }
    fn render_session_info(&self, session: &SessionState) -> Result<()> {
        self.print_section_header("Session Status")?;
        println!("Active: {}", session.active);
        if let Some(current) = &session.current_stage {
            println!("Current Stage: {current}");
        }
        println!("Total Time: {}ms", session.total_execution_time.as_millis());
        println!("Breakpoints Hit: {}", session.breakpoints_hit);
        Ok(())
    }
    // Sample data methods (in real implementation, these would come from actual debugger state)
    fn get_sample_stages(&self) -> Vec<PipelineStage> {
        use crate::runtime::dataflow_debugger::{StageType, StageStatus};
        vec![
            PipelineStage {
                stage_id: "load_data".to_string(),
                stage_name: "Load CSV Data".to_string(),
                stage_type: StageType::Load,
                status: StageStatus::Completed,
                input_schema: None,
                output_schema: None,
                execution_time: Some(Duration::from_millis(120)),
                memory_usage: Some(1024 * 1024 * 5), // 5MB
                rows_processed: Some(10000),
                metadata: HashMap::new(),
            },
            PipelineStage {
                stage_id: "filter_age".to_string(),
                stage_name: "Filter by Age".to_string(),
                stage_type: StageType::Filter,
                status: StageStatus::Completed,
                input_schema: None,
                output_schema: None,
                execution_time: Some(Duration::from_millis(45)),
                memory_usage: Some(1024 * 1024 * 3), // 3MB
                rows_processed: Some(7500),
                metadata: HashMap::new(),
            },
            PipelineStage {
                stage_id: "group_by_city".to_string(),
                stage_name: "Group by City".to_string(),
                stage_type: StageType::GroupBy,
                status: StageStatus::Running,
                input_schema: None,
                output_schema: None,
                execution_time: None,
                memory_usage: None,
                rows_processed: None,
                metadata: HashMap::new(),
            },
        ]
    }
    fn get_sample_stage(&self, _stage_id: &str) -> PipelineStage {
        use crate::runtime::dataflow_debugger::{StageType, StageStatus, DataSchema, ColumnDef, DataType};
        PipelineStage {
            stage_id: "load_data".to_string(),
            stage_name: "Load CSV Data".to_string(),
            stage_type: StageType::Load,
            status: StageStatus::Completed,
            input_schema: None,
            output_schema: Some(DataSchema {
                columns: vec![
                    ColumnDef {
                        name: "id".to_string(),
                        data_type: DataType::Integer,
                        nullable: false,
                    },
                    ColumnDef {
                        name: "name".to_string(),
                        data_type: DataType::String,
                        nullable: false,
                    },
                    ColumnDef {
                        name: "age".to_string(),
                        data_type: DataType::Integer,
                        nullable: false,
                    },
                    ColumnDef {
                        name: "city".to_string(),
                        data_type: DataType::String,
                        nullable: true,
                    },
                ],
                schema_hash: 12345,
            }),
            execution_time: Some(Duration::from_millis(120)),
            memory_usage: Some(1024 * 1024 * 5), // 5MB
            rows_processed: Some(10000),
            metadata: HashMap::from([
                ("file_path".to_string(), "/data/users.csv".to_string()),
                ("encoding".to_string(), "UTF-8".to_string()),
            ]),
        }
    }
    fn get_sample_materialized_data(&self, _stage_id: &str) -> MaterializedFrame {
        use crate::runtime::dataflow_debugger::{
            MaterializedFrame, DataSchema, ColumnDef, DataType, DataRow, DataValue
        };
        MaterializedFrame {
            stage_id: "load_data".to_string(),
            schema: DataSchema {
                columns: vec![
                    ColumnDef {
                        name: "id".to_string(),
                        data_type: DataType::Integer,
                        nullable: false,
                    },
                    ColumnDef {
                        name: "name".to_string(),
                        data_type: DataType::String,
                        nullable: false,
                    },
                    ColumnDef {
                        name: "age".to_string(),
                        data_type: DataType::Integer,
                        nullable: false,
                    },
                ],
                schema_hash: 12345,
            },
            sample_data: vec![
                DataRow {
                    values: vec![
                        DataValue::Integer(1),
                        DataValue::String("Alice".to_string()),
                        DataValue::Integer(30),
                    ],
                },
                DataRow {
                    values: vec![
                        DataValue::Integer(2),
                        DataValue::String("Bob".to_string()),
                        DataValue::Integer(25),
                    ],
                },
                DataRow {
                    values: vec![
                        DataValue::Integer(3),
                        DataValue::String("Charlie".to_string()),
                        DataValue::Integer(35),
                    ],
                },
            ],
            total_rows: 10000,
            timestamp: std::time::SystemTime::now(),
            memory_size: 1024 * 50, // 50KB
        }
    }
}
/// UI action results
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIAction {
    /// Continue UI loop
    Continue,
    /// Exit UI
    Exit,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::dataflow_debugger::{
        DataflowDebugger, DataflowConfig
    };
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    // Helper functions for consistent test setup
    fn create_test_debugger() -> Arc<Mutex<DataflowDebugger>> {
        let config = DataflowConfig::default();
        Arc::new(Mutex::new(DataflowDebugger::new(config)))
    }
    fn create_test_ui_config() -> UIConfig {
        UIConfig {
            max_preview_rows: 5,
            max_history_events: 10,
            auto_refresh: false,
            refresh_interval_ms: 500,
            show_metrics: true,
            enable_colors: false, // Disable for consistent testing
            compact_mode: true,
        }
    }
    fn create_test_ui_with_config(config: UIConfig) -> DataflowUI {
        let debugger = create_test_debugger();
        DataflowUI::new(debugger, config)
    }
    fn create_test_ui() -> DataflowUI {
        create_test_ui_with_config(create_test_ui_config())
    }
    fn create_test_pipeline_stage() -> PipelineStage {
        use crate::runtime::dataflow_debugger::{StageType, StageStatus};
        PipelineStage {
            stage_id: "test_stage".to_string(),
            stage_name: "Test Stage".to_string(),
            stage_type: StageType::Filter,
            status: StageStatus::Running,
            input_schema: None,
            output_schema: None,
            execution_time: Some(Duration::from_millis(150)),
            memory_usage: Some(1024 * 64), // 64KB
            rows_processed: Some(500),
            metadata: std::collections::HashMap::new(),
        }
    }
    fn create_test_materialized_frame() -> MaterializedFrame {
        use crate::runtime::dataflow_debugger::{DataSchema, ColumnDef, DataType, DataRow, DataValue};
        use std::time::SystemTime;
        MaterializedFrame {
            stage_id: "test_stage".to_string(),
            schema: DataSchema {
                columns: vec![
                    ColumnDef {
                        name: "id".to_string(),
                        data_type: DataType::Integer,
                        nullable: false,
                    },
                    ColumnDef {
                        name: "name".to_string(),
                        data_type: DataType::String,
                        nullable: false,
                    },
                ],
                schema_hash: 54321,
            },
            sample_data: vec![
                DataRow {
                    values: vec![
                        DataValue::Integer(1),
                        DataValue::String("Test User".to_string()),
                    ],
                },
            ],
            total_rows: 100,
            timestamp: SystemTime::now(),
            memory_size: 1024 * 8, // 8KB
        }
    }
    // ========== UIConfig Tests ==========
    #[test]
    fn test_ui_config_default() {
        let config = UIConfig::default();
        assert_eq!(config.max_preview_rows, 20);
        assert_eq!(config.max_history_events, 100);
        assert!(config.auto_refresh);
        assert_eq!(config.refresh_interval_ms, 1000);
        assert!(config.show_metrics);
        assert!(config.enable_colors);
        assert!(!config.compact_mode);
    }
    #[test]
    fn test_ui_config_clone() {
        let config1 = UIConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.max_preview_rows, config2.max_preview_rows);
        assert_eq!(config1.auto_refresh, config2.auto_refresh);
    }
    #[test]
    fn test_ui_config_debug() {
        let config = UIConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("UIConfig"));
        assert!(debug_str.contains("max_preview_rows"));
        assert!(debug_str.contains("auto_refresh"));
    }
    // ========== DisplayMode Tests ==========
    #[test]
    fn test_display_mode_variants() {
        let modes = vec![
            DisplayMode::Overview,
            DisplayMode::StageDetail("test_stage".to_string()),
            DisplayMode::Breakpoints,
            DisplayMode::DataViewer("test_data".to_string()),
            DisplayMode::Metrics,
            DisplayMode::History,
            DisplayMode::Diff("stage1".to_string(), "stage2".to_string()),
            DisplayMode::Help,
        ];
        assert_eq!(modes.len(), 8);
        assert_eq!(modes[0], DisplayMode::Overview);
    }
    #[test]
    fn test_display_mode_equality() {
        let mode1 = DisplayMode::StageDetail("test".to_string());
        let mode2 = DisplayMode::StageDetail("test".to_string());
        let mode3 = DisplayMode::StageDetail("other".to_string());
        assert_eq!(mode1, mode2);
        assert_ne!(mode1, mode3);
        assert_ne!(mode1, DisplayMode::Overview);
    }
    #[test]
    fn test_display_mode_clone() {
        let mode1 = DisplayMode::Diff("a".to_string(), "b".to_string());
        let mode2 = mode1.clone();
        assert_eq!(mode1, mode2);
    }
    #[test]
    fn test_display_mode_debug() {
        let mode = DisplayMode::DataViewer("test_viewer".to_string());
        let debug_str = format!("{mode:?}");
        assert!(debug_str.contains("DataViewer"));
        assert!(debug_str.contains("test_viewer"));
    }
    // ========== UIAction Tests ==========
    #[test]
    fn test_ui_action_variants() {
        let actions = [UIAction::Continue, UIAction::Exit];
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], UIAction::Continue);
        assert_eq!(actions[1], UIAction::Exit);
    }
    #[test]
    fn test_ui_action_equality() {
        assert_eq!(UIAction::Continue, UIAction::Continue);
        assert_eq!(UIAction::Exit, UIAction::Exit);
        assert_ne!(UIAction::Continue, UIAction::Exit);
    }
    #[test]
    fn test_ui_action_clone_debug() {
        let action = UIAction::Continue;
        let cloned = action.clone();
        assert_eq!(action, cloned);
        let debug_str = format!("{action:?}");
        assert!(debug_str.contains("Continue"));
    }
    // ========== DataflowUI Creation Tests ==========
    #[test]
    fn test_dataflow_ui_creation() {
        let debugger = create_test_debugger();
        let config = create_test_ui_config();
        let ui = DataflowUI::new(debugger, config.clone());
        assert_eq!(ui.display_mode, DisplayMode::Overview);
        assert_eq!(ui.config.max_preview_rows, config.max_preview_rows);
        assert!(!ui.colors_enabled);
    }
    #[test]
    fn test_dataflow_ui_with_default_config() {
        let debugger = create_test_debugger();
        let ui = DataflowUI::new(debugger, UIConfig::default());
        assert_eq!(ui.display_mode, DisplayMode::Overview);
        assert_eq!(ui.config.max_preview_rows, 20);
        assert!(ui.colors_enabled);
    }
    #[test]
    fn test_dataflow_ui_terminal_size() {
        let ui = create_test_ui();
        // Terminal size should be initialized
        assert!(ui.terminal_size.0 > 0);
        assert!(ui.terminal_size.1 > 0);
    }
    #[test]
    fn test_dataflow_ui_refresh_timing() {
        let ui = create_test_ui();
        let now = Instant::now();
        // Last refresh should be recent
        assert!(now.duration_since(ui.last_refresh) < Duration::from_secs(1));
    }
    // ========== Display Mode Management Tests ==========
    #[test]
    fn test_set_display_mode() {
        let mut ui = create_test_ui();
        assert_eq!(ui.display_mode, DisplayMode::Overview);
        ui.display_mode = DisplayMode::Metrics;
        assert_eq!(ui.display_mode, DisplayMode::Metrics);
        ui.display_mode = DisplayMode::StageDetail("test".to_string());
        assert_eq!(ui.display_mode, DisplayMode::StageDetail("test".to_string()));
    }
    #[test]
    fn test_get_current_display_mode() {
        let mut ui = create_test_ui();
        assert_eq!(ui.display_mode, DisplayMode::Overview);
        ui.display_mode = DisplayMode::Help;
        assert_eq!(ui.display_mode, DisplayMode::Help);
    }
    #[test]
    fn test_toggle_colors() {
        let mut ui = create_test_ui();
        let initial_colors = ui.colors_enabled;
        ui.colors_enabled = !ui.colors_enabled;
        assert_eq!(ui.colors_enabled, !initial_colors);
        ui.colors_enabled = !ui.colors_enabled;
        assert_eq!(ui.colors_enabled, initial_colors);
    }
    // ========== Display Rendering Tests ==========
    #[test]
    fn test_render_overview() {
        let ui = create_test_ui();
        let result = ui.render_overview();
        assert!(result.is_ok());
    }
    #[test]
    fn test_render_stage_detail() {
        let ui = create_test_ui();
        let stage_id = "test_stage";
        let result = ui.render_stage_detail(stage_id);
        assert!(result.is_ok());
    }
    #[test]
    fn test_render_breakpoints() {
        let ui = create_test_ui();
        let result = ui.render_breakpoints();
        assert!(result.is_ok());
    }
    #[test]
    fn test_render_data_viewer() {
        let ui = create_test_ui();
        let stage_id = "test_stage";
        let result = ui.render_data_viewer(stage_id);
        assert!(result.is_ok());
    }
    #[test]
    fn test_render_metrics() {
        let ui = create_test_ui();
        let result = ui.render_metrics();
        assert!(result.is_ok());
    }
    #[test]
    fn test_render_history() {
        let ui = create_test_ui();
        let result = ui.render_history();
        assert!(result.is_ok());
    }
    #[test]
    fn test_render_diff() {
        let ui = create_test_ui();
        let stage1 = "stage_a";
        let stage2 = "stage_b";
        let result = ui.render_diff(stage1, stage2);
        assert!(result.is_ok());
    }
    #[test]
    fn test_render_help() {
        let ui = create_test_ui();
        let result = ui.render_help();
        assert!(result.is_ok());
    }
    // ========== Input Handling Tests ==========
    #[test]
    fn test_handle_command_navigation() {
        let mut ui = create_test_ui();
        // Test overview mode navigation
        assert_eq!(ui.handle_command("overview").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::Overview);
        assert_eq!(ui.handle_command("metrics").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::Metrics);
        assert_eq!(ui.handle_command("history").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::History);
        assert_eq!(ui.handle_command("help").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::Help);
    }
    #[test]
    fn test_handle_command_quit() {
        let mut ui = create_test_ui();
        assert_eq!(ui.handle_command("quit").unwrap(), UIAction::Exit);
        assert_eq!(ui.handle_command("exit").unwrap(), UIAction::Exit);
        assert_eq!(ui.handle_command("q").unwrap(), UIAction::Exit);
    }
    #[test]
    fn test_handle_command_breakpoints() {
        let mut ui = create_test_ui();
        assert_eq!(ui.handle_command("breakpoints").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::Breakpoints);
    }
    #[test]
    fn test_handle_command_colors() {
        let mut ui = create_test_ui();
        ui.handle_command("colors on").unwrap();
        assert!(ui.colors_enabled);
        ui.handle_command("colors off").unwrap();
        assert!(!ui.colors_enabled);
    }
    #[test]
    fn test_handle_command_refresh() {
        let mut ui = create_test_ui();
        // Small delay to ensure time difference
        std::thread::sleep(Duration::from_millis(1));
        assert_eq!(ui.handle_command("refresh").unwrap(), UIAction::Continue);
        // The refresh command updates last_refresh to trigger a refresh
    }
    #[test]
    fn test_handle_command_unknown() {
        let mut ui = create_test_ui();
        let initial_mode = ui.display_mode.clone();
        assert_eq!(ui.handle_command("xyz").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, initial_mode); // Should not change
    }
    // ========== Refresh and Update Tests ==========
    #[test]
    fn test_refresh_timing() {
        let ui = create_test_ui();
        // Check that refresh interval is properly set
        assert_eq!(ui.refresh_interval, Duration::from_millis(500));
        // Check that last_refresh is recent
        let now = Instant::now();
        assert!(now.duration_since(ui.last_refresh) < Duration::from_secs(1));
    }
    #[test]
    fn test_auto_refresh_config() {
        let mut config = create_test_ui_config();
        config.auto_refresh = true;
        config.refresh_interval_ms = 2000;
        let ui = create_test_ui_with_config(config);
        assert!(ui.config.auto_refresh);
        assert_eq!(ui.refresh_interval, Duration::from_millis(2000));
    }
    #[test]
    fn test_terminal_size_initialization() {
        let ui = create_test_ui();
        // Terminal size should be initialized to default values
        assert!(ui.terminal_size.0 > 0);
        assert!(ui.terminal_size.1 > 0);
    }
    // ========== Data Formatting Tests ==========
    #[test]
    fn test_format_bytes() {
        let ui = create_test_ui();
        assert_eq!(ui.format_bytes(512), "512B");
        assert_eq!(ui.format_bytes(1536), "1.50KB");  // 1.5 * 1024
        assert_eq!(ui.format_bytes(1024 * 1024), "1.00MB");
        assert_eq!(ui.format_bytes(1024 * 1024 * 1024), "1.00GB");
    }
    #[test]
    fn test_format_timestamp() {
        let ui = create_test_ui();
        let timestamp = std::time::SystemTime::now();
        let formatted = ui.format_timestamp(timestamp);
        // Should be a string representation of seconds since epoch
        assert!(!formatted.is_empty());
        // Should be numeric
        assert!(formatted.parse::<u64>().is_ok());
    }
    #[test] 
    fn test_sample_data_creation() {
        let ui = create_test_ui();
        let stages = ui.get_sample_stages();
        assert_eq!(stages.len(), 3);
        assert_eq!(stages[0].stage_id, "load_data");
        assert_eq!(stages[1].stage_id, "filter_age");
        assert_eq!(stages[2].stage_id, "group_by_city");
    }
    #[test]
    fn test_sample_stage_detail() {
        let ui = create_test_ui();
        let stage = ui.get_sample_stage("any_id");
        assert_eq!(stage.stage_id, "load_data");
        assert_eq!(stage.stage_name, "Load CSV Data");
        assert!(stage.execution_time.is_some());
        assert!(stage.memory_usage.is_some());
        assert!(stage.rows_processed.is_some());
    }
    #[test]
    fn test_sample_materialized_data_creation() {
        let ui = create_test_ui();
        let frame = ui.get_sample_materialized_data("test_id");
        assert_eq!(frame.stage_id, "load_data");
        assert_eq!(frame.schema.columns.len(), 3);
        assert_eq!(frame.sample_data.len(), 3);
        assert_eq!(frame.total_rows, 10000);
    }
    // ========== Color Support Tests ==========
    #[test]
    fn test_colors_enabled() {
        let config_with_colors = UIConfig { enable_colors: true, ..create_test_ui_config() };
        let ui_with_colors = create_test_ui_with_config(config_with_colors);
        assert!(ui_with_colors.colors_enabled);
        let config_without_colors = UIConfig { enable_colors: false, ..create_test_ui_config() };
        let ui_without_colors = create_test_ui_with_config(config_without_colors);
        assert!(!ui_without_colors.colors_enabled);
    }
    #[test]
    fn test_color_configuration() {
        // Test that color setting from config is properly applied
        let mut ui = create_test_ui();
        assert!(!ui.colors_enabled); // Test config has colors disabled
        // Test that color setting can be changed
        ui.colors_enabled = true;
        assert!(ui.colors_enabled);
    }
    #[test]
    fn test_color_impact_on_rendering() {
        let mut ui = create_test_ui();
        // Test with colors disabled (should not crash)
        ui.colors_enabled = false;
        let result = ui.render_overview();
        assert!(result.is_ok());
        // Test with colors enabled (should not crash)
        ui.colors_enabled = true;
        let result = ui.render_overview();
        assert!(result.is_ok());
    }
    // ========== Configuration Tests ==========
    #[test]
    fn test_ui_configuration_settings() {
        let config = UIConfig {
            max_preview_rows: 10,
            max_history_events: 50,
            auto_refresh: false,
            refresh_interval_ms: 2000,
            show_metrics: false,
            enable_colors: true,
            compact_mode: true,
        };
        let ui = create_test_ui_with_config(config);
        assert_eq!(ui.config.max_preview_rows, 10);
        assert_eq!(ui.config.max_history_events, 50);
        assert!(!ui.config.auto_refresh);
        assert_eq!(ui.config.refresh_interval_ms, 2000);
        assert!(!ui.config.show_metrics);
        assert!(ui.config.enable_colors);
        assert!(ui.config.compact_mode);
    }
    #[test]
    fn test_display_mode_variations() {
        // Test that all display modes can be created
        let modes = vec![
            DisplayMode::Overview,
            DisplayMode::StageDetail("test".to_string()),
            DisplayMode::Breakpoints,
            DisplayMode::DataViewer("data_test".to_string()),
            DisplayMode::Metrics,
            DisplayMode::History,
            DisplayMode::Diff("a".to_string(), "b".to_string()),
            DisplayMode::Help,
        ];
        for mode in modes {
            // Each mode should be created without errors
            assert_ne!(format!("{mode:?}"), "");
        }
    }
    // ========== Integration Tests ==========
    #[test]
    fn test_refresh_display_all_modes() {
        let mut ui = create_test_ui();
        // Test different display modes
        ui.display_mode = DisplayMode::Overview;
        assert!(ui.refresh_display().is_ok());
        ui.display_mode = DisplayMode::Metrics;
        assert!(ui.refresh_display().is_ok());
        ui.display_mode = DisplayMode::Help;
        assert!(ui.refresh_display().is_ok());
        ui.display_mode = DisplayMode::History;
        assert!(ui.refresh_display().is_ok());
    }
    #[test]
    fn test_interactive_command_sequence() {
        let mut ui = create_test_ui();
        // Simulate user interaction sequence
        assert_eq!(ui.handle_command("metrics").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::Metrics);
        assert_eq!(ui.handle_command("overview").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::Overview);
        assert_eq!(ui.handle_command("breakpoints").unwrap(), UIAction::Continue);
        assert_eq!(ui.display_mode, DisplayMode::Breakpoints);
        assert_eq!(ui.handle_command("quit").unwrap(), UIAction::Exit);
    }
    #[test]
    fn test_config_variations() {
        let compact_config = UIConfig {
            max_preview_rows: 2,
            compact_mode: true,
            enable_colors: false,
            auto_refresh: false,
            ..Default::default()
        };
        let ui = create_test_ui_with_config(compact_config);
        // Should respect configuration settings
        assert_eq!(ui.config.max_preview_rows, 2);
        assert!(ui.config.compact_mode);
        assert!(!ui.colors_enabled);
        assert!(!ui.config.auto_refresh);
    }
    #[test]
    fn test_error_handling_graceful() {
        let ui = create_test_ui();
        // Test rendering with various stage IDs
        assert!(ui.render_stage_detail("nonexistent_stage").is_ok());
        assert!(ui.render_data_viewer("missing_data").is_ok());
        assert!(ui.render_diff("stage1", "stage2").is_ok());
        // All should handle gracefully without panicking
    }
}