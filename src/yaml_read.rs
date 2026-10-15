//! SEC-2 (#259): the reader. serde-saphyr resolves untagged scalars by its
//! own rules (`017` and `1_000` are numbers to it, strings to `serde_yaml_ng`;
//! a quoted `'1'` fills a `u32` field), so its deserializer cannot keep the
//! call sites' behaviour. This is `serde_yaml_ng` 0.10's event deserializer
//! (`de.rs`, MIT OR Apache-2.0) ported onto the events of granit-parser, the
//! YAML parser serde-saphyr itself uses (re-exported as
//! `serde_saphyr::granit_parser`): the same resolver and the same visitor
//! calls, so a struct, a map or a `serde_json::Value` is built exactly as
//! before. No `unsafe`, no libyaml.
//!
//! Two departures from the port, both to match what the old reader DID:
//! - granit-parser gives a block scalar whose last line ends the input
//!   without a line break a final `\n`; libyaml does not. That one `\n` is
//!   removed (`eof_block_fix`).
//! - [`from_str_singleton`] reads every enum as a one-key map (or a string
//!   for a unit variant), as `serde_yaml_ng::with::singleton_map_recursive`
//!   did for the RHL surface.

use serde::de::{
    self, DeserializeOwned, DeserializeSeed, Expected, IgnoredAny, IntoDeserializer, Unexpected,
    Visitor,
};
use serde_saphyr::granit_parser::{
    ErrorKind, Event as YamlEvent, Options as ParserOptions, ParseResult, Parser,
    ScalarStyle as YamlStyle, ScanError, Scanner, Span, StrInput, Tag, TokenType,
};
use std::collections::BTreeMap;
use std::fmt;
use std::num::ParseIntError;

/// A read error: the message, and the 1-based (line, column) it is at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    msg: String,
    mark: Option<Mark>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mark {
    line: usize,
    col: usize,
}

impl Error {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            mark: None,
        }
    }

    fn at(msg: impl Into<String>, mark: Mark) -> Self {
        Self {
            msg: msg.into(),
            mark: Some(mark),
        }
    }

    /// The 1-based (line, column) of the error, when it has one.
    #[must_use]
    pub fn location(&self) -> Option<(usize, usize)> {
        self.mark.map(|m| (m.line, m.col))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mark {
            Some(m) => write!(f, "{} at line {} column {}", self.msg, m.line, m.col),
            None => f.write_str(&self.msg),
        }
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::new(msg.to_string())
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

/// A visitor error gets the position of the event it was raised on.
fn fix_mark(e: Error, mark: Mark) -> Error {
    if e.mark.is_none() {
        Error {
            mark: Some(mark),
            ..e
        }
    } else {
        e
    }
}

// ───────────────────────── events ─────────────────────────

const TAG_BOOL: &str = "tag:yaml.org,2002:bool";
const TAG_INT: &str = "tag:yaml.org,2002:int";
const TAG_FLOAT: &str = "tag:yaml.org,2002:float";
const TAG_NULL: &str = "tag:yaml.org,2002:null";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Plain,
    Quoted,
    Literal,
    Folded,
}

#[derive(Debug)]
struct Scalar {
    value: String,
    style: Style,
    tag: Option<String>,
}

#[derive(Debug)]
enum Event {
    Alias(usize),
    Scalar(Scalar),
    SequenceStart(Option<String>),
    SequenceEnd,
    MappingStart(Option<String>),
    MappingEnd,
    Void,
}

#[derive(Debug, Default)]
struct Document {
    events: Vec<(Event, Mark)>,
    error: Option<Error>,
    /// Anchor id → index of the anchored event.
    aliases: BTreeMap<usize, usize>,
    /// Where granit-parser refused a quoted scalar's continuation line for
    /// its indentation (see [`load`]).
    quoted_indent: Option<Mark>,
}

/// The first document of `text`, and whether a second one starts.
struct Loaded {
    document: Document,
    more: bool,
}

/// A tag as libyaml spelled it: the resolved handle, then the suffix
/// (`!x` for a local tag, `tag:yaml.org,2002:str` for `!!str`).
fn tag_text(tag: Option<&Tag>) -> Option<String> {
    tag.map(|t| format!("{}{}", t.handle(), t.suffix()))
}

fn mark_of(span: &Span) -> Mark {
    Mark {
        line: span.start.line(),
        col: span.start.col() + 1,
    }
}

/// granit-parser gives a block scalar that ends the input on a line with no
/// line break a final `\n`; libyaml (and so the old reader) does not.
fn eof_block_fix(text: &str, span: &Span, style: Style, value: &mut String) {
    if !matches!(style, Style::Literal | Style::Folded)
        || text.ends_with(['\n', '\r'])
        || !value.ends_with('\n')
    {
        return;
    }
    let last_line = text.rsplit(['\n', '\r']).next().unwrap_or(text);
    if last_line.trim_matches([' ', '\t']).is_empty() {
        return;
    }
    let end = span.end.byte_offset().unwrap_or_else(|| {
        text.char_indices()
            .nth(span.end.index())
            .map_or(text.len(), |(b, _)| b)
    });
    if text
        .get(end..)
        .is_some_and(|rest| rest.chars().all(|c| c == ' ' || c == '\t'))
    {
        value.pop();
    }
}

/// What one parser event does to the document being loaded.
enum Step {
    Skip,
    Push(Event),
    DocumentEnd,
    StreamEnd,
    Fail(Error),
}

fn anchor_at(doc: &mut Document, anchor: usize) {
    if anchor != 0 {
        doc.aliases.insert(anchor, doc.events.len());
    }
}

fn style_of(style: YamlStyle) -> Style {
    match style {
        YamlStyle::Plain => Style::Plain,
        YamlStyle::Literal => Style::Literal,
        YamlStyle::Folded => Style::Folded,
        _ => Style::Quoted,
    }
}

