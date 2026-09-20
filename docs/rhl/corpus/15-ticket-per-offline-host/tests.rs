//! Task 15 — file one ticket per offline host.

fn ctx_with_hosts() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: false })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: false })
        .with_host(HostRecord { name: "runner-03".into(), cpu_pct: 5.0, online: true })
}

#[test]
fn test_task_15_one_ticket_per_offline_host() {
    let mut ctx = ctx_with_hosts();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 2);
    for ticket in &report.tickets {
        assert_eq!(ticket.repo, "paiml/fleet-ops");
        assert!(ticket.labels.contains(&"fleet-down".to_string()));
    }
}

#[test]
fn test_task_15_no_tickets_when_all_online() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true });
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
