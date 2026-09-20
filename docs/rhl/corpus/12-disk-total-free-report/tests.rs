//! Task 12 — report total free disk space summed across every declared path.

fn ctx_with_paths() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 5_000_000_000)
        .with_disk_free("/var/log", 1_000_000_000)
}

#[test]
fn test_task_12_reports_summed_total() {
    let mut ctx = ctx_with_paths();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("total_free_bytes"), Some(&"6000000000".to_string()));
}

#[test]
fn test_task_12_zero_when_no_paths_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("total_free_bytes"), Some(&"0".to_string()));
}
