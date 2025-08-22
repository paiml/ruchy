//! Unified quality scoring system for Ruchy code (RUCHY-0810)
//! Incremental scoring architecture (RUCHY-0813)

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use std::path::PathBuf;
use std::fs;

/// Analysis depth for quality scoring
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum AnalysisDepth {
    /// <100ms - AST metrics only
    Shallow,
    /// <1s - AST + type checking + basic flow
    Standard,
    /// <30s - Full property/mutation testing
    Deep,
}

/// Unified quality score with components
#[derive(Debug, Clone)]
pub struct QualityScore {
    pub value: f64,           // 0.0-1.0 normalized score
    pub components: ScoreComponents,
    pub grade: Grade,         // Human-readable grade
    pub confidence: f64,      // Confidence in score accuracy
    pub cache_hit_rate: f64,  // Percentage from cached analysis
}

/// Individual score components
#[derive(Debug, Clone)]
pub struct ScoreComponents {
    pub correctness: f64,     // 35% - Semantic correctness
    pub performance: f64,     // 25% - Runtime efficiency  
    pub maintainability: f64, // 20% - Change resilience
    pub safety: f64,          // 15% - Memory/type safety
    pub idiomaticity: f64,    // 5%  - Language conventions
}

/// Human-readable grade boundaries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grade {
    APlus,  // [0.97, 1.00] - Ship to production
    A,      // [0.93, 0.97) - Ship with confidence
    AMinus, // [0.90, 0.93) - Ship with review
    BPlus,  // [0.87, 0.90) - Acceptable
    B,      // [0.83, 0.87) - Needs work
    BMinus, // [0.80, 0.83) - Minimum viable
    CPlus,  // [0.77, 0.80) - Technical debt
    C,      // [0.73, 0.77) - Refactor advised
    CMinus, // [0.70, 0.73) - Refactor required
    D,      // [0.60, 0.70) - Major issues
    F,      // [0.00, 0.60) - Fundamental problems
}

impl Grade {
    pub fn from_score(value: f64) -> Self {
        match value {
            v if v >= 0.97 => Grade::APlus,
            v if v >= 0.93 => Grade::A,
            v if v >= 0.90 => Grade::AMinus,
            v if v >= 0.87 => Grade::BPlus,
            v if v >= 0.83 => Grade::B,
            v if v >= 0.80 => Grade::BMinus,
            v if v >= 0.77 => Grade::CPlus,
            v if v >= 0.73 => Grade::C,
            v if v >= 0.70 => Grade::CMinus,
            v if v >= 0.60 => Grade::D,
            _ => Grade::F,
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::AMinus => write!(f, "A-"),
            Grade::BPlus => write!(f, "B+"),
            Grade::B => write!(f, "B"),
            Grade::BMinus => write!(f, "B-"),
            Grade::CPlus => write!(f, "C+"),
            Grade::C => write!(f, "C"),
            Grade::CMinus => write!(f, "C-"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

/// Configuration for score weights
#[derive(Debug, Clone)]
pub struct ScoreConfig {
    pub correctness_weight: f64,
    pub performance_weight: f64,
    pub maintainability_weight: f64,
    pub safety_weight: f64,
    pub idiomaticity_weight: f64,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            correctness_weight: 0.35,
            performance_weight: 0.25,
            maintainability_weight: 0.20,
            safety_weight: 0.15,
            idiomaticity_weight: 0.05,
        }
    }
}

/// Cache key for scoring results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub file_path: PathBuf,
    pub content_hash: u64,
    pub depth: AnalysisDepth,
}

/// Cached scoring result with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub score: QualityScore,
    pub timestamp: SystemTime,
    pub dependencies: Vec<PathBuf>,
}

/// File dependency tracker
#[derive(Debug)]
pub struct DependencyTracker {
    /// Map of file -> files it depends on
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    /// Map of file -> last modified time
    file_times: HashMap<PathBuf, SystemTime>,
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
            file_times: HashMap::new(),
        }
    }

    pub fn track_dependency(&mut self, file: PathBuf, dependency: PathBuf) {
        self.dependencies.entry(file).or_default().push(dependency);
    }

    pub fn is_stale(&self, file: &PathBuf) -> bool {
        if let Some(dependencies) = self.dependencies.get(file) {
            for dep in dependencies {
                if self.is_file_modified(dep) {
                    return true;
                }
            }
        }
        false
    }

    fn is_file_modified(&self, file: &PathBuf) -> bool {
        let Ok(metadata) = fs::metadata(file) else { return true; };
        let Ok(modified) = metadata.modified() else { return true; };
        
        if let Some(&cached_time) = self.file_times.get(file) {
            modified > cached_time
        } else {
            true
        }
    }

    pub fn update_file_time(&mut self, file: PathBuf) {
        if let Ok(metadata) = fs::metadata(&file) {
            if let Ok(modified) = metadata.modified() {
                self.file_times.insert(file, modified);
            }
        }
    }
}

