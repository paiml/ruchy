//! Zero-cost abstraction verification

use serde::{Deserialize, Serialize};
use crate::frontend::ast::{Expr, ExprKind};
use super::CodeLocation;

/// Zero-cost abstraction analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractionAnalysis {
    /// Runtime overhead as percentage (0.0-1.0)
    pub runtime_overhead: f64,
    
    /// Abstraction patterns found
    pub patterns: Vec<AbstractionPattern>,
    
    /// Inlining opportunities
    pub inlining_opportunities: Vec<InliningOpportunity>,
    
    /// Memory allocation overhead
    pub allocation_overhead: f64,
    
    /// Type system overhead
    pub type_overhead: f64,
}

/// Abstraction pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractionPattern {
    /// Type of abstraction
    pub pattern_type: AbstractionType,
    
    /// Whether it's truly zero-cost
    pub is_zero_cost: bool,
    
    /// Estimated overhead if not zero-cost
    pub overhead_estimate: f64,
    
    /// Description of the pattern
    pub description: String,
    
    /// Location in code
    pub location: Option<CodeLocation>,
    
    /// Optimization suggestions
    pub suggestions: Vec<String>,
}

/// Types of abstractions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AbstractionType {
    /// Function call overhead
    FunctionCall,
    
    /// Iterator abstraction
    Iterator,
    
    /// Closure capture
    Closure,
    
    /// Generic specialization
    Generic,
    
    /// Trait object dispatch
    TraitObject,
    
    /// Option/Result monadic operations
    Monadic,
    
    /// Higher-order functions
    HigherOrder,
    
    /// Type conversion
    TypeConversion,
}

/// Inlining opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InliningOpportunity {
    /// Function or closure that could be inlined
    pub function_name: String,
    
    /// Call frequency estimate
    pub call_frequency: usize,
    
    /// Function size in nodes
    pub function_size: usize,
    
    /// Inlining benefit score (0.0-1.0)
    pub benefit_score: f64,
    
    /// Location of calls
    pub call_sites: Vec<CodeLocation>,
    
    /// Reasons why inlining would help
    pub benefits: Vec<String>,
}

/// Analyze zero-cost abstractions in AST
pub fn analyze_abstractions(ast: &Expr) -> AbstractionAnalysis {
    let mut patterns = Vec::new();
    let mut inlining_opportunities = Vec::new();
    
    analyze_abstractions_recursive(ast, &mut patterns, &mut inlining_opportunities, 0);
    
    let runtime_overhead = calculate_total_overhead(&patterns);
    let allocation_overhead = calculate_allocation_overhead(&patterns);
    let type_overhead = calculate_type_overhead(&patterns);
    
    AbstractionAnalysis {
        runtime_overhead,
        patterns,
        inlining_opportunities,
        allocation_overhead,
        type_overhead,
    }
}

fn analyze_abstractions_recursive(
    expr: &Expr,
    patterns: &mut Vec<AbstractionPattern>,
    inlining: &mut Vec<InliningOpportunity>,
    depth: usize,
) {
    match &expr.kind {
        ExprKind::Call { func, args } => {
            analyze_function_call(func, args, patterns, inlining);
            analyze_abstractions_recursive(func, patterns, inlining, depth + 1);
            for arg in args {
                analyze_abstractions_recursive(arg, patterns, inlining, depth + 1);
            }
        }
        
        ExprKind::Lambda { params, body } => {
            analyze_closure_pattern(params, body, patterns);
            analyze_abstractions_recursive(body, patterns, inlining, depth + 1);
        }
        
        ExprKind::For { var: _, iter, body } => {
            analyze_iterator_pattern(iter, body, patterns);
            analyze_abstractions_recursive(iter, patterns, inlining, depth + 1);
            analyze_abstractions_recursive(body, patterns, inlining, depth + 1);
        }
        
        ExprKind::Match { expr, arms } => {
            analyze_pattern_matching(expr, arms, patterns);
            analyze_abstractions_recursive(expr, patterns, inlining, depth + 1);
            for arm in arms {
                analyze_abstractions_recursive(&arm.body, patterns, inlining, depth + 1);
            }
        }
        
        ExprKind::Pipeline { expr, stages } => {
            analyze_pipeline_abstraction(expr, stages, patterns);
            analyze_abstractions_recursive(expr, patterns, inlining, depth + 1);
            for stage in stages {
                analyze_abstractions_recursive(&stage.op, patterns, inlining, depth + 1);
            }
        }
        
        ExprKind::Block(exprs) => {
            for expr in exprs {
                analyze_abstractions_recursive(expr, patterns, inlining, depth + 1);
            }
        }
        
        _ => {}
    }
}

