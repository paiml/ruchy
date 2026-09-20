//! Task 21 — report a named host's current CPU usage.

#[test]
fn test_task_21_reports_cpu_pct() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 42.5, online: true });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("cpu_pct"), Some(&"42.5".to_string()));
}

#[test]
fn test_task_21_refuses_when_host_undeclared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Refused { .. })));
}
