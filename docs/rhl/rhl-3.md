# RHL-3 — the YAML surface `.rhl.yaml` and `ruchy convert`

Roadmap row RHL-3; spec RHL-001 §2 (the YAML row), §5 (`convert`), §7 F3, §9 rules 5, 7
and 8; plan `docs/rhl/rhlga-1-plan.md` §3 P3. The code is `src/rhl/yaml.rs`; the verbs are
in `src/rhl/cli.rs` and `src/rhl/check.rs`. The binary only prints the result and exits
with its code (`src/bin/handlers/rhl_handler.rs`).

## What the YAML is

A `.rhl.yaml` file is a serialization of the intent tree (`src/rhl/tree.rs`), the same
tree the RHL parser builds. It is not a second language: there is one tree, one checker
and one normal form (§9 rule 5). Positions are not part of the tree, so two RHL texts that
differ only in layout have the same YAML.

The YAML is **data**. Reading it builds a tree and nothing else. It is refused, with a
diagnostic, when:

- it is not YAML, or does not have the tree's shape;
- it has a key the tree does not have (a misspelt key is refused, not dropped);
- it has a local YAML tag anywhere (`!custom`, `!python/object`): the shape has none. A
  `!!` tag (`!!str`, `!!python/object:…`) is in the YAML core namespace; the YAML library
  (serde_yaml_ng) resolves it to a plain scalar before any value is built, so it is read as
  that scalar. Nothing runs in either case;
- it has no `rhl:` key, or a schema version other than `1`;
- a phrase is empty, or has a leading, trailing or doubled space;
- it holds a tree the RHL grammar could not have produced. The test is the grammar itself:
  the tree's RHL normal form must parse back to the same tree. This refuses a keyword used
  as a word (`phrase: end`), a word the lexer does not make (`phrase: A`), a `"` inside a
  string, a number with leading zeros (`'007'` reads back as `7`), and an `or` directly
  under an `and` (RHL has no parentheses).

## The shape

The keys are the tree's own field names, in snake case. One schema family with the
vocabulary files (§9 rule 7): plain block YAML, no tags, and a phrase is one string, as a
vocabulary `term:` is.

| Tree | YAML |
|---|---|
| the file | `rhl: 1` (schema version), then `decls:`, a list |
| a declaration | a one-key map: `use:` or `unit:` |
| a statement | a one-key map named for the statement: `runs_on`, `every`, `may`, `wait_up_to`, `let`, `set`, `expect`, `give_back`, `stop_with`, `given`, `then`, `action`, `when`, `for_each`, `repeat`, `example` |
| a condition | a one-key map: `or` and `and` (a list of two conditions), `not`, `compare` (`left`, `op`, `right`), `in` (`left`, `right`), `app` |
| an application | a one-key map: `call` (`phrase`, optional `args`), `quantity`, `text` |
| a quantity | `value` (the digits, as a string) and optional `unit` |
| a phrase, a word, a string | a plain string |
| an operator | `is`, `is_not`, `is_below`, `is_above`, `is_at_least`, `is_at_most`, `is_one_of`, `contains` |