/// Incremental scoring engine with caching
pub struct ScoreEngine {
    config: ScoreConfig,
    cache: HashMap<CacheKey, CacheEntry>,
    dependency_tracker: DependencyTracker,
}

impl ScoreEngine {
    pub fn new(config: ScoreConfig) -> Self {
        Self { 
            config,
            cache: HashMap::new(),
            dependency_tracker: DependencyTracker::new(),
        }
    }
    
    pub fn score(&self, ast: &crate::frontend::ast::Expr, depth: AnalysisDepth) -> QualityScore {
        let components = match depth {
            AnalysisDepth::Shallow => Self::score_shallow(ast),
            AnalysisDepth::Standard => Self::score_standard(ast),
            AnalysisDepth::Deep => Self::score_deep(ast),
        };
        
        let value = self.calculate_weighted_score(&components);
        let grade = Grade::from_score(value);
        let confidence = Self::calculate_confidence(depth);
        
        QualityScore {
            value,
            components,
            grade,
            confidence,
            cache_hit_rate: 0.0, // No file path in legacy API
        }
    }

    /// Incremental scoring with file-based caching (RUCHY-0813)
    pub fn score_incremental(
        &mut self,
        ast: &crate::frontend::ast::Expr,
        file_path: PathBuf,
        content: &str,
        depth: AnalysisDepth,
    ) -> QualityScore {
        let content_hash = Self::hash_content(content);
        let cache_key = CacheKey {
            file_path: file_path.clone(),
            content_hash,
            depth,
        };

        // Check cache first
        if let Some(entry) = self.cache.get(&cache_key) {
            if !self.dependency_tracker.is_stale(&file_path) {
                let mut score = entry.score.clone();
                score.cache_hit_rate = 1.0;
                return score;
            }
        }

        // Fast path for small files - skip complex analysis
        let start = std::time::Instant::now();
        let is_small_file = content.len() < 1024;
        let effective_depth = if is_small_file && depth != AnalysisDepth::Deep {
            AnalysisDepth::Shallow
        } else {
            depth
        };

        let components = match effective_depth {
            AnalysisDepth::Shallow => Self::score_shallow(ast),
            AnalysisDepth::Standard => Self::score_standard(ast),
            AnalysisDepth::Deep => Self::score_deep(ast),
        };

        let value = self.calculate_weighted_score(&components);
        let grade = Grade::from_score(value);
        let confidence = if is_small_file && depth != effective_depth {
            Self::calculate_confidence(effective_depth) * 0.9 // Slightly reduced confidence for fast path
        } else {
            Self::calculate_confidence(depth)
        };
        let elapsed = start.elapsed();

        let score = QualityScore {
            value,
            components,
            grade,
            confidence,
            cache_hit_rate: 0.0,
        };

        // Cache the result only if worth caching
        let is_worth_caching = elapsed > Duration::from_millis(10) || !is_small_file;
        if is_worth_caching {
            let entry = CacheEntry {
                score: score.clone(),
                timestamp: SystemTime::now(),
                dependencies: Self::extract_dependencies(ast),
            };
            self.cache.insert(cache_key, entry);
        }

        self.dependency_tracker.update_file_time(file_path);

        // Optimize cache if scoring took too long or cache is getting large
        if elapsed > Duration::from_millis(100) || self.cache.len() > 1000 {
            self.optimize_cache();
        }

        score
    }

    /// Progressive scoring that refines analysis depth based on time budget
    pub fn score_progressive(
        &mut self,
        ast: &crate::frontend::ast::Expr,
        file_path: PathBuf,
        content: &str,
        time_budget: Duration,
    ) -> QualityScore {
        let start = std::time::Instant::now();

        // Start with shallow analysis
        let mut score = self.score_incremental(ast, file_path.clone(), content, AnalysisDepth::Shallow);
        
        if start.elapsed() < time_budget / 3 {
            // Upgrade to standard analysis
            score = self.score_incremental(ast, file_path.clone(), content, AnalysisDepth::Standard);
            
            if start.elapsed() < time_budget * 2 / 3 {
                // Upgrade to deep analysis
                score = self.score_incremental(ast, file_path, content, AnalysisDepth::Deep);
            }
        }

        score
    }

