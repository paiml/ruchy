//! Dataflow debugger for `DataFrame` pipeline debugging (RUCHY-0818)
//!
//! Provides comprehensive debugging capabilities for `DataFrame` operations,
//! including breakpoints, materialization on demand, and stage-by-stage analysis.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
/// Dataflow debugger for `DataFrame` pipeline analysis
pub struct DataflowDebugger {
    /// Pipeline execution stages
    #[allow(dead_code)] // Future feature for pipeline management
    pipeline_stages: Arc<Mutex<Vec<PipelineStage>>>,
    /// Active breakpoints by stage name
    breakpoints: Arc<Mutex<HashMap<String, Breakpoint>>>,
    /// Materialized data at breakpoints
    materialized_data: Arc<Mutex<HashMap<String, MaterializedFrame>>>,
    /// Debugger configuration
    config: DataflowConfig,
    /// Execution history for analysis
    execution_history: Arc<Mutex<VecDeque<ExecutionEvent>>>,
    /// Performance metrics per stage
    stage_metrics: Arc<Mutex<HashMap<String, StageMetrics>>>,
    /// Current debugging session state
    session_state: Arc<Mutex<SessionState>>,
}
/// Configuration for the dataflow debugger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataflowConfig {
    /// Maximum number of rows to materialize at each stage
    pub max_rows_per_stage: usize,
    /// Enable automatic materialization at each stage
    pub auto_materialize: bool,
    /// Maximum execution history events to keep
    pub max_history_events: usize,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Timeout for stage execution (in milliseconds)
    pub stage_timeout_ms: u64,
    /// Enable detailed memory tracking
    pub track_memory: bool,
    /// Enable diff computation between stages
    pub compute_diffs: bool,
    /// Sample rate for large datasets (0.0-1.0)
    pub sample_rate: f64,
}
/// Individual stage in the `DataFrame` pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    /// Unique identifier for the stage
    pub stage_id: String,
    /// Human-readable name
    pub stage_name: String,
    /// Stage type (filter, map, `group_by`, etc.)
    pub stage_type: StageType,
    /// Stage execution status
    pub status: StageStatus,
    /// Input `DataFrame` schema
    pub input_schema: Option<DataSchema>,
    /// Output `DataFrame` schema  
    pub output_schema: Option<DataSchema>,
    /// Stage execution time
    pub execution_time: Option<Duration>,
    /// Memory usage for this stage
    pub memory_usage: Option<usize>,
    /// Number of rows processed
    pub rows_processed: Option<usize>,
    /// Stage-specific metadata
    pub metadata: HashMap<String, String>,
}
/// Types of `DataFrame` operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StageType {
    /// Data loading stage
    Load,
    /// Filtering operations
    Filter,
    /// Column selection/projection
    Select,
    /// Column transformations
    Map,
    /// Grouping operations
    GroupBy,
    /// Aggregation operations
    Aggregate,
    /// Join operations
    Join,
    /// Sorting operations  
    Sort,
    /// Window functions
    Window,
    /// Union operations
    Union,
    /// Custom user-defined operations
    Custom(String),
}
/// Execution status of a pipeline stage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageStatus {
    /// Stage not yet executed
    Pending,
    /// Stage currently executing
    Running,
    /// Stage completed successfully
    Completed,
    /// Stage failed with error
    Failed(String),
    /// Stage execution was cancelled
    Cancelled,
    /// Stage paused at breakpoint
    Paused,
}
/// Breakpoint configuration for pipeline debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Stage ID where breakpoint is set
    pub stage_id: String,
    /// Breakpoint condition (optional)
    pub condition: Option<BreakpointCondition>,
    /// Whether breakpoint is currently active
    pub active: bool,
    /// Hit count for this breakpoint
    pub hit_count: usize,
    /// Actions to perform when breakpoint is hit
    pub actions: Vec<BreakpointAction>,
}
/// Conditions for triggering breakpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakpointCondition {
    /// Always break at this stage
    Always,
    /// Break if row count meets criteria
    RowCount { operator: ComparisonOp, value: usize },
    /// Break if execution time exceeds threshold
    ExecutionTime { threshold_ms: u64 },
    /// Break if memory usage exceeds threshold
    MemoryUsage { threshold_mb: usize },
    /// Break on specific data values
    DataValue { column: String, value: DataValue },
    /// Break on error conditions
    OnError,
}
/// Comparison operators for breakpoint conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}
/// Actions to perform when breakpoint is triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakpointAction {
    /// Pause execution and wait for user input
    Pause,
    /// Print debug information
    Print(String),
    /// Materialize current `DataFrame`
    Materialize,
    /// Compute diff with previous stage
    ComputeDiff,
    /// Export data to file
    Export { format: ExportFormat, path: String },
}
/// Materialized `DataFrame` data for inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedFrame {
    /// Stage ID where data was materialized
    pub stage_id: String,
    /// `DataFrame` schema
    pub schema: DataSchema,
    /// Sample of data rows (limited by config)
    pub sample_data: Vec<DataRow>,
    /// Total number of rows in full dataset
    pub total_rows: usize,
    /// Materialization timestamp
    pub timestamp: std::time::SystemTime,
    /// Memory footprint of materialized data
    pub memory_size: usize,
}
/// `DataFrame` schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSchema {
    /// Column definitions
    pub columns: Vec<ColumnDef>,
    /// Schema hash for change detection
    pub schema_hash: u64,
}
/// Column definition in `DataFrame` schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: DataType,
    /// Whether column allows null values
    pub nullable: bool,
}
/// Supported data types in `DataFrames`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Integer,
    Float,
    String,
    DateTime,
    Array(Box<DataType>),
    Struct(Vec<(String, DataType)>),
}
/// Single row of data in materialized `DataFrame`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRow {
    /// Values for each column
    pub values: Vec<DataValue>,
}
/// Individual data value in a `DataFrame` cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Null,
    Array(Vec<DataValue>),
}
/// Performance metrics for a pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageMetrics {
    /// Stage execution time
    pub execution_time: Duration,
    /// Memory peak usage during stage
    pub peak_memory: usize,
    /// Number of rows input to stage
    pub input_rows: usize,
    /// Number of rows output from stage
    pub output_rows: usize,
    /// CPU time spent in stage
    pub cpu_time: Duration,
    /// I/O operations performed
    pub io_operations: usize,
    /// Cache hit ratio (if applicable)
    pub cache_hit_ratio: Option<f64>,
}
/// Execution event in the debugging session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    /// Event timestamp
    pub timestamp: std::time::SystemTime,
    /// Event type
    pub event_type: EventType,
    /// Stage ID associated with event
    pub stage_id: String,
    /// Additional event data
    pub data: HashMap<String, String>,
}
/// Types of execution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    StageStarted,
    StageCompleted,
    StageFailed,
    BreakpointHit,
    DataMaterialized,
    DiffComputed,
    PerformanceAlert,
}
/// Current state of the debugging session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Whether debugger is actively running
    pub active: bool,
    /// Current stage being executed or paused at
    pub current_stage: Option<String>,
    /// Session start time
    pub session_start: std::time::SystemTime,
    /// Total execution time so far
    pub total_execution_time: Duration,
    /// Number of breakpoints hit
    pub breakpoints_hit: usize,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}
