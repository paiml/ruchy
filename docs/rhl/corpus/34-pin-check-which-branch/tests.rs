//! Task 34 — pin check with no named repo or branch; must refuse.

#[test]
fn test_task_34_refuses_for_missing_repo_and_branch() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_repo(RepoRecord {
        name: "paiml/core".into(),
        open_prs: 1,
        default_branch: "develop".into(),
    });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_34_refuses_with_multiple_repos_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 1, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 1, default_branch: "trunk".into() });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
