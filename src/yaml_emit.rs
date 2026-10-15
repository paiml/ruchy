//! SEC-2 (#259): the YAML writer, in-tree.
//!
//! `serde_yaml_ng::to_string` decided a scalar's style in two steps, and both
//! are reproduced here so ruchy's YAML output stays byte for byte what it was:
//!
//! 1. serde_yaml_ng asked for a style: `|` literal if the string holds a line
//!    break, single quotes if the text would read back as something other than
//!    a string (`''`, `null`, `true`, `1`, `0.5`, `007`), plain otherwise.
//! 2. libyaml's emitter analysed the text and could overrule it (plain is not
//!    allowed for `- x` or `a: b`, single quotes cannot hold a tab, ...).
//!
//! The layout (two-space indent, `- ` items at the parent key's column, `[]`
//! and `{}` for empty collections, no line folding) is libyaml's block
//! emitter, ported state for state for the events serde_yaml_ng sent it. No
//! YAML library writes here: serde-saphyr always double-quotes, which is valid
//! YAML but not the bytes ruchy wrote. The differential tests in
//! `yaml_io_tests.rs` and `rhl/yaml_tests.rs` hold this module to
//! `serde_yaml_ng` itself.
//!
//! Enums are written in the externally tagged map form (`Variant: value`),
//! the form the one enum-writing call site (RHL) asked serde_yaml_ng for with
//! `singleton_map_recursive`.

use serde::ser::{self, Serialize};
use std::fmt;

/// A value that has no YAML form here (bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteError(String);

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WriteError {}

impl ser::Error for WriteError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

/// The style serde_yaml_ng requested for a scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Any,
    Plain,
    Single,
    Literal,
}

/// The style libyaml wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Written {
    Plain,
    Single,
    Double,
    Literal,
}

#[derive(Debug, Clone)]
enum Node {
    Scalar(String, Style),
    Seq(Vec<Node>),
    Map(Vec<(Node, Node)>),
}

/// Write `value` as one YAML document, as `serde_yaml_ng::to_string` did.
pub(crate) fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String, WriteError> {
    let node = value.serialize(NodeSer)?;
    let mut e = Emitter::new();
    e.document(&node);
    Ok(e.out)
}

// ───────────────────────── serde_yaml_ng's style request ─────────────────────────

/// serde_yaml_ng's `digits_but_not_number`: a leading zero followed only by
/// digits is a string under YAML 1.2, so it is quoted to stay one.
fn digits_but_not_number(s: &str) -> bool {
    let s = s.strip_prefix(['-', '+']).unwrap_or(s);
    s.len() > 1 && s.starts_with('0') && s[1..].bytes().all(|b| b.is_ascii_digit())
}

fn radix_int(s: &str, signed: bool) -> bool {
    let unpositive = s.strip_prefix('+').unwrap_or(s);
    let positive = |p: &str, radix: u32| {
        unpositive.strip_prefix(p).is_some_and(|rest| {
            !rest.starts_with(['+', '-']) && u128::from_str_radix(rest, radix).is_ok()
        })
    };
    let negative = |p: &str, radix: u32| {
        s.strip_prefix('-')
            .and_then(|r| r.strip_prefix(p))
            .is_some_and(|rest| i128::from_str_radix(&format!("-{rest}"), radix).is_ok())
    };
    [("0x", 16), ("0o", 8), ("0b", 2)]
        .into_iter()
        .any(|(p, radix)| positive(p, radix) || (signed && negative(p, radix)))
}

/// Would serde_yaml_ng read this plain scalar back as an integer?
fn reads_as_int(s: &str) -> bool {
    let unpositive = s.strip_prefix('+').unwrap_or(s);
    let decimal = !digits_but_not_number(s);
    // parse_unsigned_int (u64, then u128)
    let unsigned = radix_int(s, false)
        || (decimal && !unpositive.starts_with(['+', '-']) && unpositive.parse::<u128>().is_ok());
    // parse_negative_int (i64, then i128)
    let negative =
        (s.starts_with('-') && radix_int(s, true)) || (decimal && s.parse::<i128>().is_ok());
    unsigned || negative
}

