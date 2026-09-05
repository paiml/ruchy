//! PMAT-097: dispatch totality + determinism contracts for the two hot dispatch paths.
//!
//! Under test:
//! - `Transpiler::transpile_expr` (`src/backend/transpiler/expr_dispatcher.rs`)
//! - the interpreter's `eval_expr_kind` dispatch, reached through the public
//!   `Interpreter::eval_expr` (`eval_expr_kind` itself is `pub(crate)`, so an
//!   integration test cannot call it directly).
//!
//! Properties (see `contracts/transpile-dispatch-v1.yaml`,
//! `contracts/eval-dispatch-v1.yaml`):
//! - `dispatch_totality`:    `∀e ∈ Expr: f(e) ∈ Ok ∪ Err` — never `⊥` (panic/divergence).
//! - `dispatch_determinism`: `f(e) = f(e)` — two calls render identically.
//!
//! The sample set holds one `Expr` per `ExprKind` variant. `all_variants_covered`
//! is a compile-time exhaustiveness guard: it has no wildcard arm, so adding a
//! variant to `ExprKind` fails to compile until this file covers it.

use ruchy::frontend::ast::{
    BinaryOp, DataFrameOp, Expr, ExprKind, Literal, Pattern, Span, Type, TypeKind, UnaryOp,
};
use ruchy::runtime::Interpreter;
use ruchy::Transpiler;
use std::sync::mpsc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Sample construction helpers (simplest valid payload per variant).
// ---------------------------------------------------------------------------

fn e(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::default())
}

fn lit() -> Expr {
    e(ExprKind::Literal(Literal::Integer(1, None)))
}

fn blit() -> Box<Expr> {
    Box::new(lit())
}

fn ident() -> Expr {
    e(ExprKind::Identifier("x".to_string()))
}

fn bident() -> Box<Expr> {
    Box::new(ident())
}

fn ty() -> Type {
    Type {
        kind: TypeKind::Named("i32".to_string()),
        span: Span::default(),
    }
}

