//! ARRAYMUT-1: the in-place `Vec` methods (`sort`, `reverse`, `append`,
//! `insert`, `remove`, `clear`, `truncate`, `dedup`, `extend`) change their
//! receiver when it is a local array or a struct field, and return what Rust
//! returns: `()` (printed as `nil`) for all of them except `remove`, which
//! returns the removed element.
//!
//! Before the fix `v.sort()` / `v.reverse()` returned a changed copy and left
//! `v` untouched, and `insert`/`remove`/`clear`/`truncate`/`dedup` failed with
//! "Unknown array method". `sorted`/`reversed` stay functional.
//!
//! `extend(…)` is not exercised here: `.extend(` does not parse yet (tracked
//! as EXTENDKW-1); its interpreter behaviour is covered by lib unit tests in
//! `src/runtime/eval_array.rs`.

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

fn output_of(mut cmd: Command) -> std::process::Output {
    cmd.output().expect("spawn command")
}

fn stdout_of(cmd: Command, what: &str) -> String {
    let out = output_of(cmd);
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

/// Run a program that must fail; returns its combined stdout+stderr.
fn run_failing(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    let out = output_of(cmd);
    assert!(!out.status.success(), "expected failure for:\n{src}");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
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

/// Transpile, compile, run; assert interpreter == rustc output.
fn differential(src: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    let expected = interpret(dir.path(), src);
    let actual = rustc_run(dir.path(), &rust);
    assert_eq!(
        actual, expected,
        "transpiled Rust disagrees with the interpreter\nsource:\n{src}\nrust:\n{rust}"
    );
}

/// `op` on a local `[3, 1, 2, 2]`: prints the call's result, then the array.
fn on_local(op: &str) -> String {
    run_file(&format!(
        "fun main() {{\n    let mut v = [3, 1, 2, 2]\n    println(v.{op})\n    println(v)\n}}\n"
    ))
}

const BAG: &str = "struct Bag { items: Vec<i32> }\n";

/// `op` on `self.items` inside a `&mut self` method, then the field.
fn on_self_field(op: &str) -> String {
    run_file(&format!(
        "{BAG}impl Bag {{
    fun apply(&mut self) {{
        println(self.items.{op})
    }}
}}
fun main() {{
    let mut b = Bag {{ items: vec![3, 1, 2, 2] }}
    b.apply()
    println(b.items)
}}
"
    ))
}

/// Same op on a local and on `self.items`: both must give `expected`.
fn assert_in_place(op: &str, expected: &str) {
    assert_eq!(on_local(op), expected, "local receiver, `{op}`");
    assert_eq!(on_self_field(op), expected, "self.field receiver, `{op}`");
}

#[test]
fn test_arraymut_1_01_sort_sorts_receiver() {
    assert_in_place("sort()", "nil\n[1, 2, 2, 3]\n");
}

#[test]
fn test_arraymut_1_02_reverse_reverses_receiver() {
    assert_in_place("reverse()", "nil\n[2, 2, 1, 3]\n");
}

#[test]
fn test_arraymut_1_03_insert_places_element() {
    assert_in_place("insert(0, 9)", "nil\n[9, 3, 1, 2, 2]\n");
    assert_in_place("insert(4, 9)", "nil\n[3, 1, 2, 2, 9]\n");
}

#[test]
fn test_arraymut_1_04_remove_returns_removed_element() {
    assert_in_place("remove(1)", "1\n[3, 2, 2]\n");
}

#[test]
fn test_arraymut_1_05_clear_empties_receiver() {
    assert_in_place("clear()", "nil\n[]\n");
}

#[test]
fn test_arraymut_1_06_truncate_shortens_receiver() {
    assert_in_place("truncate(2)", "nil\n[3, 1]\n");
    assert_in_place("truncate(10)", "nil\n[3, 1, 2, 2]\n");
}

#[test]
fn test_arraymut_1_07_dedup_removes_consecutive_duplicates() {
    assert_in_place("dedup()", "nil\n[3, 1, 2]\n");
}

#[test]
fn test_arraymut_1_08_append_moves_other_elements_in() {
    assert_in_place("append([7, 8])", "nil\n[3, 1, 2, 2, 7, 8]\n");
}

