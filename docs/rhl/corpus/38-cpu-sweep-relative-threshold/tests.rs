//! Task 38 — CPU sweep with an undefined relative threshold; must refuse.

#[test]
fn test_task_38_refuses_with_one_outlier_host() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 10.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 95.0, online: true });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_38_refuses_with_uniform_cpu_across_hosts() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 50.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 50.0, online: true });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
