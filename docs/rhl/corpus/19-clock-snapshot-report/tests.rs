//! Task 19 — report the current mocked clock time.

#[test]
fn test_task_19_reports_now_unix() {
    let mut ctx = TaskCtx::new(1_700_000_123);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("now_unix"), Some(&"1700000123".to_string()));
}

#[test]
fn test_task_19_reports_zero_clock() {
    let mut ctx = TaskCtx::new(0);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("now_unix"), Some(&"0".to_string()));
}
