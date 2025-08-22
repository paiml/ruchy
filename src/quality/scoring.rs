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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Score correctness component (35% weight)
pub fn score_correctness(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut score = 1.0;
    
    // Pattern match exhaustiveness check
    let pattern_completeness = analyze_pattern_completeness(ast);
    score *= pattern_completeness;
    
    // Error handling coverage
    let error_handling_quality = analyze_error_handling(ast);
    score *= error_handling_quality;
    
    // Type consistency analysis
    let type_consistency = analyze_type_consistency(ast);
    score *= type_consistency;
    
    // Logical soundness (basic checks)
    let logical_soundness = analyze_logical_soundness(ast);
    score *= logical_soundness;
    
    score.clamp(0.0, 1.0)
}

/// Analyze pattern match completeness
fn analyze_pattern_completeness(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut total_matches = 0;
    let mut complete_matches = 0;
    
    analyze_pattern_completeness_recursive(ast, &mut total_matches, &mut complete_matches);
    
    if total_matches == 0 {
        1.0 // No matches, assume complete
    } else {
        #[allow(clippy::cast_precision_loss)]
        let score = (complete_matches as f64) / (total_matches as f64);
        score
    }
}

fn analyze_pattern_completeness_recursive(
    expr: &crate::frontend::ast::Expr,
    total_matches: &mut usize,
    complete_matches: &mut usize,
) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Match { expr: match_expr, arms } => {
            *total_matches += 1;
            
            // Check if match has wildcard pattern or covers all cases
            let has_wildcard = arms.iter().any(|arm| {
                matches!(arm.pattern, crate::frontend::ast::Pattern::Wildcard)
            });
            
            if has_wildcard || arms.len() >= 2 { // Basic heuristic
                *complete_matches += 1;
            }
            
            analyze_pattern_completeness_recursive(match_expr, total_matches, complete_matches);
            for arm in arms {
                analyze_pattern_completeness_recursive(&arm.body, total_matches, complete_matches);
            }
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            analyze_pattern_completeness_recursive(condition, total_matches, complete_matches);
            analyze_pattern_completeness_recursive(then_branch, total_matches, complete_matches);
            if let Some(else_expr) = else_branch {
                analyze_pattern_completeness_recursive(else_expr, total_matches, complete_matches);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                analyze_pattern_completeness_recursive(e, total_matches, complete_matches);
            }
        }
        ExprKind::Function { body, .. } => {
            analyze_pattern_completeness_recursive(body, total_matches, complete_matches);
        }
        _ => {}
    }
}

/// Analyze error handling quality
fn analyze_error_handling(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut total_fallible_ops = 0;
    let mut handled_ops = 0;
    
    analyze_error_handling_recursive(ast, &mut total_fallible_ops, &mut handled_ops);
    
    if total_fallible_ops == 0 {
        1.0 // No fallible operations
    } else {
        #[allow(clippy::cast_precision_loss)]
        let base_score = (handled_ops as f64) / (total_fallible_ops as f64);
        // Boost score if some error handling is present
        if handled_ops > 0 {
            (base_score + 0.3).min(1.0)
        } else {
            0.7 // Penalty for no error handling
        }
    }
}

fn analyze_error_handling_recursive(
    expr: &crate::frontend::ast::Expr,
    total_fallible_ops: &mut usize,
    handled_ops: &mut usize,
) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Try { expr: inner } => {
            *total_fallible_ops += 1;
            *handled_ops += 1;
            analyze_error_handling_recursive(inner, total_fallible_ops, handled_ops);
        }
        ExprKind::Match { expr: match_expr, arms } => {
            // Check if matching on Result type (heuristic)
            if arms.len() >= 2 {
                *total_fallible_ops += 1;
                *handled_ops += 1;
            }
            analyze_error_handling_recursive(match_expr, total_fallible_ops, handled_ops);
            for arm in arms {
                analyze_error_handling_recursive(&arm.body, total_fallible_ops, handled_ops);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                analyze_error_handling_recursive(e, total_fallible_ops, handled_ops);
            }
        }
        ExprKind::Function { body, .. } => {
            analyze_error_handling_recursive(body, total_fallible_ops, handled_ops);
        }
        _ => {}
    }
}

/// Analyze type consistency
fn analyze_type_consistency(_ast: &crate::frontend::ast::Expr) -> f64 {
    // For now, assume good consistency since we have type checking
    // Future: integrate with type checker for real analysis
    0.95
}

