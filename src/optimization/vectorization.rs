//! Vectorization opportunity detection

use serde::{Deserialize, Serialize};
use crate::frontend::ast::{Expr, ExprKind, BinaryOp};
use super::CodeLocation;

/// Vectorization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizationOpportunity {
    /// Type of vectorization possible
    pub vector_type: VectorType,
    
    /// Description of the opportunity
    pub description: String,
    
    /// Estimated speedup potential (e.g., 4.0 for 4x speedup)
    pub speedup_potential: f64,
    
    /// Required vector width in elements
    pub required_width: usize,
    
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    
    /// Code location
    pub location: Option<CodeLocation>,
    
    /// Specific optimization suggestions
    pub suggestions: Vec<String>,
}

/// Types of vectorization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VectorType {
    /// Single Instruction Multiple Data (SIMD)
    SIMD,
    
    /// Auto-vectorizable loop
    AutoVectorization,
    
    /// Manual vectorization with intrinsics
    ManualIntrinsics,
    
    /// Parallel reduction
    ParallelReduction,
    
    /// Scatter/Gather operations
    ScatterGather,
    
    /// Matrix operations
    Matrix,
}

/// Find vectorization opportunities in AST
pub fn find_vectorization_opportunities(ast: &Expr) -> Vec<VectorizationOpportunity> {
    let mut opportunities = Vec::new();
    find_opportunities_recursive(ast, &mut opportunities, 0);
    opportunities
}

fn find_opportunities_recursive(
    expr: &Expr,
    opportunities: &mut Vec<VectorizationOpportunity>,
    depth: usize,
) {
    match &expr.kind {
        ExprKind::For { var: _, iter, body } => {
            analyze_loop_vectorization(iter, body, opportunities);
            find_opportunities_recursive(iter, opportunities, depth + 1);
            find_opportunities_recursive(body, opportunities, depth + 1);
        }
        
        ExprKind::Binary { op, left, right } => {
            analyze_arithmetic_vectorization(op, left, right, opportunities);
            find_opportunities_recursive(left, opportunities, depth + 1);
            find_opportunities_recursive(right, opportunities, depth + 1);
        }
        
        ExprKind::List(items) => {
            analyze_array_operations(items, opportunities);
            for item in items {
                find_opportunities_recursive(item, opportunities, depth + 1);
            }
        }
        
        ExprKind::Call { func, args } => {
            analyze_function_vectorization(func, args, opportunities);
            find_opportunities_recursive(func, opportunities, depth + 1);
            for arg in args {
                find_opportunities_recursive(arg, opportunities, depth + 1);
            }
        }
        
        ExprKind::Block(exprs) => {
            analyze_block_vectorization(exprs, opportunities);
            for expr in exprs {
                find_opportunities_recursive(expr, opportunities, depth + 1);
            }
        }
        
        _ => {}
    }
}

fn analyze_loop_vectorization(
    iter: &Expr,
    body: &Expr,
    opportunities: &mut Vec<VectorizationOpportunity>,
) {
    // Check for simple array iteration patterns
    if is_array_iteration(iter) {
        let body_operations = count_vectorizable_operations(body);
        
        if body_operations > 0 {
            let speedup = estimate_loop_vectorization_speedup(body);
            let confidence = calculate_vectorization_confidence(body);
            
            opportunities.push(VectorizationOpportunity {
                vector_type: VectorType::AutoVectorization,
                description: format!("Loop with {} vectorizable operations", body_operations),
                speedup_potential: speedup,
                required_width: 4, // Default to 4-wide SIMD
                confidence,
                location: None,
                suggestions: vec![
                    "Consider using iterator methods that can be auto-vectorized".to_string(),
                    "Ensure loop bounds are known at compile time".to_string(),
                    "Avoid complex control flow in loop body".to_string(),
                ],
            });
        }
    }
    
    // Check for reduction patterns
    if is_reduction_pattern(body) {
        opportunities.push(VectorizationOpportunity {
            vector_type: VectorType::ParallelReduction,
            description: "Reduction operation that can be parallelized".to_string(),
            speedup_potential: 2.0,
            required_width: 4,
            confidence: 0.8,
            location: None,
            suggestions: vec![
                "Use parallel reduction techniques".to_string(),
                "Consider using SIMD horizontal operations".to_string(),
            ],
        });
    }
}

