//! RHL-6 and MCPTOOLS-1: the §6 MCP tools of `ruchy mcp` (spec RHL-001 §6,
//! `docs/rhl/rhl-6.md`) and the `ruchy explain` verb (§5).
//!
//! Two layers. The end-to-end tests spawn the built `ruchy mcp`, speak
//! JSON-RPC over its standard input and output, and read every reply under a
//! hard timeout, so a server that hangs fails the test instead of blocking it.
//! The library tests call each tool handler with JSON and check the JSON it
//! returns, including that it is the same value the CLI verb computes.
#![cfg(feature = "mcp")]

use ruchy::mcp::rhl_tools::{
    rhl_check, rhl_convert, rhl_explain, rhl_fix, rhl_format, rhl_grammar, rhl_transpile,
    rhl_vocabulary, RHL_TOOL_NAMES,
};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

const VALID: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";
const TYPO: &str = "docs/rhl/breaks/v2/planted/typo-term/01-gx10-disk-watch/broken.rhl";
const TYPO3: &str = "docs/rhl/breaks/v2/planted/typo-term/03-gx12-runner-load-watch/broken.rhl";
const TYPO3_BASE: &str = "docs/rhl/breaks/v2/valid/03-gx12-runner-load-watch.rhl";
const MISSING_END: &str = "docs/rhl/breaks/v2/planted/missing-end/01-gx10-disk-watch/broken.rhl";
const REPLY_TIMEOUT: Duration = Duration::from_secs(30);

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).expect("read corpus file")
}

/// `{source, file_name, root}` for the corpus file `rel`.
fn args_for(rel: &str) -> Value {
    json!({
        "source": read(rel),
        "file_name": repo().join(rel).display().to_string(),
        "root": repo().display().to_string(),
    })
}