fn analyze_function_call(
    func: &Expr,
    args: &[Expr],
    patterns: &mut Vec<AbstractionPattern>,
    inlining: &mut Vec<InliningOpportunity>,
) {
    if let ExprKind::Identifier(name) = &func.kind {
        // Check for higher-order function patterns
        if is_higher_order_function(name) {
            let is_zero_cost = can_be_inlined(name, args.len());
            patterns.push(AbstractionPattern {
                pattern_type: AbstractionType::HigherOrder,
                is_zero_cost,
                overhead_estimate: if is_zero_cost { 0.0 } else { 0.1 },
                description: format!("Higher-order function call: {name}"),
                location: None,
                suggestions: if is_zero_cost {
                    vec!["This abstraction should compile to zero cost".to_string()]
                } else {
                    vec![
                        "Consider manual loop unrolling for hot paths".to_string(),
                        "Profile to verify inlining occurs".to_string(),
                    ]
                },
            });
        }
        
        // Check for inlining opportunities
        if should_consider_inlining(name, args.len()) {
            inlining.push(InliningOpportunity {
                function_name: name.clone(),
                call_frequency: estimate_call_frequency(name),
                function_size: estimate_function_size(name),
                benefit_score: calculate_inlining_benefit(name, args.len()),
                call_sites: vec![], // Would be populated with actual source locations
                benefits: vec![
                    "Eliminate function call overhead".to_string(),
                    "Enable further optimizations at call site".to_string(),
                ],
            });
        }
    }
    
    // Check for monadic operations
    if is_monadic_operation(func) {
        patterns.push(AbstractionPattern {
            pattern_type: AbstractionType::Monadic,
            is_zero_cost: true, // Option/Result operations should be zero-cost
            overhead_estimate: 0.0,
            description: "Monadic operation (Option/Result)".to_string(),
            location: None,
            suggestions: vec![
                "Verify compiler optimizes away Option/Result wrapping".to_string(),
            ],
        });
    }
}

fn analyze_closure_pattern(
    params: &[crate::frontend::ast::Param],
    body: &Expr,
    patterns: &mut Vec<AbstractionPattern>,
) {
    let capture_analysis = analyze_closure_captures(body);
    let is_zero_cost = capture_analysis.is_zero_cost;
    
    patterns.push(AbstractionPattern {
        pattern_type: AbstractionType::Closure,
        is_zero_cost,
        overhead_estimate: if is_zero_cost { 0.0 } else { capture_analysis.overhead },
        description: format!(
            "Closure with {} parameters, {} captures",
            params.len(),
            capture_analysis.capture_count
        ),
        location: None,
        suggestions: if is_zero_cost {
            vec!["Closure should compile to direct function call".to_string()]
        } else {
            vec![
                "Consider reducing captured variables".to_string(),
                "Move captures to function parameters if possible".to_string(),
            ]
        },
    });
}

fn analyze_iterator_pattern(
    _iter: &Expr,
    body: &Expr,
    patterns: &mut Vec<AbstractionPattern>,
) {
    let complexity = estimate_iterator_complexity(body);
    let is_zero_cost = complexity.can_be_optimized;
    
    patterns.push(AbstractionPattern {
        pattern_type: AbstractionType::Iterator,
        is_zero_cost,
        overhead_estimate: if is_zero_cost { 0.0 } else { 0.05 },
        description: "Iterator-based loop".to_string(),
        location: None,
        suggestions: if is_zero_cost {
            vec![
                "Iterator should optimize to simple loop".to_string(),
                "Verify LLVM eliminates iterator overhead".to_string(),
            ]
        } else {
            vec![
                "Consider manual loop if performance critical".to_string(),
                "Profile iterator vs manual loop performance".to_string(),
            ]
        },
    });
}

fn analyze_pattern_matching(
    _value: &Expr,
    arms: &[crate::frontend::ast::MatchArm],
    patterns: &mut Vec<AbstractionPattern>,
) {
    let arm_count = arms.len();
    let is_zero_cost = arm_count <= 4; // Small match expressions can be optimized well
    
    patterns.push(AbstractionPattern {
        pattern_type: AbstractionType::Generic, // Using Generic for pattern matching
        is_zero_cost,
        overhead_estimate: if is_zero_cost { 0.0 } else { 0.02 * (arm_count - 4) as f64 },
        description: format!("Pattern matching with {arm_count} arms"),
        location: None,
        suggestions: if is_zero_cost {
            vec!["Pattern matching should compile to jump table or conditional chain".to_string()]
        } else {
            vec![
                "Large match expressions may have dispatch overhead".to_string(),
                "Consider reorganizing patterns by frequency".to_string(),
            ]
        },
    });
}