fn analyze_arithmetic_vectorization(
    op: &BinaryOp,
    left: &Expr,
    right: &Expr,
    opportunities: &mut Vec<VectorizationOpportunity>,
) {
    // Check for arithmetic operations on arrays/vectors
    if is_vectorizable_operation(op) && (is_array_expression(left) || is_array_expression(right)) {
        let speedup = match op {
            BinaryOp::Add | BinaryOp::Subtract => 4.0,  // Highly vectorizable
            BinaryOp::Multiply => 2.0,                   // Moderately vectorizable
            BinaryOp::Divide => 1.5,                     // Less vectorizable
            _ => 1.2,                                     // Minimal benefit
        };
        
        opportunities.push(VectorizationOpportunity {
            vector_type: VectorType::SIMD,
            description: format!("Arithmetic operation {:?} on array data", op),
            speedup_potential: speedup,
            required_width: 4,
            confidence: 0.9,
            location: None,
            suggestions: vec![
                format!("Use SIMD instructions for {} operations", format!("{:?}", op).to_lowercase()),
                "Ensure data alignment for optimal SIMD performance".to_string(),
            ],
        });
    }
    
    // Check for fused multiply-add opportunities
    if matches!(op, BinaryOp::Add) && is_multiply_expression(left) {
        opportunities.push(VectorizationOpportunity {
            vector_type: VectorType::SIMD,
            description: "Fused multiply-add (FMA) opportunity".to_string(),
            speedup_potential: 1.5,
            required_width: 4,
            confidence: 0.85,
            location: None,
            suggestions: vec![
                "Use FMA instructions for better performance and accuracy".to_string(),
            ],
        });
    }
}

fn analyze_array_operations(
    items: &[Expr],
    opportunities: &mut Vec<VectorizationOpportunity>,
) {
    if items.len() >= 4 {
        let arithmetic_ops = items.iter()
            .filter(|item| contains_arithmetic_operations(item))
            .count();
        
        if arithmetic_ops > items.len() / 2 {
            opportunities.push(VectorizationOpportunity {
                vector_type: VectorType::SIMD,
                description: format!("Array with {} elements containing arithmetic operations", items.len()),
                speedup_potential: (items.len() / 4) as f64,
                required_width: 4,
                confidence: 0.75,
                location: None,
                suggestions: vec![
                    "Process array elements in SIMD chunks".to_string(),
                    "Use packed data structures for better cache performance".to_string(),
                ],
            });
        }
    }
}

fn analyze_function_vectorization(
    func: &Expr,
    args: &[Expr],
    opportunities: &mut Vec<VectorizationOpportunity>,
) {
    if let ExprKind::Identifier(name) = &func.kind {
        match name.as_str() {
            "map" | "filter" | "reduce" | "fold" => {
                if args.len() >= 2 && is_array_expression(&args[1]) {
                    opportunities.push(VectorizationOpportunity {
                        vector_type: VectorType::AutoVectorization,
                        description: format!("Higher-order function '{}' on array data", name),
                        speedup_potential: match name.as_str() {
                            "map" => 4.0,
                            "filter" => 2.0,
                            "reduce" | "fold" => 2.5,
                            _ => 1.5,
                        },
                        required_width: 4,
                        confidence: 0.8,
                        location: None,
                        suggestions: vec![
                            format!("Use parallel {} implementation", name),
                            "Consider SIMD-optimized implementations".to_string(),
                        ],
                    });
                }
            }
            "dot" | "cross" | "normalize" => {
                opportunities.push(VectorizationOpportunity {
                    vector_type: VectorType::Matrix,
                    description: format!("Vector operation '{}' suitable for SIMD", name),
                    speedup_potential: 3.0,
                    required_width: 4,
                    confidence: 0.95,
                    location: None,
                    suggestions: vec![
                        "Use specialized SIMD vector math libraries".to_string(),
                        "Ensure vector data is properly aligned".to_string(),
                    ],
                });
            }
            "sin" | "cos" | "sqrt" | "exp" | "log" => {
                if args.iter().any(|arg| is_array_expression(arg)) {
                    opportunities.push(VectorizationOpportunity {
                        vector_type: VectorType::SIMD,
                        description: format!("Mathematical function '{}' on array data", name),
                        speedup_potential: 2.0,
                        required_width: 4,
                        confidence: 0.7,
                        location: None,
                        suggestions: vec![
                            format!("Use SIMD implementations of {}", name),
                            "Consider approximation algorithms for better performance".to_string(),
                        ],
                    });
                }
            }
            _ => {}
        }
    }
}

