//! G2B-FIX-A (RHLGA-1): review findings on the G2 part 2 branch.
//!
//! 1. A trait default method with a by-value `self` needs `where Self: Sized`.
//! 2. `&mut self` inference sees nested/indexed field writes, field `++`/`--`,
//!    mutating std methods on a field and calls to mutating sibling methods.
//! 3. The `::` path marker survives `ruchy fmt` and non-callee transpilation.
//! 4. A user-defined `range` function is not the builtin range.
//! 5. An awaiting `main` keeps a declared non-unit return type.
//!
//! Every program is transpiled, compiled with rustc and run.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::quality::formatter::Formatter;
use ruchy::{Parser, Transpiler};
use std::process::Command;

fn scratch_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("g2b_fix_a_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn transpile(code: &str) -> String {
    let ast = Parser::new(code).parse().expect("parses");
    Transpiler::new()
        .transpile_to_program(&ast)
        .expect("transpiles")
        .to_string()
}

fn compiled_output(code: &str, name: &str) -> String {
    let rust = transpile(code);
    let dir = scratch_dir(name);
    let src = dir.join("main.rs");
    let bin = dir.join("main");
    std::fs::write(&src, &rust).expect("write rust");
    let out = Command::new("rustc")
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&src)
        .output()
        .expect("rustc runs");
    assert!(
        out.status.success(),
        "rustc rejected the transpiled program:\n{}\n--- rust ---\n{rust}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&bin).output().expect("binary runs");
    assert!(run.status.success(), "binary failed: {run:?}");
    String::from_utf8_lossy(&run.stdout).into_owned()
}

/// Whitespace-free token text, so `where Self : Sized` matches `whereSelf:Sized`.
fn squash(rust: &str) -> String {
    rust.chars().filter(|c| !c.is_whitespace()).collect()
}

// ------------------------------------------- 1. trait default by-value self

const TRAIT_DEFAULT_OWNED: &str = r"
trait Named {
    fun id(self) -> i32
    fun consume(self) -> i32 {
        self.id() + 1
    }
}
struct Token {
    v: i32
}
impl Named for Token {
    fun id(self) -> i32 {
        self.v
    }
}
fun main() {
    let t = Token { v: 41 }
    println(t.consume())
}
";

#[test]
fn test_g2bfa1_trait_default_owned_self_compiles() {
    assert_eq!(compiled_output(TRAIT_DEFAULT_OWNED, "fa1_default"), "42\n");
}

#[test]
fn test_g2bfa1_trait_default_owned_self_gets_sized_bound() {
    let rust = squash(&transpile(TRAIT_DEFAULT_OWNED));
    assert!(
        rust.contains("fnconsume(self)->i32whereSelf:Sized{"),
        "default by-value method needs `where Self: Sized`: {rust}"
    );
    assert!(
        rust.contains("fnid(self)->i32;"),
        "a bodiless by-value declaration needs no bound: {rust}"
    );
}

#[test]
fn test_g2bfa1_spec_7_1_owned_self_default_compiles() {
    let code = r#"
trait Actor {
    fun launch(self) -> i32 {
        self.id() * 2
    }
    fun id(&self) -> i32
}
struct Worker {
    n: i32
}
impl Actor for Worker {
    fun id(&self) -> i32 {
        self.n
    }
}
fun main() {
    let w = Worker { n: 21 }
    println(w.launch())
}
"#;
    assert_eq!(compiled_output(code, "fa1_spawn"), "42\n");
}

#[test]
fn test_g2bfa1_trait_default_borrowed_self_has_no_bound() {
    let code = r"
trait Shown {
    fun show(&self) -> i32 {
        7
    }
}
fun main() {
}
";
    let rust = squash(&transpile(code));
    assert!(rust.contains("fnshow(&self)->i32{"), "{rust}");
    assert!(!rust.contains("Sized"), "{rust}");
}

// ---------------------------------------------- 2. &mut self inference

fn assert_mut_self(code: &str, method: &str) {
    let rust = squash(&transpile(code));
    assert!(
        rust.contains(&format!("fn{method}(&mutself")),
        "`{method}` mutates self and must take &mut self: {rust}"
    );
}

#[test]
fn test_g2bfa2_nested_field_assignment_is_mut_self() {
    let code = r"
struct Inner {
    n: i32
}
struct Outer {
    inner: Inner
}
impl Outer {
    fun bump(self) {
        self.inner.n = self.inner.n + 1
    }
}
fun main() {
    let mut o = Outer { inner: Inner { n: 1 } }
    o.bump()
    println(o.inner.n)
}
";
    assert_mut_self(code, "bump");
    assert_eq!(compiled_output(code, "fa2_nested"), "2\n");
}

#[test]
fn test_g2bfa2_indexed_field_assignment_is_mut_self() {
    let code = r"
struct Grid {
    cells: Vec<i32>
}
impl Grid {
    fun set(self, i: usize, v: i32) {
        self.cells[i] = v
    }
}
fun main() {
    let mut g = Grid { cells: vec![0, 0, 0] }
    g.set(1, 5)
    println(g.cells[1])
}
";
    assert_mut_self(code, "set");
    assert_eq!(compiled_output(code, "fa2_index"), "5\n");
}

