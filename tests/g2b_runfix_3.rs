//! RHLGA-1 G2b run fixes, part 3.
//!
//! - FLOATDISP-1: `ruchy run` printed a float under `{}` with Debug text
//!   (`3.0`, `1000000000000000000000.0`, `-0.0`); Rust's f64 Display prints
//!   `3`, `1000000000000000000000`, `-0`. A bare `println(x)` transpiles to
//!   `println!("{:?}", x)`, so it keeps Rust's Debug text (`3.0`, `1e21`).
//! - TMPLEAK-1: `property-tests`, `fuzz` and `notebook` removed the compiled
//!   `ruchy_temp_bin_*` binary only on the success path; an error returned
//!   with `?` left it in the temp dir.
//!
//! Each FLOATDISP case states the output Rust prints for the same program; a
//! hand-written Rust reference is compiled with rustc to prove it.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

fn stdout_of(mut cmd: Command, what: &str) -> String {
    let out = cmd.output().expect("spawn command");
    assert!(
        out.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
}

fn ruchy() -> Command {
    Command::cargo_bin("ruchy").expect("ruchy binary is built for tests")
}

fn ruchy_run(dir: &Path, src: &str) -> String {
    let mut cmd = ruchy();
    cmd.arg("run")
        .arg(write_source(dir, src))
        .timeout(Duration::from_secs(10));
    stdout_of(cmd, "ruchy run")
}

fn rustc_run(dir: &Path, rust: &str) -> String {
    let main_rs = dir.join("reference.rs");
    let bin = dir.join("reference_bin");
    std::fs::write(&main_rs, rust).expect("write reference.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, "rustc on the Rust reference");
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(10));
    stdout_of(run, "Rust reference binary")
}

/// Rust prints `expected` for `rust`, and `ruchy run` prints it for `ruchy`.
fn assert_matches_rust(ruchy_src: &str, rust: &str, expected: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(rustc_run(dir.path(), rust), expected, "Rust reference");
    assert_eq!(ruchy_run(dir.path(), ruchy_src), expected, "ruchy run");
}

// ------------------------------------------------------------- FLOATDISP-1

const BRIEF_CASE: &str = r#"
fun main() {
    let x = 3.0
    println("{} {}", x, 2.5)
    println(x)
}
"#;

#[test]
fn test_floatdisp_1_brief_case_display_and_bare_println() {
    assert_matches_rust(
        BRIEF_CASE,
        r#"
fn main() {
    let x = 3.0f64;
    println!("{} {}", x, 2.5);
    println!("{:?}", x);
}
"#,
        "3 2.5\n3.0\n",
    );
}

#[test]
fn test_floatdisp_1_display_special_values() {
    assert_matches_rust(
        r#"
fun main() {
    let a = 1e21
    let b = 0.0 / 0.0
    let c = 1.0 / 0.0
    let d = -0.0
    let e = 1e-7
    let g = 0.1 + 0.2
    println("{} {} {} {} {} {} {}", a, b, c, d, e, g, -c)
}
"#,
        r#"
fn main() {
    let a = 1e21f64;
    let b = 0.0f64 / 0.0;
    let c = 1.0f64 / 0.0;
    let d = -0.0f64;
    let e = 1e-7f64;
    let g = 0.1f64 + 0.2;
    println!("{} {} {} {} {} {} {}", a, b, c, d, e, g, -c);
}
"#,
        "1000000000000000000000 NaN inf -0 0.0000001 0.30000000000000004 -inf\n",
    );
}

#[test]
fn test_floatdisp_1_debug_special_values_unchanged() {
    assert_matches_rust(
        r#"
fun main() {
    let a = 1e21
    let d = -0.0
    let e = 1e-7
    println("{:?} {:?} {:?} {:?} {:?}", 3.0, a, d, e, 2.5)
}
"#,
        r#"
fn main() {
    let a = 1e21f64;
    let d = -0.0f64;
    let e = 1e-7f64;
    println!("{:?} {:?} {:?} {:?} {:?}", 3.0f64, a, d, e, 2.5f64);
}
"#,
        "3.0 1e21 -0.0 1e-7 2.5\n",
    );
}

#[test]
fn test_floatdisp_1_bare_println_is_debug() {
    assert_matches_rust(
        r#"
fun main() {
    let a = 1e21
    let e = 1e-7
    let d = -0.0
    println(a)
    println(e)
    println(d)
    println(3.0)
}
"#,
        r#"
fn main() {
    let a = 1e21f64;
    let e = 1e-7f64;
    let d = -0.0f64;
    println!("{:?}", a);
    println!("{:?}", e);
    println!("{:?}", d);
    println!("{:?}", 3f64);
}
"#,
        "1e21\n1e-7\n-0.0\n3.0\n",
    );
}

#[test]
fn test_floatdisp_1_width_sign_and_alignment() {
    assert_matches_rust(
        r#"
fun main() {
    let x = 3.0
    println("[{:>5}] [{:+}] [{:<4}] [{:^5}] [{:05}]", x, x, x, x, -x)
}
"#,
        r#"
fn main() {
    let x = 3.0f64;
    println!("[{:>5}] [{:+}] [{:<4}] [{:^5}] [{:05}]", x, x, x, x, -x);
}
"#,
        "[    3] [+3] [3   ] [  3  ] [-0003]\n",
    );
}

#[test]
fn test_floatdisp_1_format_fn_and_fstring() {
    assert_matches_rust(
        r#"
fun main() {
    let x = 3.0
    println(format("{}", x))
    println(f"{x} {x:?} {x:>4}")
    println(x.to_string())
}
"#,
        r#"
fn main() {
    let x = 3.0f64;
    println!("{}", format!("{}", x));
    println!("{} {:?} {:>4}", x, x, x);
    println!("{}", x.to_string());
}
"#,
        "3\n3 3.0    3\n3\n",
    );
}

#[test]
fn test_floatdisp_1_run_matches_compiled_binary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_source(dir.path(), BRIEF_CASE);
    let bin = dir.path().join("prog_bin");
    let mut compile = ruchy();
    compile
        .arg("compile")
        .arg(&src)
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(120));
    stdout_of(compile, "ruchy compile");
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(10));
    let compiled = stdout_of(run, "compiled binary");
    assert_eq!(compiled, "3 2.5\n3.0\n", "compiled binary");
    assert_eq!(ruchy_run(dir.path(), BRIEF_CASE), compiled, "ruchy run");
}

