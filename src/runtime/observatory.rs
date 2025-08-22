//! Actor observatory for live system introspection (RUCHY-0817)
//!
//! Provides comprehensive monitoring and debugging capabilities for the actor system,
//! including message tracing, deadlock detection, and performance analysis.

use crate::runtime::actor::{ActorId, ActorSystem, Message};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Actor system observatory for live introspection and monitoring
pub struct ActorObservatory {
    /// Reference to the actor system being observed
    actor_system: Arc<Mutex<ActorSystem>>,
    
    /// Message trace storage
    message_traces: Arc<Mutex<VecDeque<MessageTrace>>>,
    
    /// Actor state snapshots
    actor_snapshots: Arc<Mutex<HashMap<ActorId, ActorSnapshot>>>,
    
    /// Deadlock detection state
    deadlock_detector: Arc<Mutex<DeadlockDetector>>,
    
    /// Observatory configuration
    config: ObservatoryConfig,
    
    /// Active filters for message tracing
    filters: Vec<MessageFilter>,
    
    /// Performance metrics
    metrics: Arc<Mutex<SystemMetrics>>,
    
    /// Observatory start time
    start_time: Instant,
}

/// Configuration for the actor observatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservatoryConfig {
    /// Maximum number of message traces to keep
    pub max_traces: usize,
    
    /// Maximum age for message traces (in seconds)
    pub trace_retention_seconds: u64,
    
    /// Enable deadlock detection
    pub enable_deadlock_detection: bool,
    
    /// Deadlock detection interval (in milliseconds)
    pub deadlock_check_interval_ms: u64,
    
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    
    /// Metrics collection interval (in milliseconds)
    pub metrics_interval_ms: u64,
    
    /// Maximum number of actor snapshots to keep
    pub max_snapshots: usize,
}

impl Default for ObservatoryConfig {
    fn default() -> Self {
        Self {
            max_traces: 10000,
            trace_retention_seconds: 3600, // 1 hour
            enable_deadlock_detection: true,
            deadlock_check_interval_ms: 1000, // 1 second
            enable_metrics: true,
            metrics_interval_ms: 5000, // 5 seconds
            max_snapshots: 1000,
        }
    }
}

/// Message trace entry for debugging and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTrace {
    /// Unique trace ID
    pub trace_id: u64,
    
    /// Timestamp when the message was traced
    pub timestamp: u64,
    
    /// Source actor ID (None for external messages)
    pub source: Option<ActorId>,
    
    /// Destination actor ID
    pub destination: ActorId,
    
    /// The traced message
    pub message: Message,
    
    /// Message processing status
    pub status: MessageStatus,
    
    /// Processing duration in microseconds
    pub processing_duration_us: Option<u64>,
    
    /// Error information if message processing failed
    pub error: Option<String>,
    
    /// Stack depth for nested message calls
    pub stack_depth: usize,
    
    /// Correlation ID for tracking message chains
    pub correlation_id: Option<String>,
}

/// Status of a traced message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Message is queued for processing
    Queued,
    /// Message is currently being processed
    Processing,
    /// Message was processed successfully
    Completed,
    /// Message processing failed
    Failed,
    /// Message was dropped due to actor failure
    Dropped,
}

/// Snapshot of an actor's state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSnapshot {
    /// Actor ID
    pub actor_id: ActorId,
    
    /// Actor name
    pub name: String,
    
    /// Snapshot timestamp
    pub timestamp: u64,
    
    /// Current state of the actor
    pub state: ActorState,
    
    /// Number of messages in the actor's mailbox
    pub mailbox_size: usize,
    
    /// Actor's supervision parent (if any)
    pub parent: Option<ActorId>,
    
    /// Actor's supervised children
    pub children: Vec<ActorId>,
    
    /// Recent message processing statistics
    pub message_stats: MessageStats,
    
    /// Memory usage estimate (in bytes)
    pub memory_usage: Option<usize>,
}

