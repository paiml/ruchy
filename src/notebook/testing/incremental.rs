// SPRINT6-004: Incremental testing with smart caching
// PMAT Complexity: <10 per function
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
/// Incremental test executor with dependency tracking
pub struct IncrementalTester {
    cache: TestResultCache,
    dependency_tracker: DependencyTracker,
    config: IncrementalConfig,
}
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    pub cache_directory: PathBuf,
    pub max_cache_size: usize,
    pub cache_ttl: Duration,
    pub force_rerun_threshold: f64,
    pub dependency_analysis: bool,
}
#[derive(Debug)]
pub struct IncrementalResult {
    pub executed_cells: Vec<String>,
    pub cached_cells: Vec<String>,
    pub dependency_graph: DependencyGraph,
    pub cache_stats: CacheStatistics,
}
/// Cache for test results with TTL and LRU eviction
#[derive(Debug)]
pub struct TestResultCache {
    pub cache: HashMap<String, CachedTestResult>,
    pub access_order: Vec<String>,
    pub max_size: usize,
    pub cache_dir: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTestResult {
    pub cell_id: String,
    pub source_hash: String,
    pub dependencies_hash: String,
    pub result: TestResult,
    pub timestamp: SystemTime,
    pub access_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
    pub memory_used: usize,
}
/// Tracks dependencies between notebook cells
#[derive(Debug)]
pub struct DependencyTracker {
    pub dependencies: HashMap<String, HashSet<String>>,
    pub variable_definitions: HashMap<String, String>, // variable -> defining cell
}
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
    pub execution_order: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hit_rate: f64,
    pub total_lookups: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub evictions: usize,
    // Legacy aliases for backward compatibility with tests
    pub hits: usize,
    pub misses: usize,
    pub size: usize,
}
impl Default for IncrementalTester {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalTester {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::incremental::IncrementalTester;
    ///
    /// let instance = IncrementalTester::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::incremental::IncrementalTester;
    ///
    /// let instance = IncrementalTester::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::incremental::IncrementalTester;
    ///
    /// let instance = IncrementalTester::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self::with_config(IncrementalConfig::default())
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::incremental::IncrementalTester;
    ///
    /// let mut instance = IncrementalTester::new();
    /// let result = instance.with_config();
    /// // Verify behavior
    /// ```
    pub fn with_config(config: IncrementalConfig) -> Self {
        let cache = TestResultCache::new(config.cache_directory.clone(), config.max_cache_size);
        let dependency_tracker = DependencyTracker::new();
        Self {
            cache,
            dependency_tracker,
            config,
        }
    }
    /// Execute notebook with incremental testing
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::incremental::IncrementalTester;
    ///
    /// let mut instance = IncrementalTester::new();
    /// let result = instance.execute_incremental();
    /// // Verify behavior
    /// ```
    pub fn execute_incremental(
        &mut self,
        notebook: &Notebook,
        changed_cells: &[String],
    ) -> IncrementalResult {
        let mut executed_cells = Vec::new();
        let mut cached_cells = Vec::new();
        // Build dependency graph
        if self.config.dependency_analysis {
            self.dependency_tracker.analyze_dependencies(notebook);
        }
        // Determine execution order
        let execution_order = self.dependency_tracker.topological_sort(notebook);
        // Find cells that need re-execution
        let cells_to_execute =
            self.find_cells_to_execute(notebook, changed_cells, &execution_order);
        // Execute or retrieve from cache
        for cell_id in &execution_order {
            let cell = notebook.get_cell(cell_id).unwrap();
            if cells_to_execute.contains(cell_id) {
                // Execute and cache
                let result = self.execute_cell(cell);
                self.cache.store(
                    cell_id,
                    &cell.source,
                    &self.get_dependencies_hash(cell_id),
                    result,
                );
                executed_cells.push(cell_id.clone());
            } else {
                // Try to use cached result
                if let Some(_cached) = self.cache.get(cell_id) {
                    cached_cells.push(cell_id.clone());
                } else {
                    // Cache miss - need to execute
                    let result = self.execute_cell(cell);
                    self.cache.store(
                        cell_id,
                        &cell.source,
                        &self.get_dependencies_hash(cell_id),
                        result,
                    );
                    executed_cells.push(cell_id.clone());
                }
            }
        }
        IncrementalResult {
            executed_cells,
            cached_cells,
            dependency_graph: self.dependency_tracker.get_graph(),
            cache_stats: self.cache.get_statistics(),
        }
    }
    fn find_cells_to_execute(
        &mut self,
        notebook: &Notebook,
        changed_cells: &[String],
        execution_order: &[String],
    ) -> HashSet<String> {
        let mut to_execute = HashSet::new();
        // Add directly changed cells
        for cell_id in changed_cells {
            to_execute.insert(cell_id.clone());
        }
        // Add dependent cells (cascade invalidation)
        for cell_id in execution_order {
            if to_execute.contains(cell_id) {
                continue;
            }
            // Check if any dependencies have changed
            let dependencies = self.dependency_tracker.get_dependencies(cell_id);
            if dependencies.iter().any(|dep| to_execute.contains(dep)) {
                to_execute.insert(cell_id.clone());
            }
            // Check if cached result is stale
            let cell = notebook.get_cell(cell_id).unwrap();
            if !self.is_cache_valid(cell_id, &cell.source) {
                to_execute.insert(cell_id.clone());
            }
        }
        to_execute
    }
    fn execute_cell(&self, cell: &Cell) -> TestResult {
        let start = std::time::Instant::now();
        // Simulate cell execution
        let success = !cell.source.contains("error");
        let output = if success {
            "OK".to_string()
        } else {
            "Error occurred".to_string()
        };
        let duration = start.elapsed();
        TestResult {
            success,
            output,
            duration_ms: duration.as_millis() as u64,
            memory_used: 1024, // Simulate memory usage
        }
    }
    fn is_cache_valid(&mut self, cell_id: &str, source: &str) -> bool {
        if let Some(cached) = self.cache.get(cell_id) {
            let source_hash = self.calculate_hash(source);
            let deps_hash = self.get_dependencies_hash(cell_id);
            // Check if source or dependencies changed
            if cached.source_hash != source_hash || cached.dependencies_hash != deps_hash {
                return false;
            }
            // Check TTL
            if let Ok(elapsed) = cached.timestamp.elapsed() {
                if elapsed > self.config.cache_ttl {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
    fn get_dependencies_hash(&self, cell_id: &str) -> String {
        let dependencies = self.dependency_tracker.get_dependencies(cell_id);
        let mut combined = dependencies
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<_>>();
        combined.sort_unstable();
        self.calculate_hash(&combined.join(","))
    }
    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
}
impl TestResultCache {
    pub fn new(cache_dir: PathBuf, max_size: usize) -> Self {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).ok();
        }
        let mut cache = Self {
            cache: HashMap::new(),
            access_order: Vec::new(),
            max_size,
            cache_dir,
        };
        // Load existing cache from disk
        cache.load_from_disk();
        cache
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::incremental::store;
    ///
    /// let result = store("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn store(
        &mut self,
        cell_id: &str,
        source: &str,
        dependencies_hash: &str,
        result: TestResult,
    ) {
        let source_hash = self.calculate_hash(source);
        let cached_result = CachedTestResult {
            cell_id: cell_id.to_string(),
            source_hash,
            dependencies_hash: dependencies_hash.to_string(),
            result,
            timestamp: SystemTime::now(),
            access_count: 0,
        };
        // Remove old entry from access order
        self.access_order.retain(|id| id != cell_id);
        // Add to front of access order
        self.access_order.insert(0, cell_id.to_string());
        // Store in cache
        self.cache.insert(cell_id.to_string(), cached_result);
        // Evict if over capacity
        if self.cache.len() > self.max_size {
            self.evict_lru();
        }
        // Persist to disk
        self.save_to_disk(cell_id);
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::incremental::get;
    ///
    /// let result = get("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get(&mut self, cell_id: &str) -> Option<CachedTestResult> {
        if let Some(mut cached) = self.cache.get(cell_id).cloned() {
            // Update access order
            self.access_order.retain(|id| id != cell_id);
            self.access_order.insert(0, cell_id.to_string());
            // Update access count
            cached.access_count += 1;
            self.cache.insert(cell_id.to_string(), cached.clone());
            Some(cached)
        } else {
            None
        }
    }
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.access_order.pop() {
            self.cache.remove(&lru_key);
            // Remove from disk
            let cache_file = self.cache_dir.join(format!("{lru_key}.json"));
            std::fs::remove_file(cache_file).ok();
        }
    }
    fn load_from_disk(&mut self) {
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            if let Ok(cached) = serde_json::from_str::<CachedTestResult>(&content) {
                                self.cache.insert(cached.cell_id.clone(), cached.clone());
                                self.access_order.push(cached.cell_id);
                            }
                        }
                    }
                }
            }
        }
    }
    fn save_to_disk(&self, cell_id: &str) {
        if let Some(cached) = self.cache.get(cell_id) {
            let cache_file = self.cache_dir.join(format!("{cell_id}.json"));
            if let Ok(content) = serde_json::to_string_pretty(cached) {
                std::fs::write(cache_file, content).ok();
            }
        }
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::incremental::get_statistics;
    ///
    /// let result = get_statistics(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_statistics(&self) -> CacheStatistics {
        let total_access_count: usize = self.cache.values().map(|c| c.access_count).sum();
        let total_lookups = total_access_count + self.cache.len(); // Approximate
        CacheStatistics {
            hit_rate: if total_lookups > 0 {
                total_access_count as f64 / total_lookups as f64
            } else {
                0.0
            },
            total_lookups,
            cache_hits: total_access_count,
            cache_misses: self.cache.len(),
            evictions: 0, // Would track in real implementation
            // Legacy aliases for backward compatibility
            hits: total_access_count,
            misses: self.cache.len(),
            size: self.cache.len(),
        }
    }
    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
}
impl Default for DependencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            variable_definitions: HashMap::new(),
        }
    }
    /// Analyze dependencies between cells
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::incremental::DependencyTracker;
    ///
    /// let mut instance = DependencyTracker::new();
    /// let result = instance.analyze_dependencies();
    /// // Verify behavior
    /// ```
    pub fn analyze_dependencies(&mut self, notebook: &Notebook) {
        self.dependencies.clear();
        self.variable_definitions.clear();
        // First pass: find variable definitions
        for cell in &notebook.cells {
            if matches!(cell.cell_type, CellType::Code) {
                let defined_vars = self.extract_definitions(&cell.source);
                for var in defined_vars {
                    self.variable_definitions.insert(var, cell.id.clone());
                }
            }
        }
        // Second pass: find variable usages and create dependencies
        for cell in &notebook.cells {
            if matches!(cell.cell_type, CellType::Code) {
                let used_vars = self.extract_usages(&cell.source);
                let mut cell_deps = HashSet::new();
                for var in used_vars {
                    if let Some(defining_cell) = self.variable_definitions.get(&var) {
                        if defining_cell != &cell.id {
                            cell_deps.insert(defining_cell.clone());
                        }
                    }
                }
                if !cell_deps.is_empty() {
                    self.dependencies.insert(cell.id.clone(), cell_deps);
                }
            }
        }
    }
    fn extract_definitions(&self, source: &str) -> Vec<String> {
        let mut definitions = Vec::new();
        for line in source.lines() {
            let line = line.trim();
            // Look for variable assignments
            if let Some(pos) = line.find(" = ") {
                let var_part = &line[..pos];
                if let Some(var) = var_part.split_whitespace().last() {
                    definitions.push(var.to_string());
                }
            }
            // Look for let declarations
            if line.starts_with("let ") {
                if let Some(var) = line[4..].split_whitespace().next() {
                    definitions.push(var.to_string());
                }
            }
        }
        definitions
    }
    fn extract_usages(&self, source: &str) -> Vec<String> {
        // Simple variable extraction - split on non-alphanumeric chars
        source
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_alphabetic() || c == '_'))
            .map(std::string::ToString::to_string)
            .collect()
    }
    /// Get dependencies for a specific cell
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::incremental::get_dependencies;
    ///
    /// let result = get_dependencies("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_dependencies(&self, cell_id: &str) -> HashSet<String> {
        self.dependencies.get(cell_id).cloned().unwrap_or_default()
    }
    /// Perform topological sort for execution order
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::incremental::topological_sort;
    ///
    /// let result = topological_sort(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn topological_sort(&self, notebook: &Notebook) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        for cell in &notebook.cells {
            if !visited.contains(&cell.id) {
                self.dfs_visit(&cell.id, &mut visited, &mut visiting, &mut result);
            }
        }
        result
    }
    fn dfs_visit(
        &self,
        cell_id: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visiting.contains(cell_id) {
            // Cycle detected - skip for now
            return;
        }
        if visited.contains(cell_id) {
            return;
        }
        visiting.insert(cell_id.to_string());
        if let Some(deps) = self.dependencies.get(cell_id) {
            for dep in deps {
                self.dfs_visit(dep, visited, visiting, result);
            }
        }
        visiting.remove(cell_id);
        visited.insert(cell_id.to_string());
        result.push(cell_id.to_string());
    }
    /// Get dependency graph representation
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::incremental::get_graph;
    ///
    /// let result = get_graph(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_graph(&self) -> DependencyGraph {
        let mut nodes = HashSet::new();
        let mut edges = Vec::new();
        for (cell_id, deps) in &self.dependencies {
            nodes.insert(cell_id.clone());
            for dep in deps {
                nodes.insert(dep.clone());
                edges.push((dep.clone(), cell_id.clone()));
            }
        }
        DependencyGraph {
            nodes: nodes.into_iter().collect(),
            edges,
            execution_order: Vec::new(), // Would be filled by topological sort
        }
    }
}
impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            cache_directory: PathBuf::from(".ruchy_cache"),
            max_cache_size: 1000,
            cache_ttl: Duration::from_secs(24 * 60 * 60), // 24 hours
            force_rerun_threshold: 0.1,                   // 10% change threshold
            dependency_analysis: true,
        }
    }
}
// Supporting types
pub struct Notebook {
    pub cells: Vec<Cell>,
}
pub struct Cell {
    pub id: String,
    pub source: String,
    pub cell_type: CellType,
}
pub enum CellType {
    Code,
    Markdown,
}
impl Notebook {
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::incremental::get_cell;
    ///
    /// let result = get_cell("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_cell(&self, cell_id: &str) -> Option<&Cell> {
        self.cells.iter().find(|c| c.id == cell_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> IncrementalConfig {
        IncrementalConfig {
            cache_directory: PathBuf::from("/tmp/test_cache"),
            max_cache_size: 100,
            cache_ttl: Duration::from_secs(3600),
            force_rerun_threshold: 0.1,
            dependency_analysis: true,
        }
    }

    fn create_test_cell(id: &str, source: &str) -> Cell {
        Cell {
            id: id.to_string(),
            source: source.to_string(),
            cell_type: CellType::Code,
        }
    }

    fn create_test_notebook() -> Notebook {
        Notebook {
            cells: vec![
                create_test_cell("cell1", "x = 1"),
                create_test_cell("cell2", "y = x + 1"),
                create_test_cell("cell3", "z = y * 2"),
            ],
        }
    }

    #[test]
    fn test_incremental_tester_new() {
        let tester = IncrementalTester::new();
        assert_eq!(tester.config.max_cache_size, 1000);
        assert_eq!(tester.config.cache_ttl, Duration::from_secs(24 * 60 * 60));
        assert!(tester.config.dependency_analysis);
    }

    #[test]
    fn test_incremental_tester_default() {
        let tester = IncrementalTester::default();
        assert_eq!(tester.config.max_cache_size, 1000);
    }

    #[test]
    fn test_incremental_tester_with_config() {
        let config = create_test_config();
        let tester = IncrementalTester::with_config(config);
        assert_eq!(tester.config.max_cache_size, 100);
        assert_eq!(tester.config.cache_ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_execute_incremental_empty_notebook() {
        let mut tester = IncrementalTester::new();
        let notebook = Notebook { cells: vec![] };
        let result = tester.execute_incremental(&notebook, &[]);
        assert!(result.executed_cells.is_empty());
        assert!(result.cached_cells.is_empty());
    }

    #[test]
    fn test_test_result_cache_new() {
        let temp_dir = TempDir::new().unwrap();
        let cache = TestResultCache::new(temp_dir.path().to_path_buf(), 10);
        assert_eq!(cache.max_size, 10);
        assert!(cache.cache.is_empty());
        assert!(cache.access_order.is_empty());
    }

    #[test]
    fn test_cache_store_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = TestResultCache::new(temp_dir.path().to_path_buf(), 10);

        let test_result = TestResult {
            success: true,
            output: "test output".to_string(),
            duration_ms: 100,
            memory_used: 1024,
        };

        cache.store("test_cell", "println('hello')", "dep_hash", test_result);

        let cached = cache.get("test_cell");
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.cell_id, "test_cell");
        assert_eq!(cached.result.output, "test output");
        assert_eq!(cached.result.duration_ms, 100);
    }

    #[test]
    fn test_dependency_tracker_new() {
        let tracker = DependencyTracker::new();
        assert!(tracker.dependencies.is_empty());
        assert!(tracker.variable_definitions.is_empty());
    }

    #[test]
    fn test_dependency_tracker_default() {
        let tracker = DependencyTracker::default();
        assert!(tracker.dependencies.is_empty());
    }

    #[test]
    fn test_analyze_dependencies() {
        let mut tracker = DependencyTracker::new();
        let notebook = create_test_notebook();

        tracker.analyze_dependencies(&notebook);

        // Check that dependencies are tracked
        let deps = tracker.get_dependencies("cell2");
        assert!(!deps.is_empty());
    }

    #[test]
    fn test_get_dependencies_empty() {
        let tracker = DependencyTracker::new();
        let deps = tracker.get_dependencies("nonexistent");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_topological_sort() {
        let mut tracker = DependencyTracker::new();
        let notebook = create_test_notebook();

        tracker.analyze_dependencies(&notebook);
        let order = tracker.topological_sort(&notebook);
        assert_eq!(order.len(), 3);
        // cell1 should come before cell2, cell2 before cell3
        let pos1 = order.iter().position(|x| x == "cell1").unwrap();
        let pos2 = order.iter().position(|x| x == "cell2").unwrap();
        let pos3 = order.iter().position(|x| x == "cell3").unwrap();
        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
    }

    #[test]
    fn test_notebook_get_cell() {
        let notebook = create_test_notebook();

        let cell = notebook.get_cell("cell1");
        assert!(cell.is_some());
        assert_eq!(cell.unwrap().source, "x = 1");

        let nonexistent = notebook.get_cell("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_incremental_config_default() {
        let config = IncrementalConfig::default();
        assert_eq!(config.max_cache_size, 1000);
        assert_eq!(config.cache_ttl, Duration::from_secs(24 * 60 * 60));
        assert_eq!(config.force_rerun_threshold, 0.1);
        assert!(config.dependency_analysis);
    }
}