fn step(text: &str, doc: &mut Document, event: YamlEvent<'_>, span: &Span) -> Step {
    match event {
        YamlEvent::StreamEnd => Step::StreamEnd,
        YamlEvent::DocumentEnd => Step::DocumentEnd,
        YamlEvent::Alias(id) if doc.aliases.contains_key(&id) => Step::Push(Event::Alias(id)),
        YamlEvent::Alias(_) => Step::Fail(Error::at("unknown anchor", mark_of(span))),
        YamlEvent::Scalar(value, style, anchor, tag) => {
            anchor_at(doc, anchor);
            let style = style_of(style);
            let mut value = value.into_owned();
            // PMAT-264: granit spells an empty node `~`; libyaml spells it as
            // the empty string, and a `String` field reads what is spelled. A
            // `~` the file spells has a span; an empty node has none.
            if style == Style::Plain && value == "~" && span.start.index() == span.end.index() {
                value.clear();
            }
            eof_block_fix(text, span, style, &mut value);
            Step::Push(Event::Scalar(Scalar {
                value,
                style,
                tag: tag_text(tag.as_deref()),
            }))
        }
        YamlEvent::SequenceStart(_, anchor, tag) => {
            anchor_at(doc, anchor);
            Step::Push(Event::SequenceStart(tag_text(tag.as_deref())))
        }
        YamlEvent::MappingStart(_, anchor, tag) => {
            anchor_at(doc, anchor);
            Step::Push(Event::MappingStart(tag_text(tag.as_deref())))
        }
        YamlEvent::SequenceEnd => Step::Push(Event::SequenceEnd),
        YamlEvent::MappingEnd => Step::Push(Event::MappingEnd),
        // stream and document starts, comments, and any event kind
        // granit-parser may add later carry no node
        _ => Step::Skip,
    }
}

fn mark_of_scan(e: &ScanError) -> Mark {
    let m = e.marker();
    Mark {
        line: m.line(),
        col: m.col() + 1,
    }
}

fn scan_error(e: &ScanError) -> Error {
    Error::at(e.info().to_string(), mark_of_scan(e))
}

/// After the first document ends: does anything but the stream's end
/// follow? A second document, or an error where one would start, is "more
/// than one document" to the old reader.
fn more_follows<'i>(parser: &mut Parser<'i, StrInput<'i>>) -> bool {
    loop {
        match parser.next_event() {
            None | Some(Ok((YamlEvent::StreamEnd, _))) => return false,
            Some(Ok((YamlEvent::Comment(..), _))) => {}
            Some(_) => return true,
        }
    }
}

/// Move the first document's events into `document`, stopping at its end or
/// at the first error; `true` when another document follows it.
fn feed<'i>(text: &str, parser: &mut Parser<'i, StrInput<'i>>, document: &mut Document) -> bool {
    loop {
        match parser.next_event().map(|next| absorb(text, document, next)) {
            Some(Flow::Continue) => {}
            Some(Flow::DocumentEnd) => return more_follows(parser),
            Some(Flow::Stop) | None => return false,
        }
    }
}

/// What [`feed`] does after one parser event.
enum Flow {
    Continue,
    DocumentEnd,
    Stop,
}

fn absorb(text: &str, document: &mut Document, next: ParseResult<'_>) -> Flow {
    if let Err(e) = &next {
        if *e.kind() == ErrorKind::InvalidQuotedScalarIndent {
            document.quoted_indent = Some(mark_of_scan(e));
        }
    }
    let outcome = next
        .map_err(|e| scan_error(&e))
        .map(|(event, span)| (step(text, document, event, &span), mark_of(&span)));
    match outcome {
        Ok((Step::Skip, _)) => Flow::Continue,
        Ok((Step::Push(event), mark)) => {
            document.events.push((event, mark));
            Flow::Continue
        }
        Ok((Step::Fail(e), _)) | Err(e) => {
            document.error = Some(e);
            Flow::Stop
        }
        Ok((Step::DocumentEnd, _)) => Flow::DocumentEnd,
        Ok((Step::StreamEnd, _)) => Flow::Stop,
    }
}

/// libyaml (the old reader's scanner) reads an anchor or alias NAME as a run of
/// `[0-9A-Za-z_-]` and refuses any other character after it but a blank, the
/// end of input or one of `?:,]}%@` and a backtick. granit-parser takes YAML 1.2's
/// wider `ns-anchor-char` (anything but a blank or a flow indicator), so
/// `unit: &!our` was a null here and a scan error there (PMAT-260, found by
/// `test_sec2_prop_damaged_yaml_reads_as_the_oracle_reads_it`). This refuses
/// every name libyaml refuses, using granit-parser's own scanner for the token
/// boundaries; it is stricter than libyaml only where a name runs into a
/// character libyaml would stop at (`&a:b`), which then names a different
/// anchor in each reader anyway.
fn libyaml_refuses_a_name(text: &str) -> Option<Error> {
    for token in Scanner::new(StrInput::new(text)) {
        let Ok(token) = token else {
            return None; // the parser reports scan errors itself
        };
        let (span, kind) = token.into_parts();
        let name = match kind {
            TokenType::Anchor(name) | TokenType::Alias(name) => name,
            _ => continue,
        };
        let word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
        let after = span
            .end
            .byte_offset()
            .and_then(|b| text.get(b..))
            .and_then(|rest| rest.chars().next());
        if !name.chars().all(word) || matches!(after, Some('[' | '{')) {
            return Some(Error::at(
                "did not find expected alphabetic or numeric character, while scanning an anchor",
                mark_of(&span),
            ));
        }
    }
    None
}

/// The first document of `text` as granit-parser reads it.
fn load_once(text: &str) -> Loaded {
    let mut options = ParserOptions::default();
    options.emit_comments = false;
    let mut parser = Parser::new_from_str_with_options(text, options);
    let mut document = Document::default();
    let more = feed(text, &mut parser, &mut document);
    if document.error.is_none() {
        document.error = libyaml_refuses_a_name(text);
    }
    // the stream ended with no node at all: it reads as an empty document
    if document.events.is_empty() && document.error.is_none() {
        document
            .events
            .push((Event::Void, Mark { line: 1, col: 1 }));
    }
    Loaded { document, more }
}

/// The byte range of each line of `text` (1-based line n is index n-1),
/// without its break; `\r\n`, `\r` and `\n` each end one line, as in YAML.
fn line_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = text.as_bytes();
    let mut lines = Vec::new();
    let (mut start, mut i) = (0, 0);
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(start..i);
                start = i + 1;
            }
            b'\r' => {
                lines.push(start..i);
                if bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    lines.push(start..bytes.len());
    lines
}

