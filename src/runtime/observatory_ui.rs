//! Terminal UI dashboard for actor observatory (RUCHY-0817)
//!
//! Provides a real-time terminal interface for monitoring actor systems,
//! displaying message traces, system metrics, and deadlock information.

use crate::runtime::observatory::{
    ActorObservatory, ActorState, MessageStatus,
};
use anyhow::Result;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Terminal UI dashboard for actor system monitoring
pub struct ObservatoryDashboard {
    /// Reference to the observatory
    observatory: Arc<Mutex<ActorObservatory>>,
    
    /// Dashboard configuration
    config: DashboardConfig,
    
    /// Current display mode
    display_mode: DisplayMode,
    
    /// Last update time
    last_update: Instant,
    
    /// Terminal dimensions
    terminal_size: (u16, u16), // (width, height)
    
    /// Scroll positions for different views
    #[allow(dead_code)] // Future feature for scrolling
    scroll_positions: HashMap<DisplayMode, usize>,
}

/// Configuration for the dashboard display
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Refresh interval in milliseconds
    pub refresh_interval_ms: u64,
    
    /// Maximum number of traces to display
    pub max_traces_display: usize,
    
    /// Maximum number of actors to display in actor list
    pub max_actors_display: usize,
    
    /// Enable color output
    pub enable_colors: bool,
    
    /// Show detailed actor information
    pub show_actor_details: bool,
    
    /// Show message processing times
    pub show_timing_info: bool,
    
    /// Auto-refresh the display
    pub auto_refresh: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: 1000, // 1 second
            max_traces_display: 50,
            max_actors_display: 20,
            enable_colors: true,
            show_actor_details: true,
            show_timing_info: true,
            auto_refresh: true,
        }
    }
}

/// Different display modes for the dashboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayMode {
    /// Overview of the entire system
    Overview,
    /// Actor list with detailed information
    ActorList,
    /// Message trace view
    MessageTraces,
    /// System metrics and performance
    Metrics,
    /// Deadlock detection and analysis
    Deadlocks,
    /// Help screen
    Help,
}

/// Color codes for terminal output
#[allow(dead_code)]
pub struct Colors {
    pub reset: &'static str,
    pub bold: &'static str,
    pub red: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub blue: &'static str,
    pub magenta: &'static str,
    pub cyan: &'static str,
    pub white: &'static str,
    pub gray: &'static str,
}

impl Colors {
    pub const fn new(enable_colors: bool) -> Self {
        if enable_colors {
            Self {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                red: "\x1b[31m",
                green: "\x1b[32m",
                yellow: "\x1b[33m",
                blue: "\x1b[34m",
                magenta: "\x1b[35m",
                cyan: "\x1b[36m",
                white: "\x1b[37m",
                gray: "\x1b[90m",
            }
        } else {
            Self {
                reset: "",
                bold: "",
                red: "",
                green: "",
                yellow: "",
                blue: "",
                magenta: "",
                cyan: "",
                white: "",
                gray: "",
            }
        }
    }
}

impl ObservatoryDashboard {
    /// Create a new dashboard
    pub fn new(observatory: Arc<Mutex<ActorObservatory>>, config: DashboardConfig) -> Self {
        Self {
            observatory,
            config,
            display_mode: DisplayMode::Overview,
            last_update: Instant::now(),
            terminal_size: (80, 24), // Default size
            scroll_positions: HashMap::new(),
        }
    }
    
    /// Start the interactive dashboard
    pub fn start_interactive(&mut self) -> Result<()> {
        // Clear screen and hide cursor
        print!("\x1b[2J\x1b[?25l");
        io::stdout().flush()?;
        
        loop {
            self.update_terminal_size()?;
            self.render_current_view()?;
            
            if self.config.auto_refresh {
                // In a real implementation, we would handle keyboard input here
                // For now, just refresh after the interval
                std::thread::sleep(Duration::from_millis(self.config.refresh_interval_ms));
            } else {
                // Wait for user input
                break;
            }
        }
        
        // Restore cursor and clear screen
        print!("\x1b[?25h\x1b[2J\x1b[H");
        io::stdout().flush()?;
        
        Ok(())
    }
    
    /// Render the current view to the terminal
    pub fn render_current_view(&mut self) -> Result<()> {
        // Clear screen and move to top
        print!("\x1b[2J\x1b[H");
        
        match self.display_mode {
            DisplayMode::Overview => self.render_overview(),
            DisplayMode::ActorList => self.render_actor_list(),
            DisplayMode::MessageTraces => self.render_message_traces(),
            DisplayMode::Metrics => self.render_metrics(),
            DisplayMode::Deadlocks => self.render_deadlocks(),
            DisplayMode::Help => self.render_help(),
        }?;
        
        // Render status bar at bottom
        self.render_status_bar()?;
        
        io::stdout().flush()?;
        self.last_update = Instant::now();
        
        Ok(())
    }
    
