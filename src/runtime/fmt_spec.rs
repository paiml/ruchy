//! FMTSPEC-1: Rust `std::fmt` format strings for the interpreter.
//!
//! One parser for the format-spec language, shared by `println`/`print`/
//! `eprintln`/`eprint`/`format` (function and macro forms) and by the
//! `{expr:spec}` f-string interpolation.
//!
//! Supported: `{}`, `{0}`, `{name}`, `{{`/`}}`, fill/align `<^>`, `+`, `#`,
//! `0`, width, `.precision`, and the traits `?`, `x`, `X`, `b`, `o`, `e`, `E`.
//! Width or precision taken from an argument (`{:1$}`, `{:.*}`, `{:w$}`) is
//! an error naming the spec, never silently wrong output.

use crate::runtime::value_format::{format_value_display, rust_debug};
use crate::runtime::Value;
use std::collections::HashMap;

/// Which argument a field refers to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgRef {
    /// `{}`: the next implicit positional argument.
    Next,
    /// `{0}`: an explicit positional argument.
    Index(usize),
    /// `{name}`: a named argument or a variable in scope.
    Name(String),
}

/// Alignment inside the field width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// The formatting trait a field selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Display,
    Debug,
    LowerHex,
    UpperHex,
    Binary,
    Octal,
    LowerExp,
    UpperExp,
}

/// A parsed `:spec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub fill: char,
    pub align: Option<Align>,
    pub plus: bool,
    pub alternate: bool,
    pub zero: bool,
    pub width: Option<usize>,
    pub precision: Option<usize>,
    pub kind: Kind,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            fill: ' ',
            align: None,
            plus: false,
            alternate: false,
            zero: false,
            width: None,
            precision: None,
            kind: Kind::Display,
        }
    }
}

/// One piece of a parsed format string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    Text(String),
    Field(ArgRef, Spec),
}

type Chars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

/// Parse a format string into text and fields.
pub fn parse(fmt: &str) -> Result<Vec<Piece>, String> {
    let mut pieces = Vec::new();
    let mut text = String::new();
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        parse_char(c, &mut chars, &mut pieces, &mut text)?;
    }
    flush_text(&mut pieces, &mut text);
    Ok(pieces)
}

/// One character of a format string: a `{{`/`}}` escape, a field, or text.
fn parse_char(
    c: char,
    chars: &mut Chars<'_>,
    pieces: &mut Vec<Piece>,
    text: &mut String,
) -> Result<(), String> {
    if matches!(c, '{' | '}') && chars.peek() == Some(&c) {
        chars.next();
        text.push(c);
        return Ok(());
    }
    match c {
        '{' => {
            flush_text(pieces, text);
            pieces.push(parse_field(&take_field(chars)?)?);
            Ok(())
        }
        '}' => Err("unmatched `}` in format string (use `}}` for a literal brace)".into()),
        _ => {
            text.push(c);
            Ok(())
        }
    }
}

fn flush_text(pieces: &mut Vec<Piece>, text: &mut String) {
    if !text.is_empty() {
        pieces.push(Piece::Text(std::mem::take(text)));
    }
}

fn take_field(chars: &mut Chars<'_>) -> Result<String, String> {
    let mut body = String::new();
    for c in chars.by_ref() {
        match c {
            '}' => return Ok(body),
            '{' => return Err(format!("invalid format field `{{{body}{{`")),
            _ => body.push(c),
        }
    }
    Err(format!(
        "unterminated format field `{{{body}` (use `{{{{` for a literal brace)"
    ))
}

fn parse_field(body: &str) -> Result<Piece, String> {
    let (arg, spec) = match body.split_once(':') {
        Some((arg, spec)) => (arg, Some(spec)),
        None => (body, None),
    };
    let spec = match spec {
        Some(text) => parse_spec(text)?,
        None => Spec::default(),
    };
    Ok(Piece::Field(parse_arg(arg.trim())?, spec))
}

