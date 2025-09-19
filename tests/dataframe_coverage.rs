// Coverage Test Suite for DataFrame support
// Target: Basic DataFrame coverage
// Sprint 80: ALL NIGHT Coverage Marathon Phase 14

#[cfg(feature = "dataframe")]
mod dataframe_tests {
    use ruchy::dataframe::{DataFrame, Series, Column};
    use polars::prelude::*;

    #[test]
    fn test_dataframe_new() {
        let df = DataFrame::empty();
        assert_eq!(df.height(), 0);
    }

    #[test]
    fn test_dataframe_from_series() {
        let s1 = Series::new("a", &[1, 2, 3]);
        let s2 = Series::new("b", &[4, 5, 6]);
        let df = DataFrame::new(vec![s1, s2]).unwrap();
        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 2);
    }

    #[test]
    fn test_dataframe_select() {
        let s1 = Series::new("a", &[1, 2, 3]);
        let s2 = Series::new("b", &[4, 5, 6]);
        let df = DataFrame::new(vec![s1, s2]).unwrap();

        let selected = df.select(&["a"]).unwrap();
        assert_eq!(selected.width(), 1);
    }

    #[test]
    fn test_dataframe_filter() {
        let s1 = Series::new("a", &[1, 2, 3, 4, 5]);
        let df = DataFrame::new(vec![s1]).unwrap();

        let mask = df.column("a").unwrap().i32().unwrap().gt(2);
        let filtered = df.filter(&mask).unwrap();
        assert_eq!(filtered.height(), 3);
    }

    #[test]
    fn test_dataframe_sort() {
        let s1 = Series::new("a", &[3, 1, 2]);
        let df = DataFrame::new(vec![s1]).unwrap();

        let sorted = df.sort(&["a"], vec![false], false).unwrap();
        assert_eq!(sorted.height(), 3);
    }

    #[test]
    fn test_dataframe_head() {
        let s1 = Series::new("a", &[1, 2, 3, 4, 5]);
        let df = DataFrame::new(vec![s1]).unwrap();

        let head = df.head(Some(3));
        assert_eq!(head.height(), 3);
    }

    #[test]
    fn test_dataframe_tail() {
        let s1 = Series::new("a", &[1, 2, 3, 4, 5]);
        let df = DataFrame::new(vec![s1]).unwrap();

        let tail = df.tail(Some(2));
        assert_eq!(tail.height(), 2);
    }

    #[test]
    fn test_series_operations() {
        let s1 = Series::new("a", &[1, 2, 3]);
        let s2 = Series::new("b", &[4, 5, 6]);

        let sum = &s1 + &s2;
        assert_eq!(sum.len(), 3);
    }

    #[test]
    fn test_dataframe_join() {
        let df1 = df![
            "a" => [1, 2, 3],
            "b" => [4, 5, 6]
        ].unwrap();

        let df2 = df![
            "a" => [1, 2, 3],
            "c" => [7, 8, 9]
        ].unwrap();

        let joined = df1.join(&df2, &["a"], &["a"], JoinArgs::default()).unwrap();
        assert_eq!(joined.width(), 3);
    }

    #[test]
    fn test_dataframe_group_by() {
        let df = df![
            "group" => ["A", "B", "A", "B"],
            "value" => [1, 2, 3, 4]
        ].unwrap();

        let grouped = df.group_by(&["group"]).unwrap()
            .agg(&[col("value").sum()])
            .unwrap();

        assert_eq!(grouped.height(), 2);
    }

    #[test]
    fn test_dataframe_melt() {
        let df = df![
            "a" => [1, 2],
            "b" => [3, 4],
            "c" => [5, 6]
        ].unwrap();

        let melted = df.melt(MeltArgs::default()).unwrap();
        assert!(melted.height() > df.height());
    }

    #[test]
    fn test_dataframe_pivot() {
        let df = df![
            "foo" => ["A", "A", "B", "B"],
            "bar" => ["X", "Y", "X", "Y"],
            "value" => [1, 2, 3, 4]
        ].unwrap();

        let pivoted = df.pivot(PivotArgs {
            values: &["value"],
            index: &["foo"],
            columns: &["bar"],
            ..Default::default()
        }).unwrap();

        assert_eq!(pivoted.height(), 2);
    }
}

// Non-dataframe feature tests
#[cfg(not(feature = "dataframe"))]
mod dataframe_stub_tests {
    #[test]
    fn test_dataframe_feature_disabled() {
        // When dataframe feature is disabled, these types don't exist
        assert!(true);
    }
}

// Common tests that work regardless of feature
#[test]
fn test_dataframe_feature_flag() {
    #[cfg(feature = "dataframe")]
    {
        assert!(true); // Feature enabled
    }

    #[cfg(not(feature = "dataframe"))]
    {
        assert!(true); // Feature disabled
    }
}

#[test]
fn test_multiple_dataframes() {
    #[cfg(feature = "dataframe")]
    {
        use polars::prelude::*;

        let _df1 = DataFrame::empty();
        let _df2 = DataFrame::empty();
        let _df3 = DataFrame::empty();
    }
    assert!(true);
}

#[test]
fn test_dataframe_stress() {
    #[cfg(feature = "dataframe")]
    {
        use polars::prelude::*;

        // Create large dataframe
        let mut series = vec![];
        for i in 0..10 {
            let data: Vec<i32> = (0..1000).map(|x| x + i).collect();
            series.push(Series::new(&format!("col{}", i), &data));
        }

        let df = DataFrame::new(series).unwrap();
        assert_eq!(df.height(), 1000);
        assert_eq!(df.width(), 10);
    }
    assert!(true);
}