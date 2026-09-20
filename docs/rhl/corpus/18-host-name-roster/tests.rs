//! Task 18 — report the names of every declared host, joined by commas.

fn ctx_with_hosts() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: true })
}

#[test]
fn test_task_18_joins_names_in_declared_order() {
    let mut ctx = ctx_with_hosts();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("roster"), Some(&"runner-01,runner-02".to_string()));
}

#[test]
fn test_task_18_empty_string_when_no_hosts_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("roster"), Some(&String::new()));
}
