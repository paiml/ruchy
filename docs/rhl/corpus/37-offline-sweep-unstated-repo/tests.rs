//! Task 37 — offline-host report with no stated destination; must refuse.

#[test]
fn test_task_37_refuses_with_offline_hosts_present() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: false });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_37_refuses_with_no_hosts_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