/// `text` with the leading spaces of line `line` (1-based) widened past every
/// indent a block opened above it, and how many spaces that added; `None`
/// when the line is not one libyaml would have folded: its indent is not
/// spaces up to `col` (1-based, where granit-parser stopped), or it is a
/// document indicator at column 0, which libyaml refuses in a quoted scalar.
fn reindent(text: &str, line: usize, col: usize) -> Option<(String, usize)> {
    let lines = line_ranges(text);
    let range = lines.get(line.checked_sub(1)?)?.clone();
    let row = &text[range.clone()];
    let lead = row.len() - row.trim_start_matches(' ').len();
    if lead + 1 != col {
        return None;
    }
    let rest = &row[lead..];
    let indicator = (rest.starts_with("---") || rest.starts_with("..."))
        && rest[3..]
            .chars()
            .next()
            .is_none_or(|c| c == ' ' || c == '\t');
    if lead == 0 && indicator {
        return None;
    }
    // A block's indent is a column on some line above, so it is less than
    // that line's length: one more than the longest is past all of them.
    let width = lines[..line - 1]
        .iter()
        .map(|r| text[r.clone()].chars().count())
        .max()
        .unwrap_or(0)
        + 1;
    let added = width.checked_sub(lead).filter(|n| *n > 0)?;
    let mut out = String::with_capacity(text.len() + added);
    out.push_str(&text[..range.start]);
    out.push_str(&" ".repeat(width));
    out.push_str(&text[range.start + lead..]);
    Some((out, added))
}

/// Put every mark on a re-indented line back on the column it has in the
/// text as written.
fn unshift(mut loaded: Loaded, added: &BTreeMap<usize, usize>) -> Loaded {
    let back = |m: &mut Mark| {
        if let Some(n) = added.get(&m.line) {
            m.col = m.col.saturating_sub(*n).max(1);
        }
    };
    loaded.document.events.iter_mut().for_each(|(_, m)| back(m));
    if let Some(m) = loaded.document.error.as_mut().and_then(|e| e.mark.as_mut()) {
        back(m);
    }
    loaded
}

/// libyaml applies no indentation rule to the continuation lines of a quoted
/// scalar in block context: it strips their leading blanks and folds them, so
/// `a:\n  b: 'x\ny'` is `{a: {b: x y}}` to it. granit-parser follows YAML 1.2
/// and refuses a continuation line at or left of the block's indent ("invalid
/// indentation in multiline quoted scalar"; PMAT-264, found by
/// `test_sec2_prop_damaged_yaml_reads_as_the_oracle_reads_it`). Each such
/// line is given enough leading spaces to pass, which the folding then strips,
/// and the text is read again; one line per pass, each line at most once.
fn load(text: &str) -> Loaded {
    let mut current = std::borrow::Cow::Borrowed(text);
    let mut added = BTreeMap::new();
    loop {
        let loaded = load_once(&current);
        let next = loaded
            .document
            .quoted_indent
            .filter(|m| !added.contains_key(&m.line))
            .and_then(|m| reindent(&current, m.line, m.col).map(|r| (m.line, r)));
        let Some((line, (wider, n))) = next else {
            return unshift(loaded, &added);
        };
        added.insert(line, n);
        current = std::borrow::Cow::Owned(wider);
    }
}

// ───────────────────────── the deserializer ─────────────────────────

fn run<T>(text: &str, singleton: bool, f: impl FnOnce(&mut De<'_>) -> Result<T>) -> Result<T> {
    let loaded = load(text);
    let mut pos = 0;
    let mut jumps = 0;
    let t = f(&mut De {
        doc: &loaded.document,
        pos: &mut pos,
        jumps: &mut jumps,
        depth: 128,
        current_enum: None,
        singleton,
    })?;
    if let Some(e) = loaded.document.error {
        return Err(e);
    }
    if loaded.more {
        return Err(Error::new(
            "deserializing from YAML containing more than one document is not supported",
        ));
    }
    Ok(t)
}

/// Read one YAML document as `T`, as `serde_yaml_ng::from_str` did.
///
/// # Errors
///
/// Returns [`Error`] when the text is not YAML or not `T`'s shape.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T> {
    run(text, false, |de| T::deserialize(de))
}

/// [`from_str`], with every enum read as a one-key map (a string for a unit
/// variant): `serde_yaml_ng::with::singleton_map_recursive`.
///
/// # Errors
///
/// As [`from_str`].
pub fn from_str_singleton<T: DeserializeOwned>(text: &str) -> Result<T> {
    run(text, true, |de| T::deserialize(de))
}

#[derive(Clone, Copy)]
struct CurrentEnum<'d> {
    name: Option<&'static str>,
    tag: &'d str,
}

struct De<'d> {
    doc: &'d Document,
    pos: &'d mut usize,
    jumps: &'d mut usize,
    depth: u8,
    current_enum: Option<CurrentEnum<'d>>,
    singleton: bool,
}

impl<'d> De<'d> {
    fn child<'c>(&'c mut self) -> De<'c>
    where
        'd: 'c,
    {
        De {
            doc: self.doc,
            pos: self.pos,
            jumps: self.jumps,
            depth: self.depth,
            current_enum: None,
            singleton: self.singleton,
        }
    }

    fn peek_event_mark(&self) -> Result<(&'d Event, Mark)> {
        match self.doc.events.get(*self.pos) {
            Some((event, mark)) => Ok((event, *mark)),
            None => Err(self
                .doc
                .error
                .clone()
                .unwrap_or_else(|| Error::new("EOF while parsing a value"))),
        }
    }

    fn peek_event(&self) -> Result<&'d Event> {
        self.peek_event_mark().map(|(e, _)| e)
    }

    fn next_event_mark(&mut self) -> Result<(&'d Event, Mark)> {
        self.peek_event_mark().map(|(event, mark)| {
            *self.pos += 1;
            self.current_enum = None;
            (event, mark)
        })
    }

    fn next_event(&mut self) -> Result<&'d Event> {
        self.next_event_mark().map(|(e, _)| e)
    }

    fn jump<'a>(&'a mut self, pos: &'a mut usize) -> Result<De<'a>> {
        *self.jumps += 1;
        if *self.jumps > self.doc.events.len() * 100 {
            return Err(Error::new("repetition limit exceeded"));
        }
        match self.doc.aliases.get(pos) {
            Some(found) => {
                *pos = *found;
                Ok(De {
                    doc: self.doc,
                    pos,
                    jumps: self.jumps,
                    depth: self.depth,
                    current_enum: None,
                    singleton: self.singleton,
                })
            }
            None => Err(Error::new("unknown anchor")),
        }
    }

    fn ignore_any(&mut self) -> Result<()> {
        let mut depth = 0usize;
        loop {
            match self.next_event()? {
                Event::Alias(_) | Event::Scalar(_) | Event::Void => {}
                Event::SequenceStart(_) | Event::MappingStart(_) => depth += 1,
                Event::SequenceEnd | Event::MappingEnd => {
                    depth = depth.saturating_sub(1);
                }
            }
            if depth == 0 {
                return Ok(());
            }
        }
    }

    fn recursion_check<T>(
        &mut self,
        mark: Mark,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let previous = self.depth;
        self.depth = match previous.checked_sub(1) {
            Some(d) => d,
            None => return Err(Error::at("recursion limit exceeded", mark)),
        };
        let r = f(self);
        self.depth = previous;
        r
    }

    fn visit_sequence<'de, V: Visitor<'de>>(&mut self, visitor: V, mark: Mark) -> Result<V::Value> {
        let (value, len) = self.recursion_check(mark, |de| {
            let mut seq = SeqAccess {
                empty: false,
                de,
                len: 0,
            };
            let value = visitor.visit_seq(&mut seq)?;
            Ok((value, seq.len))
        })?;
        self.end_sequence(len)?;
        Ok(value)
    }

    fn visit_mapping<'de, V: Visitor<'de>>(&mut self, visitor: V, mark: Mark) -> Result<V::Value> {
        let (value, len) = self.recursion_check(mark, |de| {
            let mut map = MapAccess {
                empty: false,
                de,
                len: 0,
            };
            let value = visitor.visit_map(&mut map)?;
            Ok((value, map.len))
        })?;
        self.end_mapping(len)?;
        Ok(value)
    }

    fn end_sequence(&mut self, len: usize) -> Result<()> {
        let total = {
            let mut seq = SeqAccess {
                empty: false,
                de: self,
                len,
            };
            while de::SeqAccess::next_element::<IgnoredAny>(&mut seq)?.is_some() {}
            seq.len
        };
        self.next_event()?;
        if total == len {
            Ok(())
        } else {
            Err(de::Error::invalid_length(
                total,
                &ExpectedLen("sequence of", len, "element", "elements"),
            ))
        }
    }

    fn end_mapping(&mut self, len: usize) -> Result<()> {
        let total = {
            let mut map = MapAccess {
                empty: false,
                de: self,
                len,
            };
            while de::MapAccess::next_entry::<IgnoredAny, IgnoredAny>(&mut map)?.is_some() {}
            map.len
        };
        self.next_event()?;
        if total == len {
            Ok(())
        } else {
            Err(de::Error::invalid_length(
                total,
                &ExpectedLen("map containing", len, "entry", "entries"),
            ))
        }
    }
}

