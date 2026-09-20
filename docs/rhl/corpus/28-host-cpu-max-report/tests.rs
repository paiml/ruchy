//! Task 28 — report the highest CPU usage among all declared hosts.

#[test]
fn test_task_28_reports_max_cpu() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 12.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 88.0, online: true });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("max_cpu_pct"), Some(&"88".to_string()));
}

#[test]
fn test_task_28_zero_when_no_hosts_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("max_cpu_pct"), Some(&"0".to_string()));
}
