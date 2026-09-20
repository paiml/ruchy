//! Task 36 — ticket with no named path, host, or label; must refuse.

#[test]
fn test_task_36_refuses_with_one_low_path() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_disk_free("/var/lib/docker", 1_000_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_36_refuses_with_several_paths_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 1_000_000_000)
        .with_disk_free("/var/log", 2_000_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
