//! Task 20 — report free bytes on a named path.

#[test]
fn test_task_20_reports_free_bytes() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_disk_free("/var/lib/docker", 7_500_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("free_bytes"), Some(&"7500000000".to_string()));
}

#[test]
fn test_task_20_refuses_when_path_undeclared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let result = run(&mut ctx);
    assert!(matches!(result, Err(TaskError::Refused { .. })));
}
