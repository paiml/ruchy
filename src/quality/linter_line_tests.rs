// Real source positions in lint findings (child of #112).
use super::*;
use crate::frontend::parser::Parser;

fn findings(src: &str) -> Vec<LintIssue> {
    let ast = Parser::new(src).parse().expect("parse");
    Linter::new().lint(&ast, src).expect("lint")
}

fn position_of(src: &str, rule: &str) -> (usize, usize) {
    let all = findings(src);
    let hits: Vec<_> = all.iter().filter(|i| i.rule == rule).collect();
    assert_eq!(hits.len(), 1, "expected one {rule} finding, got {all:?}");
    (hits[0].line, hits[0].column)
}

#[test]
fn unused_variable_reports_its_let_line() {
    let src = "println(\"a\")\nprintln(\"b\")\n\n\nlet unused_v = 5\n";
    assert_eq!(position_of(src, "unused_variable"), (5, 1));
}

#[test]
fn unused_variable_reports_column_of_indented_let() {
    let src = "fun f() {\n    let skipped = 1\n    2\n}\nf()\n";
    assert_eq!(position_of(src, "unused_variable"), (2, 5));
}

#[test]
fn unused_loop_variable_reports_its_for_line() {
    let src = "println(\"a\")\n\n\nfor i in [1, 2] {\n    println(\"x\")\n}\n";
    assert_eq!(position_of(src, "unused_loop_variable"), (4, 1));
}

#[test]
fn unused_match_binding_reports_its_arm() {
    let src = "let x = 1\n\nmatch x {\n    n => println(\"m\"),\n}\n";
    assert_eq!(position_of(src, "unused_match_binding"), (4, 5));
}

#[test]
fn undefined_variable_reports_its_use() {
    let src = "println(\"a\")\n\n\n\nprintln(missing_name)\n";
    assert_eq!(position_of(src, "undefined"), (5, 9));
}

#[test]
fn shadowing_reports_the_inner_let() {
    let src = "let outer = 1\n\nfun f() {\n    let outer = 2\n    outer\n}\nf()\nprintln(outer)\n";
    assert_eq!(position_of(src, "shadowing"), (4, 5));
}

#[test]
fn unused_parameter_reports_its_lambda_line() {
    let src = "let a = 1\n\n\nlet g = |p, q| p + 1\ng(a)\n";
    assert_eq!(position_of(src, "unused_parameter"), (4, 9));
}

#[test]
fn json_carries_real_line_and_column() {
    let src = "println(\"a\")\n\n\nprintln(missing_name)\n";
    let json = serde_json::to_value(&findings(src)[0]).expect("json");
    assert_eq!(json["line"], 4);
    assert_eq!(json["column"], 9);
}

#[test]
fn position_counts_columns_in_chars_not_bytes() {
    let source = "é\nxé = 1\nab";
    // offset of "=": line 2, after "xé " (3 chars, 4 bytes)
    assert_eq!(line_and_column(source, "é\nxé ".len() + 1), (2, 5));
    assert_eq!(line_and_column(source, 0), (1, 1));
    assert_eq!(line_and_column(source, "é\n".len()), (2, 1));
    // out-of-range offsets clamp to end of source instead of panicking
    assert_eq!(line_and_column(source, 10_000), (3, 3));
}

#[test]
fn linter_stays_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Linter>();
}

fn params_of(src: &str) -> Vec<(String, usize, usize)> {
    findings(src)
        .into_iter()
        .filter(|i| i.rule == "unused_parameter")
        .map(|i| (i.name, i.line, i.column))
        .collect()
}

#[test]
fn named_fun_reports_its_unused_parameter() {
    let src = "fun add(a, b) {\n    a\n}\nadd(1, 2)\n";
    assert_eq!(params_of(src), vec![("b".to_string(), 1, 12)]);
}

#[test]
fn named_fun_unused_parameter_on_a_later_line() {
    let src = "println(\"x\")\n\nfun add(a, b) {\n    b\n}\nadd(1, 2)\n";
    assert_eq!(params_of(src), vec![("a".to_string(), 3, 9)]);
}

