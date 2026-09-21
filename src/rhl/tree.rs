//! The RHL intent tree (spec RHL-001 §2): the one tree every surface produces.
//!
//! RHL text, YAML, MCP calls and voice all end here, and there is one checker
//! and one compiler over it. The shape mirrors `grammar/rhl.lalrpop`
//! production for production, so a reader can check the tree against the
//! pre-registered grammar line by line.
//!
//! # Spans are position, not meaning
//!
//! Every node carries a [`Span`] for diagnostics, but spans are
//! `#[serde(skip)]`: the serialized tree is the program's meaning, and two
//! programs that differ only in layout serialize identically. That is what
//! F2 ("`fmt` preserves the tree") compares, and it is what the lossless YAML
//! surface (row RHL-3) will round-trip.

use serde::{Deserialize, Serialize};

/// A half-open byte range `start..end` into the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// First byte of the node.
    pub start: usize,
    /// One past the last byte of the node.
    pub end: usize,
}

impl Span {
    /// The span from `start` to `end`.
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// The smallest span covering both `self` and `other`.
    #[must_use]
    pub fn to(self, other: Span) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }
}

/// A whole `.rhl` file: vocabulary imports and units, in source order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    /// Top-level declarations in source order.
    pub decls: Vec<Decl>,
}

/// One top-level declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decl {
    /// `use vocabulary <phrase>` — for example `use vocabulary fleet v1`.
    Use(UseDecl),
    /// A `job`, `command`, `check`, `pipeline` or `shape` block.
    Unit(Unit),
}

/// `use vocabulary <phrase>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseDecl {
    /// The words after the keyword, for example `fleet v1`.
    pub vocabulary: Phrase,
    /// The whole declaration line.
    #[serde(skip)]
    pub span: Span,
}

/// The five unit kinds of §3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitKind {
    /// `job`
    Job,
    /// `command`
    Command,
    /// `check`
    Check,
    /// `pipeline`
    Pipeline,
    /// `shape`
    Shape,
}

impl UnitKind {
    /// The keyword that introduces this unit kind.
    #[must_use]
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Command => "command",
            Self::Check => "check",
            Self::Pipeline => "pipeline",
            Self::Shape => "shape",
        }
    }
}

/// `<kind> "<name>" … end`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    /// Which of the five unit kinds this is.
    pub kind: UnitKind,
    /// The quoted unit name.
    pub name: Text,
    /// The statements between the header line and `end`.
    pub body: Vec<Stmt>,
    /// From the kind keyword to the closing `end`.
    #[serde(skip)]
    pub span: Span,
}

/// One line-oriented statement, or a block statement closed by `end`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stmt {
    /// What the statement is.
    pub kind: StmtKind,
    /// From the first token of the statement to its last (a block's `end`).
    #[serde(skip)]
    pub span: Span,
}

/// The statement forms of `grammar/rhl.lalrpop` (`Stmt` and `BlockStmt`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StmtKind {
    /// `runs on <app>`
    RunsOn(App),
    /// `every <quantity>`
    Every(Quantity),
    /// `may read|write|call <phrase>`
    May(Effect),
    /// `wait up to <quantity>`
    WaitUpTo(Quantity),
    /// `let <name> be <cond>`
    Let {
        /// The name being introduced.
        name: Phrase,
        /// Its value.
        value: Cond,
    },
    /// `set <name> to <cond>`
    Set {
        /// The name being assigned.
        name: Phrase,
        /// Its new value.
        value: Cond,
    },
    /// `expect <cond>`
    Expect(Cond),
    /// `give back <cond>`
    GiveBack(Cond),
    /// `stop with <cond>`
    StopWith(Cond),
    /// `given <cond>` (valid only inside `example`; a check, not a production)
    Given(Cond),
    /// `then <cond>` (valid only inside `example`; a check, not a production)
    Then(Cond),
    /// A vocabulary action, with an optional `with … end` attribute block.
    Action(Action),
    /// `when <cond> … [otherwise …] end`
    When {
        /// The condition.
        cond: Cond,
        /// Statements run when the condition holds.
        then_body: Vec<Stmt>,
        /// Statements after `otherwise`, if the block has one.
        otherwise: Option<Vec<Stmt>>,
    },
    /// `for each <name> in <cond> … end`
    ForEach {
        /// The loop variable.
        name: Phrase,
        /// The finite collection iterated over.
        collection: Cond,
        /// The loop body.
        body: Vec<Stmt>,
    },
    /// `repeat at most <n> times [until <cond>] … end`
    Repeat {
        /// The bound `n`.
        times: Int,
        /// The early-exit condition, if any.
        until: Option<Cond>,
        /// The loop body.
        body: Vec<Stmt>,
    },
    /// `example "<name>" … end`
    Example {
        /// The quoted example name.
        name: Text,
        /// Its `given`/`then` lines.
        body: Vec<Stmt>,
    },
}

/// `may <verb> <target>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    /// `read`, `write` or `call`.
    pub verb: EffectVerb,
    /// What may be touched, for example `disk`.
    pub target: Phrase,
}