/// One `Expr` per `ExprKind` variant, paired with its variant name.
///
/// The property under test is totality, not semantics: payloads are the
/// simplest valid ones, chosen so that evaluation terminates (loops break
/// immediately, `while` conditions are false, iterators are empty).
fn variants() -> Vec<(&'static str, Expr)> {
    vec![
        ("Literal", lit()),
        ("Identifier", ident()),
        (
            "QualifiedName",
            e(ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "x".to_string(),
            }),
        ),
        (
            "StringInterpolation",
            e(ExprKind::StringInterpolation { parts: vec![] }),
        ),
        (
            "Binary",
            e(ExprKind::Binary {
                left: blit(),
                op: BinaryOp::Add,
                right: blit(),
            }),
        ),
        (
            "Unary",
            e(ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: blit(),
            }),
        ),
        ("Throw", e(ExprKind::Throw { expr: blit() })),
        (
            "TryCatch",
            e(ExprKind::TryCatch {
                try_block: blit(),
                catch_clauses: vec![],
                finally_block: None,
            }),
        ),
        ("Ok", e(ExprKind::Ok { value: blit() })),
        ("Err", e(ExprKind::Err { error: blit() })),
        ("Some", e(ExprKind::Some { value: blit() })),
        ("None", e(ExprKind::None)),
        (
            "TypeCast",
            e(ExprKind::TypeCast {
                expr: blit(),
                target_type: "i64".to_string(),
            }),
        ),
        (
            "Ternary",
            e(ExprKind::Ternary {
                condition: Box::new(e(ExprKind::Literal(Literal::Bool(true)))),
                true_expr: blit(),
                false_expr: blit(),
            }),
        ),
        ("Try", e(ExprKind::Try { expr: blit() })),
        ("Await", e(ExprKind::Await { expr: blit() })),
        ("Spawn", e(ExprKind::Spawn { actor: blit() })),
        ("AsyncBlock", e(ExprKind::AsyncBlock { body: blit() })),
        ("Lazy", e(ExprKind::Lazy { expr: blit() })),
        (
            "If",
            e(ExprKind::If {
                condition: Box::new(e(ExprKind::Literal(Literal::Bool(true)))),
                then_branch: blit(),
                else_branch: None,
            }),
        ),
        (
            "IfLet",
            e(ExprKind::IfLet {
                pattern: Pattern::Wildcard,
                expr: blit(),
                then_branch: blit(),
                else_branch: None,
            }),
        ),
        (
            "Let",
            e(ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: blit(),
                body: blit(),
                is_mutable: false,
                else_block: None,
            }),
        ),
        (
            "LetPattern",
            e(ExprKind::LetPattern {
                pattern: Pattern::Identifier("x".to_string()),
                type_annotation: None,
                value: blit(),
                body: blit(),
                is_mutable: false,
                else_block: None,
            }),
        ),
        (
            "Function",
            e(ExprKind::Function {
                name: "f".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: blit(),
                is_async: false,
                is_pub: false,
            }),
        ),
        (
            "Lambda",
            e(ExprKind::Lambda {
                params: vec![],
                body: blit(),
            }),
        ),
        (
            "AsyncLambda",
            e(ExprKind::AsyncLambda {
                params: vec![],
                body: blit(),
            }),
        ),
        (
            "Struct",
            e(ExprKind::Struct {
                name: "S".to_string(),
                type_params: vec![],
                fields: vec![],
                methods: vec![],
                derives: vec![],
                is_pub: false,
            }),
        ),
        (
            "TupleStruct",
            e(ExprKind::TupleStruct {
                name: "T".to_string(),
                type_params: vec![],
                fields: vec![],
                derives: vec![],
                is_pub: false,
            }),
        ),
        (
            "Class",
            e(ExprKind::Class {
                name: "C".to_string(),
                type_params: vec![],
                superclass: None,
                traits: vec![],
                fields: vec![],
                constructors: vec![],
                methods: vec![],
                constants: vec![],
                properties: vec![],
                derives: vec![],
                decorators: vec![],
                is_pub: false,
                is_sealed: false,
                is_abstract: false,
            }),
        ),
        (
            "Enum",
            e(ExprKind::Enum {
                name: "E".to_string(),
                type_params: vec![],
                variants: vec![],
                is_pub: false,
            }),
        ),
        (
            "StructLiteral",
            e(ExprKind::StructLiteral {
                name: "S".to_string(),
                fields: vec![],
                base: None,
            }),
        ),
        (
            "ObjectLiteral",
            e(ExprKind::ObjectLiteral { fields: vec![] }),
        ),
        (
            "FieldAccess",
            e(ExprKind::FieldAccess {
                object: bident(),
                field: "f".to_string(),
            }),
        ),
        (
            "OptionalFieldAccess",
            e(ExprKind::OptionalFieldAccess {
                object: bident(),
                field: "f".to_string(),
            }),
        ),
        (
            "IndexAccess",
            e(ExprKind::IndexAccess {
                object: bident(),
                index: blit(),
            }),
        ),
        (
            "Slice",
            e(ExprKind::Slice {
                object: bident(),
                start: None,
                end: None,
            }),
        ),
        (
            "Trait",
            e(ExprKind::Trait {
                name: "Tr".to_string(),
                type_params: vec![],
                associated_types: vec![],
                methods: vec![],
                is_pub: false,
            }),
        ),
        (
            "Impl",
            e(ExprKind::Impl {
                type_params: vec![],
                trait_name: None,
                for_type: "S".to_string(),
                methods: vec![],
                is_pub: false,
            }),
        ),
        (
            "Actor",
            e(ExprKind::Actor {
                name: "A".to_string(),
                state: vec![],
                handlers: vec![],
            }),
        ),
        (
            "Effect",
            e(ExprKind::Effect {
                name: "Eff".to_string(),
                operations: vec![],
            }),
        ),
        (
            "Handle",
            e(ExprKind::Handle {
                expr: blit(),
                handlers: vec![],
            }),
        ),
        (
            "Send",
            e(ExprKind::Send {
                actor: bident(),
                message: blit(),
            }),
        ),
        (
            "Command",
            e(ExprKind::Command {
                program: "true".to_string(),
                args: vec![],
                env: vec![],
                working_dir: None,
            }),
        ),
        (
            "Ask",
            e(ExprKind::Ask {
                actor: bident(),
                message: blit(),
                timeout: None,
            }),
        ),
        (
            "ActorSend",
            e(ExprKind::ActorSend {
                actor: bident(),
                message: blit(),
            }),
        ),
        (
            "ActorQuery",
            e(ExprKind::ActorQuery {
                actor: bident(),
                message: blit(),
            }),
        ),
        (
            "Call",
            e(ExprKind::Call {
                func: bident(),
                args: vec![],
            }),
        ),
        (
            "Macro",
            e(ExprKind::Macro {
                name: "m".to_string(),
                args: vec![],
            }),
        ),
        (
            "MethodCall",
            e(ExprKind::MethodCall {
                receiver: bident(),
                method: "m".to_string(),
                args: vec![],
            }),
        ),
        (
            "OptionalMethodCall",
            e(ExprKind::OptionalMethodCall {
                receiver: bident(),
                method: "m".to_string(),
                args: vec![],
            }),
        ),
        ("Block", e(ExprKind::Block(vec![lit()]))),
        (
            "Pipeline",
            e(ExprKind::Pipeline {
                expr: blit(),
                stages: vec![],
            }),
        ),
        (
            "Match",
            e(ExprKind::Match {
                expr: blit(),
                arms: vec![],
            }),
        ),
        ("List", e(ExprKind::List(vec![lit()]))),
        ("Set", e(ExprKind::Set(vec![lit()]))),
        (
            "ArrayInit",
            e(ExprKind::ArrayInit {
                value: blit(),
                size: blit(),
            }),
        ),
        ("Tuple", e(ExprKind::Tuple(vec![lit()]))),
        ("Spread", e(ExprKind::Spread { expr: blit() })),
        (
            "ListComprehension",
            e(ExprKind::ListComprehension {
                element: blit(),
                clauses: vec![],
            }),
        ),
        (
            "SetComprehension",
            e(ExprKind::SetComprehension {
                element: blit(),
                clauses: vec![],
            }),
        ),
        (
            "DictComprehension",
            e(ExprKind::DictComprehension {
                key: blit(),
                value: blit(),
                clauses: vec![],
            }),
        ),
        ("DataFrame", e(ExprKind::DataFrame { columns: vec![] })),
        (
            "DataFrameOperation",
            e(ExprKind::DataFrameOperation {
                source: bident(),
                operation: DataFrameOp::Limit(1),
            }),
        ),
        (
            "For",
            e(ExprKind::For {
                label: None,
                var: "i".to_string(),
                pattern: None,
                iter: Box::new(e(ExprKind::List(vec![]))),
                body: blit(),
            }),
        ),
        (
            "While",
            e(ExprKind::While {
                label: None,
                condition: Box::new(e(ExprKind::Literal(Literal::Bool(false)))),
                body: blit(),
            }),
        ),
        (
            "WhileLet",
            e(ExprKind::WhileLet {
                label: None,
                pattern: Pattern::Literal(Literal::Bool(false)),
                expr: blit(),
                body: blit(),
            }),
        ),
        (
            "Loop",
            e(ExprKind::Loop {
                label: None,
                body: Box::new(e(ExprKind::Break {
                    label: None,
                    value: None,
                })),
            }),
        ),
        (
            "Range",
            e(ExprKind::Range {
                start: blit(),
                end: blit(),
                inclusive: false,
            }),
        ),
        (
            "Module",
            e(ExprKind::Module {
                name: "m".to_string(),
                body: blit(),
            }),
        ),
        (
            "ModuleDeclaration",
            e(ExprKind::ModuleDeclaration {
                name: "m".to_string(),
            }),
        ),
        (
            "Break",
            e(ExprKind::Break {
                label: None,
                value: None,
            }),
        ),
        ("Continue", e(ExprKind::Continue { label: None })),
        ("Return", e(ExprKind::Return { value: None })),
        (
            "Assign",
            e(ExprKind::Assign {
                target: bident(),
                value: blit(),
            }),
        ),
        (
            "CompoundAssign",
            e(ExprKind::CompoundAssign {
                target: bident(),
                op: BinaryOp::Add,
                value: blit(),
            }),
        ),
        (
            "PreIncrement",
            e(ExprKind::PreIncrement { target: bident() }),
        ),
        (
            "PostIncrement",
            e(ExprKind::PostIncrement { target: bident() }),
        ),
        (
            "PreDecrement",
            e(ExprKind::PreDecrement { target: bident() }),
        ),
        (
            "PostDecrement",
            e(ExprKind::PostDecrement { target: bident() }),
        ),
        (
            "Extension",
            e(ExprKind::Extension {
                target_type: "S".to_string(),
                methods: vec![],
            }),
        ),
        (
            "Import",
            e(ExprKind::Import {
                module: "std".to_string(),
                items: None,
            }),
        ),
        (
            "ImportAll",
            e(ExprKind::ImportAll {
                module: "std".to_string(),
                alias: "s".to_string(),
            }),
        ),
        (
            "ImportDefault",
            e(ExprKind::ImportDefault {
                module: "std".to_string(),
                name: "s".to_string(),
            }),
        ),
        (
            "Export",
            e(ExprKind::Export {
                expr: blit(),
                is_default: false,
            }),
        ),
        ("ExportList", e(ExprKind::ExportList { names: vec![] })),
        (
            "ReExport",
            e(ExprKind::ReExport {
                items: vec![],
                module: "std".to_string(),
            }),
        ),
        ("ExportDefault", e(ExprKind::ExportDefault { expr: blit() })),
        (
            "TypeAlias",
            e(ExprKind::TypeAlias {
                name: "A".to_string(),
                target_type: ty(),
            }),
        ),
        (
            "MacroInvocation",
            e(ExprKind::MacroInvocation {
                name: "println".to_string(),
                args: vec![],
            }),
        ),
        (
            "VecRepeat",
            e(ExprKind::VecRepeat {
                value: blit(),
                count: blit(),
            }),
        ),
        ("Yield", e(ExprKind::Yield { value: None })),
        (
            "Signal",
            e(ExprKind::Signal {
                initial_value: blit(),
            }),
        ),
        ("InfraBlock", e(ExprKind::InfraBlock { body: vec![] })),
    ]
}

