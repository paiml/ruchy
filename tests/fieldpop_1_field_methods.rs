//! FIELDPOP-1: a mutating array method called on a field (`self.items.pop()`,
//! `s.items.push(x)`, `o.items.pop()`) returns the method's own result and
//! writes the changed array back to the field. It used to evaluate the field
//! to a copy, run the array method on the copy, and return the updated copy:
//! `self.items.pop()` yielded the remaining array (so `.unwrap()` failed with
//! "Unknown array method: unwrap") and the field kept all its items.
//!
//! The field form must behave exactly like the same call on a local array
//! (`let mut a = [1, 2, 3]; a.pop()` returns `3`, leaves `a == [1, 2]`), so
//! most tests compare the two forms instead of fixing one semantics here.

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

/// Transpile, compile, run; assert interpreter == rustc output. Returns the Rust text.
fn differential(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    let expected = interpret(dir.path(), src);
    let actual = rustc_run(dir.path(), &rust);
    assert_eq!(
        actual, expected,
        "transpiled Rust disagrees with the interpreter\nsource:\n{src}\nrust:\n{rust}"
    );
    rust
}

const STACK: &str = "struct Stack { items: Vec<i32> }\n";

/// `op` applied to a local array and to a struct field must print the same
/// result and leave the same array behind.
fn assert_field_matches_local(op: &str) {
    let local = format!(
        "fun main() {{\n    let mut a = [3, 1, 2]\n    println(a.{op})\n    println(a)\n}}\n"
    );
    let field = format!(
        "{STACK}fun main() {{\n    let mut s = Stack {{ items: vec![3, 1, 2] }}\n    println(s.items.{op})\n    println(s.items)\n}}\n"
    );
    assert_eq!(
        run_file(&field),
        run_file(&local),
        "field vs local for `{op}`"
    );
}

#[test]
fn test_fieldpop_1_01_self_field_pop_returns_element_and_writes_back() {
    let src = format!(
        "{STACK}impl Stack {{
    fun pop_one(&mut self) -> i32 {{
        self.items.pop()
    }}
}}
fun main() {{
    let mut s = Stack {{ items: vec![1, 2, 3] }}
    println(s.pop_one())
    println(s.items.len())
    println(s.items)
}}
"
    );
    assert_eq!(run_file(&src), "3\n2\n[1, 2]\n");
}

#[test]
fn test_fieldpop_1_02_self_field_push_writes_back() {
    let src = format!(
        "{STACK}impl Stack {{
    fun push_two(&mut self, x: i32) {{
        self.items.push(x)
        self.items.push(x + 1)
    }}
}}
fun main() {{
    let mut s = Stack {{ items: vec![1] }}
    s.push_two(10)
    println(s.items)
}}
"
    );
    assert_eq!(run_file(&src), "[1, 10, 11]\n");
}

#[test]
fn test_fieldpop_1_03_struct_value_field_pop_and_push_in_main() {
    let src = format!(
        "{STACK}fun main() {{
    let mut s = Stack {{ items: vec![1, 2, 3] }}
    let top = s.items.pop()
    println(top)
    println(s.items)
    s.items.push(9)
    println(s.items)
}}
"
    );
    assert_eq!(run_file(&src), "3\n[1, 2]\n[1, 2, 9]\n");
}

#[test]
fn test_fieldpop_1_04_field_push_inside_while_loop_persists() {
    let src = format!(
        "{STACK}fun main() {{
    let mut s = Stack {{ items: vec![] }}
    let mut i = 0
    while i < 3 {{
        s.items.push(i)
        i = i + 1
    }}
    println(s.items)
}}
"
    );
    assert_eq!(run_file(&src), "[0, 1, 2]\n");
}

#[test]
fn test_fieldpop_1_05_object_literal_field_pop_and_push() {
    let src = "fun main() {
    let mut o = {items: [1, 2, 3]}
    println(o.items.pop())
    o.items.push(7)
    println(o.items)
}
";
    assert_eq!(run_file(src), "3\n[1, 2, 7]\n");
}

#[test]
fn test_fieldpop_1_06_class_field_push_pop_reference_semantics() {
    let src = "class Bag {
    items: Vec<i32>
    new() {
        self.items = []
    }
    fun add(&mut self, x: i32) { self.items.push(x) }
    fun take(&mut self) -> i32 { self.items.pop() }
}
let mut b = Bag::new()
let alias = b
b.add(4)
b.add(5)
println(b.take())
println(b.items)
println(alias.items)
";
    assert_eq!(run_file(src), "5\n[4]\n[4]\n");
}

#[test]
fn test_fieldpop_1_07_empty_field_pop_matches_local() {
    let local =
        run_file("fun main() {\n    let mut a = []\n    println(a.pop())\n    println(a)\n}\n");
    let field = run_file(&format!(
        "{STACK}fun main() {{\n    let mut s = Stack {{ items: vec![] }}\n    println(s.items.pop())\n    println(s.items)\n}}\n"
    ));
    assert_eq!(field, local);
}

#[test]
fn test_fieldpop_1_08_mutating_forms_match_local() {
    for op in ["pop()", "push(4)"] {
        assert_field_matches_local(op);
    }
}

#[test]
fn test_fieldpop_1_09_methods_1_and_copying_forms_match_local() {
    for op in [
        "sorted()",
        "reversed()",
        "drop(1)",
        "append([4])",
        "sort()",
        "reverse()",
        "len()",
    ] {
        assert_field_matches_local(op);
    }
}

#[test]
fn test_fieldpop_1_10_differential_push_pop_through_rustc() {
    let src = format!(
        "{STACK}impl Stack {{
    fun push_two(&mut self, x: i32) {{
        self.items.push(x)
        self.items.push(x + 1)
    }}
    fun drop_top(&mut self) -> usize {{
        self.items.pop();
        self.items.len()
    }}
}}
fun main() {{
    let mut s = Stack {{ items: vec![1, 2, 3] }}
    println(s.drop_top())
    s.push_two(10)
    println(s.items.len())
    println(s.items[s.items.len() - 1])
}}
"
    );
    differential(&src);
}

// ---------- VECLIT-1: an array literal for a `Vec<…>` struct field ----------

#[test]
fn test_veclit_1_01_array_literal_for_vec_field_compiles_as_vec() {
    let src = format!(
        "{STACK}fun main() {{
    let mut s = Stack {{ items: [1, 2, 3] }}
    s.items.push(4)
    println(s.items.len())
}}
"
    );
    let rust = differential(&src);
    let squashed: String = rust.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(squashed.contains("items:vec![1,2,3]"), "rust:\n{rust}");
}

#[test]
fn test_veclit_1_02_nested_vec_field_and_fixed_array_field_untouched() {
    let src = "struct Grid { rows: Vec<Vec<i32>>, fixed: [i32; 2] }
fun main() {
    let g = Grid { rows: [vec![1], vec![2, 3]], fixed: [7, 8] }
    println(g.rows.len())
    println(g.fixed[1])
}
";
    let rust = differential(src);
    let squashed: String = rust.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(squashed.contains("fixed:[7,8]"), "rust:\n{rust}");
}