fn parse_arg(arg: &str) -> Result<ArgRef, String> {
    if arg.is_empty() {
        return Ok(ArgRef::Next);
    }
    if let Ok(index) = arg.parse::<usize>() {
        return Ok(ArgRef::Index(index));
    }
    if is_identifier(arg) {
        return Ok(ArgRef::Name(arg.to_string()));
    }
    Err(format!("invalid format argument `{arg}`"))
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_alphabetic() || c == '_');
    first_ok && chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Parse the text after `:` in a field.
pub fn parse_spec(text: &str) -> Result<Spec, String> {
    parse_spec_chars(&text.chars().collect::<Vec<_>>())
        .map_err(|e| format!("{e} in format spec `{{:{text}}}`"))
}

fn parse_spec_chars(chars: &[char]) -> Result<Spec, String> {
    let mut spec = Spec::default();
    let mut i = parse_fill_align(chars, &mut spec);
    i = parse_flags(chars, i, &mut spec);
    let (width, mut i) = parse_count(chars, i)?;
    spec.width = width;
    if chars.get(i) == Some(&'.') {
        let (precision, next) = parse_precision(chars, i + 1)?;
        spec.precision = Some(precision);
        i = next;
    }
    spec.kind = parse_kind(&chars[i..].iter().collect::<String>())?;
    Ok(spec)
}

fn align_of(c: char) -> Option<Align> {
    match c {
        '<' => Some(Align::Left),
        '^' => Some(Align::Center),
        '>' => Some(Align::Right),
        _ => None,
    }
}

fn parse_fill_align(chars: &[char], spec: &mut Spec) -> usize {
    if let Some(align) = chars.get(1).copied().and_then(align_of) {
        spec.fill = chars[0];
        spec.align = Some(align);
        return 2;
    }
    if let Some(align) = chars.first().copied().and_then(align_of) {
        spec.align = Some(align);
        return 1;
    }
    0
}

fn parse_flags(chars: &[char], mut i: usize, spec: &mut Spec) -> usize {
    if matches!(chars.get(i), Some('+' | '-')) {
        spec.plus = chars[i] == '+';
        i += 1;
    }
    if chars.get(i) == Some(&'#') {
        spec.alternate = true;
        i += 1;
    }
    if chars.get(i) == Some(&'0') {
        spec.zero = true;
        i += 1;
    }
    i
}

/// Digits at `i`; a count taken from an argument (`1$`) is refused.
fn parse_count(chars: &[char], i: usize) -> Result<(Option<usize>, usize), String> {
    let end = i + chars[i..].iter().take_while(|c| c.is_ascii_digit()).count();
    if chars.get(end) == Some(&'$') {
        return Err("width/precision taken from an argument (`N$`) is not supported".into());
    }
    let digits: String = chars[i..end].iter().collect();
    Ok((digits.parse().ok(), end))
}

fn parse_precision(chars: &[char], i: usize) -> Result<(usize, usize), String> {
    if chars.get(i) == Some(&'*') {
        return Err("precision taken from an argument (`.*`) is not supported".into());
    }
    match parse_count(chars, i)? {
        (Some(precision), next) => Ok((precision, next)),
        (None, _) => Err("missing precision after `.`".into()),
    }
}

fn parse_kind(text: &str) -> Result<Kind, String> {
    match text {
        "" => Ok(Kind::Display),
        "?" => Ok(Kind::Debug),
        "x" => Ok(Kind::LowerHex),
        "X" => Ok(Kind::UpperHex),
        "b" => Ok(Kind::Binary),
        "o" => Ok(Kind::Octal),
        "e" => Ok(Kind::LowerExp),
        "E" => Ok(Kind::UpperExp),
        other => Err(format!("unsupported format trait `{other}`")),
    }
}

