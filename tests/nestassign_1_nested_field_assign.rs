//! NESTASSIGN-1: assignment to a nested field (`o.inner.z = 5`,
//! `o.inner.z += 1`, `self.a.b = x`, `o.items[0].z = 5`) writes the value
//! through every value-typed parent back to the root variable. It used to fail
//! with "Runtime error: Complex field access not supported" because field
//! assignment only accepted a variable as the object.
//!
//! Each program is also transpiled and compiled with rustc; the compiled
//! binary must print the same output as the interpreter.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.timeout(Duration::from_secs(30));
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
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn interpret(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    stdout_of(cmd, "ruchy run")
}

fn run_file(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    interpret(dir.path(), src)
}

fn transpile(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, "ruchy transpile")
}

fn rustc_run(dir: &Path, rust: &str) -> String {
    let main_rs = dir.join("main.rs");
    let bin = dir.join("prog_bin");
    std::fs::write(&main_rs, rust).expect("write main.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "transpiled binary")
}

/// Interpret, then transpile + rustc + run; both must print `expected`.
fn assert_both(src: &str, expected: &str) {
    assert_eq!(
        run_file(src),
        expected,
        "interpreter output\nsource:\n{src}"
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    let actual = rustc_run(dir.path(), &rust);
    assert_eq!(
        actual, expected,
        "transpiled Rust output\nsource:\n{src}\nrust:\n{rust}"
    );
}

const NEST: &str = "struct In { z: i32 }\nstruct Out { inner: In }\n";

#[test]
fn test_nestassign_1_01_two_level_assign() {
    let src = format!(
        "{NEST}fun main() {{
    let mut o = Out {{ inner: In {{ z: 1 }} }}
    o.inner.z = 5
    println(o.inner.z)
}}
"
    );
    assert_both(&src, "5\n");
}

#[test]
fn test_nestassign_1_02_two_level_compound_assign() {
    let src = format!(
        "{NEST}fun main() {{
    let mut o = Out {{ inner: In {{ z: 1 }} }}
    o.inner.z += 1
    o.inner.z *= 10
    println(o.inner.z)
}}
"
    );
    assert_both(&src, "20\n");
}

#[test]
fn test_nestassign_1_03_three_level_assign() {
    let src = "struct C { v: i32 }
struct B { c: C }
struct A { b: B }
fun main() {
    let mut a = A { b: B { c: C { v: 1 } } }
    a.b.c.v = 7
    a.b.c.v += 1
    println(a.b.c.v)
}
";
    assert_both(src, "8\n");
}

#[test]
fn test_nestassign_1_04_self_nested_assign_in_method() {
    let src = format!(
        "{NEST}impl Out {{
    fun set(&mut self, x: i32) {{
        self.inner.z = x
    }}
    fun bump(&mut self) {{
        self.inner.z += 1
    }}
}}
fun main() {{
    let mut o = Out {{ inner: In {{ z: 1 }} }}
    o.set(40)
    o.bump()
    println(o.inner.z)
}}
"
    );
    assert_both(&src, "41\n");
}

#[test]
fn test_nestassign_1_05_sibling_fields_untouched() {
    let src = "struct In { z: i32, w: i32 }
struct Out { inner: In, k: i32 }
fun main() {
    let mut o = Out { inner: In { z: 1, w: 2 }, k: 3 }
    o.inner.z = 9
    println(o.inner.z)
    println(o.inner.w)
    println(o.k)
}
";
    assert_both(src, "9\n2\n3\n");
}

/// IDXASSIGN-1: the transpiler used to emit `o.items[0 as usize].clone().z = 5`,
/// which assigned to a temporary; the compiled binary must now agree.
#[test]
fn test_nestassign_1_06_index_in_chain_assign() {
    let src = "struct In { z: i32 }
struct Out { items: Vec<In> }
fun main() {
    let mut o = Out { items: vec![In { z: 1 }, In { z: 2 }] }
    o.items[0].z = 5
    o.items[0].z += 1
    o.items[1] = In { z: 7 }
    println(o.items[0].z)
    println(o.items[1].z)
}
";
    assert_both(src, "6\n7\n");
}

#[test]
fn test_nestassign_1_07_field_on_non_object_is_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(
        dir.path(),
        "fun main() {\n    let mut n = 3\n    n.z = 5\n    println(n)\n}\n",
    );
    let out = ruchy().arg("run").arg(file).output().expect("spawn ruchy");
    assert!(
        !out.status.success(),
        "assigning a field on an integer must fail"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("non-object"),
        "error names the non-object target: {stderr}"
    );
}