/// Analyze logical soundness
fn analyze_logical_soundness(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut score = 1.0;
    
    // Check for obvious logical issues
    let has_unreachable = has_unreachable_code(ast);
    if has_unreachable {
        score *= 0.8; // Penalty for unreachable code
    }
    
    let has_infinite_loops = has_potential_infinite_loops(ast);
    if has_infinite_loops {
        score *= 0.9; // Penalty for potential infinite loops
    }
    
    score
}

fn has_unreachable_code(ast: &crate::frontend::ast::Expr) -> bool {
    use crate::frontend::ast::ExprKind;
    
    match &ast.kind {
        ExprKind::Block(exprs) => {
            for (i, expr) in exprs.iter().enumerate() {
                if i < exprs.len() - 1 && is_diverging_expr(expr) {
                    return true; // Code after diverging expression
                }
                if has_unreachable_code(expr) {
                    return true;
                }
            }
            false
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            has_unreachable_code(condition) ||
            has_unreachable_code(then_branch) ||
            else_branch.as_ref().is_some_and(|e| has_unreachable_code(e))
        }
        _ => false
    }
}

fn is_diverging_expr(expr: &crate::frontend::ast::Expr) -> bool {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Call { func, .. } => {
            // Check for known diverging functions (heuristic)
            if let ExprKind::Identifier(name) = &func.kind {
                matches!(name.as_str(), "panic" | "unreachable" | "exit")
            } else {
                false
            }
        }
        _ => false
    }
}

fn has_potential_infinite_loops(ast: &crate::frontend::ast::Expr) -> bool {
    use crate::frontend::ast::ExprKind;
    
    match &ast.kind {
        ExprKind::While { condition, body } => {
            // Check for trivial infinite loops: while true { ... }
            if let ExprKind::Literal(crate::frontend::ast::Literal::Bool(true)) = &condition.kind {
                // Check if body has break statement
                !has_break_statement(body)
            } else {
                has_potential_infinite_loops(condition) || has_potential_infinite_loops(body)
            }
        }
        ExprKind::Block(exprs) => exprs.iter().any(has_potential_infinite_loops),
        ExprKind::Function { body, .. } => has_potential_infinite_loops(body),
        _ => false
    }
}

fn has_break_statement(ast: &crate::frontend::ast::Expr) -> bool {
    use crate::frontend::ast::ExprKind;
    
    match &ast.kind {
        ExprKind::Break { .. } => true,
        ExprKind::Block(exprs) => exprs.iter().any(has_break_statement),
        ExprKind::If { condition, then_branch, else_branch } => {
            has_break_statement(condition) ||
            has_break_statement(then_branch) ||
            else_branch.as_ref().is_some_and(|e| has_break_statement(e))
        }
        _ => false
    }
}

/// Score performance component (25% weight)
pub fn score_performance(ast: &crate::frontend::ast::Expr) -> f64 {
    let metrics = analyze_ast_metrics(ast);
    let mut score = 1.0;
    
    // Complexity analysis (BigO implications)
    let complexity_score = analyze_algorithmic_complexity(ast);
    score *= complexity_score;
    
    // Penalize high cyclomatic complexity (affects branch prediction)
    if metrics.cyclomatic_complexity > 10 {
        #[allow(clippy::cast_precision_loss)]
        let penalty = ((metrics.cyclomatic_complexity - 10) as f64 * 0.02).min(0.3);
        score *= 1.0 - penalty;
    }
    
    // Penalize deep nesting (affects cache performance)
    if metrics.max_nesting > 3 {
        #[allow(clippy::cast_precision_loss)]
        let penalty = ((metrics.max_nesting - 3) as f64 * 0.05).min(0.2);
        score *= 1.0 - penalty;
    }
    
    // Allocation analysis
    let allocation_score = analyze_allocation_patterns(ast);
    score *= allocation_score;
    
    // Memory access patterns (simplified for current AST)
    // Future enhancement when Index/Dict variants are added
    
    score.clamp(0.0, 1.0)
}

