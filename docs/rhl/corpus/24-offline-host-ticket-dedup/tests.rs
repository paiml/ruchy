//! Task 24 — file exactly one ticket listing every offline host.

fn ctx_with_hosts() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: false })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: false })
        .with_host(HostRecord { name: "runner-03".into(), cpu_pct: 5.0, online: true })
}

#[test]
fn test_task_24_single_ticket_names_both_hosts() {
    let mut ctx = ctx_with_hosts();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/fleet-ops");
    assert!(report.tickets[0].title.contains("runner-01"));
    assert!(report.tickets[0].title.contains("runner-02"));
    assert!(report.tickets[0].labels.contains(&"fleet-down".to_string()));
}

#[test]
fn test_task_24_no_ticket_when_all_online() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true });
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
