//! Task 05 — pin check on one repo's default branch.

fn ctx_with_branch(branch: &str) -> TaskCtx {
    TaskCtx::new(1_700_000_000).with_repo(RepoRecord {
        name: "paiml/core".into(),
        open_prs: 3,
        default_branch: branch.into(),
    })
}

#[test]
fn test_task_05_files_ticket_when_branch_wrong() {
    let mut ctx = ctx_with_branch("develop");
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/core");
    assert!(report.tickets[0].labels.contains(&"ci".to_string()));
}

#[test]
fn test_task_05_no_ticket_when_branch_correct() {
    let mut ctx = ctx_with_branch("main");
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