/// Current state of an actor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorState {
    /// Actor is starting up
    Starting,
    /// Actor is running normally
    Running,
    /// Actor is processing a message
    Processing(String), // Message type being processed
    /// Actor is restarting due to failure
    Restarting,
    /// Actor is stopping
    Stopping,
    /// Actor has stopped
    Stopped,
    /// Actor has failed
    Failed(String), // Failure reason
}

/// Message processing statistics for an actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    /// Total messages processed
    pub total_processed: u64,
    
    /// Messages processed per second (recent average)
    pub messages_per_second: f64,
    
    /// Average message processing time in microseconds
    pub avg_processing_time_us: f64,
    
    /// Maximum message processing time in microseconds
    pub max_processing_time_us: u64,
    
    /// Number of failed message processings
    pub failed_messages: u64,
    
    /// Last processing timestamp
    pub last_processed: Option<u64>,
}

impl Default for MessageStats {
    fn default() -> Self {
        Self {
            total_processed: 0,
            messages_per_second: 0.0,
            avg_processing_time_us: 0.0,
            max_processing_time_us: 0,
            failed_messages: 0,
            last_processed: None,
        }
    }
}

/// System-wide performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Total number of active actors
    pub active_actors: usize,
    
    /// Total messages processed across all actors
    pub total_messages_processed: u64,
    
    /// System-wide messages per second
    pub system_messages_per_second: f64,
    
    /// Total memory usage estimate (in bytes)
    pub total_memory_usage: usize,
    
    /// Number of currently queued messages across all actors
    pub total_queued_messages: usize,
    
    /// Average actor mailbox size
    pub avg_mailbox_size: f64,
    
    /// Number of actor restarts in the last period
    pub recent_restarts: u64,
    
    /// Last metrics update timestamp
    pub last_updated: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            active_actors: 0,
            total_messages_processed: 0,
            system_messages_per_second: 0.0,
            total_memory_usage: 0,
            total_queued_messages: 0,
            avg_mailbox_size: 0.0,
            recent_restarts: 0,
            last_updated: current_timestamp(),
        }
    }
}

/// Filter for message tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFilter {
    /// Filter name for identification
    pub name: String,
    
    /// Actor ID to filter by (None for all actors)
    pub actor_id: Option<ActorId>,
    
    /// Actor name pattern to filter by
    pub actor_name_pattern: Option<String>,
    
    /// Message type pattern to filter by
    pub message_type_pattern: Option<String>,
    
    /// Minimum message processing time to include (microseconds)
    pub min_processing_time_us: Option<u64>,
    
    /// Only include failed messages
    pub failed_only: bool,
    
    /// Maximum stack depth to include
    pub max_stack_depth: Option<usize>,
}

/// Deadlock detection system
#[derive(Debug)]
pub struct DeadlockDetector {
    /// Graph of actor message dependencies
    dependency_graph: HashMap<ActorId, HashSet<ActorId>>,
    
    /// Currently blocked actors waiting for responses
    blocked_actors: HashMap<ActorId, Vec<BlockedRequest>>,
    
    /// Last deadlock check timestamp
    last_check: Instant,
    
    /// Detected deadlocks
    detected_deadlocks: Vec<DeadlockCycle>,
}

/// Information about a blocked request
#[derive(Debug, Clone)]
pub struct BlockedRequest {
    /// Actor making the request
    pub requester: ActorId,
    
    /// Actor being requested from
    pub target: ActorId,
    
    /// When the request was made
    pub timestamp: Instant,
    
    /// Timeout for the request
    pub timeout: Duration,
    
    /// Message correlation ID
    pub correlation_id: Option<String>,
}

/// A detected deadlock cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlockCycle {
    /// Actors involved in the deadlock
    pub actors: Vec<ActorId>,
    
    /// When the deadlock was detected
    pub detected_at: u64,
    
    /// Estimated duration of the deadlock
    pub duration_estimate_ms: u64,
    
    /// Suggested resolution strategy
    pub resolution_suggestion: String,
}

