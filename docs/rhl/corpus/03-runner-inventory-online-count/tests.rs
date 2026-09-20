//! Task 03 — report how many hosts are online.

fn ctx_with_hosts() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: false })
        .with_host(HostRecord { name: "runner-03".into(), cpu_pct: 5.0, online: true })
}

#[test]
fn test_task_03_reports_online_count() {
    let mut ctx = ctx_with_hosts();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("online_count"), Some(&"2".to_string()));
}

#[test]
fn test_task_03_zero_when_no_hosts_online() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: false });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("online_count"), Some(&"0".to_string()));
}