/// Analyze algorithmic complexity patterns
fn analyze_algorithmic_complexity(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    
    analyze_complexity_recursive(ast, &mut nested_loops, &mut recursive_calls, 0);
    
    let mut score = 1.0;
    
    // Penalize nested loops (O(n^k) complexity)
    if nested_loops > 0 {
        #[allow(clippy::cast_precision_loss)]
        let penalty = (f64::from(nested_loops) * 0.15).min(0.5);
        score *= 1.0 - penalty;
    }
    
    // Penalize recursive calls without obvious base case
    if recursive_calls > 2 {
        score *= 0.8; // May indicate exponential complexity
    }
    
    score
}

fn analyze_complexity_recursive(
    expr: &crate::frontend::ast::Expr,
    nested_loops: &mut i32,
    recursive_calls: &mut i32,
    current_nesting: i32,
) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::For { iter, body, .. } | ExprKind::While { condition: iter, body } => {
            if current_nesting > 0 {
                *nested_loops += 1;
            }
            analyze_complexity_recursive(iter, nested_loops, recursive_calls, current_nesting);
            analyze_complexity_recursive(body, nested_loops, recursive_calls, current_nesting + 1);
        }
        ExprKind::Call { func, args } => {
            // Check for potential recursive calls (heuristic)
            if let ExprKind::Identifier(_) = &func.kind {
                *recursive_calls += 1;
            }
            analyze_complexity_recursive(func, nested_loops, recursive_calls, current_nesting);
            for arg in args {
                analyze_complexity_recursive(arg, nested_loops, recursive_calls, current_nesting);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                analyze_complexity_recursive(e, nested_loops, recursive_calls, current_nesting);
            }
        }
        ExprKind::Function { body, .. } => {
            analyze_complexity_recursive(body, nested_loops, recursive_calls, 0);
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            analyze_complexity_recursive(condition, nested_loops, recursive_calls, current_nesting);
            analyze_complexity_recursive(then_branch, nested_loops, recursive_calls, current_nesting);
            if let Some(else_expr) = else_branch {
                analyze_complexity_recursive(else_expr, nested_loops, recursive_calls, current_nesting);
            }
        }
        _ => {}
    }
}

/// Analyze allocation patterns (GC pressure)
fn analyze_allocation_patterns(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut allocations = 0;
    let mut large_allocations = 0;
    
    count_allocations_recursive(ast, &mut allocations, &mut large_allocations);
    
    let mut score = 1.0;
    
    // Penalize excessive allocations
    if allocations > 10 {
        #[allow(clippy::cast_precision_loss)]
        let penalty = (f64::from(allocations - 10) * 0.01).min(0.3);
        score *= 1.0 - penalty;
    }
    
    // Penalize large allocations in loops
    if large_allocations > 0 {
        #[allow(clippy::cast_precision_loss)]
        let penalty = (f64::from(large_allocations) * 0.1).min(0.4);
        score *= 1.0 - penalty;
    }
    
    score
}

fn count_allocations_recursive(
    expr: &crate::frontend::ast::Expr,
    allocations: &mut i32,
    large_allocations: &mut i32,
) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::List(items) => {
            *allocations += 1;
            if items.len() > 100 {
                *large_allocations += 1;
            }
            for item in items {
                count_allocations_recursive(item, allocations, large_allocations);
            }
        }
        // Dictionary literals not yet implemented in AST
        // ExprKind::Dict { pairs } => { ... }
        ExprKind::StringInterpolation { parts } => {
            *allocations += 1; // String concatenation
            for part in parts {
                if let crate::frontend::ast::StringPart::Expr(e) = part {
                    count_allocations_recursive(e, allocations, large_allocations);
                }
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                count_allocations_recursive(e, allocations, large_allocations);
            }
        }
        ExprKind::Function { body, .. } => {
            count_allocations_recursive(body, allocations, large_allocations);
        }
        _ => {}
    }
}

// Memory access pattern analysis will be added when
// AST supports indexing and dictionary operations

/// Score maintainability component (20% weight)
pub fn score_maintainability(ast: &crate::frontend::ast::Expr) -> f64 {
    let metrics = analyze_ast_metrics(ast);
    let mut score = 1.0;
    
    // Coupling analysis
    let coupling_score = analyze_coupling(ast);
    score *= coupling_score;
    
    // Cohesion analysis
    let cohesion_score = analyze_cohesion(ast, &metrics);
    score *= cohesion_score;
    
    // Code duplication detection (simplified)
    let duplication_score = analyze_duplication(ast);
    score *= duplication_score;
    
    // Naming quality (basic heuristics)
    let naming_score = analyze_naming_quality(ast);
    score *= naming_score;
    
    score.clamp(0.0, 1.0)
}

