//! Task 23 — report how many hosts are declared.

#[test]
fn test_task_23_reports_host_count() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: true })
        .with_host(HostRecord { name: "runner-03".into(), cpu_pct: 5.0, online: false });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("host_count"), Some(&"3".to_string()));
}

#[test]
fn test_task_23_zero_when_no_hosts_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("host_count"), Some(&"0".to_string()));
}
