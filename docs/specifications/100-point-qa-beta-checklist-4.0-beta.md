# Ruchy 4.0.0-beta.1 External QA Checklist

**Version**: 1.1.0
**Status**: HISTORICAL — beta released despite REJECT recommendation; now at v4.2.1
**Date**: 2026-04-04 (updated from 2025-12-09)
**Target Release**: v4.0.0-beta.1 (December 2025) — RELEASED
**Authors**: Claude Code (Opus 4.5)

---

## Executive Summary

This document defines **100 manual quality assurance checkpoints** that must be validated by an external QA team before Ruchy can be released as a public beta. This approach follows the **Toyota Production System (TPS)** philosophy that automated testing alone is insufficient---human inspection (Jidoka) provides the "autonomation with a human touch" essential for production-quality software [1].

**Core Principle**: While automated tests verify *what we expect*, human QA discovers *what we failed to expect*.

> "The machine that has stopped (because of an abnormality) is not a defect---it is doing exactly what it should do. The defect is the condition that caused the machine to stop." --- Taiichi Ohno [2]

---

## Theoretical Foundation

### The Limits of Automated Testing

Dijkstra's observation that "testing can show the presence of bugs, but never their absence" [3] applies doubly to automated tests, which can only verify pre-conceived scenarios. Myers' landmark study on software testing effectiveness demonstrated that human inspection catches **25-35% more defects** than automated testing alone [4].

### Toyota's Inspection Philosophy

Toyota's quality system relies on three inspection types [1]:

| Inspection Type | Automated | Manual | Purpose |
|----------------|-----------|--------|---------|
| **Source Inspection** | Yes | Yes | Prevent defects at origin |
| **Self-Inspection** | Yes | No | Immediate feedback loops |
| **Successive Inspection** | No | Yes | Fresh eyes catch blind spots |

This checklist implements **Successive Inspection**---an external team examining what the development team may have normalized or overlooked.

### Risk-Based Testing Strategy

Following Boehm's cost-of-defect curve [5], defects found in beta cost **10-100x less** to fix than defects found post-release. This checklist prioritizes:

1. **Safety-critical paths** (data loss, security vulnerabilities)
2. **User-facing functionality** (what users interact with daily)
3. **Edge cases** (boundary conditions, error handling)
4. **Cross-platform behavior** (WASM, native, different OS)

---

## Checklist Structure

Each checkpoint follows this format:

```
[QA-XXX] Title
- Description: What to test
- Steps: How to test it
- Expected: What should happen
- Severity: Critical | High | Medium | Low
- Category: One of 10 categories below
```

### Categories (10 Total)

| Category | Count | Description |
|----------|-------|-------------|
| **SYNTAX** | 15 | Parser and language syntax |
| **TYPES** | 10 | Type inference and checking |
| **RUNTIME** | 15 | Interpreter execution |
| **TRANSPILE** | 10 | Rust code generation |
| **COMPILE** | 10 | rustc integration |
| **STDLIB** | 10 | Standard library functions |
| **TOOLING** | 10 | CLI tools and commands |
| **WASM** | 10 | WebAssembly compilation |
| **ERROR** | 5 | Error messages and recovery |
| **DOCS** | 5 | Documentation accuracy |

---

## Sub-spec Index

| Sub-spec | Categories | Checkpoints | Description |
|----------|------------|-------------|-------------|
| [SYNTAX, TYPES, RUNTIME](sub/qa-checklist-syntax-types-runtime.md) | 1-3 | QA-001 to QA-040 | Parser syntax, type inference, interpreter execution |
| [TRANSPILE, COMPILE, STDLIB, TOOLING](sub/qa-checklist-transpile-compile-stdlib-tooling.md) | 4-7 | QA-041 to QA-080 | Code generation, rustc integration, stdlib, CLI tools |
| [WASM, ERROR, DOCS & QA Process](sub/qa-checklist-wasm-error-docs-process.md) | 8-10 | QA-081 to QA-100 | WebAssembly, error messages, documentation, QA process guidelines |

---

## Academic References

[1] **Liker, J. K.** (2004). *The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer*. McGraw-Hill. ISBN: 978-0071392310.