/// serde_yaml_ng's `parse_f64`, finite results only count.
fn reads_as_float(s: &str) -> bool {
    let unpositive = match s.strip_prefix('+') {
        Some(u) if u.starts_with(['+', '-']) => return false,
        Some(u) => u,
        None => s,
    };
    if matches!(unpositive, ".inf" | ".Inf" | ".INF")
        || matches!(s, "-.inf" | "-.Inf" | "-.INF" | ".nan" | ".NaN" | ".NAN")
    {
        return true;
    }
    unpositive.parse::<f64>().is_ok_and(f64::is_finite)
}

/// The style serde_yaml_ng's `serialize_str` requested.
fn str_style(v: &str) -> Style {
    if v.contains('\n') {
        return Style::Literal;
    }
    let ambiguous = v.is_empty()
        || matches!(v, "null" | "Null" | "NULL" | "~")
        || matches!(v, "true" | "True" | "TRUE" | "false" | "False" | "FALSE")
        || reads_as_int(v)
        || (!digits_but_not_number(v) && reads_as_float(v))
        || digits_but_not_number(v);
    if ambiguous {
        Style::Single
    } else {
        Style::Any
    }
}

fn float_text<F: Copy + Into<f64>>(v: F, finite: impl FnOnce() -> String) -> String {
    let f: f64 = v.into();
    if f.is_nan() {
        ".nan".into()
    } else if f.is_infinite() {
        if f > 0.0 { ".inf" } else { "-.inf" }.into()
    } else {
        finite()
    }
}

// ───────────────────────── serde → node tree ─────────────────────────

struct NodeSer;

fn scalar(s: impl Into<String>, style: Style) -> Node {
    Node::Scalar(s.into(), style)
}

fn str_node(v: &str) -> Node {
    scalar(v, str_style(v))
}

fn single(key: &str, value: Node) -> Node {
    Node::Map(vec![(str_node(key), value)])
}

impl ser::Serializer for NodeSer {
    type Ok = Node;
    type Error = WriteError;
    type SerializeSeq = SeqSer;
    type SerializeTuple = SeqSer;
    type SerializeTupleStruct = SeqSer;
    type SerializeTupleVariant = SeqSer;
    type SerializeMap = MapSer;
    type SerializeStruct = MapSer;
    type SerializeStructVariant = MapSer;