// --------------------------------------------------------------- TMPLEAK-1

/// The compiled program replaces its own executable with a text file, so the
/// next execution fails with "Permission denied" and the handler returns `?`.
const SELF_REPLACING: &str = r#"
fun main() {
    let exe = std::env::current_exe().unwrap()
    std::fs::remove_file(&exe).unwrap()
    std::fs::write(&exe, "not a binary").unwrap()
}
"#;

const PANICKING: &str = r#"
fun main() {
    panic("boom")
}
"#;

fn leaked_binaries(tmp: &Path) -> Vec<String> {
    std::fs::read_dir(tmp)
        .expect("read temp dir")
        .map(|e| {
            e.expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.starts_with("ruchy_temp_bin_"))
        .collect()
}

/// Runs `ruchy <args…> prog.ruchy` with TMPDIR set to a fresh directory,
/// asserts the command fails, and returns the leaked temp binaries.
fn leaks_after_failing(args: &[&str], src: &str) -> Vec<String> {
    let dir = tempfile::tempdir().expect("tempdir");
    let tmp = dir.path().join("tmp");
    std::fs::create_dir(&tmp).expect("create TMPDIR");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.args(&args[..1])
        .arg(&file)
        .args(&args[1..])
        .env("TMPDIR", &tmp)
        .timeout(Duration::from_secs(180));
    let out = cmd.output().expect("spawn ruchy");
    assert!(!out.status.success(), "the {} run must fail", args[0]);
    leaked_binaries(&tmp)
}

#[test]
fn test_tmpleak_1_property_tests_error_path_removes_binary() {
    let leaked = leaks_after_failing(&["property-tests", "--cases", "3"], SELF_REPLACING);
    assert!(leaked.is_empty(), "leaked temp binaries: {leaked:?}");
}

#[test]
fn test_tmpleak_1_fuzz_error_path_removes_binary() {
    let leaked = leaks_after_failing(&["fuzz", "--iterations", "3"], SELF_REPLACING);
    assert!(leaked.is_empty(), "leaked temp binaries: {leaked:?}");
}

#[test]
fn test_tmpleak_1_property_tests_panicking_program_removes_binary() {
    let leaked = leaks_after_failing(&["property-tests", "--cases", "3"], PANICKING);
    assert!(leaked.is_empty(), "leaked temp binaries: {leaked:?}");
}

#[test]
fn test_tmpleak_1_notebook_failing_program_removes_binary() {
    let leaked = leaks_after_failing(&["notebook"], PANICKING);
    assert!(leaked.is_empty(), "leaked temp binaries: {leaked:?}");
}
