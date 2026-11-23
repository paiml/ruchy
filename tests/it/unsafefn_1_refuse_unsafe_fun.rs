#![allow(missing_docs)]
//! UNSAFEFN-1 (paiml/ruchy#244, epic #239): an `unsafe` modifier on a Ruchy `fun`
//! must be refused with a diagnostic, never transpiled to `unsafe fn`
//! (zero-unsafe policy, #132).

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fn transpile(src: &str) -> Result<String, String> {
    let ast = Parser::new(src).parse().map_err(|e| e.to_string())?;
    Transpiler::new()
        .transpile_to_program(&ast)
        .map(|t| t.to_string())
        .map_err(|e| e.to_string())
}

#[test]
fn unsafe_fun_is_refused_with_diagnostic() {
    let err = transpile("unsafe fun raw_ptr() { }").expect_err("unsafe fun must be refused");
    assert!(err.contains("unsafe"), "diagnostic names unsafe: {err}");
    assert!(
        err.contains("#132") || err.contains("zero-unsafe"),
        "cites policy: {err}"
    );
}

#[test]
fn plain_fun_still_transpiles_without_unsafe() {
    let code = transpile("fun ok() { }").expect("plain fun transpiles");
    assert!(!code.contains("unsafe"), "no unsafe in output: {code}");
}