An absent optional part (`unit`, an action's `target` and `with`, `otherwise`, `until`)
and an empty argument list are left out. An empty `with:` or `otherwise:` list is kept,
because it is a different tree from an absent one.

The §3.2 example, as RHL:

```rhl
use vocabulary fleet v1
use vocabulary tickets v1

job "gx10 disk watch"
  runs on host gx10
  every 1 hour
  may read disk
  may call tickets

  let free be disk free of "/"

  when free is below 100 GB
    file ticket in repo "paiml/infra" with
      title "gx10 disk below 100 GB"
      label "fleet"
    end
  end

  expect host is unchanged
  expect ticket count is at most 1

  example "low disk files one ticket"
    given disk free of "/" is 90 GB
    then ticket count is 1
  end

  example "healthy disk files nothing"
    given disk free of "/" is 400 GB
    then ticket count is 0
  end
end
```

and as `.rhl.yaml` (`ruchy convert example.rhl`):

```yaml
rhl: 1
decls:
- use:
    vocabulary: fleet v1
- use:
    vocabulary: tickets v1
- unit:
    kind: job
    name: gx10 disk watch
    body:
    - runs_on:
        call:
          phrase: host gx10
    - every:
        value: '1'
        unit: hour
    - may:
        verb: read
        target: disk
    - may:
        verb: call
        target: tickets
    - let:
        name: free
        value:
          app:
            call:
              phrase: disk free of
              args:
              - text: /
    - when:
        cond:
          compare:
            left:
              call:
                phrase: free
            op: is_below
            right:
              quantity:
                value: '100'
                unit: GB
        then_body:
        - action:
            head:
              call:
                phrase: file ticket
            target:
              call:
                phrase: repo
                args:
                - text: paiml/infra
            with:
            - action:
                head:
                  call:
                    phrase: title
                    args:
                    - text: gx10 disk below 100 GB
            - action:
                head:
                  call:
                    phrase: label
                    args:
                    - text: fleet
    - expect:
        compare:
          left:
            call:
              phrase: host
          op: is
          right:
            call:
              phrase: unchanged
    - expect:
        compare:
          left:
            call:
              phrase: ticket count
          op: is_at_most
          right:
            quantity:
              value: '1'
    - example:
        name: low disk files one ticket
        body:
        - given:
            compare:
              left:
                call:
                  phrase: disk free of
                  args:
                  - text: /
              op: is
              right:
                quantity:
                  value: '90'
                  unit: GB
        - then:
            compare:
              left:
                call:
                  phrase: ticket count
              op: is
              right:
                quantity:
                  value: '1'
    - example:
        name: healthy disk files nothing
        body:
        - given:
            compare:
              left:
                call:
                  phrase: disk free of
                  args:
                  - text: /
              op: is
              right:
                quantity:
                  value: '400'
                  unit: GB
        - then:
            compare:
              left:
                call:
                  phrase: ticket count
              op: is
              right:
                quantity:
                  value: '0'
```

The lib test `test_rhl3_doc_shows_the_spec_example_in_both_forms` holds both blocks equal
to the spec and to the serializer.

## `ruchy convert <input> [-o <output>] [--to rhl|yaml]`

The input's name picks the direction: `.rhl.yaml` converts to RHL, `.rhl` to YAML.
`--to` forces the output surface; the input is then read as the other one. The result goes
to `-o`, or to standard output.

- RHL output is always `fmt`'s normal form. YAML output is always the canonical YAML.
- An input that is not a tree is refused with exit **2** and the diagnostic on standard
  error: RHL text that does not parse gets its own `RHL-P…` code; YAML that is refused
  (above) gets `RHL-P004`, at the YAML's line and column, or at line 0 when the problem
  is the tree as a whole and has no one place.
- A file that cannot be read or written is exit **1**. An input whose name gives no
  direction, with no `--to`, is exit **2**.

`convert` is the one verb RHL-3 adds (§9 rule 8).

## `ruchy check` and `ruchy fmt` on `.rhl.yaml`

Both verbs extend by extension, as for `.rhl`.

- `ruchy check x.rhl.yaml` reads the tree and runs the same checker on it: the checker is
  given the tree's normal form, which is accepted only because it parses back to the same
  tree. The codes, messages, candidates, verdict and exit code are those of the `.rhl`
  form. The **span of every diagnostic is line 0, column 0** (unknown): the checker's
  positions are in the RHL text, which is not the file checked. For the same reason the
  diagnostics carry **no fixes**. To see positions or apply fixes, `ruchy convert` the file
  to `.rhl`. A YAML text that is not a tree is one `RHL-P004` diagnostic, with the verdict
  `fail` and exit 1, as a `.rhl` parse failure is.
- `ruchy fmt x.rhl.yaml` re-emits the canonical YAML of the tree.
- `ruchy fix` edits RHL text; on a `.rhl.yaml` file it refuses (exit 1) and names
  `ruchy convert`.

## F3 — lossless surfaces

The lib tests `test_rhl3_*` in `src/rhl/yaml_tests.rs` measure F3:

- For every program in `docs/rhl/breaks/valid/`, `docs/rhl/breaks/v2/valid/`, and every
  `broken.rhl` under `docs/rhl/breaks/planted/` and `docs/rhl/breaks/v2/planted/` that
  parses: `fmt(src)` → YAML → RHL is byte-identical to `fmt(src)`, and YAML → RHL → YAML
  is byte-identical to the YAML. The count is checked against the directory listing: the
  only files left out are the `missing-end` cases, which do not parse.
- The same two properties over generated trees (proptest, `PROPTEST_CASES`, default 100).
- Positive control: changing one token (`100 GB` to `101 GB`, `may read` to `may write`)
  changes the YAML bytes. Changing only the layout does not.
- Refusals: malformed YAML is `RHL-P004` with its line and column; arbitrary text and
  damaged YAML never panic.
