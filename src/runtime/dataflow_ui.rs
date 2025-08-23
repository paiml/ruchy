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
                self.last_refresh = Instant::now().checked_sub(self.refresh_interval).unwrap();
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
        format!("{:?}", timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
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