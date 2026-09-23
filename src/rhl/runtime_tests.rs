//! RHL-4b runtime tests: every runtime function is found by its binding,
//! transpiles through ruchy and compiles with `rustc`, and runs against FAKE
//! `df`, `du` and `gh` that are the only programs on PATH, so no test can
//! reach the real `gh`.

use super::{binding_function, parse_binding, UNBOUND};
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

const BINDINGS: &[&str] = &[
    "fleet::disk_free_of",
    "fleet::disk_usage_of",
    "tickets::tickets_filed_in",
    "tickets::file_ticket",
    "tickets::label_ticket",
];

/// A fake program: echoes `out` and appends its name and argv to `log`.
fn fake(dir: &Path, name: &str, out: &str, log: &Path) {
    let script = format!(
        "#!/bin/sh\necho \"{name} $*\" >> '{}'\nprintf '%s\\n' '{out}'\n",
        log.display()
    );
    let path = dir.join(name);
    std::fs::write(&path, script).expect("write fake");
    let mut perm = std::fs::metadata(&path).expect("meta").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
    std::fs::set_permissions(&path, perm).expect("chmod");
}

fn rustc_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Build `functions` + `main` with ruchy's transpiler and `rustc`, run it
/// with PATH = the fake directory only, and return (stdout, log).
fn run_with_fakes(functions: &str, main: &str) -> (String, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).expect("mkdir bin");
    let log = dir.path().join("argv.log");
    fake(&bin, "df", "Avail\n90000000000", &log);
    fake(&bin, "du", "4096\t/var", &log);
    fake(&bin, "gh", "[{\"number\":1},{\"number\":2}]", &log);
    let rust = crate::rhl::lower::to_rust(&format!("{functions}\n{main}")).expect("transpiles");
    let src = dir.path().join("j.rs");
    std::fs::write(&src, rust).expect("write");
    let exe = dir.path().join("j");
    let built = Command::new("rustc")
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg(&src)
        .output()
        .expect("rustc runs");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let out = Command::new(&exe)
        .env_clear()
        .env("PATH", &bin)
        .output()
        .expect("runs");
    // The fake directory is the whole PATH: the only `gh` the program can see.
    assert!(bin.join("gh").is_file());
    let log_text = std::fs::read_to_string(&log).unwrap_or_default();
    (String::from_utf8_lossy(&out.stdout).to_string(), log_text)
}

fn all_functions() -> String {
    BINDINGS
        .iter()
        .map(|b| binding_function(&root(), b).unwrap_or_else(|e| panic!("{e}")))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_rhl4b_runtime_every_binding_names_one_function() {
    for b in BINDINGS {
        let f = binding_function(&root(), b).unwrap_or_else(|e| panic!("{e}"));
        let name = parse_binding(b).expect("well formed").1;
        assert!(f.starts_with(&format!("fun {name}(")), "{f}");
        assert!(f.ends_with("}\n"), "{f}");
    }
}

#[test]
fn test_rhl4b_runtime_refuses_unbound_malformed_and_missing() {
    assert!(binding_function(&root(), UNBOUND).is_err());
    assert!(parse_binding("../x::y").is_none());
    assert!(parse_binding("fleet::Disk").is_none());
    assert!(binding_function(&root(), "fleet::runner_load_of").is_err());
    assert!(binding_function(&root(), "nothere::f").is_err());
}

#[test]
fn test_rhl4b_runtime_measures_read_the_fakes_and_nothing_else() {
    if !rustc_available() {
        return;
    }
    let main = "fun main() {\n    println!(\"{} {} {}\", disk_free_of(\"/\"), \
                disk_usage_of(\"/var\"), tickets_filed_in(\"paiml/infra\"))\n}\n";
    let (out, log) = run_with_fakes(&all_functions(), main);
    assert_eq!(out.trim(), "90000000000 4096 2");
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(
        lines,
        [
            "df -B1 --output=avail /",
            "du -sb /var",
            "gh issue list --repo paiml/infra --state open --json number --limit 1000",
        ]
    );
}

#[test]
fn test_rhl4b_runtime_file_ticket_calls_gh_issue_create_once() {
    if !rustc_available() {
        return;
    }
    let main = "fun main() {\n    let ok = file_ticket(\"paiml/infra\", \"gx10 disk watch\", \"fleet\")\n    \
                println!(\"{}\", ok)\n}\n";
    let (out, log) = run_with_fakes(&all_functions(), main);
    assert_eq!(out.lines().last(), Some("true"), "{out}");
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        ["gh issue create --repo paiml/infra --title gx10 disk watch --label fleet"]
    );
}