/// Analyze coupling between functions/modules
fn analyze_coupling(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut external_calls = 0;
    let mut total_functions = 0;
    
    count_coupling_metrics(ast, &mut external_calls, &mut total_functions);
    
    if total_functions == 0 {
        return 1.0;
    }
    
    #[allow(clippy::cast_precision_loss)]
    let coupling_ratio = external_calls as f64 / total_functions as f64;
    
    // Lower coupling is better
    if coupling_ratio > 5.0 {
        0.7 // High coupling penalty
    } else if coupling_ratio > 2.0 {
        0.85
    } else {
        1.0 // Good coupling
    }
}

fn count_coupling_metrics(expr: &crate::frontend::ast::Expr, external_calls: &mut i32, total_functions: &mut i32) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Function { body, .. } => {
            *total_functions += 1;
            count_coupling_metrics(body, external_calls, total_functions);
        }
        ExprKind::Call { func, args } => {
            *external_calls += 1;
            count_coupling_metrics(func, external_calls, total_functions);
            for arg in args {
                count_coupling_metrics(arg, external_calls, total_functions);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                count_coupling_metrics(e, external_calls, total_functions);
            }
        }
        _ => {}
    }
}

/// Analyze cohesion within functions
fn analyze_cohesion(_ast: &crate::frontend::ast::Expr, metrics: &AstMetrics) -> f64 {
    let mut score = 1.0;
    
    // Penalize functions that are too large (low cohesion indicator)
    if metrics.line_count > 100 {
        score *= 0.8;
    }
    
    // Penalize excessive depth (indicates mixed concerns)
    if metrics.max_depth > 15 {
        score *= 0.85;
    }
    
    // Penalize too many functions (possible low cohesion)
    if metrics.function_count > 30 {
        score *= 0.9;
    }
    
    score
}

/// Analyze code duplication (simplified heuristic)
fn analyze_duplication(ast: &crate::frontend::ast::Expr) -> f64 {
    use crate::frontend::ast::ExprKind;
    let mut expression_patterns = std::collections::HashMap::new();
    
    collect_expression_patterns(ast, &mut expression_patterns);
    
    let duplicated_patterns = expression_patterns.values().filter(|&&count| count > 1).count();
    
    if duplicated_patterns > 5 {
        0.8 // Significant duplication penalty
    } else if duplicated_patterns > 2 {
        0.9 // Some duplication
    } else {
        1.0 // Little to no duplication
    }
}

fn collect_expression_patterns(expr: &crate::frontend::ast::Expr, patterns: &mut std::collections::HashMap<String, i32>) {
    use crate::frontend::ast::ExprKind;
    
    // Simplified pattern matching - use expression kind as pattern
    let pattern = format!("{:?}", std::mem::discriminant(&expr.kind));
    *patterns.entry(pattern).or_insert(0) += 1;
    
    match &expr.kind {
        ExprKind::Block(exprs) => {
            for e in exprs {
                collect_expression_patterns(e, patterns);
            }
        }
        ExprKind::Function { body, .. } => {
            collect_expression_patterns(body, patterns);
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            collect_expression_patterns(condition, patterns);
            collect_expression_patterns(then_branch, patterns);
            if let Some(else_expr) = else_branch {
                collect_expression_patterns(else_expr, patterns);
            }
        }
        _ => {}
    }
}

/// Analyze naming quality (basic heuristics)
fn analyze_naming_quality(ast: &crate::frontend::ast::Expr) -> f64 {
    use crate::frontend::ast::ExprKind;
    let mut good_names = 0;
    let mut total_names = 0;
    
    analyze_names_recursive(ast, &mut good_names, &mut total_names);
    
    if total_names == 0 {
        return 1.0;
    }
    
    #[allow(clippy::cast_precision_loss)]
    let good_ratio = good_names as f64 / total_names as f64;
    good_ratio.max(0.5) // Minimum score for naming
}

fn analyze_names_recursive(expr: &crate::frontend::ast::Expr, good_names: &mut i32, total_names: &mut i32) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Function { name, .. } => {
            *total_names += 1;
            if is_good_name(name) {
                *good_names += 1;
            }
        }
        ExprKind::Let { name, body, .. } => {
            *total_names += 1;
            if is_good_name(name) {
                *good_names += 1;
            }
            analyze_names_recursive(body, good_names, total_names);
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                analyze_names_recursive(e, good_names, total_names);
            }
        }
        _ => {}
    }
}