/// Export formats for dataflow debugging data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Comma-separated values
    Csv,
    /// JSON format
    Json,
    /// Parquet format
    Parquet,
    /// Debug text format
    Debug,
}
impl Default for DataflowConfig {
    fn default() -> Self {
        Self {
            max_rows_per_stage: 1000,
            auto_materialize: false,
            max_history_events: 10000,
            enable_profiling: true,
            stage_timeout_ms: 30000, // 30 seconds
            track_memory: true,
            compute_diffs: false,
            sample_rate: 1.0, // No sampling by default
        }
    }
}
impl DataflowDebugger {
    /// Create a new dataflow debugger
    pub fn new(config: DataflowConfig) -> Self {
        Self {
            pipeline_stages: Arc::new(Mutex::new(Vec::new())),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            materialized_data: Arc::new(Mutex::new(HashMap::new())),
            config,
            execution_history: Arc::new(Mutex::new(VecDeque::new())),
            stage_metrics: Arc::new(Mutex::new(HashMap::new())),
            session_state: Arc::new(Mutex::new(SessionState::default())),
        }
    }
    /// Start a new debugging session
    pub fn start_session(&self) -> Result<()> {
        let mut state = self.session_state
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire session state lock"))?;
        state.active = true;
        state.session_start = std::time::SystemTime::now();
        state.total_execution_time = Duration::from_secs(0);
        state.breakpoints_hit = 0;
        state.current_stage = None;
        self.record_event(EventType::StageStarted, "session".to_string(), HashMap::new())?;
        Ok(())
    }
    /// Add a breakpoint to the debugger
    pub fn add_breakpoint(&self, breakpoint: Breakpoint) -> Result<()> {
        let mut breakpoints = self.breakpoints
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire breakpoints lock"))?;
        breakpoints.insert(breakpoint.stage_id.clone(), breakpoint);
        Ok(())
    }
    /// Remove a breakpoint by stage ID
    pub fn remove_breakpoint(&self, stage_id: &str) -> Result<bool> {
        let mut breakpoints = self.breakpoints
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire breakpoints lock"))?;
        Ok(breakpoints.remove(stage_id).is_some())
    }
    /// Execute a pipeline stage with debugging support
    pub fn execute_stage(&self, pipeline_stage: &mut PipelineStage) -> Result<StageExecutionResult> {
        let start_time = Instant::now();
        pipeline_stage.status = StageStatus::Running;
        // Update session state
        {
            let mut state = self.session_state
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire session state lock"))?;
            state.current_stage = Some(pipeline_stage.stage_id.clone());
        }
        // Check for breakpoints
        if let Some(breakpoint) = self.check_breakpoint(&pipeline_stage.stage_id)? {
            if self.should_break(pipeline_stage, &breakpoint)? {
                pipeline_stage.status = StageStatus::Paused;
                self.handle_breakpoint_hit(&pipeline_stage.stage_id, &breakpoint)?;
                return Ok(StageExecutionResult::Paused);
            }
        }
        // Simulate stage execution (in real implementation, this would execute actual DataFrame operations)
        std::thread::sleep(Duration::from_millis(10)); // Simulate work
        // Record execution metrics
        let execution_time = start_time.elapsed();
        pipeline_stage.execution_time = Some(execution_time);
        pipeline_stage.status = StageStatus::Completed;
        // Store performance metrics
        let metrics = StageMetrics {
            execution_time,
            peak_memory: 1024 * 1024, // 1MB simulated
            input_rows: pipeline_stage.rows_processed.unwrap_or(0),
            output_rows: pipeline_stage.rows_processed.unwrap_or(0),
            cpu_time: execution_time,
            io_operations: 1,
            cache_hit_ratio: Some(0.85),
        };
        let mut stage_metrics = self.stage_metrics
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire stage metrics lock"))?;
        stage_metrics.insert(pipeline_stage.stage_id.clone(), metrics);
        // Auto-materialize if configured
        if self.config.auto_materialize {
            self.materialize_stage(&pipeline_stage.stage_id)?;
        }
        self.record_event(
            EventType::StageCompleted,
            pipeline_stage.stage_id.clone(),
            HashMap::from([("duration_ms".to_string(), execution_time.as_millis().to_string())])
        )?;
        Ok(StageExecutionResult::Completed)
    }
    /// Materialize `DataFrame` data at a specific stage
    pub fn materialize_stage(&self, stage_id: &str) -> Result<MaterializedFrame> {
        // In a real implementation, this would materialize actual DataFrame data
        let materialized = MaterializedFrame {
            stage_id: stage_id.to_string(),
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
                        nullable: true,
                    },
                ],
                schema_hash: 12345,
            },
            sample_data: vec![
                DataRow {
                    values: vec![DataValue::Integer(1), DataValue::String("Alice".to_string())],
                },
                DataRow {
                    values: vec![DataValue::Integer(2), DataValue::String("Bob".to_string())],
                },
            ],
            total_rows: 1000,
            timestamp: std::time::SystemTime::now(),
            memory_size: 1024 * 50, // 50KB
        };
        let mut materialized_data = self.materialized_data
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire materialized data lock"))?;
        materialized_data.insert(stage_id.to_string(), materialized.clone());
        self.record_event(
            EventType::DataMaterialized,
            stage_id.to_string(),
            HashMap::from([("rows".to_string(), materialized.total_rows.to_string())])
        )?;
        Ok(materialized)
    }
    /// Compute diff between two pipeline stages
    pub fn compute_stage_diff(&self, stage1_id: &str, stage2_id: &str) -> Result<StageDiff> {
        let materialized_data = self.materialized_data
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire materialized data lock"))?;
        let stage1_data = materialized_data.get(stage1_id)
            .ok_or_else(|| anyhow::anyhow!("Stage {} not materialized", stage1_id))?;
        let stage2_data = materialized_data.get(stage2_id)
            .ok_or_else(|| anyhow::anyhow!("Stage {} not materialized", stage2_id))?;
        // Compute basic diff metrics
        let row_count_diff = stage2_data.total_rows as i64 - stage1_data.total_rows as i64;
        let schema_changed = stage1_data.schema.schema_hash != stage2_data.schema.schema_hash;
        let diff = StageDiff {
            stage1_id: stage1_id.to_string(),
            stage2_id: stage2_id.to_string(),
            row_count_diff,
            schema_changed,
            column_changes: self.compute_column_changes(&stage1_data.schema, &stage2_data.schema),
            data_changes: self.compute_data_changes(&stage1_data.sample_data, &stage2_data.sample_data),
        };
        self.record_event(
            EventType::DiffComputed,
            format!("{stage1_id}:{stage2_id}"),
            HashMap::from([("row_diff".to_string(), row_count_diff.to_string())])
        )?;
        Ok(diff)
    }
    /// Get current debugging session status
    pub fn get_session_status(&self) -> Result<SessionState> {
        let state = self.session_state
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire session state lock"))?;
        Ok(state.clone())
    }
    /// Get execution history
    pub fn get_execution_history(&self) -> Result<Vec<ExecutionEvent>> {
        let history = self.execution_history
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire execution history lock"))?;
        Ok(history.iter().cloned().collect())
    }
    /// Get performance metrics for all stages
    pub fn get_stage_metrics(&self) -> Result<HashMap<String, StageMetrics>> {
        let metrics = self.stage_metrics
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire stage metrics lock"))?;
        Ok(metrics.clone())
    }
    /// Export debugging data to various formats
    pub fn export_debug_data(&self, format: ExportFormat, output_path: &str) -> Result<()> {
        let history = self.get_execution_history()?;
        let metrics = self.get_stage_metrics()?;
        let session_status = self.get_session_status()?;
        let debug_data = DebugExport {
            session_status,
            execution_history: history,
            stage_metrics: metrics,
            materialized_data: {
                let data = self.materialized_data
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Failed to acquire materialized data lock"))?;
                data.clone()
            },
        };
        match format {
            ExportFormat::Json => {
                let json_data = serde_json::to_string_pretty(&debug_data)?;
                std::fs::write(output_path, json_data)?;
            }
            ExportFormat::Debug => {
                let debug_str = format!("{debug_data:#?}");
                std::fs::write(output_path, debug_str)?;
            }
            _ => {
                return Err(anyhow::anyhow!("Export format {:?} not yet implemented", format));
            }
        }
        Ok(())
    }
    // Helper methods
    fn check_breakpoint(&self, stage_id: &str) -> Result<Option<Breakpoint>> {
        let breakpoints = self.breakpoints
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire breakpoints lock"))?;
        Ok(breakpoints.get(stage_id).cloned())
    }
    fn should_break(&self, _stage: &PipelineStage, breakpoint: &Breakpoint) -> Result<bool> {
        if !breakpoint.active {
            return Ok(false);
        }
        match &breakpoint.condition {
            None | Some(BreakpointCondition::Always) => Ok(true),
            Some(BreakpointCondition::RowCount { operator: _, value: _ }) => {
                // In real implementation, would check actual row count
                Ok(true)
            }
            Some(BreakpointCondition::ExecutionTime { threshold_ms: _ }) => {
                // In real implementation, would check execution time
                Ok(false)
            }
            Some(BreakpointCondition::MemoryUsage { threshold_mb: _ }) => {
                // In real implementation, would check memory usage
                Ok(false)
            }
            Some(BreakpointCondition::DataValue { column: _, value: _ }) => {
                // In real implementation, would inspect actual data
                Ok(false)
            }
            Some(BreakpointCondition::OnError) => Ok(false),
        }
    }
    fn handle_breakpoint_hit(&self, stage_id: &str, breakpoint: &Breakpoint) -> Result<()> {
        // Update breakpoint hit count
        let mut breakpoints = self.breakpoints
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire breakpoints lock"))?;
        if let Some(bp) = breakpoints.get_mut(stage_id) {
            bp.hit_count += 1;
        }
        // Update session state
        {
            let mut state = self.session_state
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire session state lock"))?;
            state.breakpoints_hit += 1;
        }
        // Execute breakpoint actions
        for action in &breakpoint.actions {
            match action {
                BreakpointAction::Pause => {
                    // In real implementation, would pause execution and wait for user input
                }
                BreakpointAction::Print(message) => {
                    println!("Breakpoint: {message}");
                }
                BreakpointAction::Materialize => {
                    self.materialize_stage(stage_id)?;
                }
                BreakpointAction::ComputeDiff => {
                    // Would compute diff with previous stage if available
                }
                BreakpointAction::Export { format: _, path: _ } => {
                    // Would export current data
                }
            }
        }
        self.record_event(
            EventType::BreakpointHit,
            stage_id.to_string(),
            HashMap::from([("hit_count".to_string(), breakpoint.hit_count.to_string())])
        )?;
        Ok(())
    }
    fn record_event(&self, event_type: EventType, stage_id: String, data: HashMap<String, String>) -> Result<()> {
        let event = ExecutionEvent {
            timestamp: std::time::SystemTime::now(),
            event_type,
            stage_id,
            data,
        };
        let mut history = self.execution_history
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire execution history lock"))?;
        history.push_back(event);
        // Maintain history size limit
        while history.len() > self.config.max_history_events {
            history.pop_front();
        }
        Ok(())
    }
    fn compute_column_changes(&self, schema1: &DataSchema, schema2: &DataSchema) -> Vec<ColumnChange> {
        let mut changes = Vec::new();
        // Find added/removed columns (simplified implementation)
        let cols1: Vec<&str> = schema1.columns.iter().map(|c| c.name.as_str()).collect();
        let cols2: Vec<&str> = schema2.columns.iter().map(|c| c.name.as_str()).collect();
        for col in &cols2 {
            if !cols1.contains(col) {
                changes.push(ColumnChange::Added((*col).to_string()));
            }
        }
        for col in &cols1 {
            if !cols2.contains(col) {
                changes.push(ColumnChange::Removed((*col).to_string()));
            }
        }
        changes
    }
    fn compute_data_changes(&self, _data1: &[DataRow], _data2: &[DataRow]) -> Vec<DataChange> {
        // Simplified implementation - in reality would compute detailed row-level diffs
        vec![DataChange::RowCountChanged]
    }
}
/// Result of executing a pipeline stage
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageExecutionResult {
    /// Stage completed successfully
    Completed,
    /// Stage paused at breakpoint
    Paused,
    /// Stage failed with error
    Failed(String),
}
/// Diff between two pipeline stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDiff {
    /// First stage ID
    pub stage1_id: String,
    /// Second stage ID  
    pub stage2_id: String,
    /// Difference in row count (stage2 - stage1)
    pub row_count_diff: i64,
    /// Whether schema changed between stages
    pub schema_changed: bool,
    /// Specific column changes
    pub column_changes: Vec<ColumnChange>,
    /// Data-level changes
    pub data_changes: Vec<DataChange>,
}
/// Types of column changes between stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnChange {
    /// Column was added
    Added(String),
    /// Column was removed
    Removed(String),
    /// Column type changed
    TypeChanged(String, DataType, DataType),
    /// Column renamed
    Renamed(String, String),
}
/// Types of data changes between stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataChange {
    /// Row count changed
    RowCountChanged,
    /// Specific rows added
    RowsAdded(Vec<usize>),
    /// Specific rows removed
    RowsRemoved(Vec<usize>),
    /// Cell values modified
    ValuesModified(Vec<(usize, usize)>), // (row, col) indices
}
/// Complete debugging data export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugExport {
    /// Current session status
    pub session_status: SessionState,
    /// Execution history
    pub execution_history: Vec<ExecutionEvent>,
    /// Performance metrics
    pub stage_metrics: HashMap<String, StageMetrics>,
    /// Materialized data
    pub materialized_data: HashMap<String, MaterializedFrame>,
}
impl Default for SessionState {
    fn default() -> Self {
        Self {
            active: false,
            current_stage: None,
            session_start: std::time::SystemTime::now(),
            total_execution_time: Duration::from_secs(0),
            breakpoints_hit: 0,
            metadata: HashMap::new(),
        }
    }
}
impl fmt::Display for StageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load => write!(f, "Load"),
            Self::Filter => write!(f, "Filter"),
            Self::Select => write!(f, "Select"),
            Self::Map => write!(f, "Map"),
            Self::GroupBy => write!(f, "GroupBy"),
            Self::Aggregate => write!(f, "Aggregate"),
            Self::Join => write!(f, "Join"),
            Self::Sort => write!(f, "Sort"),
            Self::Window => write!(f, "Window"),
            Self::Union => write!(f, "Union"),
            Self::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}
