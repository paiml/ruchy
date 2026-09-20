//! Task 27 — report a named repo's current open PR count.

#[test]
fn test_task_27_reports_open_pr_count() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_repo(RepoRecord {
        name: "paiml/core".into(),
        open_prs: 17,
        default_branch: "main".into(),
    });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("open_prs"), Some(&"17".to_string()));
}

#[test]
fn test_task_27_refuses_when_repo_undeclared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Refused { .. })));
}
