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

fn finding_at(offset: usize) -> Finding {
    Finding {
        offset,
        severity: "warning".to_string(),
        rule: "r".to_string(),
        message: "m".to_string(),
        suggestion: "s".to_string(),
        issue_type: "t".to_string(),
        name: "n".to_string(),
    }
}

#[test]
fn resolve_turns_a_byte_offset_into_line_and_column() {
    let src = "ab\ncd\n  ef";
    let issue = finding_at(src.find("ef").unwrap()).resolve(src);
    assert_eq!((issue.line, issue.column), (3, 3));
    assert_eq!(issue.rule, "r");
    assert_eq!(issue.message, "m");
    assert_eq!(issue.suggestion, "s");
    assert_eq!(issue.issue_type, "t");
    assert_eq!(issue.name, "n");
    assert_eq!(issue.severity, "warning");
}

#[test]
fn every_finding_kind_reports_a_column_of_at_least_one() {
    // unused variable, undefined, shadowing, unused loop var, match binding, lambda param
    let src = "let a = 1\nlet outer = 1\n\n\n\nfun f() {\n    let outer = 2\n    outer\n}\nf()\nprintln(missing)\nfor i in [1] { println(\"x\") }\nmatch a {\n    n => println(\"m\"),\n}\nlet g = |p| 1\ng(1)\n";
    let all = findings(src);
    assert!(all.len() >= 5, "{all:?}");
    for i in &all {
        assert!(i.line >= 1 && i.column >= 1, "{i:?}");
        assert!(
            i.line <= src.lines().count(),
            "offset leaked into line: {i:?}"
        );
    }
}
