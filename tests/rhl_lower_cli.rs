//! RHL-4: `ruchy transpile` and `ruchy compile` on `.rhl` / `.rhl.yaml`
//! files, through the built binary (spec RHL-001 §5: extend, do not collide).
//!
//! The library decides (src/rhl/cli.rs, src/rhl/lower.rs, `cargo test --lib`);
//! this file shows what only a process can: the verb dispatch, the exit
//! codes, and that the Rust of an RHL file is byte for byte what
//! `ruchy transpile` makes of its lowered ruchy.

use assert_cmd::Command;
use predicates::prelude::*;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

const VALID: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";
const TYPO: &str = "docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl";

fn stdout_of(args: &[&str]) -> String {
    let out = ruchy_cmd()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).expect("utf-8")
}

/// A scratch root holding copies of `vocab/` and `contracts/` (files only).
fn scratch_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for sub in ["vocab", "contracts"] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir");
        for e in std::fs::read_dir(sub)
            .expect("read dir")
            .filter_map(Result::ok)
        {
            if e.path().is_file() {
                std::fs::copy(e.path(), dir.path().join(sub).join(e.file_name())).expect("copy");
            }
        }
    }
    dir
}

#[test]
fn test_rhl4_cli_transpile_emit_ruchy_prints_the_lowering() {
    ruchy_cmd()
        .args(["transpile", VALID, "--emit", "ruchy"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "fun decide(facts: Facts) -> Vec<Action>",
        ))
        .stdout(predicate::str::contains("struct Facts {"));
}

#[test]
fn test_rhl4_cli_transpile_default_is_the_rust_of_the_lowered_ruchy() {
    let rust = stdout_of(&["transpile", VALID]);
    assert!(
        rust.contains("fn decide(facts: Facts) -> Vec<Action>"),
        "{rust}"
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let ruchy = dir.path().join("j.ruchy");
    std::fs::write(&ruchy, stdout_of(&["transpile", VALID, "--emit", "ruchy"])).expect("write");
    let via_ruchy = stdout_of(&["transpile", ruchy.to_str().expect("utf-8 path")]);
    assert_eq!(
        rust, via_ruchy,
        "the .rhl path must be `ruchy transpile` of its lowering"
    );
}

#[test]
fn test_rhl4_cli_transpile_writes_output_and_yaml_matches_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("j.rs");
    ruchy_cmd()
        .args(["transpile", VALID, "-o", out.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    let written = std::fs::read_to_string(&out).expect("written");
    let root = scratch_root();
    let yaml = root.path().join("j.rhl.yaml");
    ruchy_cmd()
        .args(["convert", VALID, "-o", yaml.to_str().expect("path")])
        .assert()
        .success();
    assert_eq!(
        stdout_of(&["transpile", yaml.to_str().expect("path")]),
        written
    );
}

#[test]
fn test_rhl4_cli_transpile_declines_a_typo_and_an_unlowerable_construct() {
    ruchy_cmd()
        .args(["transpile", TYPO])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("RHL-V002"));
    let root = scratch_root();
    let file = root.path().join("g.rhl");
    std::fs::write(
        &file,
        "use vocabulary fleet v2\n\njob \"g\"\n  give back 1\nend\n",
    )
    .expect("write");
    ruchy_cmd()
        .args(["transpile", file.to_str().expect("path")])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("RHL-L001"));
}

#[test]
fn test_rhl4_cli_emit_is_for_rhl_files_only() {
    ruchy_cmd()
        .args(["transpile", VALID, "--emit", "c"])
        .assert()
        .code(2);
    let dir = tempfile::tempdir().expect("tempdir");
    let ruchy = dir.path().join("x.ruchy");
    std::fs::write(&ruchy, "fun f() -> i64 { 1 }\n").expect("write");
    ruchy_cmd()
        .args([
            "transpile",
            ruchy.to_str().expect("path"),
            "--emit",
            "ruchy",
        ])
        .assert()
        .failure();
    assert!(Path::new(VALID).is_file());
}

// ---------------------------------------------------------------- RHL-4c: compile and run

const RUNNER_LOAD: &str = "docs/rhl/breaks/v2/valid/03-gx12-runner-load-watch.rhl";
const PLAN_LINE: &str =
    "FileTicket { repo: \"paiml/infra\", title: \"gx10 disk watch\", label: \"fleet\" }";
const GH_CREATE: &str =
    "gh [issue] [create] [--repo] [paiml/infra] [--title] [gx10 disk watch] [--label] [fleet]";

/// A fake world for one test. `fakes` holds `df` and `gh` scripts that log
/// their argv (one `[arg]` per argument) to `log` and print canned output;
/// it is FIRST on `path`. The only other directory on `path` holds links to
/// `rustc`, `cc` and `ld` and nothing else, so no real `df` or `gh` is
/// reachable from the build or from the job.
struct World {
    _dir: tempfile::TempDir,
    path: OsString,
    log: PathBuf,
}

impl World {
    fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }

    fn gh_lines(&self) -> Vec<String> {
        self.log()
            .lines()
            .filter(|l| l.starts_with("gh "))
            .map(str::to_string)
            .collect()
    }

    fn ruchy(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        ruchy_cmd().env("PATH", &self.path).args(args).assert()
    }

    /// Where `/bin/sh` resolves `name` on this world's PATH.
    fn which(&self, name: &str) -> String {
        let out = std::process::Command::new("/bin/sh")
            .args(["-c", &format!("command -v {name}")])
            .env("PATH", &self.path)
            .output()
            .expect("sh runs");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
}

fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

fn real_rustc() -> Option<PathBuf> {
    let out = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()?;
    let sysroot = String::from_utf8(out.stdout).ok()?;
    Some(Path::new(sysroot.trim()).join("bin/rustc")).filter(|p| p.is_file())
}

fn fake(dir: &Path, name: &str, out: &str, log: &Path) {
    let script = format!(
        "#!/bin/sh\n{{ printf '%s' {name}; for a in \"$@\"; do printf ' [%s]' \"$a\"; done; \
         printf '\\n'; }} >> '{}'\nprintf '%s\\n' '{out}'\n",
        log.display()
    );
    let path = dir.join(name);
    std::fs::write(&path, script).expect("write fake");
    let mut perm = std::fs::metadata(&path).expect("meta").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
    std::fs::set_permissions(&path, perm).expect("chmod");
}

/// The world with `df` reporting `free` bytes available; `None` (the test
/// is skipped, and says so) when this host has no rustc, cc or ld to link.
fn world(free: u64) -> Option<World> {
    let tools = [
        ("rustc", real_rustc()),
        ("cc", on_path("cc")),
        ("ld", on_path("ld")),
    ];
    if tools.iter().any(|(_, p)| p.is_none()) {
        println!("RHL-4c: skipped: no rustc, cc or ld to build a job with");
        return None;
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let (fakes, toolchain) = (dir.path().join("fakes"), dir.path().join("toolchain"));
    std::fs::create_dir(&fakes).expect("mkdir fakes");
    std::fs::create_dir(&toolchain).expect("mkdir toolchain");
    for (name, target) in tools {
        std::os::unix::fs::symlink(target.expect("checked"), toolchain.join(name)).expect("link");
    }
    let log = dir.path().join("argv.log");
    fake(&fakes, "df", &format!("Avail\n{free}"), &log);
    fake(
        &fakes,
        "gh",
        "https://github.com/paiml/infra/issues/1",
        &log,
    );
    let path = std::env::join_paths([&fakes, &toolchain]).expect("PATH");
    let w = World {
        _dir: dir,
        path,
        log,
    };
    assert_eq!(w.which("gh"), fakes.join("gh").display().to_string());
    assert_eq!(w.which("df"), fakes.join("df").display().to_string());
    Some(w)
}

fn stdout_text(a: &assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(&a.get_output().stdout).to_string()
}

#[test]
fn test_rhl4c_cli_compile_then_run_is_a_dry_run_that_prints_the_plan() {
    let Some(w) = world(90_000_000_000) else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let bin = dir.path().join("gx10");
    w.ruchy(&["compile", VALID, "-o", bin.to_str().expect("path")])
        .success();
    let out = std::process::Command::new(&bin)
        .env_clear()
        .env("PATH", &w.path)
        .output()
        .expect("the job runs");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("{PLAN_LINE}\n")
    );
    let run = w.ruchy(&["run", VALID]).success();
    assert_eq!(stdout_text(&run), format!("{PLAN_LINE}\n"));
    let df = "df [-B1] [--output=avail] [/]";
    assert_eq!(w.log(), format!("{df}\n{df}\n"), "df read twice, gh never");
    assert!(w.gh_lines().is_empty());
}

#[test]
fn test_rhl4c_cli_run_with_apply_files_exactly_one_ticket() {
    let Some(w) = world(90_000_000_000) else {
        return;
    };
    let run = w.ruchy(&["run", VALID, "--apply"]).success();
    assert!(
        stdout_text(&run).starts_with(PLAN_LINE),
        "{}",
        stdout_text(&run)
    );
    assert_eq!(w.gh_lines(), [GH_CREATE]);
}

#[test]
fn test_rhl4c_cli_enough_free_space_is_an_empty_plan_and_no_gh_even_with_apply() {
    let Some(w) = world(400_000_000_000) else {
        return;
    };
    let dry = w.ruchy(&["run", VALID]).success();
    assert_eq!(stdout_text(&dry), "");
    let applied = w.ruchy(&["run", VALID, "--apply"]).success();
    assert_eq!(stdout_text(&applied), "");
    assert!(w.gh_lines().is_empty(), "{}", w.log());
    assert_eq!(w.log().lines().count(), 2, "{}", w.log());
}

#[test]
fn test_rhl4c_cli_an_unbound_measure_is_refused_with_l002_exit_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bin = dir.path().join("j");
    for args in [
        vec!["compile", RUNNER_LOAD, "-o", bin.to_str().expect("path")],
        vec!["run", RUNNER_LOAD],
    ] {
        ruchy_cmd()
            .args(&args)
            .assert()
            .code(2)
            .stderr(predicate::str::contains("RHL-L002"))
            .stderr(predicate::str::contains("`runner load of`"));
    }
    assert!(!bin.exists());
    ruchy_cmd()
        .args(["transpile", RUNNER_LOAD])
        .assert()
        .success();
}

#[test]
fn test_rhl4c_cli_apply_is_for_rhl_files_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruchy = dir.path().join("x.ruchy");
    std::fs::write(&ruchy, "println!(\"hi\")\n").expect("write");
    ruchy_cmd()
        .args(["run", ruchy.to_str().expect("path"), "--apply"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--apply"));
}