impl ActorObservatory {
    /// Create a new actor observatory
    pub fn new(actor_system: Arc<Mutex<ActorSystem>>, config: ObservatoryConfig) -> Self {
        Self {
            actor_system,
            message_traces: Arc::new(Mutex::new(VecDeque::new())),
            actor_snapshots: Arc::new(Mutex::new(HashMap::new())),
            deadlock_detector: Arc::new(Mutex::new(DeadlockDetector::new())),
            config,
            filters: Vec::new(),
            metrics: Arc::new(Mutex::new(SystemMetrics::default())),
            start_time: Instant::now(),
        }
    }
    
    /// Add a message filter for tracing
    pub fn add_filter(&mut self, filter: MessageFilter) {
        self.filters.push(filter);
    }
    
    /// Remove a message filter by name
    pub fn remove_filter(&mut self, name: &str) -> bool {
        let initial_len = self.filters.len();
        self.filters.retain(|f| f.name != name);
        self.filters.len() != initial_len
    }
    
    /// Get current list of filters
    pub fn get_filters(&self) -> &[MessageFilter] {
        &self.filters
    }
    
    /// Record a message trace
    pub fn trace_message(&self, trace: MessageTrace) -> Result<()> {
        let mut traces = self.message_traces
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire message traces lock"))?;
        
        // Apply filters
        if !self.message_matches_filters(&trace) {
            return Ok(());
        }
        
        traces.push_back(trace);
        
        // Enforce retention limits
        while traces.len() > self.config.max_traces {
            traces.pop_front();
        }
        
        // Remove old traces based on age
        let retention_threshold = current_timestamp() - self.config.trace_retention_seconds;
        while let Some(front) = traces.front() {
            if front.timestamp < retention_threshold {
                traces.pop_front();
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Get recent message traces with optional filtering
    pub fn get_traces(&self, limit: Option<usize>, filter_name: Option<&str>) -> Result<Vec<MessageTrace>> {
        let traces = self.message_traces
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire message traces lock"))?;
        
        let mut result: Vec<MessageTrace> = if let Some(filter_name) = filter_name {
            traces.iter()
                .filter(|trace| self.trace_matches_filter(trace, filter_name))
                .cloned()
                .collect()
        } else {
            traces.iter().cloned().collect()
        };
        
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        
        Ok(result)
    }
    
    /// Update actor snapshot
    pub fn update_actor_snapshot(&self, snapshot: ActorSnapshot) -> Result<()> {
        let mut snapshots = self.actor_snapshots
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire actor snapshots lock"))?;
        
        snapshots.insert(snapshot.actor_id, snapshot);
        
        // Enforce snapshot limits
        if snapshots.len() > self.config.max_snapshots {
            // Remove oldest snapshots
            let mut oldest_actors: Vec<_> = snapshots.iter()
                .map(|(&id, snapshot)| (id, snapshot.timestamp))
                .collect();
            oldest_actors.sort_by_key(|(_, timestamp)| *timestamp);
            
            let to_remove = snapshots.len() - self.config.max_snapshots;
            for i in 0..to_remove {
                if let Some((actor_id, _)) = oldest_actors.get(i) {
                    snapshots.remove(actor_id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get current actor snapshots
    pub fn get_actor_snapshots(&self) -> Result<HashMap<ActorId, ActorSnapshot>> {
        Ok(self.actor_snapshots
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire actor snapshots lock"))?
            .clone())
    }
    
    /// Get specific actor snapshot
    pub fn get_actor_snapshot(&self, actor_id: ActorId) -> Result<Option<ActorSnapshot>> {
        Ok(self.actor_snapshots
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire actor snapshots lock"))?
            .get(&actor_id)
            .cloned())
    }
    
    /// Perform deadlock detection
    pub fn detect_deadlocks(&self) -> Result<Vec<DeadlockCycle>> {
        if !self.config.enable_deadlock_detection {
            return Ok(Vec::new());
        }
        
        let mut detector = self.deadlock_detector
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire deadlock detector lock"))?;
        
        detector.detect_cycles()
    }
    
    /// Update system metrics
    pub fn update_metrics(&self) -> Result<()> {
        if !self.config.enable_metrics {
            return Ok(());
        }
        
        let _system = self.actor_system
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire actor system lock"))?;
        
        let mut metrics = self.metrics
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?;
        
        let snapshots = self.actor_snapshots
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire actor snapshots lock"))?;
        
        // Update metrics based on current system state
        metrics.active_actors = snapshots.len();
        metrics.total_messages_processed = snapshots.values()
            .map(|s| s.message_stats.total_processed)
            .sum();
        
        metrics.total_queued_messages = snapshots.values()
            .map(|s| s.mailbox_size)
            .sum();
        
        metrics.avg_mailbox_size = if snapshots.is_empty() {
            0.0
        } else {
            metrics.total_queued_messages as f64 / snapshots.len() as f64
        };
        
        metrics.total_memory_usage = snapshots.values()
            .filter_map(|s| s.memory_usage)
            .sum();
        
        metrics.last_updated = current_timestamp();
        
        Ok(())
    }
    
    /// Get current system metrics
    pub fn get_metrics(&self) -> Result<SystemMetrics> {
        Ok(self.metrics
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?
            .clone())
    }
    
    /// Get observatory uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Check if a message matches the configured filters
    fn message_matches_filters(&self, trace: &MessageTrace) -> bool {
        if self.filters.is_empty() {
            return true; // No filters means include all messages
        }
        
        self.filters.iter().any(|filter| self.message_matches_filter(trace, filter))
    }
    
    /// Check if a message matches a specific filter
    fn message_matches_filter(&self, trace: &MessageTrace, filter: &MessageFilter) -> bool {
        // Filter by actor ID
        if let Some(filter_actor_id) = filter.actor_id {
            if trace.destination != filter_actor_id {
                return false;
            }
        }
        
        // Filter by processing time
        if let Some(min_time) = filter.min_processing_time_us {
            if let Some(duration) = trace.processing_duration_us {
                if duration < min_time {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Filter by failed messages only
        if filter.failed_only && trace.status != MessageStatus::Failed {
            return false;
        }
        
        // Filter by stack depth
        if let Some(max_depth) = filter.max_stack_depth {
            if trace.stack_depth > max_depth {
                return false;
            }
        }
        
        true
    }
    
    /// Check if a trace matches a filter by name
    fn trace_matches_filter(&self, trace: &MessageTrace, filter_name: &str) -> bool {
        self.filters.iter()
            .find(|f| f.name == filter_name)
            .map_or(false, |filter| self.message_matches_filter(trace, filter))
    }
}

impl DeadlockDetector {
    /// Create a new deadlock detector
    pub fn new() -> Self {
        Self {
            dependency_graph: HashMap::new(),
            blocked_actors: HashMap::new(),
            last_check: Instant::now(),
            detected_deadlocks: Vec::new(),
        }
    }
    
    /// Add a blocked request to track
    pub fn add_blocked_request(&mut self, request: BlockedRequest) {
        self.blocked_actors
            .entry(request.requester)
            .or_default()
            .push(request.clone());
        
        // Update dependency graph
        self.dependency_graph
            .entry(request.requester)
            .or_default()
            .insert(request.target);
    }
    
    /// Remove a blocked request (when resolved)
    pub fn remove_blocked_request(&mut self, requester: ActorId, target: ActorId) {
        if let Some(requests) = self.blocked_actors.get_mut(&requester) {
            requests.retain(|r| r.target != target);
            if requests.is_empty() {
                self.blocked_actors.remove(&requester);
            }
        }
        
        // Update dependency graph
        if let Some(dependencies) = self.dependency_graph.get_mut(&requester) {
            dependencies.remove(&target);
            if dependencies.is_empty() {
                self.dependency_graph.remove(&requester);
            }
        }
    }
    
    /// Detect cycles in the dependency graph (potential deadlocks)
    pub fn detect_cycles(&mut self) -> Result<Vec<DeadlockCycle>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        for &actor in self.dependency_graph.keys() {
            if !visited.contains(&actor) {
                self.dfs_detect_cycle(actor, &mut visited, &mut path, &mut cycles)?;
            }
        }
        
        self.detected_deadlocks.extend(cycles.clone());
        self.last_check = Instant::now();
        
        Ok(cycles)
    }
    
    /// Depth-first search to detect cycles
    fn dfs_detect_cycle(
        &self,
        actor: ActorId,
        visited: &mut HashSet<ActorId>,
        path: &mut Vec<ActorId>,
        cycles: &mut Vec<DeadlockCycle>,
    ) -> Result<()> {
        visited.insert(actor);
        path.push(actor);
        
        if let Some(dependencies) = self.dependency_graph.get(&actor) {
            for &dependent_actor in dependencies {
                if let Some(cycle_start_index) = path.iter().position(|&a| a == dependent_actor) {
                    // Found a cycle
                    let cycle_actors = path[cycle_start_index..].to_vec();
                    let duration_estimate = self.estimate_cycle_duration(&cycle_actors);
                    
                    cycles.push(DeadlockCycle {
                        actors: cycle_actors.clone(),
                        detected_at: current_timestamp(),
                        duration_estimate_ms: duration_estimate,
                        resolution_suggestion: self.suggest_resolution(&cycle_actors),
                    });
                } else if !visited.contains(&dependent_actor) {
                    self.dfs_detect_cycle(dependent_actor, visited, path, cycles)?;
                }
            }
        }
        
        path.pop();
        Ok(())
    }
    
    /// Estimate how long a deadlock cycle has been active
    fn estimate_cycle_duration(&self, actors: &[ActorId]) -> u64 {
        let now = Instant::now();
        actors.iter()
            .filter_map(|&actor| self.blocked_actors.get(&actor))
            .flatten()
            .map(|request| now.duration_since(request.timestamp).as_millis() as u64)
            .max()
            .unwrap_or(0)
    }
    
    /// Suggest a resolution strategy for a deadlock cycle
    fn suggest_resolution(&self, actors: &[ActorId]) -> String {
        match actors.len() {
            1 => "Self-deadlock: Check for recursive message sending".to_string(),
            2 => "Binary deadlock: Consider using ask with timeout or redesign interaction pattern".to_string(),
            3..=5 => "Multi-actor deadlock: Implement hierarchical message ordering or use supervision".to_string(),
            _ => "Complex deadlock: Consider breaking into smaller subsystems or using event sourcing".to_string(),
        }
    }
}

/// Get current timestamp in seconds since Unix epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Create a simple message filter for testing
impl MessageFilter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            actor_id: None,
            actor_name_pattern: None,
            message_type_pattern: None,
            min_processing_time_us: None,
            failed_only: false,
            max_stack_depth: None,
        }
    }
    
    /// Create a filter for a specific actor
    pub fn for_actor(name: &str, actor_id: ActorId) -> Self {
        Self {
            name: name.to_string(),
            actor_id: Some(actor_id),
            actor_name_pattern: None,
            message_type_pattern: None,
            min_processing_time_us: None,
            failed_only: false,
            max_stack_depth: None,
        }
    }
    
    /// Create a filter for failed messages only
    pub fn failed_messages_only(name: &str) -> Self {
        Self {
            name: name.to_string(),
            actor_id: None,
            actor_name_pattern: None,
            message_type_pattern: None,
            min_processing_time_us: None,
            failed_only: true,
            max_stack_depth: None,
        }
    }
    
    /// Create a filter for slow messages
    pub fn slow_messages(name: &str, min_time_us: u64) -> Self {
        Self {
            name: name.to_string(),
            actor_id: None,
            actor_name_pattern: None,
            message_type_pattern: None,
            min_processing_time_us: Some(min_time_us),
            failed_only: false,
            max_stack_depth: None,
        }
    }
}