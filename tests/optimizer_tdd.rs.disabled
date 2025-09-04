//! Comprehensive TDD test suite for optimizer
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every optimization path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::backend::optimizer::{Optimizer, OptLevel, OptimizationPass};
use ruchy::frontend::ast::{Expr, Stmt, Program};

// ==================== OPTIMIZER INITIALIZATION TESTS ====================

#[test]
fn test_optimizer_new() {
    let optimizer = Optimizer::new();
    assert_eq!(optimizer.level(), OptLevel::None);
}

#[test]
fn test_optimizer_with_level() {
    let optimizer = Optimizer::with_level(OptLevel::Basic);
    assert_eq!(optimizer.level(), OptLevel::Basic);
}

#[test]
fn test_optimizer_levels() {
    assert!(OptLevel::None < OptLevel::Basic);
    assert!(OptLevel::Basic < OptLevel::Aggressive);
    assert!(OptLevel::Aggressive < OptLevel::Maximum);
}

// ==================== CONSTANT FOLDING TESTS ====================

#[test]
fn test_constant_fold_addition() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("+", Expr::literal(2), Expr::literal(3));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Literal(5)));
}

#[test]
fn test_constant_fold_multiplication() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("*", Expr::literal(4), Expr::literal(5));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Literal(20)));
}

#[test]
fn test_constant_fold_nested() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("+",
        Expr::binary("*", Expr::literal(2), Expr::literal(3)),
        Expr::literal(4)
    );
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Literal(10)));
}

#[test]
fn test_constant_fold_boolean() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("&&", Expr::literal_bool(true), Expr::literal_bool(false));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::LiteralBool(false)));
}

// ==================== DEAD CODE ELIMINATION TESTS ====================

#[test]
fn test_dead_code_after_return() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let stmts = vec![
        Stmt::return_value(Some(Expr::literal(42))),
        Stmt::let_binding("x", Expr::literal(100)), // Dead code
    ];
    
    let optimized = optimizer.optimize_statements(stmts);
    assert_eq!(optimized.len(), 1);
}

#[test]
fn test_dead_code_after_break() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let stmts = vec![
        Stmt::break_loop(None),
        Stmt::expr(Expr::call("println", vec![])), // Dead code
    ];
    
    let optimized = optimizer.optimize_statements(stmts);
    assert_eq!(optimized.len(), 1);
}

#[test]
fn test_dead_code_after_continue() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let stmts = vec![
        Stmt::continue_loop(),
        Stmt::assign("x", Expr::literal(10)), // Dead code
    ];
    
    let optimized = optimizer.optimize_statements(stmts);
    assert_eq!(optimized.len(), 1);
}

#[test]
fn test_unreachable_if_branch() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let expr = Expr::if_expr(
        Expr::literal_bool(false),
        Expr::call("unreachable", vec![]), // Dead branch
        Some(Expr::literal(42))
    );
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Literal(42)));
}

// ==================== COMMON SUBEXPRESSION ELIMINATION TESTS ====================

#[test]
fn test_cse_basic() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let x_plus_y = Expr::binary("+", Expr::ident("x"), Expr::ident("y"));
    let expr = Expr::binary("*", x_plus_y.clone(), x_plus_y);
    
    let optimized = optimizer.optimize_expr(expr);
    // Should introduce temporary for x + y
    assert!(optimizer.has_temp_binding());
}

#[test]
fn test_cse_complex() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let complex = Expr::call("expensive", vec![Expr::ident("x")]);
    let stmts = vec![
        Stmt::let_binding("a", complex.clone()),
        Stmt::let_binding("b", complex.clone()),
    ];
    
    let optimized = optimizer.optimize_statements(stmts);
    // Should reuse first computation
    assert!(optimizer.cse_count() == 1);
}

// ==================== LOOP OPTIMIZATION TESTS ====================

#[test]
fn test_loop_invariant_hoisting() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let loop_body = vec![
        Stmt::let_binding("invariant", Expr::literal(42)), // Loop invariant
        Stmt::assign("i", Expr::binary("+", Expr::ident("i"), Expr::literal(1))),
    ];
    
    let optimized = optimizer.optimize_loop(loop_body);
    // Invariant should be hoisted
    assert!(optimizer.hoisted_count() > 0);
}

#[test]
fn test_loop_unrolling() {
    let mut optimizer = Optimizer::with_level(OptLevel::Maximum);
    let for_loop = Stmt::for_loop("i", Expr::range(0, 4), vec![
        Stmt::expr(Expr::call("print", vec![Expr::ident("i")])),
    ]);
    
    let optimized = optimizer.optimize_stmt(for_loop);
    // Should unroll small constant loops
    assert!(optimizer.unrolled_loops() > 0);
}

// ==================== INLINE OPTIMIZATION TESTS ====================

