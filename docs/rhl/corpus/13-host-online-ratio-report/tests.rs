//! Task 13 — report the fraction of declared hosts that are online.

fn ctx_with_hosts() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: false })
}

#[test]
fn test_task_13_reports_half_online() {
    let mut ctx = ctx_with_hosts();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("online_ratio"), Some(&"0.50".to_string()));
}

#[test]
fn test_task_13_reports_zero_when_no_hosts_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("online_ratio"), Some(&"0.00".to_string()));
}
