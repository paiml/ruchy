//! Task 09 — CPU alert on one named host, fixed threshold.

fn ctx_with_cpu(pct: f64) -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-03".into(), cpu_pct: pct, online: true })
}

#[test]
fn test_task_09_files_ticket_when_over_threshold() {
    let mut ctx = ctx_with_cpu(95.0);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.tickets.len(), 1);
    assert_eq!(report.tickets[0].repo, "paiml/fleet-ops");
    assert!(report.tickets[0].labels.contains(&"cpu-hot".to_string()));
}

#[test]
fn test_task_09_no_ticket_at_or_below_threshold() {
    let mut ctx = ctx_with_cpu(90.0);
    let report = run(&mut ctx).expect("task should not refuse");
    assert!(report.tickets.is_empty());
}