impl fmt::Display for StageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed(err) => write!(f, "Failed: {err}"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Paused => write!(f, "Paused"),
        }
    }
}
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "Boolean"),
            Self::Integer => write!(f, "Integer"),
            Self::Float => write!(f, "Float"),
            Self::String => write!(f, "String"),
            Self::DateTime => write!(f, "DateTime"),
            Self::Array(inner) => write!(f, "Array<{inner}>"),
            Self::Struct(fields) => {
                write!(f, "Struct{{")?;
                for (i, (name, dtype)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{name}: {dtype}")?;
                }
                write!(f, "}}")
            }
        }
    }
}
impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Null => write!(f, "null"),
            Self::Array(values) => {
                write!(f, "[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{value}")?;
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_config() -> DataflowConfig {
        DataflowConfig {
            max_rows_per_stage: 1000,
            auto_materialize: true,
            max_history_events: 100,
            enable_profiling: true,
            stage_timeout_ms: 30000,
            track_memory: true,
            compute_diffs: true,
            sample_rate: 1.0,
        }
    }

    fn create_test_schema() -> DataSchema {
        DataSchema {
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                },
                ColumnDef {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    nullable: true,
                },
            ],
            schema_hash: 12345,
        }
    }

    #[test]
    fn test_dataflow_debugger_creation() {
        let config = create_test_config();
        let debugger = DataflowDebugger::new(config);

        assert_eq!(debugger.config.max_rows_per_stage, 1000);
        assert!(debugger.config.auto_materialize);
        assert!(debugger.config.enable_profiling);
    }

    #[test]
    fn test_dataflow_config_default() {
        let config = DataflowConfig::default();
        assert_eq!(config.max_rows_per_stage, 1000);
        assert!(!config.auto_materialize);
        assert_eq!(config.max_history_events, 50);
        assert!(!config.enable_profiling);
        assert_eq!(config.stage_timeout_ms, 30000);
        assert!(!config.track_memory);
        assert!(!config.compute_diffs);
        assert_eq!(config.sample_rate, 1.0);
    }

    #[test]
    fn test_session_state_default() {
        let state = SessionState::default();
        assert!(!state.active);
        assert!(state.current_stage.is_none());
        assert_eq!(state.total_execution_time, Duration::from_secs(0));
        assert_eq!(state.breakpoints_hit, 0);
        assert!(state.metadata.is_empty());
    }

    #[test]
    fn test_debugger_start_session() {
        let config = create_test_config();
        let debugger = DataflowDebugger::new(config);

        let result = debugger.start_session();
        assert!(result.is_ok());
    }

    #[test]
    fn test_debugger_stop_session() {
        let config = create_test_config();
        let debugger = DataflowDebugger::new(config);

        let _ = debugger.start_session();
        let result = debugger.stop_session();
        assert!(result.is_ok());
    }

    #[test]
    fn test_debugger_add_stage() {
        let config = create_test_config();
        let debugger = DataflowDebugger::new(config);

        let stage = PipelineStage {
            stage_id: "stage_1".to_string(),
            stage_name: "Filter Stage".to_string(),
            stage_type: StageType::Filter,
            status: StageStatus::Pending,
            input_schema: Some(create_test_schema()),
            output_schema: None,
            execution_time: None,
            memory_usage: None,
            rows_processed: None,
            metadata: HashMap::new(),
        };

        let result = debugger.add_stage(stage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_debugger_set_breakpoint() {
        let config = create_test_config();
        let debugger = DataflowDebugger::new(config);

        let breakpoint = Breakpoint {
            stage_id: "stage_1".to_string(),
            condition: Some(BreakpointCondition::Always),
            active: true,
            hit_count: 0,
            actions: vec![BreakpointAction::Pause],
        };

        let result = debugger.set_breakpoint("stage_1".to_string(), breakpoint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_debugger_remove_breakpoint() {
        let config = create_test_config();
        let debugger = DataflowDebugger::new(config);

        let breakpoint = Breakpoint {
            stage_id: "stage_1".to_string(),
            condition: Some(BreakpointCondition::Always),
            active: true,
            hit_count: 0,
            actions: vec![BreakpointAction::Pause],
        };

        let _ = debugger.set_breakpoint("stage_1".to_string(), breakpoint);
        let result = debugger.remove_breakpoint("stage_1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_stage_type_display() {
        assert_eq!(format!("{}", StageType::Load), "Load");
        assert_eq!(format!("{}", StageType::Filter), "Filter");
        assert_eq!(format!("{}", StageType::Select), "Select");
        assert_eq!(format!("{}", StageType::Map), "Map");
        assert_eq!(format!("{}", StageType::GroupBy), "GroupBy");
        assert_eq!(format!("{}", StageType::Aggregate), "Aggregate");
        assert_eq!(format!("{}", StageType::Join), "Join");
        assert_eq!(format!("{}", StageType::Sort), "Sort");
        assert_eq!(format!("{}", StageType::Window), "Window");
        assert_eq!(format!("{}", StageType::Union), "Union");
        assert_eq!(format!("{}", StageType::Custom("test".to_string())), "Custom(test)");
    }

    #[test]
    fn test_stage_status_display() {
        assert_eq!(format!("{}", StageStatus::Pending), "Pending");
        assert_eq!(format!("{}", StageStatus::Running), "Running");
        assert_eq!(format!("{}", StageStatus::Completed), "Completed");
        assert_eq!(format!("{}", StageStatus::Failed("error".to_string())), "Failed: error");
        assert_eq!(format!("{}", StageStatus::Cancelled), "Cancelled");
        assert_eq!(format!("{}", StageStatus::Paused), "Paused");
    }

    #[test]
    fn test_data_type_display() {
        assert_eq!(format!("{}", DataType::Boolean), "Boolean");
        assert_eq!(format!("{}", DataType::Integer), "Integer");
        assert_eq!(format!("{}", DataType::Float), "Float");
        assert_eq!(format!("{}", DataType::String), "String");
        assert_eq!(format!("{}", DataType::DateTime), "DateTime");
        assert_eq!(format!("{}", DataType::Array(Box::new(DataType::Integer))), "Array<Integer>");

        let struct_fields = vec![
            ("id".to_string(), DataType::Integer),
            ("name".to_string(), DataType::String),
        ];
        assert_eq!(format!("{}", DataType::Struct(struct_fields)), "Struct{id: Integer, name: String}");
    }

    #[test]
    fn test_data_value_display() {
        assert_eq!(format!("{}", DataValue::Boolean(true)), "true");
        assert_eq!(format!("{}", DataValue::Integer(42)), "42");
        assert_eq!(format!("{}", DataValue::Float(3.14)), "3.14");
        assert_eq!(format!("{}", DataValue::String("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", DataValue::Null), "null");

        let array_values = vec![DataValue::Integer(1), DataValue::Integer(2), DataValue::Integer(3)];
        assert_eq!(format!("{}", DataValue::Array(array_values)), "[1, 2, 3]");
    }

    #[test]
    fn test_breakpoint_condition_row_count() {
        let condition = BreakpointCondition::RowCount {
            operator: ComparisonOp::GreaterThan,
            value: 1000,
        };

        let breakpoint = Breakpoint {
            stage_id: "stage_1".to_string(),
            condition: Some(condition),
            active: true,
            hit_count: 0,
            actions: vec![BreakpointAction::Pause],
        };

        assert!(breakpoint.active);
        assert_eq!(breakpoint.hit_count, 0);
    }

    #[test]
    fn test_breakpoint_condition_execution_time() {
        let condition = BreakpointCondition::ExecutionTime {
            threshold_ms: 5000,
        };

        let breakpoint = Breakpoint {
            stage_id: "stage_1".to_string(),
            condition: Some(condition),
            active: true,
            hit_count: 0,
            actions: vec![BreakpointAction::Print("Slow execution detected".to_string())],
        };

        assert_eq!(breakpoint.actions.len(), 1);
    }

    #[test]
    fn test_breakpoint_condition_memory_usage() {
        let condition = BreakpointCondition::MemoryUsage {
            threshold_mb: 100,
        };

        let breakpoint = Breakpoint {
            stage_id: "stage_1".to_string(),
            condition: Some(condition),
            active: true,
            hit_count: 0,
            actions: vec![BreakpointAction::Materialize],
        };

        assert!(breakpoint.active);
    }

    #[test]
    fn test_breakpoint_condition_data_value() {
        let condition = BreakpointCondition::DataValue {
            column: "status".to_string(),
            value: DataValue::String("error".to_string()),
        };

        let breakpoint = Breakpoint {
            stage_id: "stage_1".to_string(),
            condition: Some(condition),
            active: true,
            hit_count: 0,
            actions: vec![BreakpointAction::ComputeDiff],
        };

        assert!(breakpoint.active);
    }

    #[test]
    fn test_breakpoint_action_export() {
        let action = BreakpointAction::Export {
            format: ExportFormat::Json,
            path: "/tmp/debug_data.json".to_string(),
        };

        let breakpoint = Breakpoint {
            stage_id: "stage_1".to_string(),
            condition: Some(BreakpointCondition::Always),
            active: true,
            hit_count: 0,
            actions: vec![action],
        };

        assert_eq!(breakpoint.actions.len(), 1);
    }

    #[test]
    fn test_materialized_frame_creation() {
        let schema = create_test_schema();
        let sample_data = vec![
            DataRow {
                values: vec![
                    DataValue::Integer(1),
                    DataValue::String("Alice".to_string()),
                ],
            },
            DataRow {
                values: vec![
                    DataValue::Integer(2),
                    DataValue::String("Bob".to_string()),
                ],
            },
        ];

        let materialized_frame = MaterializedFrame {
            stage_id: "stage_1".to_string(),
            schema,
            sample_data,
            total_rows: 1000,
            timestamp: std::time::SystemTime::now(),
            memory_size: 4096,
        };

        assert_eq!(materialized_frame.stage_id, "stage_1");
        assert_eq!(materialized_frame.total_rows, 1000);
        assert_eq!(materialized_frame.memory_size, 4096);
    }

    #[test]
    fn test_stage_metrics_creation() {
        let metrics = StageMetrics {
            stage_id: "stage_1".to_string(),
            execution_count: 1,
            total_execution_time: Duration::from_millis(500),
            average_execution_time: Duration::from_millis(500),
            peak_memory_usage: 1024,
            total_rows_processed: 10000,
            error_count: 0,
            last_execution: std::time::SystemTime::now(),
        };

        assert_eq!(metrics.stage_id, "stage_1");
        assert_eq!(metrics.execution_count, 1);
        assert_eq!(metrics.total_rows_processed, 10000);
        assert_eq!(metrics.error_count, 0);
    }

    #[test]
    fn test_execution_event_stage_start() {
        let event = ExecutionEvent::StageStart {
            stage_id: "stage_1".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        if let ExecutionEvent::StageStart { stage_id, .. } = event {
            assert_eq!(stage_id, "stage_1");
        } else {
            panic!("Expected StageStart event");
        }
    }

    #[test]
    fn test_execution_event_stage_complete() {
        let event = ExecutionEvent::StageComplete {
            stage_id: "stage_1".to_string(),
            timestamp: std::time::SystemTime::now(),
            execution_time: Duration::from_millis(100),
            rows_processed: 1000,
        };

        if let ExecutionEvent::StageComplete { stage_id, rows_processed, .. } = event {
            assert_eq!(stage_id, "stage_1");
            assert_eq!(rows_processed, 1000);
        } else {
            panic!("Expected StageComplete event");
        }
    }

    #[test]
    fn test_execution_event_breakpoint_hit() {
        let event = ExecutionEvent::BreakpointHit {
            stage_id: "stage_1".to_string(),
            breakpoint_id: "bp_1".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        if let ExecutionEvent::BreakpointHit { stage_id, breakpoint_id, .. } = event {
            assert_eq!(stage_id, "stage_1");
            assert_eq!(breakpoint_id, "bp_1");
        } else {
            panic!("Expected BreakpointHit event");
        }
    }

    #[test]
    fn test_execution_event_error() {
        let event = ExecutionEvent::Error {
            stage_id: "stage_1".to_string(),
            error_message: "Division by zero".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        if let ExecutionEvent::Error { stage_id, error_message, .. } = event {
            assert_eq!(stage_id, "stage_1");
            assert_eq!(error_message, "Division by zero");
        } else {
            panic!("Expected Error event");
        }
    }

    #[test]
    fn test_comparison_operators() {
        let operators = vec![
            ComparisonOp::Equal,
            ComparisonOp::NotEqual,
            ComparisonOp::GreaterThan,
            ComparisonOp::GreaterThanOrEqual,
            ComparisonOp::LessThan,
            ComparisonOp::LessThanOrEqual,
        ];

        assert_eq!(operators.len(), 6);
    }

    #[test]
    fn test_export_formats() {
        let formats = vec![
            ExportFormat::Json,
            ExportFormat::Csv,
            ExportFormat::Parquet,
            ExportFormat::Arrow,
        ];

        assert_eq!(formats.len(), 4);
    }

    #[test]
    fn test_complex_data_type_struct() {
        let fields = vec![
            ("user_id".to_string(), DataType::Integer),
            ("profile".to_string(), DataType::Struct(vec![
                ("name".to_string(), DataType::String),
                ("age".to_string(), DataType::Integer),
            ])),
        ];

        let struct_type = DataType::Struct(fields);
        let display_str = format!("{}", struct_type);
        assert!(display_str.contains("user_id: Integer"));
        assert!(display_str.contains("profile: Struct"));
    }

    #[test]
    fn test_complex_data_value_nested_array() {
        let nested_array = DataValue::Array(vec![
            DataValue::Array(vec![DataValue::Integer(1), DataValue::Integer(2)]),
            DataValue::Array(vec![DataValue::Integer(3), DataValue::Integer(4)]),
        ]);

        let display_str = format!("{}", nested_array);
        assert_eq!(display_str, "[[1, 2], [3, 4]]");
    }

    #[test]
    fn test_column_definition() {
        let column = ColumnDef {
            name: "test_column".to_string(),
            data_type: DataType::String,
            nullable: true,
        };

        assert_eq!(column.name, "test_column");
        assert!(column.nullable);
    }
}