#[test]
fn test_g2bfa2_field_increment_is_mut_self() {
    let code = r"
struct Counter {
    count: i32
}
impl Counter {
    fun tick(self) -> i32 {
        self.count++
    }
    fun untick(self) -> i32 {
        --self.count
    }
}
fun main() {
    let mut c = Counter { count: 0 }
    c.tick()
    c.tick()
    c.untick()
    println(c.count)
}
";
    assert_mut_self(code, "tick");
    assert_mut_self(code, "untick");
    assert_eq!(compiled_output(code, "fa2_incr"), "1\n");
}

#[test]
fn test_g2bfa2_mutating_std_method_on_field_is_mut_self() {
    let code = r"
struct Bag {
    items: Vec<i32>
}
impl Bag {
    fun add(self, x: i32) {
        self.items.push(x)
    }
    fun size(self) -> usize {
        self.items.len()
    }
}
fun main() {
    let mut b = Bag { items: vec![] }
    b.add(3)
    b.add(4)
    println(b.items.len())
}
";
    assert_mut_self(code, "add");
    let rust = squash(&transpile(code));
    assert!(
        rust.contains("fnsize(self)->usize"),
        "a read-only method stays an owned receiver: {rust}"
    );
    assert_eq!(compiled_output(code, "fa2_push"), "2\n");
}

#[test]
fn test_g2bfa2_calling_mutating_sibling_is_mut_self() {
    // examples/self_hosting/lexer.ruchy: skip_whitespace(self) calls self.advance()
    let code = r"
struct Lexer {
    pos: i32
}
impl Lexer {
    fun skip_whitespace(self) {
        while self.pos < 3 {
            self.advance()
        }
    }
    fun skip_twice(self) {
        self.skip_whitespace()
    }
    fun advance(self) {
        self.pos = self.pos + 1
    }
}
fun main() {
    let mut l = Lexer { pos: 0 }
    l.skip_twice()
    println(l.pos)
}
";
    assert_mut_self(code, "advance");
    assert_mut_self(code, "skip_whitespace");
    assert_mut_self(code, "skip_twice");
    assert_eq!(compiled_output(code, "fa2_sibling"), "3\n");
}

// ------------------------------------------------- 3. `::` path marker

fn fmt(code: &str) -> String {
    let ast = Parser::new(code).parse().expect("parses");
    let mut formatter = Formatter::new();
    formatter.set_source(code);
    formatter.format(&ast).expect("formats")
}

#[test]
fn test_g2bfa3_fmt_keeps_path_separator() {
    let out = fmt("helpers::double(4)\n");
    assert!(out.contains("helpers::double(4)"), "fmt output: {out}");
}

#[test]
fn test_g2bfa3_fmt_keeps_non_callee_path_and_dot_field() {
    let out = fmt("let m = i32::MAX\nlet f = p.x\n");
    assert!(out.contains("i32::MAX"), "fmt output: {out}");
    assert!(out.contains("p.x"), "fmt output: {out}");
    assert!(!out.contains("p::x"), "fmt output: {out}");
}

#[test]
fn test_g2bfa3_fmt_round_trip_is_stable() {
    let once = fmt("fun main() {\n    let v = helpers::double(4)\n}\n");
    let twice = fmt(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_g2bfa3_lowercase_root_path_constant_transpiles_with_colons() {
    let code = "fun main() {\n    let m = i32::MAX\n    println(m)\n}\n";
    let rust = squash(&transpile(code));
    assert!(rust.contains("i32::MAX"), "{rust}");
    assert_eq!(compiled_output(code, "fa3_max"), "2147483647\n");
}

// ---------------------------------------------- 4. user-defined `range`

const USER_RANGE: &str = r"
fun range(a: i32, b: i32) -> Vec<i32> {
    vec![a, b]
}
fun main() {
    println(range(0, 5).count())
    println(range(0, 5).sum())
}
";

#[test]
fn test_g2bfa4_user_defined_range_is_called_and_iterated() {
    let rust = squash(&transpile(USER_RANGE));
    assert!(rust.contains("range(0,5).iter().count()"), "{rust}");
    assert_eq!(compiled_output(USER_RANGE, "fa4_user_range"), "2\n5\n");
}

#[test]
fn test_g2bfa4_builtin_range_count_still_direct() {
    let code = "fun main() {\n    let n = range(0, 4).count()\n    println(n)\n}\n";
    assert_eq!(compiled_output(code, "fa4_builtin_range"), "4\n");
}

// --------------------------------------- 5. awaiting main return type

#[test]
fn test_g2bfa5_async_main_with_result_return_type_compiles() {
    let code = r"
async fun value() -> i32 {
    5
}
async fun main() -> Result<(), String> {
    let v = value().await
    println(v)
    Ok(())
}
";
    assert_eq!(compiled_output(code, "fa5_result_main"), "5\n");
}

#[test]
fn test_g2bfa5_async_main_propagates_error_exit() {
    let code = r#"
async fun check(x: i32) -> Result<i32, String> {
    if x > 0 { Ok(x) } else { Err(String::from("neg")) }
}
async fun main() -> Result<(), String> {
    let v = check(3).await?
    println(v)
    Ok(())
}
"#;
    assert_eq!(compiled_output(code, "fa5_try_main"), "3\n");
}