struct ExpectedLen(&'static str, usize, &'static str, &'static str);

impl Expected for ExpectedLen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let noun = if self.1 == 1 { self.2 } else { self.3 };
        write!(f, "{} {} {noun}", self.0, self.1)
    }
}

struct SeqAccess<'a, 'd> {
    empty: bool,
    de: &'a mut De<'d>,
    len: usize,
}

impl<'de> de::SeqAccess<'de> for SeqAccess<'_, '_> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.empty {
            return Ok(None);
        }
        match self.de.peek_event()? {
            Event::SequenceEnd | Event::Void => Ok(None),
            _ => {
                self.len += 1;
                seed.deserialize(&mut self.de.child()).map(Some)
            }
        }
    }
}

struct MapAccess<'a, 'd> {
    empty: bool,
    de: &'a mut De<'d>,
    len: usize,
}

impl<'de> de::MapAccess<'de> for MapAccess<'_, '_> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.empty {
            return Ok(None);
        }
        match self.de.peek_event()? {
            Event::MappingEnd | Event::Void => Ok(None),
            _ => {
                self.len += 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        seed.deserialize(&mut self.de.child())
    }
}

struct EnumAccess<'a, 'd> {
    de: &'a mut De<'d>,
    name: Option<&'static str>,
    tag: &'d str,
}

impl<'de, 'a, 'd> de::EnumAccess<'de> for EnumAccess<'a, 'd> {
    type Error = Error;
    type Variant = De<'a>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, De<'a>)> {
        let variant = seed.deserialize(IntoDeserializer::<Error>::into_deserializer(self.tag))?;
        let de = De {
            doc: self.de.doc,
            pos: self.de.pos,
            jumps: self.de.jumps,
            depth: self.de.depth,
            current_enum: Some(CurrentEnum {
                name: self.name,
                tag: self.tag,
            }),
            singleton: self.de.singleton,
        };
        Ok((variant, de))
    }
}

impl<'de> de::VariantAccess<'de> for De<'_> {
    type Error = Error;

    fn unit_variant(mut self) -> Result<()> {
        de::Deserialize::deserialize(&mut self)
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(mut self, seed: T) -> Result<T::Value> {
        seed.deserialize(&mut self)
    }

    fn tuple_variant<V: Visitor<'de>>(mut self, _len: usize, visitor: V) -> Result<V::Value> {
        de::Deserializer::deserialize_seq(&mut self, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        mut self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        de::Deserializer::deserialize_struct(&mut self, "", fields, visitor)
    }
}

struct UnitVariantAccess<'a, 'd> {
    de: &'a mut De<'d>,
}

impl<'de> de::EnumAccess<'de> for UnitVariantAccess<'_, '_> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        Ok((seed.deserialize(&mut *self.de)?, self))
    }
}

impl<'de> de::VariantAccess<'de> for UnitVariantAccess<'_, '_> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, _seed: T) -> Result<T::Value> {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value> {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

/// `singleton_map_recursive`'s enum: a one-key map, its key the variant.
struct SingletonEnum<'a, 'd> {
    de: &'a mut De<'d>,
}

impl<'de, 'a, 'd> de::EnumAccess<'de> for SingletonEnum<'a, 'd> {
    type Error = Error;
    type Variant = SingletonVariant<'a, 'd>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, SingletonVariant { de: self.de }))
    }
}

struct SingletonVariant<'a, 'd> {
    de: &'a mut De<'d>,
}

impl SingletonVariant<'_, '_> {
    /// After the one entry, the map must end.
    fn end<T>(self, value: T) -> Result<T> {
        match self.de.peek_event()? {
            Event::MappingEnd => {
                self.de.next_event()?;
                Ok(value)
            }
            _ => Err(de::Error::invalid_length(2, &"map with a single key")),
        }
    }
}

impl<'de> de::VariantAccess<'de> for SingletonVariant<'_, '_> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(de::Error::invalid_type(Unexpected::Map, &"unit variant"))
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let v = seed.deserialize(&mut self.de.child())?;
        self.end(v)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        let v = de::Deserializer::deserialize_seq(&mut self.de.child(), visitor)?;
        self.end(v)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let v = de::Deserializer::deserialize_struct(&mut self.de.child(), "", fields, visitor)?;
        self.end(v)
    }
}

// ───────────────────────── the resolver ─────────────────────────

fn parse_null(scalar: &str) -> Option<()> {
    matches!(scalar, "null" | "Null" | "NULL" | "~").then_some(())
}