/// The three effect verbs of §3.1 principle 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectVerb {
    /// `may read`
    Read,
    /// `may write`
    Write,
    /// `may call`
    Call,
}

impl EffectVerb {
    /// The verb as a vocabulary `effect:` field spells it.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Call => "call",
        }
    }
}

/// An action statement: `<app> [in <app>] [with … end]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// The action term and its arguments, for example `file ticket`.
    pub head: App,
    /// The operand after `in`, for example `repo "paiml/infra"`.
    pub target: Option<App>,
    /// The attribute lines of a `with … end` block, if present.
    pub with: Option<Vec<Stmt>>,
}

/// A condition: `or`, `and`, `not`, a comparison, `in`, or a bare application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cond {
    /// What the condition is.
    pub kind: CondKind,
    /// The whole condition.
    #[serde(skip)]
    pub span: Span,
}

/// The condition forms of `grammar/rhl.lalrpop` (`Disj`, `Conj`, `Cmp`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CondKind {
    /// `<cond> or <cond>` — left-associative.
    Or(Box<Cond>, Box<Cond>),
    /// `<cond> and <cond>` — left-associative, binds tighter than `or`.
    And(Box<Cond>, Box<Cond>),
    /// `not <cond>`
    Not(Box<Cond>),
    /// `<app> <op> <app>`
    Compare {
        /// Left operand.
        left: App,
        /// The comparison operator.
        op: CompOp,
        /// Right operand.
        right: App,
    },
    /// `<app> in <app>`
    In {
        /// Left operand.
        left: App,
        /// Right operand.
        right: App,
    },
    /// A bare application used as a value.
    App(App),
}

/// The comparison operators of §3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompOp {
    /// `is`
    Is,
    /// `is not`
    IsNot,
    /// `is below`
    IsBelow,
    /// `is above`
    IsAbove,
    /// `is at least`
    IsAtLeast,
    /// `is at most`
    IsAtMost,
    /// `is one of`
    IsOneOf,
    /// `contains`
    Contains,
}

impl CompOp {
    /// The operator's keyword spelling.
    #[must_use]
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Is => "is",
            Self::IsNot => "is not",
            Self::IsBelow => "is below",
            Self::IsAbove => "is above",
            Self::IsAtLeast => "is at least",
            Self::IsAtMost => "is at most",
            Self::IsOneOf => "is one of",
            Self::Contains => "contains",
        }
    }

    /// True for the four ordering operators, which need ordered operands.
    #[must_use]
    pub fn is_ordering(self) -> bool {
        matches!(
            self,
            Self::IsBelow | Self::IsAbove | Self::IsAtLeast | Self::IsAtMost
        )
    }
}

/// An application: a phrase with optional arguments, a quantity, or a string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum App {
    /// `<phrase> [<atom>…]` — a term, a `let` name, or an entity and instance.
    Call {
        /// The words, for example `disk free of` or `host gx10`.
        phrase: Phrase,
        /// String and quantity arguments, for example `"/"`.
        args: Vec<Atom>,
    },
    /// A quantity literal such as `100 GB`.
    Quantity(Quantity),
    /// A string literal.
    Text(Text),
}

/// An argument: a string or a quantity (grammar `Atom`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Atom {
    /// A string literal argument.
    Text(Text),
    /// A quantity argument.
    Quantity(Quantity),
}

/// `INT [unit]` — `100 GB`, `1 hour`, `5 %`, or a bare `3`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantity {
    /// The number.
    pub value: Int,
    /// The unit word exactly as written (`GB`, `hour`, `%`), if any.
    pub unit: Option<Word>,
    /// From the number to the end of the unit.
    #[serde(skip)]
    pub span: Span,
}

/// A non-negative integer literal, kept as its decimal digits.
///
/// Digits, not a machine integer, so that no literal can overflow the parser.
/// Leading zeros are dropped at parse time (`007` and `7` are one tree, so
/// `fmt` prints `7`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Int {
    /// Decimal digits with leading zeros removed (`"0"` for zero).
    pub digits: String,
    /// The literal as written.
    #[serde(skip)]
    pub span: Span,
}

/// A string literal. `value` excludes the quotes; RHL strings have no escapes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    /// The characters between the quotes.
    pub value: String,
    /// The literal including its quotes.
    #[serde(skip)]
    pub span: Span,
}

/// One or more words (grammar `Phrase = WORD+`). The grammar knows no
/// vocabulary term; which words form a term is the checker's job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phrase {
    /// The words in order; never empty.
    pub words: Vec<Word>,
    /// From the first word to the last.
    #[serde(skip)]
    pub span: Span,
}

impl Phrase {
    /// The words joined by single spaces, for example `disk free of`.
    #[must_use]
    pub fn text(&self) -> String {
        self.words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// One word, a unit symbol, or `%`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    /// The word exactly as written.
    pub text: String,
    /// Where it was written.
    #[serde(skip)]
    pub span: Span,
}
