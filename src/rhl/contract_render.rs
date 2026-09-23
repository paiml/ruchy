//! RHL-5b: the YAML bytes of a unit contract. Every string is a double-quoted
//! scalar (JSON escaping is valid YAML), so no value can change the layout.

use super::Job;

/// A double-quoted YAML scalar.
fn q(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// `  key:` followed by one `- "item"` line per item, or `key: []`.
fn list(indent: &str, key: &str, items: &[String]) -> String {
    if items.is_empty() {
        return format!("{indent}{key}: []\n");
    }
    let mut out = format!("{indent}{key}:\n");
    for i in items {
        out.push_str(&format!("{indent}  - {}\n", q(i)));
    }
    out
}

/// `GX10-DISK-WATCH`: the unit name as an id part.
fn slug(name: &str) -> String {
    let raw: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '-'
            }
        })
        .collect();
    raw.split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// One proof obligation and the falsification test that discharges it.
struct Obligation {
    kind: &'static str,
    property: String,
    formal: String,
    rule: String,
    test: String,
    if_fails: String,
}

impl Job {
    /// The contract bytes; `source_sha256` is the sha of the input file.
    pub(super) fn render(&self, source_sha256: &str) -> String {
        let mut out = format!(
            "# RHL unit contract of job {}, emitted by ruchy from its RHL source (RHL-5b).\n\
             # Regenerate it from the source; do not edit. Layout: docs/rhl/rhl-5.md.\n",
            q(&self.name)
        );
        out.push_str(&self.metadata(source_sha256));
        out.push_str(&self.equations());
        let obligations = self.obligations();
        out.push_str(&self.obligation_blocks(&obligations));
        out.push_str(&self.gates());
        out
    }

    fn metadata(&self, source_sha256: &str) -> String {
        let mut out =
            String::from("metadata:\n  version: \"1.0.0\"\n  author: \"ruchy (RHL-5b)\"\n");
        out.push_str(&format!(
            "  description: {}\n  crate: \"ruchy\"\n",
            q(&format!("RHL unit contract for job '{}'", self.name))
        ));
        let refs = [
            "RHL-001 §3.1 principle 7",
            "RHL-001 §4",
            "RHL-001 §7 F6",
            "RHL-001 §9.3",
        ];
        out.push_str(&list("  ", "references", &refs.map(String::from)));
        out.push_str(&format!(
            "  unit:\n    kind: \"job\"\n    name: {}\n    source_sha256: {}\n",
            q(&self.name),
            q(source_sha256)
        ));
        if let Some(r) = &self.runs_on {
            out.push_str(&format!("    runs_on: {}\n", q(r)));
        }
        if let Some(e) = &self.every {
            out.push_str(&format!("    every: {}\n", q(e)));
        }
        let effects: Vec<String> = self.effects.iter().map(|e| format!("may {e}")).collect();
        out.push_str(&list("    ", "effects", &effects));
        out.push_str(&list("    ", "examples", &self.examples));
        out.push_str(&self.vocabulary_block());
        out
    }

    fn vocabulary_block(&self) -> String {
        let mut out = String::from("  vocabularies:\n");
        for (name, version, sha) in &self.vocabularies {
            out.push_str(&format!(
                "    - name: {}\n      version: {version}\n      sha256: {}\n",
                q(name),
                q(sha)
            ));
        }
        out.push_str(if self.terms.is_empty() {
            "  terms: []\n"
        } else {
            "  terms:\n"
        });
        for (term, vocab, contract) in &self.terms {
            out.push_str(&format!(
                "    - term: {}\n      vocabulary: {}\n      contract: {}\n",
                q(term),
                q(vocab),
                q(contract)
            ));
        }
        out
    }

    fn domain(&self) -> String {
        format!("plan = decide(facts), facts : Facts of job '{}'", self.name)
    }

    fn surface(&self) -> String {
        format!("effects(apply(plan)) ⊆ {{{}}}", self.effects.join(", "))
    }

    fn equations(&self) -> String {
        let mut out = format!(
            "\nequations:\n  effect_surface:\n    formula: {}\n    domain: {}\n",
            q(&self.surface()),
            q(&self.domain())
        );
        let mut invariants: Vec<String> = self.effects.iter().map(|e| format!("may {e}")).collect();
        if invariants.is_empty() {
            invariants.push("the job declares no effect: apply performs none".to_string());
        }
        out.push_str(&list("    ", "invariants", &invariants));
        for (n, (_, f)) in self.expects.iter().enumerate() {
            out.push_str(&format!(
                "  expect_{n}:\n    formula: {}\n    domain: {}\n",
                q(f),
                q(&self.domain())
            ));
            out.push_str(&list("    ", "postconditions", std::slice::from_ref(f)));
        }
        out
    }