fn parse_bool(scalar: &str) -> Option<bool> {
    match scalar {
        "true" | "True" | "TRUE" => Some(true),
        "false" | "False" | "FALSE" => Some(false),
        _ => None,
    }
}

type Radix<T> = fn(&str, u32) -> Result<T, ParseIntError>;

fn prefixed<T>(rest: &str, radix: u32, from: Radix<T>) -> Option<T> {
    if rest.starts_with(['+', '-']) {
        return None;
    }
    from(rest, radix).ok()
}

/// `0x`/`0o`/`0b` digits after an optional `+` already stripped.
fn positive_prefixed<T>(unpositive: &str, from: Radix<T>) -> Option<T> {
    [("0x", 16), ("0o", 8), ("0b", 2)]
        .into_iter()
        .find_map(|(p, radix)| {
            unpositive
                .strip_prefix(p)
                .and_then(|r| prefixed(r, radix, from))
        })
}

/// `-0x`/`-0o`/`-0b` digits.
fn negative_prefixed<T>(scalar: &str, from: Radix<T>) -> Option<T> {
    [("-0x", 16), ("-0o", 8), ("-0b", 2)]
        .into_iter()
        .find_map(|(p, radix)| {
            let rest = scalar.strip_prefix(p)?;
            from(&format!("-{rest}"), radix).ok()
        })
}

/// The scalar without one leading `+`; `None` when a sign follows it.
fn without_plus(scalar: &str) -> Option<&str> {
    match scalar.strip_prefix('+') {
        Some(u) if u.starts_with(['+', '-']) => None,
        Some(u) => Some(u),
        None => Some(scalar),
    }
}

/// Base-10 digits, unless they are YAML 1.2's leading-zero string.
fn decimal<T>(scalar: &str, digits: &str, from: Radix<T>) -> Option<T> {
    if digits_but_not_number(scalar) {
        return None;
    }
    from(digits, 10).ok()
}

fn parse_unsigned_int<T>(scalar: &str, from: Radix<T>) -> Option<T> {
    let unpositive = scalar.strip_prefix('+').unwrap_or(scalar);
    if let Some(int) = positive_prefixed(unpositive, from) {
        return Some(int);
    }
    if unpositive.starts_with(['+', '-']) {
        return None;
    }
    decimal(scalar, unpositive, from)
}

fn parse_signed_int<T>(scalar: &str, from: Radix<T>) -> Option<T> {
    let unpositive = without_plus(scalar)?;
    positive_prefixed(unpositive, from)
        .or_else(|| negative_prefixed(scalar, from))
        .or_else(|| decimal(scalar, unpositive, from))
}

fn parse_negative_int<T>(scalar: &str, from: Radix<T>) -> Option<T> {
    negative_prefixed(scalar, from).or_else(|| decimal(scalar, scalar, from))
}

fn parse_f64(scalar: &str) -> Option<f64> {
    let unpositive = without_plus(scalar)?;
    if matches!(unpositive, ".inf" | ".Inf" | ".INF") {
        return Some(f64::INFINITY);
    }
    if matches!(scalar, "-.inf" | "-.Inf" | "-.INF") {
        return Some(f64::NEG_INFINITY);
    }
    if matches!(scalar, ".nan" | ".NaN" | ".NAN") {
        return Some(f64::NAN.copysign(1.0));
    }
    unpositive.parse::<f64>().ok().filter(|f| f.is_finite())
}

/// Leading zero(s) then digits: a string in YAML 1.2.
fn digits_but_not_number(scalar: &str) -> bool {
    let s = scalar.strip_prefix(['-', '+']).unwrap_or(scalar);
    s.len() > 1 && s.starts_with('0') && s[1..].bytes().all(|b| b.is_ascii_digit())
}

/// An integer as the resolver reads it, in the narrowest type that holds it.
#[derive(Clone, Copy)]
enum IntValue {
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
}

impl IntValue {
    fn parse(v: &str) -> Option<Self> {
        parse_unsigned_int(v, u64::from_str_radix)
            .map(Self::U64)
            .or_else(|| parse_negative_int(v, i64::from_str_radix).map(Self::I64))
            .or_else(|| parse_unsigned_int(v, u128::from_str_radix).map(Self::U128))
            .or_else(|| parse_negative_int(v, i128::from_str_radix).map(Self::I128))
    }

    fn visit<'de, V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self {
            Self::U64(i) => visitor.visit_u64(i),
            Self::I64(i) => visitor.visit_i64(i),
            Self::U128(i) => visitor.visit_u128(i),
            Self::I128(i) => visitor.visit_i128(i),
        }
    }
}

/// What an untagged plain scalar resolves to.
enum Plain {
    Null,
    Bool(bool),
    Int(IntValue),
    Float(f64),
    Str,
}

impl Plain {
    fn resolve(v: &str) -> Self {
        let null = (v.is_empty() || parse_null(v).is_some()).then_some(Self::Null);
        null.or_else(|| parse_bool(v).map(Self::Bool))
            .or_else(|| IntValue::parse(v).map(Self::Int))
            .or_else(|| {
                parse_f64(v)
                    .filter(|_| !digits_but_not_number(v))
                    .map(Self::Float)
            })
            .unwrap_or(Self::Str)
    }
}

fn visit_untagged_scalar<'de, V: Visitor<'de>>(visitor: V, v: &str) -> Result<V::Value> {
    match Plain::resolve(v) {
        Plain::Null => visitor.visit_unit(),
        Plain::Bool(b) => visitor.visit_bool(b),
        Plain::Int(i) => i.visit(visitor),
        Plain::Float(f) => visitor.visit_f64(f),
        Plain::Str => visitor.visit_str(v),
    }
}

fn visit_scalar<'de, V: Visitor<'de>>(
    visitor: V,
    s: &Scalar,
    tagged_already: bool,
) -> Result<V::Value> {
    match (&s.tag, tagged_already) {
        (Some(tag), false) => visit_tagged_scalar(visitor, s, tag),
        _ if s.style == Style::Plain => visit_untagged_scalar(visitor, &s.value),
        _ => visitor.visit_str(&s.value),
    }
}