/// Compile-time exhaustiveness guard: no wildcard arm.
///
/// Adding a variant to `ExprKind` breaks this build until `variants()` above
/// grows a sample for it.
fn all_variants_covered(kind: &ExprKind) {
    match kind {
        ExprKind::Literal(..)
        | ExprKind::Identifier(..)
        | ExprKind::QualifiedName { .. }
        | ExprKind::StringInterpolation { .. }
        | ExprKind::Binary { .. }
        | ExprKind::Unary { .. }
        | ExprKind::Throw { .. }
        | ExprKind::TryCatch { .. }
        | ExprKind::Ok { .. }
        | ExprKind::Err { .. }
        | ExprKind::Some { .. }
        | ExprKind::None
        | ExprKind::TypeCast { .. }
        | ExprKind::Ternary { .. }
        | ExprKind::Try { .. }
        | ExprKind::Await { .. }
        | ExprKind::Spawn { .. }
        | ExprKind::AsyncBlock { .. }
        | ExprKind::Lazy { .. }
        | ExprKind::If { .. }
        | ExprKind::IfLet { .. }
        | ExprKind::Let { .. }
        | ExprKind::LetPattern { .. }
        | ExprKind::Function { .. }
        | ExprKind::Lambda { .. }
        | ExprKind::AsyncLambda { .. }
        | ExprKind::Struct { .. }
        | ExprKind::TupleStruct { .. }
        | ExprKind::Class { .. }
        | ExprKind::Enum { .. }
        | ExprKind::StructLiteral { .. }
        | ExprKind::ObjectLiteral { .. }
        | ExprKind::FieldAccess { .. }
        | ExprKind::OptionalFieldAccess { .. }
        | ExprKind::IndexAccess { .. }
        | ExprKind::Slice { .. }
        | ExprKind::Trait { .. }
        | ExprKind::Impl { .. }
        | ExprKind::Actor { .. }
        | ExprKind::Effect { .. }
        | ExprKind::Handle { .. }
        | ExprKind::Send { .. }
        | ExprKind::Command { .. }
        | ExprKind::Ask { .. }
        | ExprKind::ActorSend { .. }
        | ExprKind::ActorQuery { .. }
        | ExprKind::Call { .. }
        | ExprKind::Macro { .. }
        | ExprKind::MethodCall { .. }
        | ExprKind::OptionalMethodCall { .. }
        | ExprKind::Block(..)
        | ExprKind::Pipeline { .. }
        | ExprKind::Match { .. }
        | ExprKind::List(..)
        | ExprKind::Set(..)
        | ExprKind::ArrayInit { .. }
        | ExprKind::Tuple(..)
        | ExprKind::Spread { .. }
        | ExprKind::ListComprehension { .. }
        | ExprKind::SetComprehension { .. }
        | ExprKind::DictComprehension { .. }
        | ExprKind::DataFrame { .. }
        | ExprKind::DataFrameOperation { .. }
        | ExprKind::For { .. }
        | ExprKind::While { .. }
        | ExprKind::WhileLet { .. }
        | ExprKind::Loop { .. }
        | ExprKind::Range { .. }
        | ExprKind::Module { .. }
        | ExprKind::ModuleDeclaration { .. }
        | ExprKind::Break { .. }
        | ExprKind::Continue { .. }
        | ExprKind::Return { .. }
        | ExprKind::Assign { .. }
        | ExprKind::CompoundAssign { .. }
        | ExprKind::PreIncrement { .. }
        | ExprKind::PostIncrement { .. }
        | ExprKind::PreDecrement { .. }
        | ExprKind::PostDecrement { .. }
        | ExprKind::Extension { .. }
        | ExprKind::Import { .. }
        | ExprKind::ImportAll { .. }
        | ExprKind::ImportDefault { .. }
        | ExprKind::Export { .. }
        | ExprKind::ExportList { .. }
        | ExprKind::ReExport { .. }
        | ExprKind::ExportDefault { .. }
        | ExprKind::TypeAlias { .. }
        | ExprKind::MacroInvocation { .. }
        | ExprKind::VecRepeat { .. }
        | ExprKind::Yield { .. }
        | ExprKind::Signal { .. }
        | ExprKind::InfraBlock { .. } => (),
    }
}