    /// Expects (ensures), the closed effect surface, each `may` line and each
    /// term's contract; never empty, because the surface is always there.
    fn obligations(&self) -> Vec<Obligation> {
        let mut out: Vec<Obligation> = self
            .expects
            .iter()
            .enumerate()
            .map(|(n, (text, f))| Obligation {
                kind: "postcondition",
                property: text.clone(),
                formal: format!("{f}, where {}", self.domain()),
                rule: format!("{text} holds on every plan decide returns"),
                test: format!("expect_{n} (the guard `ruchy test` and the built job's main check)"),
                if_fails: format!("the plan breaks `{text}`; the built job applies nothing (§9.4)"),
            })
            .collect();
        out.push(self.surface_obligation());
        out.extend(self.effects.iter().map(|e| effect_obligation(e)));
        out.extend(self.terms.iter().map(|(t, _, c)| term_obligation(t, c)));
        out
    }

    fn surface_obligation(&self) -> Obligation {
        Obligation {
            kind: "invariant",
            property: "apply performs only the declared effects (RHL-001 §3.1 principle 6)"
                .to_string(),
            formal: self.surface(),
            rule: "an effect the job does not declare is a compile error".to_string(),
            test: "ruchy check: RHL-E001 undeclared effect".to_string(),
            if_fails: "the job touches something its `may` lines do not name".to_string(),
        }
    }

    fn obligation_blocks(&self, obligations: &[Obligation]) -> String {
        let id = slug(&self.name);
        let mut po = String::from("\nproof_obligations:\n");
        let mut ft = String::from("\nfalsification_tests:\n");
        for (n, o) in obligations.iter().enumerate() {
            po.push_str(&format!(
                "  - type: {}\n    property: {}\n    formal: {}\n    applies_to: all\n",
                o.kind,
                q(&o.property),
                q(&o.formal)
            ));
            ft.push_str(&falsification(&id, n + 1, &o.rule, &o.test, &o.if_fails));
        }
        let base = obligations.len();
        for (n, name) in self.examples.iter().enumerate() {
            let test = format!("example_{n} {}", q(name));
            let rule = format!("example {} holds: its thens and every expect", q(name));
            let fails = format!("`ruchy test` reports example {} FAILED", q(name));
            ft.push_str(&falsification(&id, base + n + 1, &rule, &test, &fails));
        }
        po + &ft
    }

    fn gates(&self) -> String {
        let id = slug(&self.name);
        let mut tests: Vec<String> = (0..self.examples.len())
            .map(|n| format!("example_{n}"))
            .collect();
        tests.extend((0..self.expects.len()).map(|n| format!("expect_{n}")));
        tests.push("ruchy check".to_string());
        let mut out = format!(
            "\n# NOT APPLICABLE, declared because `pv validate` requires the block: the job's\n\
             # obligations are checked by its generated tests and guards, not by a Kani harness.\n\
             kani_harnesses:\n  - id: KANI-RHL-UNIT-{id}-001\n    obligation: PO-RHL-UNIT-{id}-001\n    \
             property: \"NOT APPLICABLE — a job's plan is checked by its examples and expect guards\"\n    \
             bound: 1\n    strategy: bounded_int\n    harness: none\n    status: declared_not_implemented\n    \
             discharged_by: \"ruchy test\"\n"
        );
        out.push_str(&format!(
            "\nqa_gate:\n  id: QA-RHL-UNIT-{id}\n  name: {}\n  min_coverage: 1.0\n  max_complexity: 10\n",
            q(&format!("rhl job: {}", self.name))
        ));
        out.push_str(&list("  ", "required_tests", &tests));
        out
    }
}

fn effect_obligation(effect: &str) -> Obligation {
    Obligation {
        kind: "invariant",
        property: format!("effect declared: may {effect}"),
        formal: format!("{effect} ∈ declared_effects(job)"),
        rule: format!("`may {effect}` is part of the job's contract (§3.1 principle 6)"),
        test: "ruchy check: RHL-E001 undeclared effect".to_string(),
        if_fails: format!("the job uses `{effect}` without declaring it"),
    }
}

fn term_obligation(term: &str, contract: &str) -> Obligation {
    Obligation {
        kind: "invariant",
        property: format!("term '{term}' carries its contract (F6)"),
        formal: format!("contract('{term}') = {contract}"),
        rule: "every term the job uses has a contract template (RHL-001 §9.3)".to_string(),
        test: "ruchy check: RHL-C001 no contract template".to_string(),
        if_fails: format!("term '{term}' has no contract at {contract}"),
    }
}

fn falsification(id: &str, n: usize, rule: &str, test: &str, if_fails: &str) -> String {
    format!(
        "  - id: FALSIFY-RHL-UNIT-{id}-{n:03}\n    rule: {}\n    test: {}\n    prediction: \"verified\"\n    if_fails: {}\n",
        q(rule),
        q(test),
        q(if_fails)
    )
}
