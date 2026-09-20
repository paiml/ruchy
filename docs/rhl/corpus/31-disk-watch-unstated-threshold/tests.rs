//! Task 31 — disk watch with no stated threshold; must refuse.

#[test]
fn test_task_31_refuses_for_missing_threshold() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_disk_free("/var/lib/docker", 1_000_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}

#[test]
fn test_task_31_refuses_even_with_plenty_of_space() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_disk_free("/var/lib/docker", 900_000_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Ambiguous { .. })));
}
