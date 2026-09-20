//! Task 07 — stale-PR sweep on one repo, fixed threshold.

fn ctx_with_open_prs(count: u32) -> TaskCtx {
    TaskCtx::new(1_700_000_000).with_repo(RepoRecord {
        name: "paiml/core".into(),
        open_prs: count,
        default_branch: "main".into(),
    })
}

#[test]
fn test_task_07_files_ticket_above_threshold() {
    let mut ctx = ctx_with_open_prs(21);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert!(report.tickets[0].labels.contains(&"stale-pr".to_string()));
}

#[test]
fn test_task_07_no_ticket_at_or_below_threshold() {
    let mut ctx = ctx_with_open_prs(20);
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
