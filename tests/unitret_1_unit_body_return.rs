//! UNITRET-1: a function without a declared return type whose tail
//! expression is a method call that returns unit in Rust (`push`, `clear`,
//! `sort`, ...) must not get an inferred return type.
//!
//! Before the fix `fun f(xs: &mut Vec<i32>) { xs.push(1) }` transpiled to
//! `fn f(xs: &mut Vec<i32>) -> i32 { xs.push(1) }` and rustc failed with
//! E0308 (expected `i32`, found `()`).
//!
//! Bodies with a value-producing tail keep their inferred return type.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.timeout(Duration::from_secs(10));
    cmd
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
}

fn stdout_of(mut cmd: Command, what: &str) -> String {
    let out = cmd.output().expect("spawn command");
    assert!(
        out.status.success(),
        "{what} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn transpile(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, "ruchy transpile")
}

/// `ruchy compile` (rustc) the source, then run the binary.
fn compile_and_run(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let bin = dir.path().join("prog_bin");
    let mut cmd = ruchy();
    cmd.arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(60));
    stdout_of(cmd, &format!("ruchy compile of:\n{src}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "compiled binary")
}

/// Strip whitespace so `fn f ( x ) -> i32` and `fn f(x)->i32` compare equal.
fn squash(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// The transpiled signature of `f`, from `fn f` up to the body brace.
fn signature_of_f(rust: &str) -> String {
    let flat = squash(rust);
    let start = flat.find("fnf(").expect("fn f in transpiled output");
    let rest = &flat[start..];
    let end = rest.find('{').expect("body brace after fn f");
    rest[..end].to_string()
}

/// Unit-returning `Vec` method calls used as a function's tail expression.
const UNIT_TAILS: &[&str] = &[
    "xs.push(1)",
    "xs.insert(0, 7)",
    "xs.extend(vec![4, 5])",
    "xs.clear()",
    "xs.sort()",
    "xs.reverse()",
    "xs.truncate(1)",
    "xs.retain(|x| *x > 1)",
    "xs.dedup()",
    "xs.swap(0, 1)",
];

#[test]
fn test_unitret_1_unit_method_tail_has_no_return_type() {
    for tail in UNIT_TAILS {
        let src = format!("fun f(xs: &mut Vec<i32>) {{ {tail} }}\n");
        let sig = signature_of_f(&transpile(&src));
        assert!(
            !sig.contains("->"),
            "`{tail}` returns unit; f must not infer a return type: {sig}"
        );
    }
}

#[test]
fn test_unitret_1_push_tail_compiles_and_runs() {
    let src = "fun f(xs: &mut Vec<i32>) { xs.push(1) }\n\
               fun main() { let mut v: Vec<i32> = vec![]\n f(&mut v)\n f(&mut v)\n println!(\"{}\", v.len()) }\n";
    assert_eq!(compile_and_run(src).trim(), "2");
}

#[test]
fn test_unitret_1_every_unit_tail_compiles() {
    let fns: String = UNIT_TAILS
        .iter()
        .enumerate()
        .map(|(i, tail)| format!("fun f{i}(xs: &mut Vec<i32>) {{ {tail} }}\n"))
        .collect();
    // Called last to first so `swap` runs before `clear` empties the Vec.
    let calls: String = (0..UNIT_TAILS.len())
        .rev()
        .map(|i| format!(" f{i}(&mut v)\n"))
        .collect();
    let src = format!(
        "{fns}fun main() {{ let mut v: Vec<i32> = vec![3, 1, 2]\n{calls} println!(\"{{:?}}\", v) }}\n"
    );
    assert_eq!(compile_and_run(&src).trim(), "[7, 4, 5, 1]");
}

#[test]
fn test_unitret_1_unit_tail_after_statements() {
    let src = "fun add_two(xs: &mut Vec<i32>) { let n = xs.len()\n xs.push(1)\n xs.push(2) }\n\
               fun main() { let mut v: Vec<i32> = vec![]\n add_two(&mut v)\n println!(\"{:?}\", v) }\n";
    assert_eq!(compile_and_run(src).trim(), "[1, 2]");
}

#[test]
fn test_unitret_1_unit_tail_in_if_branches() {
    let src =
        "fun f(xs: &mut Vec<i32>, grow: bool) { if grow { xs.push(9) } else { xs.clear() } }\n";
    let sig = signature_of_f(&transpile(src));
    assert!(!sig.contains("->"), "both branches are unit: {sig}");
}

#[test]
fn test_unitret_1_arithmetic_tail_keeps_return_type() {
    let src = "fun f(a: i32, b: i32) { a + b }\nfun main() { println!(\"{}\", f(2, 3)) }\n";
    assert!(signature_of_f(&transpile(src)).contains("->i32"));
    assert_eq!(compile_and_run(src).trim(), "5");
}

#[test]
fn test_unitret_1_value_method_tail_keeps_return_type() {
    let src = "fun f(x: i32) { x.abs() }\nfun main() { println!(\"{}\", f(-4)) }\n";
    assert!(signature_of_f(&transpile(src)).contains("->i32"));
    assert_eq!(compile_and_run(src).trim(), "4");
}

#[test]
fn test_unitret_1_len_tail_keeps_a_return_type() {
    let src = "fun f(xs: &Vec<i32>) { xs.len() }\n";
    assert!(signature_of_f(&transpile(src)).contains("->"));
}

// ==================== LENRET-1: usize-valued tails ====================
// `len()`/`count()` are `usize` in the transpiled Rust (an unannotated
// `let n = v.len()` is a `usize` too), so a function whose tail is one of
// them returns `usize` rather than an `i32` it cannot produce.

#[test]
fn test_lenret_1_len_tail_infers_usize() {
    let src = "fun f(xs: &Vec<i32>) { xs.len() }\n\
               fun main() { let v = vec![1, 2, 3]\n println!(\"{}\", f(&v)) }\n";
    assert!(signature_of_f(&transpile(src)).contains("->usize"));
    assert_eq!(compile_and_run(src).trim(), "3");
}

#[test]
fn test_lenret_1_len_tail_after_statements_and_in_loop_bound() {
    let src = "fun f(xs: &Vec<i32>) { let k = 1\n xs.len() }\n\
               fun main() { let v = vec![4, 5]\n let mut t = 0\n \
               for i in 0..f(&v) { t = t + v[i] }\n println!(\"{} {}\", f(&v), t) }\n";
    assert_eq!(compile_and_run(src).trim(), "2 9");
}

#[test]
fn test_lenret_1_string_len_tail_compiles() {
    let src = "fun f(s: &str) { s.len() }\nfun main() { println!(\"{}\", f(\"abcd\")) }\n";
    assert!(signature_of_f(&transpile(src)).contains("->usize"));
    assert_eq!(compile_and_run(src).trim(), "4");
}

/// `xs.count()` transpiles to `xs.iter().count()`. (`s.chars().count()` is
/// emitted as `s.chars().iter().count()`, a separate method-emission defect
/// reported with this ticket, so it is not used here.)
#[test]
fn test_lenret_1_count_tail_compiles() {
    let src = "fun f(xs: &Vec<i32>) { xs.count() }\n\
               fun main() { println!(\"{}\", f(&vec![7, 8, 9])) }\n";
    assert!(signature_of_f(&transpile(src)).contains("->usize"));
    assert_eq!(compile_and_run(src).trim(), "3");
}
