# RHL-6: the RHL tools of `ruchy mcp` (and RHL-9's `ruchy explain`)

Spec: RHL-001 §6 (MCP tools), §5 (`ruchy explain`), §9.4 (channel text is data;
output is a draft, never an action). Defect fixed alongside: MCPTOOLS-1, where
`ruchy mcp` built a server with no tools registered.

## Server

`ruchy mcp` serves over stdio (newline-delimited JSON-RPC, pmcp 1.20.0). The
server is built by `ruchy::mcp::create_named_mcp_server`, which registers every
tool of `ruchy::mcp::all_tools()`: the seven `ruchy-*` tools and the eight
`rhl_*` tools below. `tools/list` reports each tool's name, description and
input schema. A tool's result is its JSON value, serialized as the text of the
single `content` item of the `tools/call` result.

## Tools

Every tool calls the library function its CLI verb calls, so the tool and the
verb cannot disagree. No tool writes a file or acts on the world.

Common arguments: `source` (required, the program text), `file_name` (default
`input.rhl`; a `.rhl.yaml` name selects the YAML surface), `root` (the
vocabulary root; default: the nearest ancestor of the server's working
directory holding `vocab/`, as the CLI finds it).

A program a tool cannot take is data, not an error: `{"report": R}`, where `R`
is the §3.5 check report. Only a malformed request (no `source`, an unknown
`to` or `emit`, no vocabulary root for `rhl_vocabulary`) is a JSON-RPC error.

| Tool | Input | Output | Library call (CLI verb) |
|---|---|---|---|
| `rhl_check` | `{source, file_name?, root?}` | the §3.5 report `{file, verdict, diagnostics, unverified}` — equal to `ruchy check --format json` | `rhl::cli::check_source` (`ruchy check`) |
| `rhl_fix` | `{source, file_name?, root?}` | `{fixed_source, applied, reverted, rounds, report}`; `report` checks `fixed_source`; nothing is written | `rhl::fix::fix_safe` (`ruchy fix --safe`) |
| `rhl_format` | `{source, file_name?}` | `{source}` in normal form, or `{report}` | `rhl::cli::format_file_source` (`ruchy fmt`) |
| `rhl_convert` | `{source, to?: "rhl"\|"yaml", file_name?}` | `{output, to}`, or `{report}`; without `to`, the other surface than `file_name`'s | `rhl::cli::convert_source` (`ruchy convert`) |
| `rhl_transpile` | `{source, emit?: "ruchy"\|"rust", file_name?, root?}` | `{ruchy_source}` or `{rust_source}` (default `rust`); a program that does not check, or an `RHL-L…` lowering refusal, is `{report}`; a ruchy→Rust failure is `{error: {stage: "rust", message}}` | `rhl::cli::lower_named` + `rhl::lower::to_rust` (`ruchy transpile`) |
| `rhl_explain` | `{source, file_name?}` | `{text}`, byte-identical to `ruchy explain`'s standard output, or `{report}` | `rhl::explain::explain` (`ruchy explain`) |
| `rhl_vocabulary` | `{kind?, gives?, query?, root?}` | `[{vocabulary, version, term, kind, takes, gives, effect}]`: every term of every loadable vocabulary under the root with that `kind`, that `gives` type, and `query` inside its spelling; by vocabulary, version, file order | `rhl::vocab_cli::list` + `rhl::vocab::load` (`ruchy vocab`) |
| `rhl_grammar` | `{}` | `{grammar}`: the text of `src/rhl/grammar.lalrpop`, the grammar the parser is generated from | — |

`rhl_transpile` does not offer `emit: "contract"`; `ruchy transpile --emit
contract` remains the way to get a unit contract.

## `explain` rules

`ruchy explain <file.rhl | file.rhl.yaml>` prints the rendering and exits 0; a
file that is not a tree prints its `RHL-P…` diagnostic on standard error and
exits 2; an unreadable file exits 1. The rendering is a function of the tree
alone (no model, no check, no vocabulary): the same tree gives the same bytes,
and a `.rhl` file and its `.rhl.yaml` give the same text.

- `use vocabulary N vK` → `Uses vocabulary N vK.`
- A unit → `Job "name":` (`Command`, `Check`, `Pipeline`, `Shape`), preceded by
  one blank line unless it is the first line; its body is indented two spaces
  per block depth.
- Expressions (conditions, applications, quantities, texts) are printed in
  their RHL spelling, as `ruchy fmt` prints them.

| Statement | Line |
|---|---|
| `runs on X` | `Runs on X.` |
| `every Q` | `Runs every Q.` |
| `may V T` | `May V T.` |
| `wait up to Q` | `Waits up to Q.` |
| `let n be E` | `Sets n to E.` |
| `set n to E` | `Changes n to E.` |
| `expect C` / `give back C` / `stop with C` | `Expects C.` / `Gives back C.` / `Stops with C.` |
| `given C` / `then C` | `Given C.` / `Then C.` |
| action `A` / `A in T` | `Does A.` / `Does A in T.` |
| action with a `with` block | `Does A in T, with:` then one `name: value` line per attribute (an attribute is an action with arguments and no target or block of its own); any other statement in the block is rendered by these rules |
| `when C … otherwise … end` | `When C:` body, `Otherwise:` body |
| `for each n in C … end` | `For each n in C:` body |
| `repeat at most N times [until C] … end` | `Repeats at most N times[ until C]:` body |
| `example "x" … end` | `Example "x":` body |

Example (`docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl`):

```
Uses vocabulary fleet v2.
Uses vocabulary tickets v2.

Job "gx10 disk watch":
  Runs on host gx10.
  Runs every 1 hour.
  May read disk.
  May write tickets.
  Sets free to disk free of "/".
  When free is below 100 GB:
    Does file ticket in repo "paiml/infra", with:
      title: "gx10 disk watch"
      label: "fleet"
  Expects ticket count is at most 1.
  Example "gx10 disk watch example":
    Given free is 90 GB.
    Then ticket count is 1.
```

## Tests

- `tests/rhl_mcp.rs` (`--features mcp`, 11 tests): two stdio end-to-end tests
  spawn `ruchy mcp`, perform the `initialize` handshake (protocol 2025-06-18),
  and read every reply under a 30 s timeout — `tools/list` holds the eight
  `rhl_*` names; `tools/call rhl_check` on a v2 typo program reports
  `RHL-V002`; one test runs `ruchy explain` twice and compares it with
  `rhl_explain`; one library test per tool checks its JSON in and out
  (`rhl_check` equals `check_files(..., Json)`).
- `src/rhl/explain_tests.rs` (5 lib tests): the rules above.
- `src/bin/handlers/mcp_handler.rs`: `prepare_server` tests build the server
  and assert the registered tool names without running the stdio loop.
- Hand mutation: dropping the registration in `create_named_mcp_server`
  fails both stdio e2e tests.
