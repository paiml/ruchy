//! Task 14 — report total open pull requests summed across every declared repo.

fn ctx_with_repos() -> TaskCtx {
    TaskCtx::new(1_700_000_000)
        .with_repo(RepoRecord { name: "paiml/core".into(), open_prs: 12, default_branch: "main".into() })
        .with_repo(RepoRecord { name: "paiml/fleet-ops".into(), open_prs: 3, default_branch: "main".into() })
}

#[test]
fn test_task_14_reports_summed_open_prs() {
    let mut ctx = ctx_with_repos();
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("open_pr_total"), Some(&"15".to_string()));
}

#[test]
fn test_task_14_zero_when_no_repos_declared() {
    let mut ctx = TaskCtx::new(1_700_000_000);
    let report = run(&mut ctx).expect("task should not refuse");
    assert_eq!(report.values.get("open_pr_total"), Some(&"0".to_string()));
}