/// Render parsed pieces. `named` resolves `{name}` fields.
pub fn render(
    pieces: &[Piece],
    args: &[Value],
    named: &dyn Fn(&str) -> Option<Value>,
) -> Result<String, String> {
    let mut out = String::new();
    let mut next = 0;
    for piece in pieces {
        match piece {
            Piece::Text(text) => out.push_str(text),
            Piece::Field(arg, spec) => {
                let value = resolve(arg, args, &mut next, named)?;
                out.push_str(&format_value(&value, spec)?);
            }
        }
    }
    Ok(out)
}

fn resolve(
    arg: &ArgRef,
    args: &[Value],
    next: &mut usize,
    named: &dyn Fn(&str) -> Option<Value>,
) -> Result<Value, String> {
    match arg {
        ArgRef::Next => {
            *next += 1;
            args.get(*next - 1).cloned().ok_or_else(|| {
                format!(
                    "format string needs {} positional argument(s), {} given",
                    *next,
                    args.len()
                )
            })
        }
        ArgRef::Index(i) => args.get(*i).cloned().ok_or_else(|| {
            format!(
                "invalid reference to positional argument {i} ({} given)",
                args.len()
            )
        }),
        ArgRef::Name(name) => {
            named(name).ok_or_else(|| format!("there is no argument named `{name}`"))
        }
    }
}

/// Parse and render `fmt` in one step.
pub fn format_str(
    fmt: &str,
    args: &[Value],
    named: &dyn Fn(&str) -> Option<Value>,
) -> Result<String, String> {
    render(&parse(fmt)?, args, named)
}

/// Text of an output call (`println(args…)`, `format(args…)`).
///
/// With `template`, the first value is the format string. A template with no
/// fields and extra arguments keeps the historical ruchy form
/// (`println("a", 1)` prints `a 1`), which Rust rejects at compile time.
/// Without a template the values are joined by spaces.
pub fn format_call(
    values: &[Value],
    template: bool,
    named: &dyn Fn(&str) -> Option<Value>,
) -> Result<String, String> {
    let Some((Value::String(fmt), rest)) = values.split_first() else {
        return Ok(join_display(values));
    };
    if !template {
        return Ok(join_display(values));
    }
    let pieces = parse(fmt)?;
    let has_field = pieces.iter().any(|p| matches!(p, Piece::Field(..)));
    if !has_field && !rest.is_empty() {
        return Ok(join_display(values));
    }
    render(&pieces, rest, named)
}

fn join_display(values: &[Value]) -> String {
    values
        .iter()
        .map(format_value_display)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format one value under a spec.
pub fn format_value(value: &Value, spec: &Spec) -> Result<String, String> {
    if spec.kind == Kind::Debug && is_compound(value) {
        return debug_compound(value, spec);
    }
    format_scalar(value, spec)
}

fn is_compound(value: &Value) -> bool {
    matches!(
        value,
        Value::Array(_)
            | Value::Tuple(_)
            | Value::Struct { .. }
            | Value::Object(_)
            | Value::ObjectMut(_)
            | Value::EnumVariant { .. }
    )
}

fn format_scalar(value: &Value, spec: &Spec) -> Result<String, String> {
    let (sign, body, numeric) = match value {
        Value::Integer(i) => (int_sign(*i, spec), int_digits(*i, spec), true),
        Value::Float(f) => (float_sign(*f, spec), float_digits(*f, spec)?, true),
        // `impl Debug for str` writes the quoted text and ignores width.
        Value::String(_) if spec.kind == Kind::Debug => return Ok(rust_debug(value)),
        _ => (String::new(), text_body(value, spec)?, false),
    };
    Ok(pad(&sign, &body, spec, numeric))
}

fn is_radix(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::LowerHex | Kind::UpperHex | Kind::Binary | Kind::Octal
    )
}

