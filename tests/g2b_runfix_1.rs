//! RHLGA-1 G2b run fixes: three interpreter defects measured with `ruchy run`.
//!
//! - IDXCOMPOUND-1: compound assignment to an index target (`v[1] += 5`), a
//!   field of an element (`o.items[0].z += 1`) and an element of a field
//!   (`o.xs[1] -= 2`) failed with "Invalid compound assignment target".
//! - IDXPUSHWB-1: an in-place array method on a chain that contains an index
//!   (`t.rows[0].vals.push(9)`, `m[0].push(x)`) changed a copy and was lost.
//! - COUNTITER-1 (interpreter half): `"abc".chars().count()` failed with
//!   "Unknown array method: count".
//!
//! Each case states the stdout that Rust prints for the same program. A
//! hand-written Rust reference is compiled with rustc to prove that string,
//! and `ruchy run` must print it too.
//!
//! `ruchy compile` output is not compared here: at fefd3886 the transpiler
//! emits `v[1 as usize].clone() += 5` and `t.rows[0 as usize].clone().vals
//! .push(9)` (rustc E0067 / a lost write) and `.chars().iter().count()`
//! (E0599). Those are the transpiler halves, fixed in another worktree; every
//! program in this file uses a feature one of them breaks.

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

fn ruchy_run(dir: &Path, src: &str) -> String {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.arg("run").arg(file).timeout(Duration::from_secs(10));
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
fn assert_matches_rust(ruchy: &str, rust: &str, expected: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(rustc_run(dir.path(), rust), expected, "Rust reference");
    assert_eq!(ruchy_run(dir.path(), ruchy), expected, "ruchy run");
}

// ---------------------------------------------------------------- IDXCOMPOUND-1

#[test]
fn test_idxcompound_1_local_array_all_operators() {
    let ruchy = r#"
fun main() {
    let mut v = [10, 20, 30, 40, 50]
    v[0] += 5
    v[1] -= 3
    v[2] *= 2
    v[3] /= 3
    v[4] %= 7
    println("{} {} {} {} {}", v[0], v[1], v[2], v[3], v[4])
}
"#;
    let rust = r#"
fn main() {
    let mut v = [10, 20, 30, 40, 50];
    v[0] += 5;
    v[1] -= 3;
    v[2] *= 2;
    v[3] /= 3;
    v[4] %= 7;
    println!("{} {} {} {} {}", v[0], v[1], v[2], v[3], v[4]);
}
"#;
    assert_matches_rust(ruchy, rust, "15 17 60 13 1\n");
}

#[test]
fn test_idxcompound_1_variable_index_in_loop() {
    let ruchy = r#"
fun main() {
    let mut v = [1, 1, 1, 1]
    for i in 0..4 {
        v[i] += i * 10
    }
    println("{} {} {} {}", v[0], v[1], v[2], v[3])
}
"#;
    let rust = r#"
fn main() {
    let mut v = [1, 1, 1, 1];
    for i in 0..4 {
        v[i] += i * 10;
    }
    println!("{} {} {} {}", v[0], v[1], v[2], v[3]);
}
"#;
    assert_matches_rust(ruchy, rust, "1 11 21 31\n");
}

#[test]
fn test_idxcompound_1_field_of_element_and_element_of_field() {
    let ruchy = r#"
struct P { z: i32 }
struct O { items: Vec<P>, xs: Vec<i32> }
fun main() {
    let mut o = O { items: vec![P { z: 1 }, P { z: 7 }], xs: vec![10, 20, 30] }
    o.items[0].z += 1
    o.items[1].z *= 3
    o.xs[1] -= 2
    o.xs[2] /= 4
    println("{} {} {} {} {}", o.items[0].z, o.items[1].z, o.xs[0], o.xs[1], o.xs[2])
}
"#;
    let rust = r#"
struct P { z: i32 }
struct O { items: Vec<P>, xs: Vec<i32> }
fn main() {
    let mut o = O { items: vec![P { z: 1 }, P { z: 7 }], xs: vec![10, 20, 30] };
    o.items[0].z += 1;
    o.items[1].z *= 3;
    o.xs[1] -= 2;
    o.xs[2] /= 4;
    println!("{} {} {} {} {}", o.items[0].z, o.items[1].z, o.xs[0], o.xs[1], o.xs[2]);
}
"#;
    assert_matches_rust(ruchy, rust, "2 21 10 18 7\n");
}

#[test]
fn test_idxcompound_1_nested_index_and_index_of_nested_field() {
    let ruchy = r#"
struct G { grid: Vec<Vec<i32>> }
fun main() {
    let mut m = vec![vec![1, 2], vec![3, 4]]
    m[1][0] += 100
    let mut g = G { grid: vec![vec![5, 6], vec![7, 8]] }
    g.grid[0][1] -= 6
    println("{} {} {}", m[1][0], g.grid[0][1], g.grid[1][1])
}
"#;
    let rust = r#"
struct G { grid: Vec<Vec<i32>> }
fn main() {
    let mut m = vec![vec![1, 2], vec![3, 4]];
    m[1][0] += 100;
    let mut g = G { grid: vec![vec![5, 6], vec![7, 8]] };
    g.grid[0][1] -= 6;
    println!("{} {} {}", m[1][0], g.grid[0][1], g.grid[1][1]);
}
"#;
    assert_matches_rust(ruchy, rust, "103 0 8\n");
}

// ---------------------------------------------------------------- IDXPUSHWB-1

#[test]
fn test_idxpushwb_1_push_through_field_index_field_chain() {
    let ruchy = r#"
struct R { vals: Vec<i32> }
struct T { rows: Vec<R> }
fun main() {
    let mut t = T { rows: vec![R { vals: vec![1] }] }
    t.rows[0].vals.push(9)
    println("{:?}", t.rows[0].vals)
}
"#;
    let rust = r#"
struct R { vals: Vec<i32> }
struct T { rows: Vec<R> }
fn main() {
    let mut t = T { rows: vec![R { vals: vec![1] }] };
    t.rows[0].vals.push(9);
    println!("{:?}", t.rows[0].vals);
}
"#;
    assert_matches_rust(ruchy, rust, "[1, 9]\n");
}

#[test]
fn test_idxpushwb_1_methods_on_element_of_local_array() {
    let ruchy = r#"
fun main() {
    let mut m = vec![vec![3, 1, 2], vec![5]]
    m[0].push(0)
    m[0].sort()
    let i = 1
    m[i].insert(0, 4)
    let p = m[i].pop()
    println("{:?} {:?} {:?}", m[0], m[1], p)
}
"#;
    let rust = r#"
fn main() {
    let mut m = vec![vec![3, 1, 2], vec![5]];
    m[0].push(0);
    m[0].sort();
    let i = 1;
    m[i].insert(0, 4);
    let p = m[i].pop();
    println!("{:?} {:?} {:?}", m[0], m[1], p);
}
"#;
    assert_matches_rust(ruchy, rust, "[0, 1, 2, 3] [4] Some(5)\n");
}

#[test]
fn test_idxpushwb_1_sort_and_remove_through_variable_index() {
    let ruchy = r#"
struct R { vals: Vec<i32> }
struct T { rows: Vec<R> }
fun main() {
    let mut t = T { rows: vec![R { vals: vec![1] }, R { vals: vec![9, 4, 6] }] }
    let i = 1
    t.rows[i].vals.sort()
    let r = t.rows[i].vals.remove(0)
    println("{} {:?} {:?}", r, t.rows[i].vals, t.rows[0].vals)
}
"#;
    let rust = r#"
struct R { vals: Vec<i32> }
struct T { rows: Vec<R> }
fn main() {
    let mut t = T { rows: vec![R { vals: vec![1] }, R { vals: vec![9, 4, 6] }] };
    let i = 1;
    t.rows[i].vals.sort();
    let r = t.rows[i].vals.remove(0);
    println!("{} {:?} {:?}", r, t.rows[i].vals, t.rows[0].vals);
}
"#;
    assert_matches_rust(ruchy, rust, "4 [6, 9] [1]\n");
}

// ---------------------------------------------------------------- COUNTITER-1

#[test]
fn test_countiter_1_chars_count_matches_rust() {
    let ruchy = r#"
fun main() {
    let s = "abc"
    let n = s.chars().count()
    let u = "héllo".chars().count()
    println("{} {} {}", n, u, "".chars().count())
}
"#;
    let rust = r#"
fn main() {
    let s = "abc";
    let n = s.chars().count();
    let u = "héllo".chars().count();
    println!("{} {} {}", n, u, "".chars().count());
}
"#;
    assert_matches_rust(ruchy, rust, "3 5 0\n");
}

#[test]
fn test_countiter_1_iter_and_bytes_count_matches_rust() {
    let ruchy = r#"
fun main() {
    let v = vec![1, 2, 3, 4]
    let e: Vec<i32> = vec![]
    println("{} {} {}", v.iter().count(), e.iter().count(), "héllo".bytes().count())
}
"#;
    let rust = r#"
fn main() {
    let v = vec![1, 2, 3, 4];
    let e: Vec<i32> = vec![];
    println!("{} {} {}", v.iter().count(), e.iter().count(), "héllo".bytes().count());
}
"#;
    assert_matches_rust(ruchy, rust, "4 0 6\n");
}