    /// Render the overview screen
    fn render_overview(&self) -> Result<()> {
        let colors = Colors::new(self.config.enable_colors);
        
        println!("{}{}Ruchy Actor Observatory - System Overview{}", 
                 colors.bold, colors.cyan, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        // Get system metrics
        let observatory = self.observatory.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire observatory lock"))?;
        
        let metrics = observatory.get_metrics()?;
        let snapshots = observatory.get_actor_snapshots()?;
        let recent_traces = observatory.get_traces(Some(10), None)?;
        let deadlocks = observatory.detect_deadlocks()?;
        
        // System status summary
        println!("{}System Status:{}", colors.bold, colors.reset);
        println!("  Active Actors: {}{}{}", colors.green, metrics.active_actors, colors.reset);
        println!("  Messages Processed: {}{}{}", colors.blue, metrics.total_messages_processed, colors.reset);
        println!("  Messages/sec: {}{:.2}{}", colors.yellow, metrics.system_messages_per_second, colors.reset);
        println!("  Avg Mailbox Size: {}{:.1}{}", colors.cyan, metrics.avg_mailbox_size, colors.reset);
        
        if !deadlocks.is_empty() {
            println!("  {}Deadlocks Detected: {}{}{}", 
                     colors.red, colors.bold, deadlocks.len(), colors.reset);
        }
        
        println!();
        
        // Actor states summary
        println!("{}Actor States:{}", colors.bold, colors.reset);
        let mut state_counts = HashMap::new();
        for snapshot in snapshots.values() {
            *state_counts.entry(&snapshot.state).or_insert(0) += 1;
        }
        
        for (state, count) in &state_counts {
            let state_color = match state {
                ActorState::Running => colors.green,
                ActorState::Processing(_) => colors.yellow,
                ActorState::Failed(_) => colors.red,
                ActorState::Restarting => colors.magenta,
                _ => colors.gray,
            };
            println!("  {}: {}{}{}", state_display(state), state_color, count, colors.reset);
        }
        
        println!();
        
        // Recent message activity
        println!("{}Recent Messages:{}", colors.bold, colors.reset);
        if recent_traces.is_empty() {
            println!("  No recent messages");
        } else {
            for trace in recent_traces.iter().take(5) {
                let status_color = match trace.status {
                    MessageStatus::Completed => colors.green,
                    MessageStatus::Failed => colors.red,
                    MessageStatus::Processing => colors.yellow,
                    _ => colors.gray,
                };
                
                let duration_str = if let Some(duration) = trace.processing_duration_us {
                    format!(" ({duration}µs)")
                } else {
                    String::new()
                };
                
                println!("  {} → {}: {}{:?}{}{}", 
                         trace.source.map_or("external".to_string(), |id| id.to_string()),
                         trace.destination,
                         status_color,
                         trace.status,
                         colors.reset,
                         duration_str);
            }
        }
        
        Ok(())
    }
    
    /// Render the actor list view
    fn render_actor_list(&self) -> Result<()> {
        let colors = Colors::new(self.config.enable_colors);
        
        println!("{}{}Ruchy Actor Observatory - Actor List{}", 
                 colors.bold, colors.cyan, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        let observatory = self.observatory.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire observatory lock"))?;
        
        let snapshots = observatory.get_actor_snapshots()?;
        
        if snapshots.is_empty() {
            println!("No active actors");
            return Ok(());
        }
        
        // Table headers
        println!("{}ID        Name            State       Mailbox  Messages  Avg Time{}", 
                 colors.bold, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        // Sort actors by ID for consistent display
        let mut sorted_snapshots: Vec<_> = snapshots.values().collect();
        sorted_snapshots.sort_by_key(|s| s.actor_id.0);
        
        for snapshot in sorted_snapshots.iter().take(self.config.max_actors_display) {
            let state_color = match snapshot.state {
                ActorState::Running => colors.green,
                ActorState::Processing(_) => colors.yellow,
                ActorState::Failed(_) => colors.red,
                ActorState::Restarting => colors.magenta,
                _ => colors.gray,
            };
            
            let state_display = match &snapshot.state {
                ActorState::Processing(msg_type) => format!("Proc({msg_type})"),
                ActorState::Failed(reason) => format!("Failed({reason})"),
                other => state_display(other),
            };
            
            println!("{:<9} {:<15} {}{:<11}{} {:<8} {:<9} {:.1}µs",
                     snapshot.actor_id,
                     snapshot.name,
                     state_color,
                     state_display,
                     colors.reset,
                     snapshot.mailbox_size,
                     snapshot.message_stats.total_processed,
                     snapshot.message_stats.avg_processing_time_us);
        }
        
        Ok(())
    }
    
    /// Render the message traces view
    fn render_message_traces(&self) -> Result<()> {
        let colors = Colors::new(self.config.enable_colors);
        
        println!("{}{}Ruchy Actor Observatory - Message Traces{}", 
                 colors.bold, colors.cyan, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        let observatory = self.observatory.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire observatory lock"))?;
        
        let traces = observatory.get_traces(Some(self.config.max_traces_display), None)?;
        
        if traces.is_empty() {
            println!("No message traces available");
            return Ok(());
        }
        
        // Table headers
        println!("{}Time     Source    Destination  Status      Duration  Message{}", 
                 colors.bold, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        for trace in &traces {
            let timestamp = format_timestamp(trace.timestamp);
            let source = trace.source.map_or("external".to_string(), |id| id.to_string());
            
            let status_color = match trace.status {
                MessageStatus::Completed => colors.green,
                MessageStatus::Failed => colors.red,
                MessageStatus::Processing => colors.yellow,
                MessageStatus::Queued => colors.blue,
                MessageStatus::Dropped => colors.gray,
            };
            
            let duration_str = if let Some(duration) = trace.processing_duration_us {
                format!("{duration:>7}µs")
            } else {
                "       -".to_string()
            };
            
            let message_preview = format_message_preview(&trace.message);
            
            println!("{} {:<9} {:<12} {}{:<11}{} {} {}",
                     timestamp,
                     source,
                     trace.destination,
                     status_color,
                     format!("{:?}", trace.status),
                     colors.reset,
                     duration_str,
                     message_preview);
        }
        
        Ok(())
    }
    
    /// Render the system metrics view
    fn render_metrics(&self) -> Result<()> {
        let colors = Colors::new(self.config.enable_colors);
        
        println!("{}{}Ruchy Actor Observatory - System Metrics{}", 
                 colors.bold, colors.cyan, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        let observatory = self.observatory.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire observatory lock"))?;
        
        let metrics = observatory.get_metrics()?;
        let uptime = observatory.uptime();
        
        // System metrics
        println!("{}System Information:{}", colors.bold, colors.reset);
        println!("  Observatory Uptime: {}", format_duration(uptime));
        println!("  Last Updated: {}", format_timestamp(metrics.last_updated));
        println!();
        
        println!("{}Actor Metrics:{}", colors.bold, colors.reset);
        println!("  Active Actors: {}{}{}", colors.green, metrics.active_actors, colors.reset);
        println!("  Total Queued Messages: {}{}{}", colors.yellow, metrics.total_queued_messages, colors.reset);
        println!("  Average Mailbox Size: {}{:.2}{}", colors.cyan, metrics.avg_mailbox_size, colors.reset);
        println!("  Recent Restarts: {}{}{}", colors.red, metrics.recent_restarts, colors.reset);
        println!();
        
        println!("{}Performance Metrics:{}", colors.bold, colors.reset);
        println!("  Total Messages Processed: {}{}{}", colors.blue, metrics.total_messages_processed, colors.reset);
        println!("  System Messages/sec: {}{:.2}{}", colors.green, metrics.system_messages_per_second, colors.reset);
        println!("  Estimated Memory Usage: {}{}{}", colors.magenta, format_bytes(metrics.total_memory_usage), colors.reset);
        
        Ok(())
    }
    
    /// Render the deadlocks view
    fn render_deadlocks(&self) -> Result<()> {
        let colors = Colors::new(self.config.enable_colors);
        
        println!("{}{}Ruchy Actor Observatory - Deadlock Detection{}", 
                 colors.bold, colors.cyan, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        let observatory = self.observatory.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire observatory lock"))?;
        
        let deadlocks = observatory.detect_deadlocks()?;
        
        if deadlocks.is_empty() {
            println!("{}✓ No deadlocks detected{}", colors.green, colors.reset);
            println!();
            println!("The system is currently free of detected deadlocks.");
            println!("Deadlock detection runs automatically in the background.");
        } else {
            println!("{}⚠ {} Deadlock(s) Detected{}", colors.red, deadlocks.len(), colors.reset);
            println!();
            
            for (i, deadlock) in deadlocks.iter().enumerate() {
                println!("{}Deadlock #{}{}", colors.bold, i + 1, colors.reset);
                println!("  Detected: {}", format_timestamp(deadlock.detected_at));
                println!("  Duration: {}ms", deadlock.duration_estimate_ms);
                println!("  Actors Involved: {}{:?}{}", colors.yellow, deadlock.actors, colors.reset);
                println!("  {}Suggestion:{} {}", colors.cyan, colors.reset, deadlock.resolution_suggestion);
                println!();
            }
        }
        
        Ok(())
    }
    
    /// Render the help screen
    fn render_help(&self) -> Result<()> {
        let colors = Colors::new(self.config.enable_colors);
        
        println!("{}{}Ruchy Actor Observatory - Help{}", 
                 colors.bold, colors.cyan, colors.reset);
        println!("{}", "─".repeat(self.terminal_size.0 as usize));
        
        println!("{}Navigation:{}", colors.bold, colors.reset);
        println!("  1 - Overview screen");
        println!("  2 - Actor list");
        println!("  3 - Message traces");
        println!("  4 - System metrics");
        println!("  5 - Deadlock detection");
        println!("  h - This help screen");
        println!("  q - Quit");
        println!();
        
        println!("{}Features:{}", colors.bold, colors.reset);
        println!("  • Live monitoring of actor system state");
        println!("  • Message tracing with filtering capabilities");
        println!("  • Automatic deadlock detection");
        println!("  • Performance metrics and statistics");
        println!("  • Real-time updates every {} seconds", self.config.refresh_interval_ms / 1000);
        
        Ok(())
    }
    
    /// Render the status bar at the bottom of the screen
    fn render_status_bar(&self) -> Result<()> {
        let colors = Colors::new(self.config.enable_colors);
        
        // Move to bottom of screen
        print!("\x1b[{};1H", self.terminal_size.1);
        
        let mode_name = match self.display_mode {
            DisplayMode::Overview => "Overview",
            DisplayMode::ActorList => "Actors",
            DisplayMode::MessageTraces => "Messages",
            DisplayMode::Metrics => "Metrics",
            DisplayMode::Deadlocks => "Deadlocks",
            DisplayMode::Help => "Help",
        };
        
        let status_bar = format!(
            "{}[{}]{} | Last updated: {} | Press 'h' for help | Press 'q' to quit{}",
            colors.bold,
            mode_name,
            colors.reset,
            format_timestamp_short(self.last_update),
            colors.reset
        );
        
        // Print status bar with background
        print!("\x1b[7m{:<width$}\x1b[0m", status_bar, width = self.terminal_size.0 as usize);
        
        Ok(())
    }
    
    /// Update terminal size
    fn update_terminal_size(&mut self) -> Result<()> {
        // In a real implementation, we would get the actual terminal size
        // For now, use default values
        self.terminal_size = (80, 24);
        Ok(())
    }
    
    /// Switch to a different display mode
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }
    
    /// Get current display mode
    pub fn get_display_mode(&self) -> DisplayMode {
        self.display_mode
    }
}

/// Format a state for display
fn state_display(state: &ActorState) -> String {
    match state {
        ActorState::Starting => "Starting".to_string(),
        ActorState::Running => "Running".to_string(),
        ActorState::Processing(msg) => format!("Proc({msg})"),
        ActorState::Restarting => "Restarting".to_string(),
        ActorState::Stopping => "Stopping".to_string(),
        ActorState::Stopped => "Stopped".to_string(),
        ActorState::Failed(reason) => format!("Failed({reason})"),
    }
}

/// Format a timestamp for display
fn format_timestamp(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let now = duration.as_secs();
        if timestamp > now - 60 {
            format!("{}s ago", now - timestamp)
        } else {
            "old".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

/// Format a timestamp for display (short form)
fn format_timestamp_short(instant: Instant) -> String {
    let elapsed = instant.elapsed();
    if elapsed < Duration::from_secs(60) {
        format!("{}s ago", elapsed.as_secs())
    } else {
        format!("{}m ago", elapsed.as_secs() / 60)
    }
}

/// Format a duration for display
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

/// Format bytes for display
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    #[allow(clippy::cast_precision_loss)] // Acceptable for display formatting
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format a message for preview display
fn format_message_preview(message: &crate::runtime::actor::Message) -> String {
    use crate::runtime::actor::Message;
    
    match message {
        Message::Start => "Start".to_string(),
        Message::Stop => "Stop".to_string(),
        Message::Restart => "Restart".to_string(),
        Message::User(msg_type, _) => format!("User({msg_type})"),
        Message::Error(err) => format!("Error({err})"),
        Message::ChildFailed(actor_id, reason) => format!("ChildFailed({actor_id}, {reason})"),
        Message::ChildRestarted(actor_id) => format!("ChildRestarted({actor_id})"),
    }
}