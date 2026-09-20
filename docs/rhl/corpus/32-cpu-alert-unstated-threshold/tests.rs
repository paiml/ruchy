//! Task 32 — CPU alert with no named host and no stated threshold; must refuse.

#[test]
fn test_task_32_refuses_for_missing_host_and_threshold() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 97.0, online: true });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_32_refuses_with_multiple_hosts() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 10.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 99.0, online: true });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
