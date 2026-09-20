//! Task 08 — stale-PR sweep across every declared repo.

fn ctx_with_repos() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 25, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 3, default_branch: "main".into() })
}

#[test]
fn test_task_08_one_ticket_per_repo_over_threshold() {
    let mut ctx = ctx_with_repos();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/core");
    assert!(report.tickets[0].labels.contains(&"stale-pr".to_string()));
}

#[test]
fn test_task_08_no_tickets_when_all_under_threshold() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 2, default_branch: "main".into() });
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