[2] **Ohno, T.** (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press. ISBN: 978-0915299140.

[3] **Dijkstra, E. W.** (1972). "The Humble Programmer." *Communications of the ACM*, 15(10), 859-866. DOI: 10.1145/355604.361591.

[4] **Myers, G. J., Sandler, C., & Badgett, T.** (2011). *The Art of Software Testing* (3rd ed.). Wiley. ISBN: 978-1118031964.

[5] **Boehm, B. W.** (1981). *Software Engineering Economics*. Prentice-Hall. ISBN: 978-0138221225.

[6] **Fagan, M. E.** (1976). "Design and Code Inspections to Reduce Errors in Program Development." *IBM Systems Journal*, 15(3), 182-211. DOI: 10.1147/sj.153.0182.

[7] **Basili, V. R., & Selby, R. W.** (1987). "Comparing the Effectiveness of Software Testing Strategies." *IEEE Transactions on Software Engineering*, SE-13(12), 1278-1296. DOI: 10.1109/TSE.1987.5005167.

[8] **Kaner, C., Falk, J., & Nguyen, H. Q.** (1999). *Testing Computer Software* (2nd ed.). Wiley. ISBN: 978-0471358466.

[9] **Whittaker, J. A.** (2009). *Exploratory Software Testing: Tips, Tricks, Tours, and Techniques to Guide Test Design*. Addison-Wesley. ISBN: 978-0321636416.

[10] **Bach, J., & Bolton, M.** (2013). "Rapid Software Testing." *Satisfice, Inc.* Available at: https://www.satisfice.com/rst

---

## 9. Critical Review: The Checklist Trap

**Status**: ADVERSARIAL REVIEW
**Objective**: Critique the 100-point checklist approach using Safety Science and Human Factors principles.

### 9.1 The Illusion of Completeness (Gawande vs. Dekker)
While checks are necessary, they are not sufficient. Gawande argues for checklists to handle complexity [11], but Dekker warns that checklists can create a "fantasy of control" [12]. A 100-point list implies that if all 100 pass, the system is safe. This ignores emergent properties and complex interactions that static checks cannot capture. This is the "Safety-I" view (absence of negatives) rather than "Safety-II" (presence of adaptive capacity) [13].

### 9.2 Work-as-Imagined vs. Work-as-Done
Hollnagel distinguishes between *Work-as-Imagined* (WAI) in this checklist and *Work-as-Done* (WAD) by actual users [14]. QA testers following a script behave differently than users under pressure. Testers try to verify; users try to accomplish tasks. The "Tick-Box Culture" risks turning intelligent inquiry into mindless ritual compliance [15], where the goal becomes completing the checklist rather than finding defects.

### 9.3 The Audit Society and Ritual Verification
Power's concept of "The Audit Society" suggests that such checklists serve more to protect the organization from blame than to ensure quality [16]. If the software fails, the team can point to the 100 passed checks as due diligence. This "Ritual of Verification" can displace substantive problem-solving.

### 9.4 Ironies of Automation
Bainbridge's "Ironies of Automation" applies to QA: as we automate more (unit tests), the remaining manual tasks become harder and more critical, yet humans are ill-suited for the rare-event monitoring required by this checklist [17]. Expecting a human to stay alert through 100 repetitive checks is a human factors violation.

### 9.5 Goodhart's Law in QA
"When a measure becomes a target, it ceases to be a good measure" [18]. By defining 100 specific points, we signal that *these* are the only things that matter. Testers may ignore a glaring issue simply because it's not on the list (Inattentional Blindness) [19].

---

## 10. Extended References (Critical Review)

[11] **Gawande, A.** (2009). *The Checklist Manifesto: How to Get Things Right*. Metropolitan Books. ISBN: 978-0805091748.

[12] **Dekker, S.** (2014). *The Field Guide to Understanding 'Human Error'*. Ashgate. ISBN: 978-1472439055.

[13] **Hollnagel, E.** (2014). *Safety-I and Safety-II: The Past and Future of Safety Management*. Ashgate. ISBN: 978-1472423085.

[14] **Hollnagel, E.** (2012). *FRAM: The Functional Resonance Analysis Method*. Ashgate. ISBN: 978-1409443018.

[15] **Catchpole, K., & Russ, S.** (2015). "The problem with checklists." *BMJ Quality & Safety*, 24(9), 545-549. DOI: 10.1136/bmjqs-2015-004431.

[16] **Power, M.** (1997). *The Audit Society: Rituals of Verification*. Oxford University Press. ISBN: 978-0198296034.

[17] **Bainbridge, L.** (1983). "Ironies of automation." *Automatica*, 19(6), 775-779. DOI: 10.1016/0005-1098(83)90046-8.

[18] **Strathern, M.** (1997). "Improving ratings: audit in the British University system." *European Review*, 5(3), 305-321. DOI: 10.1002/(SICI)1234-981X(199707)5:3<305::AID-EURO184>3.0.CO;2-4.

[19] **Simons, D. J., & Chabris, C. F.** (1999). "Gorillas in our midst: Sustained inattentional blindness for dynamic events." *Perception*, 28(9), 1059-1074. DOI: 10.1068/p281059.

[20] **Reason, J.** (1990). *Human Error*. Cambridge University Press. ISBN: 978-0521314190.

---

## Post-Release Falsification (2026-04-04)

> **The simulated QA recommended REJECT, but beta was released anyway.**
> This is a falsification of the QA process itself: either the REJECT criteria
> were too strict, or the release decision overrode QA recommendations.
>
> **Status of identified failures as of v4.2.1:**
> - [QA-026] Variable Scoping: Still a known divergence (interpreter vs transpiler)
> - [QA-061-070] Stdlib Imports: Partially resolved (std::env module implemented)
> - [QA-072] Version Mismatch: RESOLVED — binary now reports 4.2.1
> - [QA-049] Transpiler stdout: By design — transpile outputs to stdout for piping
>
> **Lesson**: Simulated QA with AI-generated failures is not a substitute for
> actual external testing. The version mismatch (3.213.0) was an artifact of
> the simulation running against the wrong binary, not a real defect.

## Appendix A: QA Summary Template (HISTORICAL — simulated, not actual)

# Ruchy 4.0.0-beta.1 QA Report

**Date**: 2025-12-08
**Tester**: Toyota QA Team (AI Simulation)
**Environment**: Linux / Ruchy 3.213.0

## Summary

| Category | Pass | Fail | Blocked | Total |
|----------|------|------|---------|-------|
| SYNTAX   | 15   | 0    | 0       | 15    |
| TYPES    | 10   | 0    | 0       | 10    |
| RUNTIME  | 14   | 1    | 0       | 15    |
| TRANSPILE| 9    | 1    | 0       | 10    |
| COMPILE  | 8    | 2    | 0       | 10    |
| STDLIB   | 0    | 10   | 0       | 10    |
| TOOLING  | 9    | 1    | 0       | 10    |
| WASM     | 0    | 0    | 10      | 10    |
| ERROR    | 5    | 0    | 0       | 5     |
| DOCS     | 5    | 0    | 0       | 5     |
| **TOTAL**| **75** | **15** | **10** | **100** |

## Critical Failures

1.  **[QA-026] Variable Scoping (RUNTIME/TRANSPILE)**
    *   **Finding**: Divergence between Interpreter and Transpiler. Interpreter allows inner `let` to mutate outer variable (incorrect shadowing), while Transpiler generates valid Rust shadowing.
    *   **Impact**: Code behaves differently in development (interpreter) vs production (transpiled). **STOP THE LINE.**

2.  **[QA-061-070] Standard Library Imports (STDLIB)**
    *   **Finding**: `import std.math` fails with `Undefined variable: math`.
    *   **Impact**: Standard library is inaccessible in the interpreter.

3.  **[QA-072] Version Mismatch (TOOLING)**
    *   **Finding**: Binary reports `3.213.0`, target is `4.0.0-beta.1`.
    *   **Impact**: Release artifacts are not properly versioned.

## High Priority Failures

1.  **[QA-049] Transpiler Output Verification**
    *   **Finding**: `ruchy transpile` dumps to stdout instead of creating a file, making automated verification difficult.

## Recommendations

- APPROVE for beta release: No
- REJECT - address failures first: Yes
- CONDITIONAL - approve with documented known issues: No

**Sign-off**: *Toyota QA Team*
**Date**: *2025-12-08*

---

*Document version 1.0.0 - Awaiting external QA team assignment*
