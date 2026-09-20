//! Task 30 — report every declared repo's default branch, joined by commas.

#[test]
fn test_task_30_joins_branches_in_declared_order() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 1, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 1, default_branch: "trunk".into() });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("branch_roster"), Some(&"main,trunk".to_string()));
}

#[test]
fn test_task_30_empty_string_when_no_repos_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("branch_roster"), Some(&String::new()));
}
