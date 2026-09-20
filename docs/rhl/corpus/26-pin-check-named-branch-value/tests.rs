//! Task 26 — report whether a named repo's default branch equals main.

fn ctx_with_branch(branch: &str) -> TaskCtx {
    TaskCtx::new(1_700_000_000).with_repo(RepoRecord {
        name: "paiml/core".into(),
        open_prs: 1,
        default_branch: branch.into(),
    })
}

#[test]
fn test_task_26_reports_true_when_main() {
    let mut ctx = ctx_with_branch("main");
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("is_main"), Some(&"true".to_string()));
}

#[test]
fn test_task_26_reports_false_when_not_main() {
    let mut ctx = ctx_with_branch("develop");
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("is_main"), Some(&"false".to_string()));
}