/// A scalar under its own tag: a core tag must resolve as that type, a local
/// one leaves a plain scalar to the resolver, anything else is a string.
fn visit_tagged_scalar<'de, V: Visitor<'de>>(
    visitor: V,
    s: &Scalar,
    tag: &str,
) -> Result<V::Value> {
    let v = s.value.as_str();
    let bad = |what: &'static str| -> Result<V::Value> {
        Err(de::Error::invalid_value(Unexpected::Str(v), &what))
    };
    match tag {
        TAG_BOOL => parse_bool(v).map_or_else(|| bad("a boolean"), |b| visitor.visit_bool(b)),
        TAG_INT => IntValue::parse(v).map_or_else(|| bad("an integer"), |i| i.visit(visitor)),
        TAG_FLOAT => parse_f64(v).map_or_else(|| bad("a float"), |f| visitor.visit_f64(f)),
        TAG_NULL => parse_null(v).map_or_else(|| bad("null"), |()| visitor.visit_unit()),
        t if t.starts_with('!') && s.style == Style::Plain => visit_untagged_scalar(visitor, v),
        _ => visitor.visit_str(v),
    }
}

/// The enum tag a node carries, if any.
fn enum_tag_of(next: &Event) -> Option<&str> {
    match next {
        Event::Scalar(Scalar { tag, .. })
        | Event::SequenceStart(tag)
        | Event::MappingStart(tag) => parse_tag(tag.as_ref()),
        _ => None,
    }
}

/// `deserialize_option`: whether a scalar is `Some`. A quoted scalar always
/// is; `!!null` must spell null; an untagged plain one is unless it is null.
fn scalar_is_some(s: &Scalar, tagged_already: bool) -> Result<bool> {
    match (&s.tag, tagged_already) {
        _ if s.style != Style::Plain => Ok(true),
        (Some(tag), false) if tag == TAG_NULL => parse_null(&s.value)
            .map(|()| false)
            .ok_or_else(|| de::Error::invalid_value(Unexpected::Str(&s.value), &"null")),
        (Some(_), false) => Ok(true),
        _ => Ok(!s.value.is_empty() && parse_null(&s.value).is_none()),
    }
}

/// `deserialize_unit`: whether a scalar reads as `()`.
fn scalar_is_null(s: &Scalar, tagged_already: bool) -> bool {
    match (&s.tag, tagged_already) {
        _ if s.style != Style::Plain => false,
        (Some(tag), false) => tag == TAG_NULL && parse_null(&s.value).is_some(),
        _ => s.value.is_empty() || parse_null(&s.value).is_some(),
    }
}

fn is_plain_or_tagged_literal_scalar(expected: &str, s: &Scalar, tagged_already: bool) -> bool {
    match (s.style, &s.tag, tagged_already) {
        (Style::Plain, _, _) => true,
        (Style::Literal, Some(tag), false) => tag == expected,
        _ => false,
    }
}

fn invalid_type(event: &Event, exp: &dyn Expected) -> Error {
    enum Void {}
    struct InvalidType<'a> {
        exp: &'a dyn Expected,
    }
    impl<'de> Visitor<'de> for InvalidType<'_> {
        type Value = Void;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.exp.fmt(f)
        }
    }
    match event {
        Event::Scalar(s) => match visit_scalar(InvalidType { exp }, s, false) {
            Ok(void) => match void {},
            Err(e) => e,
        },
        Event::SequenceStart(_) => de::Error::invalid_type(Unexpected::Seq, exp),
        Event::MappingStart(_) => de::Error::invalid_type(Unexpected::Map, exp),
        Event::Void => Error::new("EOF while parsing a value"),
        Event::Alias(_) | Event::SequenceEnd | Event::MappingEnd => Error::new("unexpected event"),
    }
}

/// A `!` tag names an enum variant (libyaml's `!x` → `x`; a bare `!` stays).
fn parse_tag(tag: Option<&String>) -> Option<&str> {
    let t = tag?.as_str();
    let rest = t.strip_prefix('!')?;
    Some(if rest.is_empty() { t } else { rest })
}

macro_rules! deserialize_number {
    ($method:ident, $visit:ident, $tag:expr, $parse:expr) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let tagged_already = self.current_enum.is_some();
            let (next, mark) = self.next_event_mark()?;
            let parsed = match next {
                Event::Alias(pos) => {
                    let mut pos = *pos;
                    return self
                        .jump(&mut pos)?
                        .$method(visitor)
                        .map_err(|e| fix_mark(e, mark));
                }
                Event::Scalar(s) if is_plain_or_tagged_literal_scalar($tag, s, tagged_already) => {
                    $parse(s.value.as_str())
                }
                _ => None,
            };
            match parsed {
                Some(n) => visitor.$visit(n),
                None => Err(invalid_type(next, &visitor)),
            }
            .map_err(|e| fix_mark(e, mark))
        }
    };
}

