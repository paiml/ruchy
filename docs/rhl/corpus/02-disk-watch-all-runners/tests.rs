//! Task 02 — disk watch on a named path, every host, fixed threshold.

fn base_ctx() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 2_000_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 10.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 20.0, online: true })
}

#[test]
fn test_task_02_one_ticket_per_low_host() {
    let mut ctx = base_ctx();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 2);
    for ticket in &report.tickets {
        assert_eq!(ticket.repo, "paiml/fleet-ops");
        assert!(ticket.labels.contains(&"disk".to_string()));
    }
}

#[test]
fn test_task_02_no_tickets_when_all_hosts_have_space() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 900_000_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 10.0, online: true });
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