fn int_sign(i: i64, spec: &Spec) -> String {
    let radix = is_radix(spec.kind);
    let sign = if i < 0 && !radix {
        "-"
    } else if spec.plus {
        "+"
    } else {
        ""
    };
    let prefix = match spec.kind {
        Kind::LowerHex | Kind::UpperHex if spec.alternate => "0x",
        Kind::Binary if spec.alternate => "0b",
        Kind::Octal if spec.alternate => "0o",
        _ => "",
    };
    format!("{sign}{prefix}")
}

fn int_digits(i: i64, spec: &Spec) -> String {
    let magnitude = i.unsigned_abs();
    match (spec.kind, spec.precision) {
        (Kind::LowerHex, _) => format!("{i:x}"),
        (Kind::UpperHex, _) => format!("{i:X}"),
        (Kind::Binary, _) => format!("{i:b}"),
        (Kind::Octal, _) => format!("{i:o}"),
        (Kind::LowerExp, Some(p)) => format!("{magnitude:.p$e}"),
        (Kind::LowerExp, None) => format!("{magnitude:e}"),
        (Kind::UpperExp, Some(p)) => format!("{magnitude:.p$E}"),
        (Kind::UpperExp, None) => format!("{magnitude:E}"),
        (Kind::Display | Kind::Debug, _) => magnitude.to_string(),
    }
}

fn float_sign(f: f64, spec: &Spec) -> String {
    let sign = if f.is_sign_negative() && !f.is_nan() {
        "-"
    } else if spec.plus {
        "+"
    } else {
        ""
    };
    sign.to_string()
}

fn float_digits(f: f64, spec: &Spec) -> Result<String, String> {
    let a = f.abs();
    Ok(match (spec.kind, spec.precision) {
        (Kind::LowerExp, Some(p)) => format!("{a:.p$e}"),
        (Kind::LowerExp, None) => format!("{a:e}"),
        (Kind::UpperExp, Some(p)) => format!("{a:.p$E}"),
        (Kind::UpperExp, None) => format!("{a:E}"),
        (Kind::Display | Kind::Debug, Some(p)) => format!("{a:.p$}"),
        (Kind::Display, None) => format_value_display(&Value::Float(a)),
        (Kind::Debug, None) => rust_debug(&Value::Float(a)),
        (kind, _) => {
            return Err(format!(
                "format trait {kind:?} needs an integer, got a float"
            ))
        }
    })
}

fn text_body(value: &Value, spec: &Spec) -> Result<String, String> {
    match spec.kind {
        // Rust's `pad` truncates str and bool; other Display impls ignore precision.
        Kind::Display if matches!(value, Value::String(_) | Value::Bool(_)) => {
            Ok(truncate(&format_value_display(value), spec.precision))
        }
        Kind::Display => Ok(format_value_display(value)),
        Kind::Debug => Ok(rust_debug(value)),
        kind => Err(format!(
            "format trait {kind:?} needs a number, got {}",
            value.type_name()
        )),
    }
}

fn truncate(text: &str, precision: Option<usize>) -> String {
    match precision {
        Some(p) => text.chars().take(p).collect(),
        None => text.to_string(),
    }
}

/// Apply width, fill and alignment. `0` pads numbers after the sign.
fn pad(sign: &str, body: &str, spec: &Spec, numeric: bool) -> String {
    let len = sign.chars().count() + body.chars().count();
    let gap = spec.width.unwrap_or(0).saturating_sub(len);
    if spec.zero && numeric {
        return format!("{sign}{}{body}", "0".repeat(gap));
    }
    let default = if numeric { Align::Right } else { Align::Left };
    let (left, right) = match spec.align.unwrap_or(default) {
        Align::Left => (0, gap),
        Align::Right => (gap, 0),
        Align::Center => (gap / 2, gap - gap / 2),
    };
    let fill = |n: usize| std::iter::repeat_n(spec.fill, n).collect::<String>();
    format!("{}{sign}{body}{}", fill(left), fill(right))
}

// ------------------------------------------------------------- compound `{:?}`

