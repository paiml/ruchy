//! Task 17 — report a named repo's declared default branch.

fn ctx_with_branch(branch: &str) -> TaskCtx {
    TaskCtx::new(1_700_000_000).with_repo(RepoRecord {
        name: "paiml/core".into(),
        open_prs: 1,
        default_branch: branch.into(),
    })
}

#[test]
fn test_task_17_reports_declared_branch() {
    let mut ctx = ctx_with_branch("main");
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("default_branch"), Some(&"main".to_string()));
}

#[test]
fn test_task_17_reports_non_main_branch_verbatim() {
    let mut ctx = ctx_with_branch("release-1.0");
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("default_branch"), Some(&"release-1.0".to_string()));
}