fn is_good_name(name: &str) -> bool {
    // Basic heuristics for good naming
    if name.len() < 2 || name.starts_with('_') {
        return false;
    }
    
    // Check for descriptive names (not single letters or abbreviations)
    name.len() >= 3 && !name.chars().all(|c| c.is_ascii_uppercase())
}

/// Score safety component (15% weight)
pub fn score_safety(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut score = 1.0;
    
    // Error handling coverage (reuse from correctness)
    let error_handling_quality = analyze_error_handling(ast);
    score *= error_handling_quality;
    
    // Null safety analysis
    let null_safety_score = analyze_null_safety(ast);
    score *= null_safety_score;
    
    // Resource management analysis
    let resource_score = analyze_resource_management(ast);
    score *= resource_score;
    
    // Bounds checking (implicit in type system, give good score)
    score *= 0.95; // Slight penalty for not having explicit bounds checks
    
    score.clamp(0.0, 1.0)
}

/// Analyze null safety patterns
fn analyze_null_safety(ast: &crate::frontend::ast::Expr) -> f64 {
    use crate::frontend::ast::ExprKind;
    let mut option_uses = 0;
    let mut unsafe_accesses = 0;
    
    analyze_null_safety_recursive(ast, &mut option_uses, &mut unsafe_accesses);
    
    if option_uses + unsafe_accesses == 0 {
        return 1.0; // No nullable types used
    }
    
    // Prefer Option types over unsafe accesses
    if unsafe_accesses == 0 {
        1.0 // All nullable accesses are safe
    } else {
        #[allow(clippy::cast_precision_loss)]
        let safety_ratio = option_uses as f64 / (option_uses + unsafe_accesses) as f64;
        safety_ratio.max(0.5) // Minimum safety score
    }
}

fn analyze_null_safety_recursive(expr: &crate::frontend::ast::Expr, option_uses: &mut i32, unsafe_accesses: &mut i32) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Some { .. } | ExprKind::None => {
            *option_uses += 1;
        }
        ExprKind::Match { arms, .. } => {
            // Check if matching on Option (heuristic)
            if arms.len() >= 2 {
                *option_uses += 1;
            }
            for arm in arms {
                analyze_null_safety_recursive(&arm.body, option_uses, unsafe_accesses);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                analyze_null_safety_recursive(e, option_uses, unsafe_accesses);
            }
        }
        ExprKind::Function { body, .. } => {
            analyze_null_safety_recursive(body, option_uses, unsafe_accesses);
        }
        _ => {}
    }
}

/// Analyze resource management patterns
fn analyze_resource_management(ast: &crate::frontend::ast::Expr) -> f64 {
    use crate::frontend::ast::ExprKind;
    let mut resource_allocations = 0;
    let mut proper_cleanup = 0;
    
    analyze_resources_recursive(ast, &mut resource_allocations, &mut proper_cleanup);
    
    if resource_allocations == 0 {
        return 1.0; // No resources to manage
    }
    
    #[allow(clippy::cast_precision_loss)]
    let cleanup_ratio = proper_cleanup as f64 / resource_allocations as f64;
    cleanup_ratio.max(0.7) // Minimum score for resource management
}

fn analyze_resources_recursive(expr: &crate::frontend::ast::Expr, allocations: &mut i32, cleanup: &mut i32) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::TryCatch { try_block, finally_block, .. } => {
            *allocations += 1;
            if finally_block.is_some() {
                *cleanup += 1; // Finally block suggests proper cleanup
            }
            analyze_resources_recursive(try_block, allocations, cleanup);
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                analyze_resources_recursive(e, allocations, cleanup);
            }
        }
        ExprKind::Function { body, .. } => {
            analyze_resources_recursive(body, allocations, cleanup);
        }
        _ => {}
    }
}

/// Score idiomaticity component (5% weight)
pub fn score_idiomaticity(ast: &crate::frontend::ast::Expr) -> f64 {
    let mut score = 1.0;
    
    // Pattern matching usage (idiomatic in Ruchy)
    let pattern_score = analyze_pattern_usage(ast);
    score *= pattern_score;
    
    // Iterator usage (functional style)
    let iterator_score = analyze_iterator_usage(ast);
    score *= iterator_score;
    
    // Lambda usage (functional programming)
    let lambda_score = analyze_lambda_usage(ast);
    score *= lambda_score;
    
    score.clamp(0.0, 1.0)
}

