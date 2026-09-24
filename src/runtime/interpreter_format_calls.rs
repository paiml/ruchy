//! FMTSPEC-1 / EPRINTLN-1 / FORMATFN-1: the output and format calls
//! (`println`, `print`, `eprintln`, `eprint`, `format`, function or macro
//! form) all format through [`crate::runtime::fmt_spec`].
//!
//! Calls made through the interpreter resolve `{name}` against explicit named
//! arguments (`name = expr`) and then variables in scope, as Rust 2021 inline
//! format arguments do. A string literal first argument is always a format
//! string, as in Rust.

use crate::frontend::ast::{Expr, ExprKind, Literal};
use crate::runtime::fmt_spec;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::io::Write;

/// Where the formatted text goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatSink {
    Print,
    Println,
    Eprint,
    Eprintln,
    Return,
}

impl FormatSink {
    /// The sink of a builtin output/format function or macro name.
    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "print" => Some(Self::Print),
            "println" => Some(Self::Println),
            "eprint" => Some(Self::Eprint),
            "eprintln" => Some(Self::Eprintln),
            "format" => Some(Self::Return),
            _ => None,
        }
    }

    /// The sink of a `__builtin_*__` marker.
    pub fn for_marker(marker: &str) -> Option<Self> {
        let name = marker.strip_prefix("__builtin_")?.strip_suffix("__")?;
        Self::for_name(name)
    }

    fn marker(self) -> &'static str {
        match self {
            Self::Print => "__builtin_print__",
            Self::Println => "__builtin_println__",
            Self::Eprint => "__builtin_eprint__",
            Self::Eprintln => "__builtin_eprintln__",
            Self::Return => "__builtin_format__",
        }
    }
}

/// Deliver `text` to `sink`; `format` returns it as a string value.
pub fn emit(sink: FormatSink, text: String) -> Value {
    match sink {
        FormatSink::Return => return Value::from_string(text),
        FormatSink::Print => write_stdout(&text),
        FormatSink::Println => write_stdout(&format!("{text}\n")),
        FormatSink::Eprint => eprint!("{text}"),
        FormatSink::Eprintln => eprintln!("{text}"),
    }
    Value::Nil
}

fn write_stdout(text: &str) {
    // Notebook capture reads the output buffer.
    if let Ok(mut buf) = crate::runtime::builtins::OUTPUT_BUFFER.lock() {
        buf.push_str(text);
    }
    print!("{text}");
    let _ = std::io::stdout().flush();
}

fn format_error(e: String) -> InterpreterError {
    InterpreterError::RuntimeError(format!("format error: {e}"))
}

/// The builtin path (no scope): `println` called through a value. The first
/// value is a format string when more values follow it.
pub fn eval_builtin_format(sink: FormatSink, args: &[Value]) -> Result<Value, InterpreterError> {
    let template = args.len() > 1;
    let text = fmt_spec::format_call(args, template, &|_| None).map_err(format_error)?;
    Ok(emit(sink, text))
}

fn is_string_literal(expr: &Expr) -> bool {
    matches!(expr.kind, ExprKind::Literal(Literal::String(_)))
}

/// `name = expr` in argument position (not the first argument).
fn named_arg(expr: &Expr) -> Option<(&str, &Expr)> {
    match &expr.kind {
        ExprKind::Assign { target, value } => match &target.kind {
            ExprKind::Identifier(name) => Some((name.as_str(), value.as_ref())),
            _ => None,
        },
        _ => None,
    }
}

type NamedArgs = Vec<(String, Value)>;

impl Interpreter {
    /// A call expression. Output/format builtins that the program has not
    /// shadowed format with scope access; every other call is ordinary.
    pub(crate) fn eval_call_expr(
        &mut self,
        func: &Expr,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        match self.format_sink_for(func) {
            Some(sink) => self.eval_format_call(sink, args),
            None => self.eval_function_call(func, args),
        }
    }

    fn format_sink_for(&self, func: &Expr) -> Option<FormatSink> {
        let ExprKind::Identifier(name) = &func.kind else {
            return None;
        };
        let sink = FormatSink::for_name(name)?;
        match self.lookup_variable(name) {
            Ok(Value::String(marker)) if marker.as_ref() == sink.marker() => Some(sink),
            _ => None,
        }
    }

    /// Format `args` (a format string and its arguments) and deliver the text.
    pub(crate) fn eval_format_call(
        &mut self,
        sink: FormatSink,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        let (positional, named) = self.eval_format_args(args)?;
        // FMTTEMPLATE-1: only a string literal is a format template, as in
        // the transpiler; `println(s, 1)` prints its arguments joined by
        // spaces.
        let template = args.first().is_some_and(is_string_literal);
        let resolve = |name: &str| {
            named
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .or_else(|| self.lookup_variable(name).ok())
        };
        let text = fmt_spec::format_call(&positional, template, &resolve).map_err(format_error)?;
        Ok(emit(sink, text))
    }

    fn eval_format_args(
        &mut self,
        args: &[Expr],
    ) -> Result<(Vec<Value>, NamedArgs), InterpreterError> {
        let mut positional = Vec::new();
        let mut named = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            match named_arg(arg).filter(|_| i > 0) {
                Some((name, value)) => named.push((name.to_string(), self.eval_expr(value)?)),
                None => positional.push(self.eval_expr(arg)?),
            }
        }
        Ok((positional, named))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmtspec_1_sink_names_and_markers() {
        for name in ["print", "println", "eprint", "eprintln", "format"] {
            let sink = FormatSink::for_name(name).expect(name);
            assert_eq!(FormatSink::for_marker(sink.marker()), Some(sink));
        }
        assert_eq!(FormatSink::for_name("dbg"), None);
        assert_eq!(FormatSink::for_marker("println"), None);
    }

    #[test]
    fn test_formatfn_1_builtin_format_returns_string() {
        let args = [
            Value::from_string("{:?}".to_string()),
            Value::from_array(vec![Value::Integer(1), Value::Integer(2)]),
        ];
        let got = eval_builtin_format(FormatSink::Return, &args).unwrap();
        assert_eq!(got, Value::from_string("[1, 2]".to_string()));
    }

    #[test]
    fn test_fmtspec_1_builtin_format_error_is_runtime_error() {
        let args = [Value::from_string("{:1$}".to_string()), Value::Integer(1)];
        let err = eval_builtin_format(FormatSink::Return, &args).unwrap_err();
        assert!(format!("{err:?}").contains("1$"));
    }
}
