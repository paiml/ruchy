//! Task 16 — file one ticket per declared path below a fixed threshold.

fn ctx_with_paths() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 1_000_000_000)
        .with_disk_free("/var/log", 2_000_000_000)
        .with_disk_free("/home", 900_000_000_000)
}

#[test]
fn test_task_16_one_ticket_per_low_path() {
    let mut ctx = ctx_with_paths();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 2);
    for ticket in &report.tickets {
        assert_eq!(ticket.repo, "paiml/fleet-ops");
        assert!(ticket.labels.contains(&"disk".to_string()));
    }
}

#[test]
fn test_task_16_no_tickets_when_all_paths_healthy() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_disk_free("/home", 900_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