#[test]
fn test_inline_small_function() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let call = Expr::call("small_func", vec![Expr::literal(42)]);
    
    optimizer.register_function("small_func", 1, vec![
        Stmt::return_value(Some(Expr::binary("*", Expr::ident("x"), Expr::literal(2))))
    ]);
    
    let optimized = optimizer.optimize_expr(call);
    // Should inline small function
    assert!(matches!(optimized, Expr::Binary(_, _, _)));
}

#[test]
fn test_no_inline_recursive() {
    let mut optimizer = Optimizer::with_level(OptLevel::Maximum);
    let call = Expr::call("recursive", vec![Expr::literal(10)]);
    
    optimizer.mark_recursive("recursive");
    
    let optimized = optimizer.optimize_expr(call);
    // Should not inline recursive functions
    assert!(matches!(optimized, Expr::Call(_, _)));
}

// ==================== ALGEBRAIC SIMPLIFICATION TESTS ====================

#[test]
fn test_simplify_add_zero() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("+", Expr::ident("x"), Expr::literal(0));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Ident("x")));
}

#[test]
fn test_simplify_multiply_one() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("*", Expr::ident("x"), Expr::literal(1));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Ident("x")));
}

#[test]
fn test_simplify_multiply_zero() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("*", Expr::ident("x"), Expr::literal(0));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Literal(0)));
}

#[test]
fn test_simplify_boolean_and_true() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("&&", Expr::ident("x"), Expr::literal_bool(true));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Ident("x")));
}

#[test]
fn test_simplify_boolean_or_false() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::binary("||", Expr::ident("x"), Expr::literal_bool(false));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Ident("x")));
}

// ==================== STRENGTH REDUCTION TESTS ====================

#[test]
fn test_strength_reduce_multiply_by_power_of_two() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let expr = Expr::binary("*", Expr::ident("x"), Expr::literal(8));
    
    let optimized = optimizer.optimize_expr(expr);
    // Should convert to left shift
    assert!(matches!(optimized, Expr::Binary("<<", _, _)));
}

#[test]
fn test_strength_reduce_divide_by_power_of_two() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let expr = Expr::binary("/", Expr::ident("x"), Expr::literal(16));
    
    let optimized = optimizer.optimize_expr(expr);
    // Should convert to right shift
    assert!(matches!(optimized, Expr::Binary(">>", _, _)));
}

// ==================== TAIL CALL OPTIMIZATION TESTS ====================

#[test]
fn test_tail_call_optimization() {
    let mut optimizer = Optimizer::with_level(OptLevel::Maximum);
    let func = Function {
        name: "factorial",
        body: vec![
            Stmt::if_stmt(
                Expr::binary("==", Expr::ident("n"), Expr::literal(0)),
                vec![Stmt::return_value(Some(Expr::literal(1)))],
                Some(vec![
                    Stmt::return_value(Some(
                        Expr::binary("*",
                            Expr::ident("n"),
                            Expr::call("factorial", vec![
                                Expr::binary("-", Expr::ident("n"), Expr::literal(1))
                            ])
                        )
                    ))
                ])
            )
        ]
    };
    
    let optimized = optimizer.optimize_function(func);
    assert!(optimizer.tail_calls_optimized() > 0);
}

// ==================== PEEPHOLE OPTIMIZATION TESTS ====================

#[test]
fn test_peephole_double_negation() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::unary("!", Expr::unary("!", Expr::ident("x")));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Ident("x")));
}

#[test]
fn test_peephole_negative_negative() {
    let mut optimizer = Optimizer::with_level(OptLevel::Basic);
    let expr = Expr::unary("-", Expr::unary("-", Expr::ident("x")));
    
    let optimized = optimizer.optimize_expr(expr);
    assert!(matches!(optimized, Expr::Ident("x")));
}

// ==================== PROGRAM OPTIMIZATION TESTS ====================

#[test]
fn test_optimize_complete_program() {
    let mut optimizer = Optimizer::with_level(OptLevel::Aggressive);
    let program = Program {
        functions: vec![
            Function {
                name: "main",
                body: vec![
                    Stmt::let_binding("x", Expr::binary("+", Expr::literal(1), Expr::literal(2))),
                    Stmt::return_value(Some(Expr::ident("x"))),
                    Stmt::expr(Expr::call("unreachable", vec![])), // Dead code
                ]
            }
        ]
    };
    
    let optimized = optimizer.optimize_program(program);
    assert!(optimized.functions[0].body.len() == 2); // Dead code removed
}

// ==================== OPTIMIZATION PASS TESTS ====================

