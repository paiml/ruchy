//! Task 35 — disk watch with an unresolved pronoun; must refuse.

#[test]
fn test_task_35_refuses_with_one_host_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_disk_free("/var/lib/docker", 1_000_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_35_refuses_with_multiple_hosts_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: true });
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