    fn hash_content(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    fn extract_dependencies(_ast: &crate::frontend::ast::Expr) -> Vec<PathBuf> {
        // Extract import/use dependencies from AST
        // Implementation in RUCHY-0814 with type checker integration
        Vec::new()
    }

    fn optimize_cache(&mut self) {
        // Remove old cache entries to maintain <100ms performance
        let now = SystemTime::now();
        let cutoff = Duration::from_secs(300); // 5 minutes
        let max_entries = 500; // Maximum cache entries for performance

        // First pass: Remove old entries
        self.cache.retain(|_, entry| {
            if let Ok(age) = now.duration_since(entry.timestamp) {
                age < cutoff
            } else {
                false
            }
        });

        // Second pass: If still too many entries, remove least recently used
        if self.cache.len() > max_entries {
            let mut entries: Vec<_> = self.cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
            entries.sort_by_key(|(_, timestamp)| *timestamp);
            
            let to_remove = self.cache.len() - max_entries;
            let keys_to_remove: Vec<_> = entries.iter()
                .take(to_remove)
                .map(|(k, _)| k.clone())
                .collect();
            
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
    }

    /// Clear all caches - useful for memory management
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.dependency_tracker = DependencyTracker::new();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            memory_usage_estimate: self.cache.len() * 1024, // Rough estimate
        }
    }
    
    fn score_shallow(ast: &crate::frontend::ast::Expr) -> ScoreComponents {
        // Fast AST-only analysis (<100ms)
        let metrics = analyze_ast_metrics(ast);
        
        let correctness = 1.0;
        let mut performance = 1.0;
        let mut maintainability = 1.0;
        let safety = 1.0;
        let idiomaticity = 1.0;
        
        // Penalize high complexity
        if metrics.max_depth > 10 {
            maintainability *= 0.9;
        }
        if metrics.function_count > 50 {
            maintainability *= 0.95;
        }
        
        // Penalize deep nesting
        if metrics.max_nesting > 5 {
            performance *= 0.9;
            maintainability *= 0.9;
        }
        
        // Penalize excessive lines
        if metrics.line_count > 1000 {
            maintainability *= 0.95;
        }
        
        ScoreComponents {
            correctness,
            performance,
            maintainability,
            safety,
            idiomaticity,
        }
    }
    
    fn score_standard(ast: &crate::frontend::ast::Expr) -> ScoreComponents {
        // Standard analysis with type checking (<1s)
        let mut components = Self::score_shallow(ast);
        
        // Additional type-based analysis
        // Type checker integration in RUCHY-0814
        components.correctness *= 0.95;
        components.safety *= 0.95;
        
        components
    }
    
    fn score_deep(ast: &crate::frontend::ast::Expr) -> ScoreComponents {
        // Deep analysis with property testing (<30s)
        let mut components = Self::score_standard(ast);
        
        // Additional deep analysis
        // Property testing and mutation testing in RUCHY-0816
        components.correctness *= 0.98;
        
        components
    }
    
    fn calculate_weighted_score(&self, components: &ScoreComponents) -> f64 {
        components.correctness * self.config.correctness_weight
            + components.performance * self.config.performance_weight
            + components.maintainability * self.config.maintainability_weight
            + components.safety * self.config.safety_weight
            + components.idiomaticity * self.config.idiomaticity_weight
    }
    
    fn calculate_confidence(depth: AnalysisDepth) -> f64 {
        match depth {
            AnalysisDepth::Shallow => 0.6,
            AnalysisDepth::Standard => 0.8,
            AnalysisDepth::Deep => 0.95,
        }
    }
}

/// AST metrics for analysis
#[derive(Debug)]
struct AstMetrics {
    function_count: usize,
    max_depth: usize,
    max_nesting: usize,
    line_count: usize,
    cyclomatic_complexity: usize,
}

fn analyze_ast_metrics(ast: &crate::frontend::ast::Expr) -> AstMetrics {
    let mut metrics = AstMetrics {
        function_count: 0,
        max_depth: 0,
        max_nesting: 0,
        line_count: 0,
        cyclomatic_complexity: 1, // Base complexity
    };
    
    analyze_expr(ast, &mut metrics, 0, 0);
    metrics
}

fn analyze_expr(expr: &crate::frontend::ast::Expr, metrics: &mut AstMetrics, depth: usize, nesting: usize) {
    use crate::frontend::ast::ExprKind;
    
    metrics.max_depth = metrics.max_depth.max(depth);
    metrics.max_nesting = metrics.max_nesting.max(nesting);
    
    match &expr.kind {
        ExprKind::Function { body, .. } => {
            metrics.function_count += 1;
            analyze_expr(body, metrics, depth + 1, 0);
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                analyze_expr(e, metrics, depth + 1, nesting);
            }
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            metrics.cyclomatic_complexity += 1;
            analyze_expr(condition, metrics, depth + 1, nesting + 1);
            analyze_expr(then_branch, metrics, depth + 1, nesting + 1);
            if let Some(else_expr) = else_branch {
                analyze_expr(else_expr, metrics, depth + 1, nesting + 1);
            }
        }
        ExprKind::While { condition, body } => {
            metrics.cyclomatic_complexity += 1;
            analyze_expr(condition, metrics, depth + 1, nesting + 1);
            analyze_expr(body, metrics, depth + 1, nesting + 1);
        }
        ExprKind::For { iter, body, .. } => {
            metrics.cyclomatic_complexity += 1;
            analyze_expr(iter, metrics, depth + 1, nesting + 1);
            analyze_expr(body, metrics, depth + 1, nesting + 1);
        }
        ExprKind::Match { expr: match_expr, arms } => {
            analyze_expr(match_expr, metrics, depth + 1, nesting);
            for arm in arms {
                metrics.cyclomatic_complexity += 1;
                if let Some(guard) = &arm.guard {
                    analyze_expr(guard, metrics, depth + 1, nesting + 1);
                }
                analyze_expr(&arm.body, metrics, depth + 1, nesting + 1);
            }
        }
        _ => {}
    }
}