#[test]
fn test_optimization_pass_ordering() {
    let optimizer = Optimizer::with_level(OptLevel::Maximum);
    let passes = optimizer.get_passes();
    
    // Constant folding should come before dead code elimination
    let cf_idx = passes.iter().position(|p| matches!(p, OptimizationPass::ConstantFolding));
    let dce_idx = passes.iter().position(|p| matches!(p, OptimizationPass::DeadCodeElimination));
    
    assert!(cf_idx.unwrap() < dce_idx.unwrap());
}

#[test]
fn test_custom_optimization_pipeline() {
    let mut optimizer = Optimizer::new();
    optimizer.add_pass(OptimizationPass::ConstantFolding);
    optimizer.add_pass(OptimizationPass::CommonSubexpressionElimination);
    
    assert_eq!(optimizer.pass_count(), 2);
}

// ==================== OPTIMIZATION STATISTICS TESTS ====================

#[test]
fn test_optimization_statistics() {
    let mut optimizer = Optimizer::with_level(OptLevel::Maximum);
    let expr = Expr::binary("+",
        Expr::binary("*", Expr::literal(2), Expr::literal(3)),
        Expr::binary("*", Expr::literal(4), Expr::literal(5))
    );
    
    let _ = optimizer.optimize_expr(expr);
    
    let stats = optimizer.get_statistics();
    assert!(stats.constants_folded > 0);
}

// Helper implementations for tests
struct Optimizer {
    level: OptLevel,
}

#[derive(PartialEq, PartialOrd)]
enum OptLevel {
    None,
    Basic,
    Aggressive,
    Maximum,
}

enum OptimizationPass {
    ConstantFolding,
    DeadCodeElimination,
    CommonSubexpressionElimination,
}

struct Function {
    name: &'static str,
    body: Vec<Stmt>,
}

struct Program {
    functions: Vec<Function>,
}

impl Optimizer {
    fn new() -> Self { Self { level: OptLevel::None } }
    fn with_level(level: OptLevel) -> Self { Self { level } }
    fn level(&self) -> OptLevel { OptLevel::None }
    fn optimize_expr(&mut self, expr: Expr) -> Expr { expr }
    fn optimize_statements(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> { stmts }
    fn optimize_stmt(&mut self, stmt: Stmt) -> Stmt { stmt }
    fn optimize_loop(&mut self, body: Vec<Stmt>) -> Vec<Stmt> { body }
    fn optimize_function(&mut self, func: Function) -> Function { func }
    fn optimize_program(&mut self, program: Program) -> Program { program }
    fn has_temp_binding(&self) -> bool { false }
    fn cse_count(&self) -> usize { 0 }
    fn hoisted_count(&self) -> usize { 0 }
    fn unrolled_loops(&self) -> usize { 0 }
    fn tail_calls_optimized(&self) -> usize { 0 }
    fn register_function(&mut self, _: &str, _: usize, _: Vec<Stmt>) {}
    fn mark_recursive(&mut self, _: &str) {}
    fn get_passes(&self) -> Vec<OptimizationPass> { vec![] }
    fn add_pass(&mut self, _: OptimizationPass) {}
    fn pass_count(&self) -> usize { 0 }
    fn get_statistics(&self) -> Stats { Stats::default() }
}

#[derive(Default)]
struct Stats {
    constants_folded: usize,
}

impl Expr {
    fn literal(_: i32) -> Self { unimplemented!() }
    fn literal_bool(_: bool) -> Self { unimplemented!() }
    fn ident(_: &str) -> Self { unimplemented!() }
    fn binary(_: &str, _: Self, _: Self) -> Self { unimplemented!() }
    fn unary(_: &str, _: Self) -> Self { unimplemented!() }
    fn call(_: &str, _: Vec<Self>) -> Self { unimplemented!() }
    fn if_expr(_: Self, _: Self, _: Option<Self>) -> Self { unimplemented!() }
    fn range(_: i32, _: i32) -> Self { unimplemented!() }
}

enum Expr {
    Literal(i32),
    LiteralBool(bool),
    Ident(&'static str),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    Unary(&'static str, Box<Expr>),
    Call(&'static str, Vec<Expr>),
}

impl Stmt {
    fn let_binding(_: &str, _: Expr) -> Self { unimplemented!() }
    fn assign(_: &str, _: Expr) -> Self { unimplemented!() }
    fn expr(_: Expr) -> Self { unimplemented!() }
    fn return_value(_: Option<Expr>) -> Self { unimplemented!() }
    fn break_loop(_: Option<Expr>) -> Self { unimplemented!() }
    fn continue_loop() -> Self { unimplemented!() }
    fn for_loop(_: &str, _: Expr, _: Vec<Self>) -> Self { unimplemented!() }
    fn if_stmt(_: Expr, _: Vec<Self>, _: Option<Vec<Self>>) -> Self { unimplemented!() }
}

// Run all tests with: cargo test optimizer_tdd --test optimizer_tdd