    fn serialize_bool(self, v: bool) -> Result<Node, WriteError> {
        Ok(scalar(if v { "true" } else { "false" }, Style::Plain))
    }
    fn serialize_i8(self, v: i8) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_i16(self, v: i16) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_i32(self, v: i32) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_i64(self, v: i64) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_i128(self, v: i128) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_u8(self, v: u8) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_u16(self, v: u16) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_u32(self, v: u32) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_u64(self, v: u64) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_u128(self, v: u128) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Plain))
    }
    fn serialize_f32(self, v: f32) -> Result<Node, WriteError> {
        // ryu, as serde_yaml_ng wrote a finite float (`1e20`, not `1e+20`)
        let text = float_text(v, || ryu::Buffer::new().format_finite(v).to_string());
        Ok(scalar(text, Style::Plain))
    }
    fn serialize_f64(self, v: f64) -> Result<Node, WriteError> {
        let text = float_text(v, || ryu::Buffer::new().format_finite(v).to_string());
        Ok(scalar(text, Style::Plain))
    }
    fn serialize_char(self, v: char) -> Result<Node, WriteError> {
        Ok(scalar(v.to_string(), Style::Single))
    }
    fn serialize_str(self, v: &str) -> Result<Node, WriteError> {
        Ok(str_node(v))
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<Node, WriteError> {
        Err(WriteError(
            "serialization and deserialization of bytes in YAML is not implemented".into(),
        ))
    }
    fn serialize_none(self) -> Result<Node, WriteError> {
        self.serialize_unit()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<Node, WriteError> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<Node, WriteError> {
        Ok(scalar("null", Style::Plain))
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Node, WriteError> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Node, WriteError> {
        Ok(str_node(variant))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<Node, WriteError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<Node, WriteError> {
        Ok(single(variant, v.serialize(NodeSer)?))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSer, WriteError> {
        Ok(SeqSer {
            items: Vec::with_capacity(len.unwrap_or(0)),
            variant: None,
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<SeqSer, WriteError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<SeqSer, WriteError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SeqSer, WriteError> {
        Ok(SeqSer {
            items: Vec::with_capacity(len),
            variant: Some(variant),
        })
    }
    fn serialize_map(self, len: Option<usize>) -> Result<MapSer, WriteError> {
        Ok(MapSer {
            entries: Vec::with_capacity(len.unwrap_or(0)),
            key: None,
            variant: None,
        })
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<MapSer, WriteError> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<MapSer, WriteError> {
        Ok(MapSer {
            entries: Vec::with_capacity(len),
            key: None,
            variant: Some(variant),
        })
    }
}

struct SeqSer {
    items: Vec<Node>,
    variant: Option<&'static str>,
}

impl SeqSer {
    fn push<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<(), WriteError> {
        self.items.push(v.serialize(NodeSer)?);
        Ok(())
    }
    fn finish(self) -> Node {
        let seq = Node::Seq(self.items);
        match self.variant {
            Some(v) => single(v, seq),
            None => seq,
        }
    }
}

impl ser::SerializeSeq for SeqSer {
    type Ok = Node;
    type Error = WriteError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<(), WriteError> {
        self.push(v)
    }
    fn end(self) -> Result<Node, WriteError> {
        Ok(self.finish())
    }
}

impl ser::SerializeTuple for SeqSer {
    type Ok = Node;
    type Error = WriteError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<(), WriteError> {
        self.push(v)
    }
    fn end(self) -> Result<Node, WriteError> {
        Ok(self.finish())
    }
}

impl ser::SerializeTupleStruct for SeqSer {
    type Ok = Node;
    type Error = WriteError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<(), WriteError> {
        self.push(v)
    }
    fn end(self) -> Result<Node, WriteError> {
        Ok(self.finish())
    }
}

impl ser::SerializeTupleVariant for SeqSer {
    type Ok = Node;
    type Error = WriteError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<(), WriteError> {
        self.push(v)
    }
    fn end(self) -> Result<Node, WriteError> {
        Ok(self.finish())
    }
}

struct MapSer {
    entries: Vec<(Node, Node)>,
    key: Option<Node>,
    variant: Option<&'static str>,
}

impl MapSer {
    fn finish(self) -> Node {
        let map = Node::Map(self.entries);
        match self.variant {
            Some(v) => single(v, map),
            None => map,
        }
    }
}

impl ser::SerializeMap for MapSer {
    type Ok = Node;
    type Error = WriteError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, k: &T) -> Result<(), WriteError> {
        self.key = Some(k.serialize(NodeSer)?);
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<(), WriteError> {
        let k = self
            .key
            .take()
            .ok_or_else(|| WriteError("serialize_value called before serialize_key".into()))?;
        self.entries.push((k, v.serialize(NodeSer)?));
        Ok(())
    }
    fn end(self) -> Result<Node, WriteError> {
        Ok(self.finish())
    }
}

impl ser::SerializeStruct for MapSer {
    type Ok = Node;
    type Error = WriteError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), WriteError> {
        self.entries.push((str_node(key), v.serialize(NodeSer)?));
        Ok(())
    }
    fn end(self) -> Result<Node, WriteError> {
        Ok(self.finish())
    }
}

impl ser::SerializeStructVariant for MapSer {
    type Ok = Node;
    type Error = WriteError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), WriteError> {
        self.entries.push((str_node(key), v.serialize(NodeSer)?));
        Ok(())
    }
    fn end(self) -> Result<Node, WriteError> {
        Ok(self.finish())
    }
}

// ───────────────────────── libyaml's analysis ─────────────────────────

/// libyaml `IS_PRINTABLE` (with `unicode` on, as serde_yaml_ng set it).
fn printable(c: char) -> bool {
    let u = c as u32;
    u == 0x0A
        || (0x20..=0x7E).contains(&u)
        || (0xA0..=0xD7FF).contains(&u)
        || ((0xE000..=0xFFFD).contains(&u) && u != 0xFEFF)
        || (0x1_0000..=0x10_FFFF).contains(&u)
}

fn is_break(c: char) -> bool {
    matches!(c, '\r' | '\n' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

/// libyaml `IS_BLANKZ`; `None` is the string's end (libyaml's NUL).
fn blankz(c: Option<char>) -> bool {
    match c {
        None => true,
        Some(c) => c == ' ' || c == '\t' || c == '\0' || is_break(c),
    }
}

#[allow(clippy::struct_excessive_bools)]
struct Analysis {
    multiline: bool,
    flow_plain: bool,
    block_plain: bool,
    single: bool,
    block: bool,
}

/// The (flow, block) indicators `c` raises: libyaml's first-character
/// table when `first`, its inner-character table otherwise.
fn indicators(c: char, first: bool, preceded_ws: bool, followed_ws: bool) -> (bool, bool) {
    if first {
        first_indicators(c, followed_ws)
    } else {
        inner_indicators(c, preceded_ws, followed_ws)
    }
}

fn first_indicators(c: char, followed_ws: bool) -> (bool, bool) {
    match c {
        '#' | ',' | '[' | ']' | '{' | '}' | '&' | '*' | '!' | '|' | '>' | '\'' | '"' | '%'
        | '@' | '`' => (true, true),
        '-' => (followed_ws, followed_ws),
        '?' | ':' => (true, followed_ws),
        _ => (false, false),
    }
}

fn inner_indicators(c: char, preceded_ws: bool, followed_ws: bool) -> (bool, bool) {
    match c {
        ',' | '?' | '[' | ']' | '{' | '}' => (true, false),
        ':' => (true, followed_ws),
        '#' => (preceded_ws, preceded_ws),
        _ => (false, false),
    }
}

/// The (flow, block) indicator flags over the whole scalar.
fn scan_indicators(chars: &[char], doc_marker: bool) -> (bool, bool) {
    let blank_at = |i: usize| blankz(chars.get(i).copied());
    chars
        .iter()
        .enumerate()
        .fold((doc_marker, doc_marker), |(flow, block), (i, &c)| {
            let preceded_ws = i == 0 || blank_at(i - 1);
            let (f, b) = indicators(c, i == 0, preceded_ws, blank_at(i + 1));
            (flow || f, block || b)
        })
}

fn space(c: char) -> bool {
    c == ' '
}

/// Where a scalar's blanks, breaks and non-printables fall.
#[allow(clippy::struct_excessive_bools)]
struct Blanks {
    line_breaks: bool,
    special: bool,
    edge: bool,
    trailing_space: bool,
    break_space: bool,
    space_break: bool,
}

impl Blanks {
    fn of(chars: &[char], head: char, tail: char) -> Self {
        let pair =
            |a: fn(char) -> bool, b: fn(char) -> bool| chars.windows(2).any(|w| a(w[0]) && b(w[1]));
        let trailing_space = space(tail);
        Self {
            line_breaks: chars.iter().any(|&c| is_break(c)),
            special: chars.iter().any(|&c| !printable(c)),
            edge: space(head) || is_break(head) || trailing_space || is_break(tail),
            trailing_space,
            break_space: pair(is_break, space),
            space_break: pair(space, is_break),
        }
    }
}

/// libyaml `yaml_emitter_analyze_scalar`.
fn analyze(s: &str) -> Analysis {
    let chars: Vec<char> = s.chars().collect();
    let (Some(&head), Some(&tail)) = (chars.first(), chars.last()) else {
        return Analysis {
            multiline: false,
            flow_plain: false,
            block_plain: true,
            single: true,
            block: false,
        };
    };
    let doc_marker = s.starts_with("---") || s.starts_with("...");
    let (flow_ind, block_ind) = scan_indicators(&chars, doc_marker);
    let b = Blanks::of(&chars, head, tail);
    let broken = b.space_break || b.special;
    let plain = !(b.edge || b.break_space || broken || b.line_breaks);
    Analysis {
        multiline: b.line_breaks,
        flow_plain: plain && !flow_ind,
        block_plain: plain && !block_ind,
        single: !(b.break_space || broken),
        block: !(b.trailing_space || broken),
    }
}

// ───────────────────────── libyaml's emitter ─────────────────────────

/// libyaml's emitter state, for the events serde_yaml_ng sent: one implicit
/// document, no anchors, no tags, block collections (empty ones in flow).
#[allow(clippy::struct_excessive_bools)]
struct Emitter {
    out: String,
    column: usize,
    indent: i64,
    indents: Vec<i64>,
    whitespace: bool,
    indention: bool,
    open_ended: u8,
    flow_level: usize,
    mapping_context: bool,
    simple_key_context: bool,
}

const BEST_INDENT: i64 = 2;

impl Emitter {
    fn new() -> Self {
        Self {
            out: String::new(),
            column: 0,
            indent: -1,
            indents: Vec::new(),
            whitespace: true,
            indention: true,
            open_ended: 0,
            flow_level: 0,
            mapping_context: false,
            simple_key_context: false,
        }
    }

    fn put(&mut self, c: char) {
        self.out.push(c);
        self.column += 1;
    }

    fn put_break(&mut self) {
        self.out.push('\n');
        self.column = 0;
    }

    /// libyaml `WRITE_BREAK`.
    fn write_break(&mut self, c: char) {
        if c == '\n' {
            self.put_break();
        } else {
            self.out.push(c);
            self.column = 0;
        }
    }

    fn increase_indent(&mut self, flow: bool, indentless: bool) {
        self.indents.push(self.indent);
        if self.indent < 0 {
            self.indent = if flow { BEST_INDENT } else { 0 };
        } else if !indentless {
            self.indent += BEST_INDENT;
        }
    }

    fn pop_indent(&mut self) {
        self.indent = self.indents.pop().unwrap_or(-1);
    }

    fn write_indent(&mut self) {
        let indent = usize::try_from(self.indent.max(0)).unwrap_or(0);
        if !self.indention || self.column > indent || (self.column == indent && !self.whitespace) {
            self.put_break();
        }
        while self.column < indent {
            self.put(' ');
        }
        self.whitespace = true;
        self.indention = true;
    }

    fn write_indicator(&mut self, ind: &str, need_ws: bool, is_ws: bool, is_indention: bool) {
        if need_ws && !self.whitespace {
            self.put(' ');
        }
        for c in ind.chars() {
            self.put(c);
        }
        self.whitespace = is_ws;
        self.indention = self.indention && is_indention;
    }

    fn document(&mut self, root: &Node) {
        // DOCUMENT-START, implicit: nothing written
        self.open_ended = 0;
        self.node(root, false, false);
        // DOCUMENT-END, implicit
        self.write_indent();
        if self.open_ended == 0 {
            self.open_ended = 1;
        }
        // No STREAM-END: `serde_yaml_ng::to_string` drops its serializer
        // without emitting one (only `into_inner` does), so the `...` that
        // libyaml writes after a keep-chomped (`|+`) block scalar never appears.
    }

    fn node(&mut self, n: &Node, mapping: bool, simple_key: bool) {
        self.mapping_context = mapping;
        self.simple_key_context = simple_key;
        match n {
            Node::Scalar(s, style) => self.scalar(s, *style),
            Node::Seq(items) if items.is_empty() || self.flow_level > 0 => self.flow_seq(items),
            Node::Seq(items) => self.block_seq(items),
            Node::Map(entries) if entries.is_empty() || self.flow_level > 0 => {
                self.flow_map(entries);
            }
            Node::Map(entries) => self.block_map(entries),
        }
    }

    fn flow_seq(&mut self, items: &[Node]) {
        self.write_indicator("[", true, true, false);
        self.increase_indent(true, false);
        self.flow_level += 1;
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                self.write_indicator(",", false, false, false);
            }
            self.node(item, false, false);
        }
        self.flow_level -= 1;
        self.pop_indent();
        self.write_indicator("]", false, false, false);
    }

    fn flow_map(&mut self, entries: &[(Node, Node)]) {
        self.write_indicator("{", true, true, false);
        self.increase_indent(true, false);
        self.flow_level += 1;
        for (i, (k, v)) in entries.iter().enumerate() {
            if i > 0 {
                self.write_indicator(",", false, false, false);
            }
            if simple_key(k) {
                self.node(k, true, true);
                self.write_indicator(":", false, false, false);
            } else {
                self.write_indicator("?", true, false, false);
                self.node(k, true, false);
                self.write_indicator(":", true, false, false);
            }
            self.node(v, true, false);
        }
        self.flow_level -= 1;
        self.pop_indent();
        self.write_indicator("}", false, false, false);
    }

    fn block_seq(&mut self, items: &[Node]) {
        let indentless = self.mapping_context && !self.indention;
        self.increase_indent(false, indentless);
        for item in items {
            self.write_indent();
            self.write_indicator("-", true, false, true);
            self.node(item, false, false);
        }
        self.pop_indent();
    }

    fn block_map(&mut self, entries: &[(Node, Node)]) {
        self.increase_indent(false, false);
        for (k, v) in entries {
            self.write_indent();
            if simple_key(k) {
                self.node(k, true, true);
                self.write_indicator(":", false, false, false);
            } else {
                self.write_indicator("?", true, false, true);
                self.node(k, true, false);
                self.write_indent();
                self.write_indicator(":", true, false, true);
            }
            self.node(v, true, false);
        }
        self.pop_indent();
    }

    /// libyaml `yaml_emitter_select_scalar_style`.
    fn select(&self, a: &Analysis, len: usize, style: Style) -> Written {
        let requested = match style {
            Style::Any | Style::Plain => Written::Plain,
            Style::Single => Written::Single,
            Style::Literal => Written::Literal,
        };
        let w = if self.simple_key_context && a.multiline {
            Written::Double
        } else {
            requested
        };
        let w = match w {
            Written::Plain if !self.plain_allowed(a, len) => Written::Single,
            w => w,
        };
        match w {
            Written::Single if !a.single => Written::Double,
            Written::Literal if !self.literal_allowed(a) => Written::Double,
            w => w,
        }
    }

    fn plain_allowed(&self, a: &Analysis, len: usize) -> bool {
        let flow = self.flow_level > 0;
        let plain_ok = if flow { a.flow_plain } else { a.block_plain };
        plain_ok && !(len == 0 && (flow || self.simple_key_context))
    }

    fn literal_allowed(&self, a: &Analysis) -> bool {
        a.block && self.flow_level == 0 && !self.simple_key_context
    }

    fn scalar(&mut self, s: &str, style: Style) {
        let a = analyze(s);
        let w = self.select(&a, s.len(), style);
        self.increase_indent(true, false);
        match w {
            Written::Plain => self.plain(s),
            Written::Single => self.single_quoted(s),
            Written::Double => self.double_quoted(s),
            Written::Literal => self.literal(s),
        }
        self.pop_indent();
    }

    fn plain(&mut self, s: &str) {
        if !self.whitespace && (!s.is_empty() || self.flow_level > 0) {
            self.put(' ');
        }
        // no folding (serde_yaml_ng set the width to unlimited) and no breaks
        // (a line break rules plain out)
        for c in s.chars() {
            self.put(c);
        }
        self.whitespace = false;
        self.indention = false;
    }

    fn single_quoted(&mut self, s: &str) {
        self.write_indicator("'", true, false, false);
        let breaks = s
            .chars()
            .fold(false, |breaks, c| self.single_quoted_char(c, breaks));
        if breaks {
            self.write_indent();
        }
        self.write_indicator("'", false, false, false);
        self.whitespace = false;
        self.indention = false;
    }

    /// One character of a single-quoted scalar; `breaks` is whether the last
    /// one was a line break, and the return is whether this one is.
    fn single_quoted_char(&mut self, c: char, breaks: bool) -> bool {
        match c {
            ' ' => {
                self.put(c);
                breaks
            }
            c if is_break(c) => {
                if !breaks && c == '\n' {
                    self.put_break();
                }
                self.write_break(c);
                self.indention = true;
                true
            }
            _ => {
                if breaks {
                    self.write_indent();
                }
                if c == '\'' {
                    self.put('\'');
                }
                self.put(c);
                self.indention = false;
                false
            }
        }
    }

    fn double_quoted(&mut self, s: &str) {
        self.write_indicator("\"", true, false, false);
        for c in s.chars() {
            if needs_escape(c) {
                self.put('\\');
                for e in escape(c).chars() {
                    self.put(e);
                }
            } else {
                self.put(c);
            }
        }
        self.write_indicator("\"", false, false, false);
        self.whitespace = false;
        self.indention = false;
    }

    fn literal(&mut self, s: &str) {
        self.write_indicator("|", true, false, false);
        self.block_scalar_hints(s);
        self.put_break();
        self.indention = true;
        self.whitespace = true;
        let mut breaks = true;
        for c in s.chars() {
            breaks = self.literal_char(c, breaks);
        }
    }

    /// libyaml `yaml_emitter_write_block_scalar_hints`.
    fn block_scalar_hints(&mut self, s: &str) {
        if s.starts_with(' ') || s.starts_with(is_break) {
            self.write_indicator(&BEST_INDENT.to_string(), false, false, false);
        }
        self.open_ended = 0;
        let mut rev = s.chars().rev();
        match (rev.next(), rev.next()) {
            (None, _) => self.write_indicator("-", false, false, false),
            (Some(last), _) if !is_break(last) => {
                self.write_indicator("-", false, false, false);
            }
            (Some(_), None) => {
                self.write_indicator("+", false, false, false);
                self.open_ended = 2;
            }
            (Some(_), Some(prev)) if is_break(prev) => {
                self.write_indicator("+", false, false, false);
                self.open_ended = 2;
            }
            _ => {}
        }
    }

    /// One character of a literal scalar, as [`Self::single_quoted_char`].
    fn literal_char(&mut self, c: char, breaks: bool) -> bool {
        if is_break(c) {
            self.write_break(c);
            self.indention = true;
            return true;
        }
        if breaks {
            self.write_indent();
        }
        self.put(c);
        self.indention = false;
        false
    }
}

/// Whether a double-quoted scalar escapes `c`.
fn needs_escape(c: char) -> bool {
    !printable(c) || c == '\u{FEFF}' || is_break(c) || c == '"' || c == '\\'
}

/// The escape after the `\` for `c`: libyaml's short form, else hex.
fn escape(c: char) -> String {
    let u = c as u32;
    short_escape(u).map_or_else(
        || match u {
            0..=0xFF => format!("x{u:02X}"),
            0x100..=0xFFFF => format!("u{u:04X}"),
            _ => format!("U{u:08X}"),
        },
        String::from,
    )
}

fn short_escape(u: u32) -> Option<char> {
    match u {
        0x00 => Some('0'),
        0x07 => Some('a'),
        0x08 => Some('b'),
        0x09 => Some('t'),
        0x0A => Some('n'),
        0x0B => Some('v'),
        0x0C => Some('f'),
        0x0D => Some('r'),
        0x1B => Some('e'),
        0x22 => Some('"'),
        0x5C => Some('\\'),
        0x85 => Some('N'),
        0xA0 => Some('_'),
        0x2028 => Some('L'),
        0x2029 => Some('P'),
        _ => None,
    }
}

/// libyaml `yaml_emitter_check_simple_key`, for a key node.
fn simple_key(k: &Node) -> bool {
    match k {
        Node::Scalar(s, _) => !analyze(s).multiline && s.len() <= 128,
        Node::Seq(items) => items.is_empty(),
        Node::Map(entries) => entries.is_empty(),
    }
}