// ---------------------------------------------------------------------------
// Outcome helpers.
// ---------------------------------------------------------------------------

/// `Ok(rendered)` when the call returned (`Ok` or `Err`), `Err(())` when it panicked.
type Outcome = Result<Result<String, String>, ()>;

fn silently<T>(body: impl FnOnce() -> T) -> T {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = body();
    std::panic::set_hook(previous);
    out
}

fn transpile_outcome(expr: &Expr) -> Outcome {
    silently(|| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Transpiler::new()
                .transpile_expr(expr)
                .map(|tokens| tokens.to_string())
                .map_err(|err| err.to_string())
        }))
    })
    .map_err(|_| ())
}

/// Evaluate `variants()[index]` on a fresh `Interpreter` in a worker thread.
///
/// The worker isolates two distinct failures of totality: a panic (the sender
/// is dropped, so the channel disconnects) and divergence (no result inside the
/// timeout). Both are `⊥`.
fn eval_outcome(index: usize) -> Result<Result<String, String>, &'static str> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rendered = silently(|| {
            let (_, expr) = variants().swap_remove(index);
            // `Display`, never `Debug`: `Value::Closure` holds an `Rc` to the
            // environment that binds it, so the derived `Debug` recurses
            // forever on `fun f() { 1 }` and overflows the stack. `Display`
            // renders a closure as `<function>`.
            Interpreter::new()
                .eval_expr(&expr)
                .map(|value| format!("{value}"))
                .map_err(|err| format!("{err}"))
        });
        drop(tx.send(rendered));
    });
    match rx.recv_timeout(Duration::from_secs(30)) {
        Ok(rendered) => Ok(rendered),
        Err(mpsc::RecvTimeoutError::Timeout) => Err("diverged (no result in 30s)"),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err("panicked"),
    }
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[test]
fn test_pmat_097_variants_exhaustive_over_exprkind() {
    let samples = variants();
    for (name, expr) in &samples {
        all_variants_covered(&expr.kind);
        assert!(!name.is_empty(), "variant sample must be named");
    }
    let mut names: Vec<&str> = samples.iter().map(|(name, _)| *name).collect();
    names.sort_unstable();
    let count = names.len();
    names.dedup();
    assert_eq!(names.len(), count, "duplicate variant sample in variants()");
    assert!(
        count >= 93,
        "ExprKind has 93 variants; variants() covers only {count}"
    );
}

