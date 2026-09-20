//! Task 33 — stale-PR sweep with no stated count or age; must refuse.

#[test]
fn test_task_33_refuses_for_missing_threshold() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_repo(RepoRecord {
        name: "paiml/core".into(),
        open_prs: 200,
        default_branch: "main".into(),
    });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_33_refuses_even_with_no_repos_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