impl QualityScore {
    /// Explain changes from a baseline score
    pub fn explain_delta(&self, baseline: &QualityScore) -> ScoreExplanation {
        let delta = self.value - baseline.value;
        let mut changes = Vec::new();
        let mut tradeoffs = Vec::new();
        
        // Track component changes
        let components = [
            ("Correctness", self.components.correctness, baseline.components.correctness),
            ("Performance", self.components.performance, baseline.components.performance),
            ("Maintainability", self.components.maintainability, baseline.components.maintainability),
            ("Safety", self.components.safety, baseline.components.safety),
            ("Idiomaticity", self.components.idiomaticity, baseline.components.idiomaticity),
        ];
        
        for (name, current, baseline) in components {
            let diff = current - baseline;
            if diff.abs() > 0.01 {
                changes.push(format!("{}: {}{:.1}%", 
                    name,
                    if diff > 0.0 { "+" } else { "" },
                    diff * 100.0
                ));
            }
        }
        
        // Detect tradeoffs
        if self.components.performance > baseline.components.performance 
            && self.components.maintainability < baseline.components.maintainability {
            tradeoffs.push("Performance improved at the cost of maintainability".to_string());
        }
        
        if self.components.safety > baseline.components.safety 
            && self.components.performance < baseline.components.performance {
            tradeoffs.push("Safety improved at the cost of performance".to_string());
        }
        
        ScoreExplanation {
            delta,
            changes,
            tradeoffs,
            grade_change: format!("{} → {}", baseline.grade, self.grade),
        }
    }
}

/// Explanation of score changes
pub struct ScoreExplanation {
    pub delta: f64,
    pub changes: Vec<String>,
    pub tradeoffs: Vec<String>,
    pub grade_change: String,
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub memory_usage_estimate: usize,
}

/// Score correctness component
pub fn score_correctness(_ast: &crate::frontend::ast::Expr) -> f64 {
    // Advanced correctness metrics in RUCHY-0816:
    // - Property test coverage
    // - Refinement type proofs
    // - Pattern match exhaustiveness
    // - Mutation score
    
    1.0
}

/// Score performance component
pub fn score_performance(ast: &crate::frontend::ast::Expr) -> f64 {
    let metrics = analyze_ast_metrics(ast);
    let mut score = 1.0;
    
    // Penalize high cyclomatic complexity
    if metrics.cyclomatic_complexity > 10 {
        #[allow(clippy::cast_precision_loss)]
        let penalty = ((metrics.cyclomatic_complexity - 10) as f64 * 0.01).min(0.2);
        score *= 0.9 - penalty;
    }
    
    // Penalize deep nesting (affects cache performance)
    if metrics.max_nesting > 3 {
        #[allow(clippy::cast_precision_loss)]
        let penalty = ((metrics.max_nesting - 3) as f64 * 0.05).min(0.2);
        score *= 0.95 - penalty;
    }
    
    score
}

/// Score maintainability component
pub fn score_maintainability(ast: &crate::frontend::ast::Expr) -> f64 {
    let metrics = analyze_ast_metrics(ast);
    let mut score = 1.0;
    
    // Penalize excessive depth
    if metrics.max_depth > 15 {
        score *= 0.9;
    }
    
    // Penalize too many functions (low cohesion)
    if metrics.function_count > 30 {
        score *= 0.95;
    }
    
    score
}

/// Score safety component
pub fn score_safety(_ast: &crate::frontend::ast::Expr) -> f64 {
    // Safety metrics implementation in RUCHY-0817:
    // - Unsafe block density
    // - Error handling quality
    // - Lifetime correctness
    
    0.9 // Default conservative score
}

/// Score idiomaticity component
pub fn score_idiomaticity(_ast: &crate::frontend::ast::Expr) -> f64 {
    // Idiomaticity metrics implementation in RUCHY-0818:
    // - Iterator usage ratio
    // - Pattern match usage
    // - Error propagation patterns
    
    0.85 // Default score
}