#[test]
fn test_pmat_097_transpile_dispatch_totality_all_variants() {
    let mut bottom: Vec<&str> = Vec::new();
    for (name, expr) in variants() {
        if transpile_outcome(&expr).is_err() {
            bottom.push(name);
        }
    }
    assert!(
        bottom.is_empty(),
        "transpile_expr dispatch_totality violated (panicked) for variants: {bottom:?}"
    );
}

#[test]
fn test_pmat_097_transpile_dispatch_determinism_all_variants() {
    let mut divergent: Vec<&str> = Vec::new();
    for (name, expr) in variants() {
        let first = transpile_outcome(&expr);
        let second = transpile_outcome(&expr);
        if first != second {
            divergent.push(name);
        }
    }
    assert!(
        divergent.is_empty(),
        "transpile_expr dispatch_determinism violated for variants: {divergent:?}"
    );
}

#[test]
fn test_pmat_097_eval_dispatch_totality_all_variants() {
    let mut bottom: Vec<String> = Vec::new();
    for (index, (name, _)) in variants().into_iter().enumerate() {
        if let Err(why) = eval_outcome(index) {
            bottom.push(format!("{name}: {why}"));
        }
    }
    assert!(
        bottom.is_empty(),
        "eval_expr dispatch_totality violated for variants: {bottom:?}"
    );
}

#[test]
fn test_pmat_097_eval_dispatch_determinism_all_variants() {
    let mut divergent: Vec<&str> = Vec::new();
    for (index, (name, _)) in variants().into_iter().enumerate() {
        let first = eval_outcome(index);
        let second = eval_outcome(index);
        if first.is_ok() && second.is_ok() && first != second {
            divergent.push(name);
        }
    }
    assert!(
        divergent.is_empty(),
        "eval_expr dispatch_determinism violated for variants: {divergent:?}"
    );
}
