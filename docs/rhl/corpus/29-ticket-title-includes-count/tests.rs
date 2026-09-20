//! Task 29 — file a ticket whose title states the count of low-disk paths.

#[test]
fn test_task_29_title_states_low_path_count() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 1_000_000_000)
        .with_disk_free("/var/log", 2_000_000_000)
        .with_disk_free("/home", 900_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert!(report.tickets[0].title.contains('2'));
    assert!(report.tickets[0].labels.contains(&"disk".to_string()));
}

#[test]
fn test_task_29_no_ticket_when_none_low() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_disk_free("/home", 900_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
