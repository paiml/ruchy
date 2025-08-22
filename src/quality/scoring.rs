//! Unified quality scoring system for Ruchy code (RUCHY-0810)

/// Analysis depth for quality scoring
#[derive(Debug, Clone, Copy)]
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

/// Unified scoring engine
pub struct ScoreEngine {
    config: ScoreConfig,
}

impl ScoreEngine {
    pub fn new(config: ScoreConfig) -> Self {
        Self { config }
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
            cache_hit_rate: 0.0, // Caching implementation in RUCHY-0813
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