//! EXTENDKW-1: member names after `.` and `?.`.
//!
//! A keyword token in member position (`v.extend(x)`, `o.type`, `o.match(p)`)
//! names a method or field; it is spelled as its source text. `await` is not
//! listed: `x.await` is an await expression and is handled before this lookup.
//! `self`/`super`/`crate`/`Self` are not listed: Rust cannot name a member
//! with them, not even as a raw identifier.
use super::Token;

/// Keyword tokens accepted as a member name, paired with their source text.
static KEYWORD_MEMBERS: &[(Token, &str)] = &[
    (Token::Extend, "extend"),
    (Token::Match, "match"),
    (Token::Type, "type"),
    (Token::Loop, "loop"),
    (Token::Impl, "impl"),
    (Token::Mod, "mod"),
    (Token::Use, "use"),
    (Token::As, "as"),
    (Token::In, "in"),
    (Token::Where, "where"),
    (Token::Static, "static"),
    (Token::Const, "const"),
    (Token::Trait, "trait"),
    (Token::Struct, "struct"),
    (Token::Enum, "enum"),
    (Token::Try, "try"),
    (Token::Yield, "yield"),
    (Token::Final, "final"),
    (Token::Abstract, "abstract"),
    (Token::Override, "override"),
    (Token::Async, "async"),
    (Token::Return, "return"),
    (Token::Break, "break"),
    (Token::Continue, "continue"),
    (Token::If, "if"),
    (Token::Else, "else"),
    (Token::For, "for"),
    (Token::While, "while"),
    (Token::Let, "let"),
    (Token::Fn, "fn"),
    (Token::Pub, "pub"),
    (Token::Mut, "mut"),
    (Token::Unsafe, "unsafe"),
    (Token::From, "from"),
    (Token::Default, "default"),
    (Token::Module, "module"),
    (Token::Var, "var"),
    (Token::With, "with"),
    (Token::Import, "import"),
    (Token::Export, "export"),
    (Token::Class, "class"),
    (Token::Handle, "handle"),
    (Token::Handler, "handler"),
    (Token::Effect, "effect"),
    (Token::Property, "property"),
    (Token::Private, "private"),
    (Token::Protected, "protected"),
    (Token::Sealed, "sealed"),
    (Token::Mixin, "mixin"),
    (Token::Operator, "operator"),
    (Token::Interface, "interface"),
    (Token::Implements, "implements"),
    (Token::Receive, "receive"),
    (Token::Spawn, "spawn"),
    (Token::Actor, "actor"),
    (Token::Lazy, "lazy"),
    (Token::Catch, "catch"),
    (Token::Finally, "finally"),
    (Token::Throw, "throw"),
    (Token::Signal, "signal"),
    (Token::Infra, "infra"),
    (Token::Requires, "requires"),
    (Token::Ensures, "ensures"),
    (Token::Invariant_, "invariant"),
    (Token::Decreases, "decreases"),
    (Token::Fun, "fun"),
    (Token::Send, "send"),
    (Token::Ask, "ask"),
];

/// Source text of a keyword token usable as a member name (complexity: 1).
fn keyword_member_name(token: &Token) -> Option<&'static str> {
    KEYWORD_MEMBERS
        .iter()
        .find(|(keyword, _)| keyword == token)
        .map(|(_, name)| *name)
}

/// Member name spelled by `token`: an identifier or a keyword (complexity: 2).
pub(super) fn member_name(token: &Token) -> Option<String> {
    match token {
        Token::Identifier(name) => Some(name.clone()),
        other => keyword_member_name(other).map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extendkw_1_member_name_identifier() {
        assert_eq!(
            member_name(&Token::Identifier("len".to_string())),
            Some("len".to_string())
        );
    }

    #[test]
    fn test_extendkw_1_member_name_keywords() {
        assert_eq!(member_name(&Token::Extend), Some("extend".to_string()));
        assert_eq!(member_name(&Token::Match), Some("match".to_string()));
        assert_eq!(
            member_name(&Token::Invariant_),
            Some("invariant".to_string())
        );
    }

    #[test]
    fn test_extendkw_1_member_name_rejects_await_and_paths() {
        for token in [
            Token::Await,
            Token::Self_,
            Token::Super,
            Token::Crate,
            Token::Integer("0".to_string()),
        ] {
            assert_eq!(member_name(&token), None, "{token:?}");
        }
    }

    #[test]
    fn test_extendkw_1_keyword_member_spelling_matches_lexer() {
        for (token, name) in KEYWORD_MEMBERS {
            let mut stream = crate::frontend::lexer::TokenStream::new(name);
            let first = stream.next().map(|(t, _)| t);
            assert_eq!(first.as_ref(), Some(token), "{name}");
        }
    }
}
