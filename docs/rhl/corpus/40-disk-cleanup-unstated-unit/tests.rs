//! Task 40 — disk cleanup with an unstated unit; must refuse.

#[test]
fn test_task_40_refuses_for_missing_unit() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_disk_free("/var/lib/docker", 4_000_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_40_refuses_even_with_ample_space() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_disk_free("/var/lib/docker", 900_000_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