impl<'de> de::Deserializer<'de> for &mut De<'_> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let tagged_already = self.current_enum.is_some();
        let (next, mark) = self.next_event_mark()?;
        let enum_tag = |tag: &Option<String>| !tagged_already && parse_tag(tag.as_ref()).is_some();
        let r = match next {
            Event::Alias(pos) => {
                let mut pos = *pos;
                self.jump(&mut pos)?.deserialize_any(visitor)
            }
            Event::Scalar(Scalar { tag, .. })
            | Event::SequenceStart(tag)
            | Event::MappingStart(tag)
                if enum_tag(tag) =>
            {
                *self.pos -= 1;
                let tag = parse_tag(tag.as_ref()).unwrap_or_default();
                visitor.visit_enum(EnumAccess {
                    de: self,
                    name: None,
                    tag,
                })
            }
            Event::Scalar(s) => visit_scalar(visitor, s, tagged_already),
            Event::SequenceStart(_) => self.visit_sequence(visitor, mark),
            Event::MappingStart(_) => self.visit_mapping(visitor, mark),
            Event::SequenceEnd | Event::MappingEnd => {
                Err(Error::new("unexpected end of collection"))
            }
            Event::Void => visitor.visit_none(),
        };
        r.map_err(|e| fix_mark(e, mark))
    }

    deserialize_number!(deserialize_bool, visit_bool, TAG_BOOL, parse_bool);

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i64(visitor)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i64(visitor)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i64(visitor)
    }
    deserialize_number!(deserialize_i64, visit_i64, TAG_INT, |v| parse_signed_int(
        v,
        i64::from_str_radix
    ));
    deserialize_number!(deserialize_i128, visit_i128, TAG_INT, |v| parse_signed_int(
        v,
        i128::from_str_radix
    ));
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_u64(visitor)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_u64(visitor)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_u64(visitor)
    }
    deserialize_number!(deserialize_u64, visit_u64, TAG_INT, |v| parse_unsigned_int(
        v,
        u64::from_str_radix
    ));
    deserialize_number!(deserialize_u128, visit_u128, TAG_INT, |v| {
        parse_unsigned_int(v, u128::from_str_radix)
    });
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_f64(visitor)
    }
    deserialize_number!(deserialize_f64, visit_f64, TAG_FLOAT, parse_f64);

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Scalar(s) => visitor.visit_str(&s.value),
            Event::Alias(pos) => {
                let mut pos = *pos;
                self.jump(&mut pos)?.deserialize_str(visitor)
            }
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|e| fix_mark(e, mark))
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::new(
            "serialization and deserialization of bytes in YAML is not implemented",
        ))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let is_some = match self.peek_event()? {
            Event::Alias(pos) => {
                *self.pos += 1;
                let mut pos = *pos;
                return self.jump(&mut pos)?.deserialize_option(visitor);
            }
            Event::Scalar(s) => scalar_is_some(s, self.current_enum.is_some())?,
            Event::SequenceStart(_) | Event::MappingStart(_) => true,
            Event::SequenceEnd | Event::MappingEnd => {
                return Err(Error::new("unexpected end of collection"))
            }
            Event::Void => false,
        };
        if is_some {
            visitor.visit_some(self)
        } else {
            *self.pos += 1;
            self.current_enum = None;
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let tagged_already = self.current_enum.is_some();
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Scalar(s) if scalar_is_null(s, tagged_already) => visitor.visit_unit(),
            Event::Scalar(s) => Err(de::Error::invalid_value(Unexpected::Str(&s.value), &"null")),
            Event::Alias(pos) => {
                let mut pos = *pos;
                self.jump(&mut pos)?.deserialize_unit(visitor)
            }
            Event::Void => visitor.visit_unit(),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|e| fix_mark(e, mark))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        let (_, mark) = self.peek_event_mark()?;
        self.recursion_check(mark, |de| visitor.visit_newtype_struct(de))
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Alias(pos) => {
                let mut pos = *pos;
                self.jump(&mut pos)?.deserialize_seq(visitor)
            }
            Event::SequenceStart(_) => self.visit_sequence(visitor, mark),
            other if is_empty_plain(other) => visitor.visit_seq(SeqAccess {
                empty: true,
                de: self,
                len: 0,
            }),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|e| fix_mark(e, mark))
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (next, mark) = self.next_event_mark()?;
        match next {
            Event::Alias(pos) => {
                let mut pos = *pos;
                self.jump(&mut pos)?.deserialize_map(visitor)
            }
            Event::MappingStart(_) => self.visit_mapping(visitor, mark),
            other if is_empty_plain(other) => visitor.visit_map(MapAccess {
                empty: true,
                de: self,
                len: 0,
            }),
            other => Err(invalid_type(other, &visitor)),
        }
        .map_err(|e| fix_mark(e, mark))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if self.singleton && self.current_enum.is_none() {
            return self.singleton_enum(visitor);
        }
        let (next, mark) = self.peek_event_mark()?;
        if let (None, Some(tag)) = (self.current_enum, enum_tag_of(next)) {
            return visitor.visit_enum(EnumAccess {
                de: self,
                name: Some(name),
                tag,
            });
        }
        let r = match self.current_enum {
            Some(current) => self.nested_enum(next, current, visitor),
            None => self.untagged_enum(next, mark, name, variants, visitor),
        };
        r.map_err(|e| fix_mark(e, mark))
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.ignore_any()?;
        visitor.visit_unit()
    }
}

fn is_empty_plain(event: &Event) -> bool {
    match event {
        Event::Void => true,
        Event::Scalar(s) => s.value.is_empty() && s.style == Style::Plain,
        _ => false,
    }
}

impl<'d> De<'d> {
    /// An enum inside a tagged enum: only a unit variant, by name.
    fn nested_enum<'de, V: Visitor<'de>>(
        &mut self,
        next: &Event,
        current: CurrentEnum<'_>,
        visitor: V,
    ) -> Result<V::Value> {
        match (next, current.name) {
            (Event::Scalar(s), _) if !s.value.is_empty() => {
                visitor.visit_enum(UnitVariantAccess { de: self })
            }
            (_, Some(n)) => Err(Error::new(format!(
                "deserializing nested enum in {n}::{} from YAML is not supported yet",
                current.tag
            ))),
            (_, None) => Err(Error::new(format!(
                "deserializing nested enum in !{} from YAML is not supported yet",
                current.tag
            ))),
        }
    }

    /// An enum with no tag of its own: a scalar names a unit variant.
    fn untagged_enum<'de, V: Visitor<'de>>(
        &mut self,
        next: &'d Event,
        mark: Mark,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let expected = &"a YAML tag starting with '!'";
        match next {
            Event::Alias(pos) => {
                *self.pos += 1;
                let mut pos = *pos;
                de::Deserializer::deserialize_enum(
                    &mut self.jump(&mut pos)?,
                    name,
                    variants,
                    visitor,
                )
            }
            Event::Scalar(_) => visitor.visit_enum(UnitVariantAccess { de: self }),
            Event::MappingStart(_) => Err(fix_mark(
                de::Error::invalid_type(Unexpected::Map, expected),
                mark,
            )),
            Event::SequenceStart(_) => Err(fix_mark(
                de::Error::invalid_type(Unexpected::Seq, expected),
                mark,
            )),
            Event::SequenceEnd | Event::MappingEnd => {
                Err(Error::new("unexpected end of collection"))
            }
            Event::Void => Err(Error::new("EOF while parsing a value")),
        }
    }

    /// `singleton_map_recursive`: the enum is read through `deserialize_any`,
    /// where a string is a unit variant and a one-key map is any variant;
    /// everything else is the wrong type.
    fn singleton_enum<'de, V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        let (next, mark) = self.peek_event_mark()?;
        let r = match next {
            Event::Alias(pos) => {
                *self.pos += 1;
                let mut pos = *pos;
                self.jump(&mut pos)?.singleton_enum(visitor)
            }
            Event::Scalar(s) => {
                // the scalar as `deserialize_any` resolves it: only a string
                // names a unit variant
                match resolved_str(s) {
                    Some(text) => {
                        *self.pos += 1;
                        visitor.visit_enum(IntoDeserializer::<Error>::into_deserializer(text))
                    }
                    None => Err(invalid_type(next, &"map with a single key")),
                }
            }
            Event::MappingStart(tag) if parse_tag(tag.as_ref()).is_none() => {
                *self.pos += 1;
                self.recursion_check(mark, |de| match de.peek_event()? {
                    Event::MappingEnd => {
                        Err(de::Error::invalid_length(0, &"map with a single key"))
                    }
                    _ => visitor.visit_enum(SingletonEnum { de }),
                })
            }
            other => Err(invalid_type(other, &"map with a single key")),
        };
        r.map_err(|e| fix_mark(e, mark))
    }
}