struct Delims<'a> {
    open: &'a str,
    close: &'a str,
    spaced: bool,
    one_tuple: bool,
}

const LIST: Delims<'static> = Delims {
    open: "[",
    close: "]",
    spaced: false,
    one_tuple: false,
};

/// Rust's derived/std Debug for a compound value: the spec reaches every
/// leaf (`{:5?}` pads each element) and `#` selects the pretty form.
fn debug_compound(value: &Value, spec: &Spec) -> Result<String, String> {
    match value {
        Value::Array(items) => debug_seq(&LIST, items, spec),
        Value::Tuple(items) => debug_seq(&tuple_delims(items.len()), items, spec),
        Value::EnumVariant {
            variant_name, data, ..
        } => debug_variant(variant_name, data.as_deref().unwrap_or(&[]), spec),
        Value::Struct { name, fields } => debug_map(Some(name), fields, spec),
        Value::Object(map) => debug_object(map, spec),
        Value::ObjectMut(cell) => {
            let map = cell
                .lock()
                .map_err(|_| "object lock poisoned".to_string())?;
            debug_object(&map, spec)
        }
        _ => format_scalar(value, spec),
    }
}

fn tuple_delims(len: usize) -> Delims<'static> {
    Delims {
        open: "(",
        close: ")",
        spaced: false,
        one_tuple: len == 1,
    }
}

fn debug_seq(delims: &Delims<'_>, items: &[Value], spec: &Spec) -> Result<String, String> {
    let parts = items
        .iter()
        .map(|v| format_value(v, spec))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(wrap(delims, &parts, spec.alternate))
}

fn debug_variant(name: &str, data: &[Value], spec: &Spec) -> Result<String, String> {
    if data.is_empty() {
        return Ok(name.to_string());
    }
    let open = format!("{name}(");
    let delims = Delims {
        open: &open,
        close: ")",
        spaced: false,
        one_tuple: false,
    };
    debug_seq(&delims, data, spec)
}

fn debug_object(map: &HashMap<String, Value>, spec: &Spec) -> Result<String, String> {
    match map.get("__class") {
        Some(Value::String(class)) => debug_map(Some(class), map, spec),
        _ => debug_map(None, map, spec),
    }
}

/// A struct (`Name { a: 1 }`) or, without a name, a map (`{"a": 1}`).
fn debug_map(
    name: Option<&str>,
    map: &HashMap<String, Value>,
    spec: &Spec,
) -> Result<String, String> {
    let mut entries: Vec<_> = map.iter().filter(|(k, _)| !k.starts_with("__")).collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let parts = entries
        .into_iter()
        .map(|(k, v)| {
            let key = if name.is_some() {
                k.clone()
            } else {
                format!("{k:?}")
            };
            format_value(v, spec).map(|v| format!("{key}: {v}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    match name {
        Some(name) if parts.is_empty() => Ok(name.to_string()),
        Some(name) => {
            let open = format!("{name} {{");
            let delims = Delims {
                open: &open,
                close: "}",
                spaced: true,
                one_tuple: false,
            };
            Ok(wrap(&delims, &parts, spec.alternate))
        }
        None => {
            let delims = Delims {
                open: "{",
                close: "}",
                spaced: false,
                one_tuple: false,
            };
            Ok(wrap(&delims, &parts, spec.alternate))
        }
    }
}

fn wrap(delims: &Delims<'_>, parts: &[String], pretty: bool) -> String {
    let (open, close) = (delims.open, delims.close);
    if parts.is_empty() {
        return format!("{open}{close}");
    }
    if pretty {
        let body: String = parts.iter().map(|p| format!("{},\n", indent(p))).collect();
        return format!("{open}\n{body}{close}");
    }
    let space = if delims.spaced { " " } else { "" };
    let trail = if delims.one_tuple { "," } else { "" };
    format!("{open}{space}{}{trail}{space}{close}", parts.join(", "))
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "fmt_spec_tests.rs"]
mod tests;
