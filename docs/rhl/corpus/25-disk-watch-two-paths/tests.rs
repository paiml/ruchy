//! Task 25 — disk watch on two named paths, fixed threshold.

#[test]
fn test_task_25_tickets_only_the_low_path() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 1_000_000_000)
        .with_disk_free("/var/log", 9_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/fleet-ops");
    assert!(report.tickets[0].labels.contains(&"disk".to_string()));
}

#[test]
fn test_task_25_no_tickets_when_both_healthy() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 9_000_000_000)
        .with_disk_free("/var/log", 9_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
