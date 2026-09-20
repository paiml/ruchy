//! Task 39 — consistency check with no stated property; must refuse.

#[test]
fn test_task_39_refuses_with_branch_mismatch_present() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 5, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 5, default_branch: "trunk".into() });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_39_refuses_with_all_repos_matching() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 5, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 5, default_branch: "main".into() });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
