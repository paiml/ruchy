//! Types and units (spec RHL-001 §3.1 principle 4): the value of a phrase,
//! and which code a mismatch between two values is.

use crate::rhl::codes;
use crate::rhl::tree::CompOp;

/// The type of an RHL value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ty {
    /// A string.
    Text,
    /// A quantity of storage, for example `100 GB`.
    Size,
    /// A quantity of time, for example `1 hour`.
    Duration,
    /// A proportion, for example `5 %`.
    Percent,
    /// A plain count.
    Count,
    /// A filed ticket.
    Ticket,
    /// The result of a comparison or a logical operator.
    Bool,
    /// An instance of the named entity term, for example `host`.
    Entity(String),
    /// A value whose type is not known because of an earlier diagnostic.
    /// It fits everywhere, so one mistake yields one diagnostic.
    Unknown,
}

impl Ty {
    /// The type a vocabulary names in `gives:` or a parameter's `type:`.
    pub(crate) fn from_name(name: &str) -> Ty {
        match name {
            "Text" => Ty::Text,
            "Size" => Ty::Size,
            "Duration" => Ty::Duration,
            "Percent" => Ty::Percent,
            "Count" => Ty::Count,
            "Ticket" => Ty::Ticket,
            "Bool" => Ty::Bool,
            _ => Ty::Unknown,
        }
    }

    /// Size, Duration, Percent or Count.
    pub(crate) fn is_quantity(&self) -> bool {
        matches!(self, Ty::Size | Ty::Duration | Ty::Percent | Ty::Count)
    }

    /// A quantity that carries a unit: Size, Duration or Percent.
    pub(crate) fn has_unit(&self) -> bool {
        matches!(self, Ty::Size | Ty::Duration | Ty::Percent)
    }

    /// The name a diagnostic prints.
    pub(crate) fn name(&self) -> String {
        match self {
            Ty::Entity(e) => format!("Entity({e})"),
            other => format!("{other:?}"),
        }
    }
}

/// A typed value. `bare` marks an integer literal written without a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Val {
    /// Its type.
    pub(crate) ty: Ty,
    /// True only for a bare integer literal such as `5`.
    pub(crate) bare: bool,
}

impl Val {
    /// A value of type `ty` that is not a bare literal.
    pub(crate) fn of(ty: Ty) -> Self {
        Self { ty, bare: false }
    }

    /// The unknown value, after a diagnostic.
    pub(crate) fn unknown() -> Self {
        Self::of(Ty::Unknown)
    }

    /// Whether the type is unknown.
    pub(crate) fn is_unknown(&self) -> bool {
        self.ty == Ty::Unknown
    }
}

/// The code for two values that should have had one type: T003 for a bare
/// number against a unit-bearing quantity, T001 for two quantities of
/// different dimensions, T002 otherwise.
pub(crate) fn mismatch(l: &Val, r: &Val) -> &'static str {
    let bare_vs_unit = (l.bare && r.ty.has_unit()) || (r.bare && l.ty.has_unit());
    if bare_vs_unit {
        codes::T003
    } else if l.ty.is_quantity() && r.ty.is_quantity() {
        codes::T001
    } else {
        codes::T002
    }
}

/// The code for `l <op> r`, or `None` when the operands fit the operator.
/// An unknown operand fits every operator.
pub(crate) fn compare_code(op: CompOp, l: &Val, r: &Val) -> Option<&'static str> {
    if l.is_unknown() || r.is_unknown() {
        return None;
    }
    let fits = match op {
        CompOp::Contains => l.ty == Ty::Text && r.ty == Ty::Text,
        CompOp::IsOneOf => false,
        CompOp::Is | CompOp::IsNot => l.ty == r.ty,
        _ => l.ty == r.ty && l.ty.is_quantity(),
    };
    match (fits, op) {
        (true, _) => None,
        (false, CompOp::Contains | CompOp::IsOneOf) => Some(codes::T002),
        (false, _) => Some(mismatch(l, r)),
    }
}

/// The code for `v` where a Duration is needed (`every`, `wait up to`).
pub(crate) fn duration_code(v: &Val) -> Option<&'static str> {
    if v.is_unknown() || v.ty == Ty::Duration {
        return None;
    }
    Some(mismatch(v, &Val::of(Ty::Duration)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare() -> Val {
        Val {
            ty: Ty::Count,
            bare: true,
        }
    }

    #[test]
    fn test_rhl_1_check_types_ordering_needs_one_dimension() {
        let size = Val::of(Ty::Size);
        let hour = Val::of(Ty::Duration);
        assert_eq!(compare_code(CompOp::IsBelow, &size, &size), None);
        assert_eq!(
            compare_code(CompOp::IsBelow, &size, &hour),
            Some(codes::T001)
        );
        assert_eq!(
            compare_code(CompOp::IsBelow, &size, &bare()),
            Some(codes::T003)
        );
        let text = Val::of(Ty::Text);
        assert_eq!(
            compare_code(CompOp::IsBelow, &text, &text),
            Some(codes::T002)
        );
        assert_eq!(
            compare_code(CompOp::IsBelow, &size, &text),
            Some(codes::T002)
        );
        let count = Val::of(Ty::Count);
        assert_eq!(compare_code(CompOp::IsAtMost, &count, &bare()), None);
    }

    #[test]
    fn test_rhl_1_check_types_is_contains_one_of_and_unknown() {
        let text = Val::of(Ty::Text);
        let size = Val::of(Ty::Size);
        assert_eq!(compare_code(CompOp::Is, &text, &text), None);
        assert_eq!(compare_code(CompOp::IsNot, &text, &size), Some(codes::T002));
        assert_eq!(compare_code(CompOp::Contains, &text, &text), None);
        assert_eq!(
            compare_code(CompOp::Contains, &size, &size),
            Some(codes::T002)
        );
        assert_eq!(
            compare_code(CompOp::IsOneOf, &text, &text),
            Some(codes::T002)
        );
        assert_eq!(compare_code(CompOp::IsOneOf, &Val::unknown(), &text), None);
    }

    #[test]
    fn test_rhl_1_check_types_duration_positions() {
        assert_eq!(duration_code(&Val::of(Ty::Duration)), None);
        assert_eq!(duration_code(&bare()), Some(codes::T003));
        assert_eq!(duration_code(&Val::of(Ty::Size)), Some(codes::T001));
        assert_eq!(duration_code(&Val::unknown()), None);
        assert_eq!(Ty::from_name("Nope"), Ty::Unknown);
        assert_eq!(Ty::Entity("host".into()).name(), "Entity(host)");
    }
}