/// Analyze usage of pattern matching (idiomatic)
fn analyze_pattern_usage(ast: &crate::frontend::ast::Expr) -> f64 {
    use crate::frontend::ast::ExprKind;
    let mut matches = 0;
    let mut conditionals = 0;
    
    count_pattern_vs_conditional(ast, &mut matches, &mut conditionals);
    
    let total = matches + conditionals;
    if total == 0 {
        return 1.0;
    }
    
    #[allow(clippy::cast_precision_loss)]
    let pattern_ratio = matches as f64 / total as f64;
    
    // Higher ratio of matches vs if-else is more idiomatic
    if pattern_ratio > 0.7 {
        1.0
    } else if pattern_ratio > 0.4 {
        0.9
    } else {
        0.8
    }
}

fn count_pattern_vs_conditional(expr: &crate::frontend::ast::Expr, matches: &mut i32, conditionals: &mut i32) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            *matches += 1;
            for arm in arms {
                count_pattern_vs_conditional(&arm.body, matches, conditionals);
            }
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            *conditionals += 1;
            count_pattern_vs_conditional(condition, matches, conditionals);
            count_pattern_vs_conditional(then_branch, matches, conditionals);
            if let Some(else_expr) = else_branch {
                count_pattern_vs_conditional(else_expr, matches, conditionals);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                count_pattern_vs_conditional(e, matches, conditionals);
            }
        }
        ExprKind::Function { body, .. } => {
            count_pattern_vs_conditional(body, matches, conditionals);
        }
        _ => {}
    }
}

/// Analyze iterator usage patterns
fn analyze_iterator_usage(ast: &crate::frontend::ast::Expr) -> f64 {
    use crate::frontend::ast::ExprKind;
    let mut iterators = 0;
    let mut loops = 0;
    
    count_iterator_vs_loops(ast, &mut iterators, &mut loops);
    
    let total = iterators + loops;
    if total == 0 {
        return 1.0;
    }
    
    #[allow(clippy::cast_precision_loss)]
    let iterator_ratio = iterators as f64 / total as f64;
    
    // Higher ratio of iterators vs manual loops is more idiomatic
    if iterator_ratio > 0.6 {
        1.0
    } else if iterator_ratio > 0.3 {
        0.9
    } else {
        0.8
    }
}

fn count_iterator_vs_loops(expr: &crate::frontend::ast::Expr, iterators: &mut i32, loops: &mut i32) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::For { .. } => {
            *iterators += 1; // For loops in Ruchy are iterator-based
        }
        ExprKind::While { .. } => {
            *loops += 1;
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                count_iterator_vs_loops(e, iterators, loops);
            }
        }
        ExprKind::Function { body, .. } => {
            count_iterator_vs_loops(body, iterators, loops);
        }
        _ => {}
    }
}

/// Analyze lambda/closure usage
fn analyze_lambda_usage(ast: &crate::frontend::ast::Expr) -> f64 {
    use crate::frontend::ast::ExprKind;
    let mut lambdas = 0;
    let mut total_functions = 0;
    
    count_lambda_usage(ast, &mut lambdas, &mut total_functions);
    
    if total_functions == 0 {
        return 1.0;
    }
    
    #[allow(clippy::cast_precision_loss)]
    let lambda_ratio = lambdas as f64 / total_functions as f64;
    
    // Some lambda usage indicates functional style
    if lambda_ratio > 0.3 {
        1.0
    } else if lambda_ratio > 0.1 {
        0.95
    } else {
        0.9 // Still good if other patterns are used
    }
}

fn count_lambda_usage(expr: &crate::frontend::ast::Expr, lambdas: &mut i32, total_functions: &mut i32) {
    use crate::frontend::ast::ExprKind;
    
    match &expr.kind {
        ExprKind::Lambda { .. } => {
            *lambdas += 1;
            *total_functions += 1;
        }
        ExprKind::Function { body, .. } => {
            *total_functions += 1;
            count_lambda_usage(body, lambdas, total_functions);
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                count_lambda_usage(e, lambdas, total_functions);
            }
        }
        _ => {}
    }
}