#[test]
fn impl_method_reports_unused_parameter_not_self() {
    let src = "impl Foo {\n    fun go(self, unused_m, used_m) {\n        used_m\n    }\n}\n";
    assert_eq!(params_of(src), vec![("unused_m".to_string(), 2, 18)]);
}

#[test]
fn destructured_parameter_reports_unused_binding() {
    let src = "fun d((m, n), k) {\n    m + k\n}\nd((1, 2), 3)\n";
    assert_eq!(params_of(src), vec![("n".to_string(), 1, 7)]);
}

#[test]
fn underscore_prefixed_parameters_stay_silent() {
    let src =
        "fun skip(_ignored, _x, used) {\n    used\n}\nskip(1, 2, 3)\nlet g = |_a, b| b\ng(1, 2)\n";
    assert!(params_of(src).is_empty(), "{:?}", findings(src));
}

#[test]
fn parameter_used_only_in_nested_constructs_counts_as_used() {
    let src = "fun f(p, q, r, s, t) {\n    let h = |z| p + z\n    for i in [1] {\n        println(q + i)\n    }\n    match 1 {\n        n => println(r + n),\n    }\n    if true {\n        println(s)\n    } else {\n        println(t)\n    }\n    h(1)\n}\nf(1, 2, 3, 4, 5)\n";
    assert!(params_of(src).is_empty(), "{:?}", findings(src));
}

#[test]
fn parameter_rule_follows_the_rules_filter() {
    let src = "fun add(a, b) {\n    a\n}\nadd(1, 2)\n";
    let ast = Parser::new(src).parse().expect("parse");
    let mut linter = Linter::new();
    linter.set_rules("complexity");
    let all = linter.lint(&ast, src).expect("lint");
    assert!(all.iter().all(|i| i.rule != "unused_parameter"), "{all:?}");
    linter.set_rules("unused");
    let all = linter.lint(&ast, src).expect("lint");
    assert!(all.iter().any(|i| i.rule == "unused_parameter"), "{all:?}");
}

#[test]
fn unused_function_parameter_json_has_rule_line_and_column() {
    let src = "fun add(a, b) {\n    a\n}\nadd(1, 2)\n";
    let issue = findings(src)
        .into_iter()
        .find(|i| i.rule == "unused_parameter")
        .expect("finding");
    let json = serde_json::to_value(&issue).expect("json");
    assert_eq!(json["rule"], "unused_parameter");
    assert_eq!(json["line"], 1);
    assert_eq!(json["column"], 12);
    assert_eq!(json["name"], "b");
}

#[test]
fn outer_variable_used_inside_nested_scopes_is_not_reported() {
    let src = "let a = 1\nlet b = 2\nfor i in [1] {\n    println(a + i)\n}\nlet h = |z| b + z\nprintln(h(1))\n";
    assert!(
        findings(src).iter().all(|i| i.rule != "unused_variable"),
        "{:?}",
        findings(src)
    );
}

#[test]
fn parameter_used_in_object_literal_or_try_is_not_reported() {
    let src = "fun mk(x, y) {\n    { x: x, y: y }\n}\nfun conv(s) {\n    try {\n        Ok(s.to_int())\n    } catch e {\n        Err(e)\n    }\n}\nmk(1, 2)\nconv(\"1\")\n";
    assert!(params_of(src).is_empty(), "{:?}", findings(src));
}

#[test]
fn method_bodies_only_contribute_unused_parameter_findings() {
    let src = "impl Foo {\n    fun go(self, a) {\n        let dy = 1\n        missing_name + self.x\n    }\n}\n";
    let all = findings(src);
    assert_eq!(all.len(), 1, "{all:?}");
    assert_eq!(
        (all[0].rule.as_str(), all[0].name.as_str()),
        ("unused_parameter", "a")
    );
    assert_eq!((all[0].line, all[0].column), (2, 18));
}
