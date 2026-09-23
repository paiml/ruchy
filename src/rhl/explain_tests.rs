//! `explain` (RHL-9 verb, RHL-6 tool): byte-determinism and the rendering rules.

use super::{explain, explain_file, tree_of};
use crate::rhl::repo_root;

const VALID: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";

fn explain_text(source: &str) -> String {
    explain(&crate::rhl::parse(source).expect("parses"))
}

#[test]
fn test_rhl9_explain_renders_a_job_one_plain_line_per_statement() {
    let source = std::fs::read_to_string(repo_root().join(VALID)).expect("read");
    let got = explain_text(&source);
    assert!(got.starts_with(
        "Uses vocabulary fleet v2.\nUses vocabulary tickets v2.\n\nJob \"gx10 disk watch\":\n"
    ));
    assert!(got.contains("\n  Sets free to disk free of \"/\".\n"));
    assert!(got.contains("\n  When free is below 100 GB:\n    Does file ticket in repo \"paiml/infra\", with:\n      title: \"gx10 disk watch\"\n"));
    assert!(got.ends_with("    Then ticket count is 1.\n"));
    assert_eq!(got, explain_text(&source), "explain is byte-deterministic");
}

#[test]
fn test_rhl9_explain_renders_every_block_form() {
    let source = "\
job \"sweep\"
  wait up to 5 minute
  for each r in runners
    when load of r is above 90 percent
      restart runner
    otherwise
      stop with \"ok\"
    end
  end
  repeat at most 3 times until done
    set done to 1
  end
  give back 1
end
";
    let expected = "\
Job \"sweep\":
  Waits up to 5 minute.
  For each r in runners:
    When load of r is above 90 percent:
      Does restart runner.
    Otherwise:
      Stops with \"ok\".
  Repeats at most 3 times until done:
    Changes done to 1.
  Gives back 1.
";
    assert_eq!(explain_text(source), expected);
}

#[test]
fn test_rhl9_explain_names_each_unit_kind_and_separates_units() {
    let source = "command \"a\"\n  give back 1\nend\n\ncheck \"b\"\n  expect 1 is 1\nend\n";
    assert_eq!(
        explain_text(source),
        "Command \"a\":\n  Gives back 1.\n\nCheck \"b\":\n  Expects 1 is 1.\n"
    );
}

#[test]
fn test_rhl9_explain_reads_yaml_to_the_same_text() {
    let source = std::fs::read_to_string(repo_root().join(VALID)).expect("read");
    let program = crate::rhl::parse(&source).expect("parses");
    let yaml = crate::rhl::yaml::to_yaml(&program);
    let from_yaml = tree_of("x.rhl.yaml", &yaml).expect("yaml tree");
    assert_eq!(explain(&from_yaml), explain(&program));
}

#[test]
fn test_rhl9_explain_file_exit_codes() {
    let ok = explain_file(&repo_root().join(VALID));
    assert_eq!((ok.exit, ok.stderr.as_str()), (0, ""));
    assert!(ok.stdout.starts_with("Uses vocabulary"));
    let dir = tempfile::tempdir().expect("temp dir");
    let broken = dir.path().join("x.rhl");
    std::fs::write(&broken, "job \"a\"\n  every 1 hour\n").expect("write");
    let refused = explain_file(&broken);
    assert_eq!(refused.exit, 2);
    assert!(refused.stderr.contains("RHL-P"), "{}", refused.stderr);
    assert!(refused.stdout.is_empty());
    let missing = explain_file(&dir.path().join("absent.rhl"));
    assert_eq!(missing.exit, 1);
    assert!(missing.stderr.contains("cannot read"));
}
