//! Task 22 — report how many repos are declared.

#[test]
fn test_task_22_reports_repo_count() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 1, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 1, default_branch: "main".into() });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("repo_count"), Some(&"2".to_string()));
}

#[test]
fn test_task_22_zero_when_no_repos_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("repo_count"), Some(&"0".to_string()));
}