fn analyze_block_vectorization(
    exprs: &[Expr],
    opportunities: &mut Vec<VectorizationOpportunity>,
) {
    // Look for independent operations that can be executed in parallel
    let independent_ops = count_independent_operations(exprs);
    
    if independent_ops >= 4 {
        opportunities.push(VectorizationOpportunity {
            vector_type: VectorType::SIMD,
            description: format!("Block with {} independent operations", independent_ops),
            speedup_potential: (independent_ops / 4) as f64,
            required_width: 4,
            confidence: 0.6,
            location: None,
            suggestions: vec![
                "Reorder operations to maximize SIMD utilization".to_string(),
                "Consider using instruction-level parallelism".to_string(),
            ],
        });
    }
}

// Helper functions
fn is_array_iteration(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(_) => true, // Could be an array
        ExprKind::List(_) => true,
        _ => false,
    }
}

fn is_array_expression(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::List(_) | ExprKind::Identifier(_))
}

fn is_multiply_expression(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Binary { op: BinaryOp::Multiply, .. })
}

fn is_vectorizable_operation(op: &BinaryOp) -> bool {
    matches!(op, 
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | 
        BinaryOp::Divide | BinaryOp::And | BinaryOp::Or
    )
}

fn contains_arithmetic_operations(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            is_vectorizable_operation(op) || 
            contains_arithmetic_operations(left) || 
            contains_arithmetic_operations(right)
        }
        ExprKind::Block(exprs) => exprs.iter().any(|e| contains_arithmetic_operations(e)),
        _ => false,
    }
}

fn count_vectorizable_operations(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            let count = if is_vectorizable_operation(op) { 1 } else { 0 };
            count + count_vectorizable_operations(left) + count_vectorizable_operations(right)
        }
        ExprKind::Block(exprs) => exprs.iter().map(|e| count_vectorizable_operations(e)).sum(),
        ExprKind::Call { args, .. } => args.iter().map(|e| count_vectorizable_operations(e)).sum(),
        _ => 0,
    }
}

fn count_independent_operations(exprs: &[Expr]) -> usize {
    // Simplified analysis - in practice, would need data dependency analysis
    exprs.iter()
        .filter(|expr| contains_arithmetic_operations(expr))
        .count()
}

fn is_reduction_pattern(expr: &Expr) -> bool {
    // Detect common reduction patterns (sum, product, min, max, etc.)
    match &expr.kind {
        ExprKind::Binary { op, .. } => {
            matches!(op, BinaryOp::Add | BinaryOp::Multiply)
        }
        ExprKind::Call { func, .. } => {
            if let ExprKind::Identifier(name) = &func.kind {
                matches!(name.as_str(), "sum" | "product" | "min" | "max" | "reduce")
            } else {
                false
            }
        }
        _ => false,
    }
}

fn estimate_loop_vectorization_speedup(body: &Expr) -> f64 {
    let operations = count_vectorizable_operations(body);
    match operations {
        0 => 1.0,
        1..=2 => 2.0,
        3..=5 => 3.0,
        _ => 4.0,
    }
}

fn calculate_vectorization_confidence(body: &Expr) -> f64 {
    let complexity = calculate_control_flow_complexity(body);
    match complexity {
        0..=1 => 0.9,   // Simple straight-line code
        2..=3 => 0.7,   // Some branching
        4..=6 => 0.5,   // Complex control flow
        _ => 0.3,       // Very complex, hard to vectorize
    }
}

fn calculate_control_flow_complexity(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::If { condition, then_branch, else_branch } => {
            1 + calculate_control_flow_complexity(condition)
            + calculate_control_flow_complexity(then_branch)
            + else_branch.as_ref().map_or(0, |e| calculate_control_flow_complexity(e))
        }
        ExprKind::Match { arms, .. } => {
            arms.len() + arms.iter().map(|arm| calculate_control_flow_complexity(&arm.body)).sum::<usize>()
        }
        ExprKind::While { condition, body } => {
            2 + calculate_control_flow_complexity(condition) + calculate_control_flow_complexity(body)
        }
        ExprKind::Block(exprs) => exprs.iter().map(|e| calculate_control_flow_complexity(e)).sum(),
        _ => 0,
    }
}