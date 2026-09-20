//! Task 01 — disk watch on a named path, fixed threshold.

fn ctx_with_free_bytes(bytes: u64) -> TaskCtx {
    TaskCtx::new(1_700_000_000).with_disk_free("/var/lib/docker", bytes)
}

#[test]
fn test_task_01_files_ticket_when_below_threshold() {
    let mut ctx = ctx_with_free_bytes(4_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/fleet-ops");
    assert!(report.tickets[0].labels.contains(&"disk".to_string()));
}

#[test]
fn test_task_01_no_ticket_when_above_threshold() {
    let mut ctx = ctx_with_free_bytes(400_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
