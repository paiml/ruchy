//! Task 06 — pin check across every declared repo.

fn ctx_with_repos() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 1, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 1, default_branch: "master".into() })
}

#[test]
fn test_task_06_one_ticket_per_wrong_branch_repo() {
    let mut ctx = ctx_with_repos();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/fleet-ops");
    assert!(report.tickets[0].labels.contains(&"ci".to_string()));
}

#[test]
fn test_task_06_no_tickets_when_all_repos_correct() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 1, default_branch: "main".into() });
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
