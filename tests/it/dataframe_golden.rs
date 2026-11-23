#![cfg(feature = "dataframe")]
//! Golden tests for the `dataframe` feature (polars + arrow) — [PMAT-101]
//!
//! These tests only exist when the crate is compiled with `--features dataframe`.
//! They are the falsification tests for `contracts/feature-matrix-v1.yaml`:
//! if `src/stdlib/dataframe.rs` or `src/backend/arrow_integration.rs` drift away
//! from the locked polars API, this test target stops compiling and the golden
//! stdout below stops being produced.
//!
//! Run with: `cargo test --features dataframe --test dataframe_golden`

use assert_cmd::Command;
use ruchy::stdlib::dataframe;
use tempfile::TempDir;

/// Helper: the repository's own `ruchy` binary (never a binary resolved from PATH).
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// The exact stdout `examples/dataframe/36_dataframe_basics.ruchy` must produce.
const GOLDEN_STDOUT: &str = "\
rows=3
cols=3
first_col=name
last_col=score
selected_cols=1
selected_rows=3
name0=alice
age2=35
score1=82
age_sum=90
";

#[test]
fn test_pmat_101_example_basics_exact_stdout() {
    let output = ruchy_cmd()
        .arg("run")
        .arg("examples/dataframe/36_dataframe_basics.ruchy")
        .output()
        .expect("failed to run the ruchy binary");

    assert!(
        output.status.success(),
        "ruchy run failed: status={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    assert_eq!(stdout, GOLDEN_STDOUT, "golden stdout drifted");
}

#[test]
fn test_pmat_101_example_basics_is_deterministic() {
    let run_once = || {
        let output = ruchy_cmd()
            .arg("run")
            .arg("examples/dataframe/36_dataframe_basics.ruchy")
            .output()
            .expect("failed to run the ruchy binary");
        String::from_utf8(output.stdout).expect("stdout was not valid UTF-8")
    };

    assert_eq!(
        run_once(),
        run_once(),
        "example output is not deterministic"
    );
}

#[test]
fn test_pmat_101_csv_roundtrip_preserves_two_row_frame() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let path = temp.path().join("roundtrip.csv");
    let path_str = path.to_str().expect("temp path was not valid UTF-8");

    let mut written = dataframe::from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])])
        .expect("from_columns should build a 2x2 frame");
    dataframe::write_csv(&mut written, path_str).expect("write_csv should succeed");

    let read_back = dataframe::read_csv(path_str).expect("read_csv should succeed");

    assert_eq!(dataframe::shape(&read_back).expect("shape"), (2, 2));
    assert_eq!(
        dataframe::columns(&read_back).expect("columns"),
        vec!["age".to_string(), "score".to_string()]
    );
    assert_eq!(dataframe::row_count(&read_back).expect("row_count"), 2);
    assert_eq!(
        dataframe::columns(&written).expect("columns"),
        dataframe::columns(&read_back).expect("columns"),
        "CSV round-trip changed the column set"
    );
}

#[test]
fn test_pmat_101_arrow_roundtrip_preserves_nulls() {
    use polars::prelude::{NamedFrom, Series};

    // A column with a null in the middle: the null must survive the Arrow hop
    // and must NOT silently become 0.
    let series: Series = Series::new("v".into(), &[Some(1_i64), None, Some(3_i64)]);
    let df = polars::prelude::DataFrame::new_infer_height(vec![series.into()])
        .expect("DataFrame::new_infer_height should succeed");

    let batch = ruchy::backend::arrow_integration::dataframe_to_arrow(&df)
        .expect("dataframe_to_arrow should succeed");
    let back = ruchy::backend::arrow_integration::arrow_to_dataframe(&batch)
        .expect("arrow_to_dataframe should succeed");

    assert_eq!(back.height(), 3);
    let col = back.column("v").expect("column v should exist");
    let values: Vec<Option<i64>> = col
        .as_materialized_series()
        .i64()
        .expect("column v should be i64")
        .iter()
        .collect();
    assert_eq!(
        values,
        vec![Some(1), None, Some(3)],
        "null was not preserved"
    );
}