#[test]
fn test_arraymut_1_09_field_on_local_struct_is_written_back() {
    let src = format!(
        "{BAG}fun main() {{
    let mut b = Bag {{ items: vec![3, 1, 2] }}
    b.items.sort()
    b.items.insert(0, 0)
    println(b.items.remove(3))
    println(b.items)
}}
"
    );
    assert_eq!(run_file(&src), "3\n[0, 1, 2]\n");
}

#[test]
fn test_arraymut_1_10_out_of_range_insert_is_runtime_error() {
    let src = "fun main() {\n    let mut v = [1, 2]\n    v.insert(3, 9)\n    println(v)\n}\n";
    let out = run_failing(src);
    assert!(out.contains("insert"), "error names the method: {out}");
}

#[test]
fn test_arraymut_1_11_out_of_range_remove_is_runtime_error() {
    let src = "fun main() {\n    let mut v = [1, 2]\n    v.remove(2)\n    println(v)\n}\n";
    let out = run_failing(src);
    assert!(out.contains("remove"), "error names the method: {out}");
}

#[test]
fn test_arraymut_1_12_sorted_and_reversed_stay_functional() {
    let src = "fun main() {\n    let v = [3, 1, 2]\n    println(v.sorted())\n    println(v.reversed())\n    println(v)\n}\n";
    assert_eq!(run_file(src), "[1, 2, 3]\n[2, 1, 3]\n[3, 1, 2]\n");
}

#[test]
fn test_arraymut_1_13_mutation_inside_loop_persists() {
    let src = "fun main() {\n    let mut v = [5, 4, 3, 2, 1]\n    let mut i = 0\n    while i < 2 {\n        v.remove(0)\n        i = i + 1\n    }\n    v.sort()\n    println(v)\n}\n";
    assert_eq!(run_file(src), "[1, 2, 3]\n");
}

#[test]
fn test_arraymut_1_14_differential_local_vec() {
    differential(
        "fun main() {
    let mut v = vec![3, 1, 2, 2]
    v.sort()
    v.dedup()
    v.reverse()
    v.insert(0, 9)
    let r = v.remove(1)
    v.truncate(3)
    println(\"{} {:?}\", r, v)
    v.clear()
    println(\"{}\", v.len())
}
",
    );
}

#[test]
fn test_arraymut_1_15_differential_self_field() {
    differential(&format!(
        "{BAG}impl Bag {{
    fun tidy(&mut self) -> i32 {{
        self.items.sort()
        self.items.dedup()
        self.items.insert(0, 7)
        self.items.truncate(3)
        self.items.remove(0)
    }}
}}
fun main() {{
    let mut b = Bag {{ items: vec![3, 1, 2, 2, 5] }}
    let r = b.tidy()
    println(\"{{}} {{:?}}\", r, b.items)
}}
"
    ));
}

// ---------- RHLGA-1 review F3/F4: `&mut self` inference ----------

/// Transpile, compile with rustc, run; the rustc output.
fn through_rustc(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    rustc_run(dir.path(), &rust)
}

#[test]
fn test_arraymut_1_16_resize_and_extend_from_slice_infer_mut_self() {
    let src = "struct Bag { items: Vec<i32> }
impl Bag {
    fun grow(self) {
        self.items.resize(3, 0)
    }
    fun more(self) {
        self.items.extend_from_slice(&[9])
    }
}
fun main() {
    let mut b = Bag { items: vec![1] }
    b.grow()
    b.more()
    println(b.items)
}
";
    assert_eq!(through_rustc(src), "[1, 0, 0, 9]\n");
}

#[test]
fn test_arraymut_1_17_mut_borrow_of_self_field_infers_mut_self() {
    let src = "struct Bag { items: Vec<i32> }
fun add_one(xs: &mut Vec<i32>) -> usize {
    xs.push(1)
    xs.len()
}
impl Bag {
    fun fill(self) -> usize {
        add_one(&mut self.items)
    }
}
fun main() {
    let mut b = Bag { items: vec![7] }
    println(b.fill())
    println(b.items)
}
";
    assert_eq!(through_rustc(src), "2\n[7, 1]\n");
}