fn codes(report: &Value) -> Vec<String> {
    report["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .map(|d| d["code"].as_str().expect("code").to_string())
        .collect()
}

// ───────────────────────── end to end: `ruchy mcp` over stdio ─────────────────────────

/// A running `ruchy mcp`; killed on drop.
struct Session {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
}

impl Session {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ruchy"))
            .arg("mcp")
            .current_dir(repo())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ruchy mcp");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let (tx, lines) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let mut session = Self {
            child,
            stdin,
            lines,
        };
        session.handshake();
        session
    }

    fn send(&mut self, message: &Value) {
        writeln!(self.stdin, "{message}").expect("write request");
        self.stdin.flush().expect("flush request");
    }

    /// Send a request and wait, at most [`REPLY_TIMEOUT`], for its reply.
    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let line = self
                .lines
                .recv_timeout(REPLY_TIMEOUT)
                .unwrap_or_else(|e| panic!("no reply to `{method}` within {REPLY_TIMEOUT:?}: {e}"));
            let Ok(message) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if message["id"] == json!(id) {
                return message;
            }
        }
    }

    fn handshake(&mut self) {
        let reply = self.request(
            1,
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "rhl_mcp test", "version": "0"},
            }),
        );
        assert!(
            reply["result"]["capabilities"]["tools"].is_object(),
            "initialize must declare tools: {reply}"
        );
        self.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    }

    /// `tools/call name args`, and the JSON the tool returned in its text content.
    fn call(&mut self, id: u64, name: &str, args: Value) -> Value {
        let reply = self.request(id, "tools/call", json!({"name": name, "arguments": args}));
        let text = reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("`{name}` returned no text content: {reply}"));
        serde_json::from_str(text).expect("tool content is JSON")
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn test_rhl6_mcp_tools_list_registers_the_eight_rhl_tools_and_the_ruchy_tools() {
    let mut session = Session::start();
    let reply = session.request(2, "tools/list", json!({}));
    let names: Vec<&str> = reply["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("tools/list returned no tools: {reply}"))
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for name in RHL_TOOL_NAMES
        .iter()
        .chain(&["ruchy-score", "ruchy-transpile"])
    {
        assert!(names.contains(name), "`{name}` missing from {names:?}");
    }
}

#[test]
fn test_rhl6_mcp_tools_call_rhl_check_reports_v002_for_a_typo() {
    let mut session = Session::start();
    // No `root`: the server finds `vocab/` from its working directory.
    let report = session.call(
        2,
        "rhl_check",
        json!({"source": read(TYPO), "file_name": "broken.rhl"}),
    );
    assert_eq!(report["verdict"], "refused", "{report}");
    assert!(codes(&report).contains(&"RHL-V002".to_string()), "{report}");
}

#[test]
fn test_rhl6_explain_verb_is_byte_deterministic_and_matches_the_tool() {
    let run = || {
        let out = Command::new(env!("CARGO_BIN_EXE_ruchy"))
            .arg("explain")
            .arg(repo().join(VALID))
            .output()
            .expect("run ruchy explain");
        assert_eq!(out.status.code(), Some(0), "{out:?}");
        String::from_utf8(out.stdout).expect("utf-8")
    };
    let first = run();
    assert_eq!(first, run());
    let tool = rhl_explain(&args_for(VALID)).expect("rhl_explain");
    assert_eq!(tool["text"], first);
}

// ───────────────────────── library: each tool's JSON in and out ─────────────────────────

#[test]
fn test_rhl6_rhl_check_is_the_check_verbs_json_report() {
    let tool = rhl_check(&args_for(TYPO)).expect("rhl_check");
    let path = repo().join(TYPO);
    let cli = ruchy::rhl::cli::check_files(&[path.as_path()], ruchy::rhl::cli::OutputFormat::Json);
    let verb: Value = serde_json::from_str(&cli.stdout).expect("check --format json");
    assert_eq!(tool, verb);
    assert!(codes(&tool).contains(&"RHL-V002".to_string()));
}

#[test]
fn test_rhl6_rhl_fix_returns_the_fixed_source_and_writes_nothing() {
    let before = read(TYPO3);
    let fixed = rhl_fix(&args_for(TYPO3)).expect("rhl_fix");
    assert_eq!(fixed["fixed_source"], read(TYPO3_BASE));
    assert_eq!(fixed["applied"][0]["code"], "RHL-V002");
    assert_eq!(fixed["report"]["verdict"], "pass");
    assert_eq!(read(TYPO3), before, "rhl_fix must not write the file");
}

#[test]
fn test_rhl6_rhl_format_returns_the_normal_form_or_the_report() {
    let messy = read(VALID).replace("  every 1 hour", "  every   1   hour");
    let formatted = rhl_format(&json!({"source": messy})).expect("rhl_format");
    assert_eq!(formatted["source"], read(VALID));
    let broken = rhl_format(&args_for(MISSING_END)).expect("rhl_format");
    assert!(broken.get("source").is_none(), "{broken}");
    assert!(codes(&broken["report"])[0].starts_with("RHL-P"), "{broken}");
}

#[test]
fn test_rhl6_rhl_convert_round_trips_rhl_and_yaml() {
    let yaml = rhl_convert(&json!({"source": read(VALID), "to": "yaml"})).expect("to yaml");
    assert_eq!(yaml["to"], "yaml");
    let text = yaml["output"].as_str().expect("yaml output");
    let back = rhl_convert(&json!({"source": text, "to": "rhl"})).expect("to rhl");
    assert_eq!(back["to"], "rhl");
    assert_eq!(back["output"], read(VALID));
}

#[test]
fn test_rhl6_rhl_transpile_emits_ruchy_and_rust_and_refuses_as_a_report() {
    let mut args = args_for(VALID);
    args["emit"] = json!("ruchy");
    let ruchy = rhl_transpile(&args).expect("emit ruchy");
    assert!(ruchy["ruchy_source"]
        .as_str()
        .expect("ruchy_source")
        .contains("fun "));
    args["emit"] = json!("rust");
    let rust = rhl_transpile(&args).expect("emit rust");
    assert!(rust["rust_source"]
        .as_str()
        .expect("rust_source")
        .contains("fn "));
    let refused = rhl_transpile(&args_for(TYPO)).expect("typo");
    assert!(
        codes(&refused["report"]).contains(&"RHL-V002".to_string()),
        "{refused}"
    );
    assert!(refused.get("rust_source").is_none());
}

#[test]
fn test_rhl6_rhl_explain_renders_the_tree_in_plain_lines() {
    let explained = rhl_explain(&args_for(VALID)).expect("rhl_explain");
    let expected = "\
Uses vocabulary fleet v2.
Uses vocabulary tickets v2.

Job \"gx10 disk watch\":
  Runs on host gx10.
  Runs every 1 hour.
  May read disk.
  May write tickets.
  Sets free to disk free of \"/\".
  When free is below 100 GB:
    Does file ticket in repo \"paiml/infra\", with:
      title: \"gx10 disk watch\"
      label: \"fleet\"
  Expects ticket count is at most 1.
  Example \"gx10 disk watch example\":
    Given free is 90 GB.
    Then ticket count is 1.
";
    assert_eq!(explained["text"], expected);
}

#[test]
fn test_rhl6_rhl_vocabulary_filters_terms_by_kind_gives_and_query() {
    let root = repo().display().to_string();
    let rows = rhl_vocabulary(&json!({"root": root, "kind": "measure", "gives": "Size"}))
        .expect("rhl_vocabulary");
    let rows = rows.as_array().expect("array of terms");
    assert!(rows
        .iter()
        .any(|r| r["term"] == "disk free of" && r["version"] == 2));
    assert!(rows
        .iter()
        .all(|r| r["kind"] == "measure" && r["gives"] == "Size"));
    let all = rhl_vocabulary(&json!({"root": root})).expect("unfiltered");
    let all = all.as_array().expect("array");
    assert!(all.len() > rows.len());
    for key in [
        "vocabulary",
        "version",
        "term",
        "kind",
        "takes",
        "gives",
        "effect",
    ] {
        assert!(all[0].get(key).is_some(), "row lacks `{key}`: {}", all[0]);
    }
    let ticket = rhl_vocabulary(&json!({"root": root, "query": "ticket"})).expect("query");
    let ticket = ticket.as_array().expect("array");
    assert!(!ticket.is_empty());
    assert!(ticket
        .iter()
        .all(|r| r["term"].as_str().unwrap_or("").contains("ticket")));
}

#[test]
fn test_rhl6_rhl_grammar_returns_the_generated_parsers_grammar() {
    let grammar = rhl_grammar(&json!({})).expect("rhl_grammar");
    let on_disk = std::fs::read_to_string(repo().join("src/rhl/grammar.lalrpop")).expect("grammar");
    assert_eq!(grammar["grammar"], on_disk);
}