/// The text of a scalar that `deserialize_any` reads as a string, else None.
fn resolved_str(s: &Scalar) -> Option<String> {
    struct StrOnly;
    impl<'de> Visitor<'de> for StrOnly {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a string")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<String>, E> {
            Ok(Some(v.to_string()))
        }
        fn visit_bool<E: de::Error>(self, _: bool) -> Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_u64<E: de::Error>(self, _: u64) -> Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_i64<E: de::Error>(self, _: i64) -> Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_u128<E: de::Error>(self, _: u128) -> Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_i128<E: de::Error>(self, _: i128) -> Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_f64<E: de::Error>(self, _: f64) -> Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<String>, E> {
            Ok(None)
        }
    }
    if parse_tag(s.tag.as_ref()).is_some() {
        return None;
    }
    visit_scalar(StrOnly, s, false).ok().flatten()
}

// ───────────────────────── the untagged check ─────────────────────────

/// A key as `serde_yaml_ng::Value` compares it: resolved scalars, then
/// structure.
#[derive(PartialEq)]
enum Key {
    Null,
    Bool(bool),
    Int(i128),
    Big(u128),
    Float(u64),
    Str(String),
    Seq(Vec<Key>),
    Map(Vec<(Key, Key)>),
}

impl Key {
    fn of_scalar(s: &Scalar) -> Self {
        struct K;
        impl<'de> Visitor<'de> for K {
            type Value = Key;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a scalar")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Key, E> {
                Ok(Key::Str(v.to_string()))
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Key, E> {
                Ok(Key::Bool(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Key, E> {
                Ok(Key::Int(i128::from(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Key, E> {
                Ok(Key::Int(i128::from(v)))
            }
            fn visit_u128<E: de::Error>(self, v: u128) -> Result<Key, E> {
                Ok(Key::Big(v))
            }
            fn visit_i128<E: de::Error>(self, v: i128) -> Result<Key, E> {
                Ok(Key::Int(v))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Key, E> {
                Ok(Key::Float(if v.is_nan() {
                    f64::NAN.to_bits()
                } else {
                    v.to_bits()
                }))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Key, E> {
                Ok(Key::Null)
            }
        }
        visit_scalar(K, s, false).unwrap_or_else(|_| Key::Str(s.value.clone()))
    }
}

/// Walk one node from `pos`, building its key form; refuse a `!` tag or a
/// duplicate key, as reading it into `serde_yaml_ng::Value` and searching
/// for `Value::Tagged` did.
fn walk(doc: &Document, pos: &mut usize, jumps: &mut usize) -> Result<Key, String> {
    let Some((event, _)) = doc.events.get(*pos) else {
        return Ok(Key::Null);
    };
    *pos += 1;
    match event {
        Event::Alias(id) => walk_alias(doc, id, jumps),
        Event::Scalar(s) => local_tag(s.tag.as_ref()).map(|()| Key::of_scalar(s)),
        Event::SequenceStart(tag) => {
            local_tag(tag.as_ref())?;
            walk_seq(doc, pos, jumps)
        }
        Event::MappingStart(tag) => {
            local_tag(tag.as_ref())?;
            walk_map(doc, pos, jumps)
        }
        Event::SequenceEnd | Event::MappingEnd | Event::Void => Ok(Key::Null),
    }
}

/// `Err(tag)` for a local (`!x`) tag, which `serde_yaml_ng::Value` kept as
/// `Value::Tagged`.
fn local_tag(tag: Option<&String>) -> Result<(), String> {
    match tag {
        Some(t) if parse_tag(Some(t)).is_some() => Err(t.clone()),
        _ => Ok(()),
    }
}

fn walk_alias(doc: &Document, id: &usize, jumps: &mut usize) -> Result<Key, String> {
    *jumps += 1;
    if *jumps > doc.events.len() * 100 {
        return Err("repetition limit exceeded".into());
    }
    let mut at = *doc.aliases.get(id).ok_or("unknown anchor")?;
    walk(doc, &mut at, jumps)
}

/// Whether the collection being walked (a map when `map`) ends at `pos`.
fn at_end(doc: &Document, pos: usize, map: bool) -> bool {
    match doc.events.get(pos) {
        None => true,
        Some((Event::SequenceEnd, _)) => !map,
        Some((Event::MappingEnd, _)) => map,
        Some(_) => false,
    }
}

fn walk_seq(doc: &Document, pos: &mut usize, jumps: &mut usize) -> Result<Key, String> {
    let mut items = Vec::new();
    while !at_end(doc, *pos, false) {
        items.push(walk(doc, pos, jumps)?);
    }
    *pos += 1;
    Ok(Key::Seq(items))
}

/// A repeated key is `Err("")`: `serde_yaml_ng` refused one while building
/// its `Value`.
fn walk_map(doc: &Document, pos: &mut usize, jumps: &mut usize) -> Result<Key, String> {
    let mut entries: Vec<(Key, Key)> = Vec::new();
    while !at_end(doc, *pos, true) {
        let k = walk(doc, pos, jumps)?;
        let v = walk(doc, pos, jumps)?;
        if entries.iter().any(|(e, _)| *e == k) {
            return Err(String::new());
        }
        entries.push((k, v));
    }
    *pos += 1;
    Ok(Key::Map(entries))
}

/// Why a document would not read as an untagged `serde_yaml_ng::Value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Untagged {
    /// The text does not read at all.
    Unreadable(Error),
    /// A `!` tag, as libyaml spelled it.
    Tag(String),
    /// A mapping repeats a key.
    DuplicateKey,
}

/// `Ok` when `text` reads as one document with no `!` tag and no repeated
/// key, as `serde_yaml_ng::from_str::<Value>` plus a search for
/// `Value::Tagged` decided.
///
/// # Errors
///
/// Returns why it does not.
pub fn untagged(text: &str) -> Result<(), Untagged> {
    let loaded = load(text);
    if let Some(e) = loaded.document.error {
        return Err(Untagged::Unreadable(e));
    }
    if loaded.more {
        return Err(Untagged::Unreadable(Error::new(
            "deserializing from YAML containing more than one document is not supported",
        )));
    }
    let mut pos = 0;
    let mut jumps = 0;
    match walk(&loaded.document, &mut pos, &mut jumps) {
        Ok(_) => Ok(()),
        Err(t) if t.is_empty() => Err(Untagged::DuplicateKey),
        Err(t) if t.starts_with('!') => Err(Untagged::Tag(t)),
        Err(t) => Err(Untagged::Unreadable(Error::new(t))),
    }
}
