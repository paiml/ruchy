//! Task 11 — report which declared path has the least free disk space.

fn ctx_with_paths() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_disk_free("/var/lib/docker", 5_000_000_000)
        .with_disk_free("/var/log", 1_000_000_000)
}

#[test]
fn test_task_11_reports_lowest_path() {
    let mut ctx = ctx_with_paths();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("lowest_path"), Some(&"/var/log".to_string()));
}

#[test]
fn test_task_11_single_path_is_its_own_lowest() {
    let mut ctx = TaskCtx::new(1_700_000_000).with_disk_free("/var/lib/docker", 5_000_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("lowest_path"), Some(&"/var/lib/docker".to_string()));
}