fn analyze_pipeline_abstraction(
    _expr: &Expr,
    _stages: &[crate::frontend::ast::PipelineStage],
    patterns: &mut Vec<AbstractionPattern>,
) {
    patterns.push(AbstractionPattern {
        pattern_type: AbstractionType::HigherOrder,
        is_zero_cost: true, // Pipeline should be zero-cost syntactic sugar
        overhead_estimate: 0.0,
        description: "Pipeline operation".to_string(),
        location: None,
        suggestions: vec![
            "Pipeline operator should desugar to direct function calls".to_string(),
        ],
    });
}

// Helper functions
struct ClosureCaptureAnalysis {
    capture_count: usize,
    is_zero_cost: bool,
    overhead: f64,
}

fn analyze_closure_captures(body: &Expr) -> ClosureCaptureAnalysis {
    let capture_count = count_free_variables(body);
    let is_zero_cost = capture_count == 0 || can_capture_by_value(body);
    
    ClosureCaptureAnalysis {
        capture_count,
        is_zero_cost,
        overhead: if is_zero_cost { 0.0 } else { capture_count as f64 * 0.01 },
    }
}

struct IteratorComplexityAnalysis {
    can_be_optimized: bool,
}

fn estimate_iterator_complexity(body: &Expr) -> IteratorComplexityAnalysis {
    let has_side_effects = contains_side_effects(body);
    let has_complex_control_flow = contains_complex_control_flow(body);
    
    IteratorComplexityAnalysis {
        can_be_optimized: !has_side_effects && !has_complex_control_flow,
    }
}

fn count_free_variables(_expr: &Expr) -> usize {
    // Simplified - would need proper scope analysis
    0
}

fn can_capture_by_value(_expr: &Expr) -> bool {
    // Simplified - would analyze capture types
    true
}

fn contains_side_effects(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { .. } => true, // Conservatively assume calls have side effects
        ExprKind::Block(exprs) => exprs.iter().any(contains_side_effects),
        _ => false,
    }
}

fn contains_complex_control_flow(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::While { .. } => true,
        ExprKind::Block(exprs) => exprs.iter().any(contains_complex_control_flow),
        _ => false,
    }
}

fn is_higher_order_function(name: &str) -> bool {
    matches!(name, "map" | "filter" | "reduce" | "fold" | "for_each" | "find" | "any" | "all")
}

fn can_be_inlined(name: &str, _arg_count: usize) -> bool {
    // Simple functions are usually inlined
    matches!(name, "map" | "filter" | "is_empty" | "len" | "get")
}

fn should_consider_inlining(name: &str, arg_count: usize) -> bool {
    // Consider inlining small, frequently called functions
    (name.len() < 10 && arg_count <= 3) || matches!(name, "get" | "set" | "len" | "is_empty")
}

fn estimate_call_frequency(name: &str) -> usize {
    match name {
        "len" | "is_empty" | "get" => 100, // High frequency
        "map" | "filter" => 50,            // Medium frequency
        _ => 10,                           // Low frequency
    }
}

fn estimate_function_size(name: &str) -> usize {
    match name {
        "len" | "is_empty" => 1, // Very small
        "get" | "set" => 3,      // Small
        "map" | "filter" => 10,  // Medium
        _ => 20,                 // Large
    }
}

fn calculate_inlining_benefit(name: &str, _arg_count: usize) -> f64 {
    let frequency = estimate_call_frequency(name) as f64;
    let size = estimate_function_size(name) as f64;
    
    // Higher frequency and smaller size = higher benefit
    (frequency / 100.0) * (1.0 - (size / 50.0).min(1.0))
}

fn is_monadic_operation(func: &Expr) -> bool {
    if let ExprKind::Identifier(name) = &func.kind {
        matches!(name.as_str(), "map" | "and_then" | "or_else" | "unwrap_or" | "ok_or")
    } else {
        false
    }
}

fn calculate_total_overhead(patterns: &[AbstractionPattern]) -> f64 {
    patterns.iter()
        .filter(|p| !p.is_zero_cost)
        .map(|p| p.overhead_estimate)
        .sum::<f64>()
        .min(1.0) // Cap at 100% overhead
}

fn calculate_allocation_overhead(patterns: &[AbstractionPattern]) -> f64 {
    patterns.iter()
        .filter(|p| matches!(p.pattern_type, AbstractionType::Closure | AbstractionType::TraitObject))
        .map(|p| p.overhead_estimate * 0.5) // Allocation is part of total overhead
        .sum::<f64>()
        .min(0.2) // Cap at 20% allocation overhead
}

fn calculate_type_overhead(patterns: &[AbstractionPattern]) -> f64 {
    patterns.iter()
        .filter(|p| matches!(p.pattern_type, AbstractionType::Generic | AbstractionType::TraitObject))
        .map(|p| p.overhead_estimate * 0.3) // Type overhead is part of total
        .sum::<f64>()
        .min(0.1) // Cap at 10% type overhead
}