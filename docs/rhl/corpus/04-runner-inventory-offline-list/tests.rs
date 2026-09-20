//! Task 04 — list the names of every offline host.

fn ctx_with_hosts() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: false })
        .with_host(HostRecord { name: "runner-02".into(), cpu_pct: 5.0, online: true })
        .with_host(HostRecord { name: "runner-03".into(), cpu_pct: 5.0, online: false })
}

#[test]
fn test_task_04_lists_offline_host_names_in_order() {
    let mut ctx = ctx_with_hosts();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("offline"), Some(&"runner-01,runner-03".to_string()));
}

#[test]
fn test_task_04_empty_string_when_all_online() {
    let mut ctx = TaskCtx::new(1_700_000_000)
        .with_host(HostRecord { name: "runner-01".into(), cpu_pct: 5.0, online: true });
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("offline"), Some(&String::new()));
}
