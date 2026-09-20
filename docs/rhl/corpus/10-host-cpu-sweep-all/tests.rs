//! Task 10 — CPU sweep across every online host, fixed threshold.

fn ctx_with_hosts() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 95.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 97.0, online: false })
        .with_host(HostRecord { name: "runner-03".into(), cpu_pct: 10.0, online: true })
}

#[test]
fn test_task_10_skips_offline_hosts() {
    let mut ctx = ctx_with_hosts();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/fleet-ops");
}

#[test]
fn test_task_10_no_tickets_when_all_cool() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true });
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
