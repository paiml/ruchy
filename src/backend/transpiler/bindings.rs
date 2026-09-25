//! Let Binding Transpilation
//!
//! This module handles transpilation of let bindings:
//! - Simple let bindings
//! - Pattern destructuring
//! - Type-annotated bindings
//! - Let-else patterns
//!
//! **EXTREME TDD Round 54**: Extracted from statements.rs for modularization.
#![allow(clippy::doc_markdown)]

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal, Pattern, Type, TypeKind};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Helper: Generate let binding statement with mutability and optional Vec type hint
    /// Reduces cognitive complexity by extracting repeated 4-branch pattern
    pub(super) fn generate_let_binding(
        name_ident: &proc_macro2::Ident,
        is_mutable: bool,
        needs_vec_type_hint: bool,
        value_tokens: &TokenStream,
    ) -> TokenStream {
        match (is_mutable, needs_vec_type_hint) {
            (true, true) => quote! { let mut #name_ident: Vec<_> = #value_tokens; },
            (true, false) => quote! { let mut #name_ident = #value_tokens; },
            (false, true) => quote! { let #name_ident: Vec<_> = #value_tokens; },
            (false, false) => quote! { let #name_ident = #value_tokens; },
        }
    }

    /// Helper: Validate exact argument count (CERTEZA-001: Reduce duplication)
    /// Complexity: 1 (within Toyota Way limits)
    #[inline]
    pub(super) fn require_exact_args(
        method: &str,
        args: &[TokenStream],
        expected: usize,
    ) -> Result<()> {
        if args.len() != expected {
            bail!(
                "{method} requires exactly {expected} argument{}",
                if expected == 1 { "" } else { "s" }
            );
        }
        Ok(())
    }

    /// Helper: Validate no arguments (CERTEZA-001: Reduce duplication)
    /// Complexity: 1 (within Toyota Way limits)
    #[inline]
    pub(super) fn require_no_args(method: &str, args: &[TokenStream]) -> Result<()> {
        if !args.is_empty() {
            bail!("{method} requires no arguments");
        }
        Ok(())
    }

    /// Transpiles let bindings
    /// Complexity: 9 (within Toyota Way limits)
    pub fn transpile_let(
        &self,
        name: &str,
        value: &Expr,
        body: &Expr,
        is_mutable: bool,
    ) -> Result<TokenStream> {
        // Handle Rust reserved keywords by prefixing with r#
        // RHLGA-1 F7: `safe_ident` never r#-escapes `self`/`Self`/`super`/`crate`.
        let name_ident = Self::safe_ident(name);
        self.track_string_binding(name, value, None);
        // NESTARRVEC-1: inner array literals of a binding whose elements grow
        let inner_vec_value = elem_grown_value(name, value, body);
        let value = inner_vec_value.as_ref().unwrap_or(value);
        self.track_inner_vec_binding(name, value);

        // Auto-detect mutability
        let effective_mutability = is_mutable
            || self.mutable_vars.contains(name)
            || super::mutation_detection::is_variable_mutated(name, body);

        // TRANSPILER-007: Detect empty list literals that need type hints
        let (value_tokens, needs_vec_type_hint) = match &value.kind {
            ExprKind::Literal(Literal::String(s)) => (quote! { #s.to_string() }, false),
            ExprKind::List(items) if items.is_empty() => (self.transpile_expr(value)?, true),
            _ => (self.transpile_let_value(name, value)?, false),
        };
        let value_tokens = Self::growable_list_tokens(name, value, None, body, value_tokens);

        // HOTFIX: If body is Unit, this is a top-level let statement without scoping
        if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
            Ok(Self::generate_let_binding(
                &name_ident,
                effective_mutability,
                needs_vec_type_hint,
                &value_tokens,
            ))
        } else {
            // GLOBALSHADOW-1: the body sees the local, not a global of the same name
            self.with_globals_shadowed(&[name.to_string()], || {
                self.transpile_let_with_body(
                    &name_ident,
                    effective_mutability,
                    needs_vec_type_hint,
                    &value_tokens,
                    body,
                )
            })
        }
    }

    /// Helper for transpile_let when body is not Unit
    fn transpile_let_with_body(
        &self,
        name_ident: &proc_macro2::Ident,
        effective_mutability: bool,
        needs_vec_type_hint: bool,
        value_tokens: &TokenStream,
        body: &Expr,
    ) -> Result<TokenStream> {
        // Check if body is a Block containing sequential let statements
        if let ExprKind::Block(exprs) = &body.kind {
            self.transpile_let_block(
                name_ident,
                effective_mutability,
                needs_vec_type_hint,
                value_tokens,
                exprs,
            )
        } else if let ExprKind::Let {
            name: inner_name,
            value: inner_value,
            body: inner_body,
            is_mutable: inner_mutable,
            type_annotation: _,
            else_block: _,
        } = &body.kind
        {
            // Flatten nested let expressions
            let mut statements = Vec::new();
            statements.push(Self::generate_let_binding(
                name_ident,
                effective_mutability,
                needs_vec_type_hint,
                value_tokens,
            ));
            let inner_tokens =
                self.transpile_let(inner_name, inner_value, inner_body, *inner_mutable)?;
            statements.push(inner_tokens);
            Ok(quote! { #(#statements)* })
        } else {
            // Traditional let-in expression with proper scoping
            let body_tokens = self.transpile_expr(body)?;
            let let_binding = Self::generate_let_binding(
                name_ident,
                effective_mutability,
                needs_vec_type_hint,
                value_tokens,
            );
            Ok(quote! {
                {
                    #let_binding
                    #body_tokens
                }
            })
        }
    }

    /// Helper for transpile_let with block body
    fn transpile_let_block(
        &self,
        name_ident: &proc_macro2::Ident,
        effective_mutability: bool,
        needs_vec_type_hint: bool,
        value_tokens: &TokenStream,
        exprs: &[Expr],
    ) -> Result<TokenStream> {
        let mut statements = Vec::new();
        statements.push(Self::generate_let_binding(
            name_ident,
            effective_mutability,
            needs_vec_type_hint,
            value_tokens,
        ));

        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            let is_let = matches!(
                &expr.kind,
                ExprKind::Let { .. } | ExprKind::LetPattern { .. }
            );
            if is_let {
                statements.push(expr_tokens);
            } else if i < exprs.len() - 1 {
                statements.push(quote! { #expr_tokens; });
            } else if super::function_analysis::is_void_expression(expr) {
                statements.push(quote! { #expr_tokens; });
            } else {
                statements.push(expr_tokens);
            }
        }
        Ok(quote! { #(#statements)* })
    }

    /// Transpiles let pattern bindings (destructuring)
    pub fn transpile_let_pattern(
        &self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let mut value_tokens = self.transpile_expr(value)?;

        // Check if we're pattern matching on a list that needs to be converted to a slice
        if self.pattern_needs_slice(pattern) && self.value_creates_vec(value) {
            value_tokens = quote! { &#value_tokens[..] };
        }

        // HOTFIX: If body is Unit, this is a top-level let statement
        if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
            self.transpile_let_pattern_unit_body(pattern, &value_tokens, &pattern_tokens)
        } else {
            // GLOBALSHADOW-1: the body sees the pattern's bindings, not globals
            let bound = super::pattern_bindings::extract_pattern_bindings(pattern);
            let body_tokens = self.with_globals_shadowed(&bound, || self.transpile_expr(body))?;
            Ok(quote! {
                {
                    let #pattern_tokens = #value_tokens;
                    #body_tokens
                }
            })
        }
    }

    /// Helper for transpile_let_pattern when body is Unit
    fn transpile_let_pattern_unit_body(
        &self,
        pattern: &Pattern,
        value_tokens: &TokenStream,
        pattern_tokens: &TokenStream,
    ) -> Result<TokenStream> {
        match pattern {
            Pattern::List(patterns) => {
                let mut assignments = Vec::new();
                for (i, pat) in patterns.iter().enumerate() {
                    if let Pattern::Identifier(name) = pat {
                        let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
                        assignments.push(quote! {
                            let #ident = #value_tokens[#i].clone();
                        });
                    }
                }
                Ok(quote! { #(#assignments)* })
            }
            _ => Ok(quote! { let #pattern_tokens = #value_tokens }),
        }
    }

    /// Transpiles let bindings with optional type annotations
    /// Complexity: 9 (within Toyota Way limits)
    pub fn transpile_let_with_type(
        &self,
        name: &str,
        type_annotation: Option<&Type>,
        value: &Expr,
        body: &Expr,
        is_mutable: bool,
        is_const: bool,
    ) -> Result<TokenStream> {
        // RHLGA-1 F7: `safe_ident` never r#-escapes `self`/`Self`/`super`/`crate`.
        let name_ident = Self::safe_ident(name);
        self.track_string_binding(name, value, type_annotation);
        self.track_inner_vec_binding(name, value);

        // PARSER-073: Generate const/let keyword based on const attribute
        let is_mutable_var = is_mutable
            || self.mutable_vars.contains(name)
            || super::mutation_detection::is_variable_mutated(name, body);

        let var_keyword = if is_const {
            quote! { const }
        } else if is_mutable_var {
            quote! { let mut }
        } else {
            quote! { let }
        };

        // Handle value tokens and type hints
        let (value_tokens, needs_vec_type_hint) =
            self.process_let_value_with_type(name, value, type_annotation, is_mutable_var)?;
        let value_tokens = if is_const {
            value_tokens
        } else {
            Self::growable_list_tokens(name, value, type_annotation, body, value_tokens)
        };
        let value_tokens = Self::cast_usize_to_annotation(value, type_annotation, value_tokens);

        // Generate type annotation
        let type_tokens = self.generate_type_tokens(type_annotation, needs_vec_type_hint)?;

        // Check if body is Unit
        if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
            Ok(quote! {
                #var_keyword #name_ident #type_tokens = #value_tokens;
            })
        } else {
            // GLOBALSHADOW-1: the body sees the local, not a global of the same name
            let body_tokens =
                self.with_globals_shadowed(&[name.to_string()], || self.transpile_expr(body))?;
            Ok(quote! {
                {
                    #var_keyword #name_ident #type_tokens = #value_tokens;
                    #body_tokens
                }
            })
        }
    }

    /// INTLEN-1: `let n: int = v.len()` casts the `usize` value to the
    /// annotated integer type. (complexity: 2)
    fn cast_usize_to_annotation(
        value: &Expr,
        type_annotation: Option<&Type>,
        tokens: TokenStream,
    ) -> TokenStream {
        match type_annotation {
            Some(ty) => super::return_type_helpers::cast_usize_tokens(
                value,
                &Self::type_to_string(ty),
                tokens,
            ),
            None => tokens,
        }
    }

    /// LETVEC-1: a non-empty array literal bound by `let` becomes `vec![..]`
    /// when the binding is annotated `Vec<..>` or is grown later in `body`
    /// (the receiver of a Vec-only method). Otherwise `tokens` is returned
    /// unchanged, so a never-grown literal stays a fixed-size array.
    fn growable_list_tokens(
        name: &str,
        value: &Expr,
        type_annotation: Option<&Type>,
        body: &Expr,
        tokens: TokenStream,
    ) -> TokenStream {
        let is_literal = matches!(&value.kind, ExprKind::List(items) if !items.is_empty());
        let text = tokens.to_string();
        let is_array_tokens = text.starts_with('[') && text.ends_with(']');
        let needs_vec = type_annotation.is_some_and(is_vec_annotation)
            || super::mutation_detection::is_grown_as_vec(name, body);
        if is_literal && is_array_tokens && needs_vec {
            quote! { vec! #tokens }
        } else {
            tokens
        }
    }

    /// PRINTPARAM-1: the value of `let name = value`; a closure value knows
    /// the name it is called by. (complexity: 2)
    fn transpile_let_value(&self, name: &str, value: &Expr) -> Result<TokenStream> {
        match &value.kind {
            ExprKind::Lambda { params, body } => {
                self.transpile_named_lambda(Some(name), params, body)
            }
            _ => self.transpile_expr(value),
        }
    }

    /// NESTPUSHLIT-1: remember whether `name` is bound to a list literal
    /// whose inner literals are `vec![..]` (the NESTARRVEC-1 rewrite); any
    /// other binding of `name` clears the record. (complexity: 2)
    fn track_inner_vec_binding(&self, name: &str, value: &Expr) {
        let mut lists = self.inner_vec_lists.borrow_mut();
        if has_vec_inner_literals(value) {
            lists.insert(name.to_string());
        } else {
            lists.remove(name);
        }
    }

    /// NESTPUSHLIT-1: the arguments of `object.push(x)` / `object.insert(i, x)`
    /// with an array literal `x` as `vec![..]`, when `object` is a binding
    /// whose inner literals are `vec![..]`; `None` otherwise. (complexity: 4)
    pub(crate) fn inner_vec_push_args(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Option<Vec<Expr>> {
        let ExprKind::Identifier(name) = &object.kind else {
            return None;
        };
        if !matches!(method, "push" | "insert") || !self.inner_vec_lists.borrow().contains(name) {
            return None;
        }
        let mut rewritten = args.to_vec();
        let element = rewritten.last_mut()?;
        *element = list_as_vec(element);
        Some(rewritten)
    }

    /// LETVEC-1: transpile one statement of a block. A statement-level `let`
    /// (Unit body) whose scope is the following siblings `rest` gets its
    /// array literal emitted as `vec![..]` when a sibling grows the binding.
    pub(crate) fn transpile_block_statement(
        &self,
        expr: &Expr,
        rest: &[Expr],
    ) -> Result<TokenStream> {
        match grown_let_as_vec(expr, rest) {
            Some(rewritten) => self.transpile_expr(&rewritten),
            None => self.transpile_expr(expr),
        }
    }

    /// Process value for let-with-type, handling string/list conversions
    fn process_let_value_with_type(
        &self,
        name: &str,
        value: &Expr,
        type_annotation: Option<&Type>,
        is_mutable_var: bool,
    ) -> Result<(TokenStream, bool)> {
        match (&value.kind, type_annotation) {
            (ExprKind::Literal(Literal::String(s)), Some(type_ann)) if matches!(&type_ann.kind, TypeKind::Named(n) if n == "String") =>
            {
                // TRANSPILER-STRING-CONCAT-001 FIX: Register string variable for concatenation tracking
                self.string_vars.borrow_mut().insert(name.to_string());
                Ok((quote! { #s.to_string() }, false))
            }
            (ExprKind::Literal(Literal::String(s)), None) if is_mutable_var => {
                // TRANSPILER-STRING-CONCAT-001 FIX: Register string variable for concatenation tracking
                self.string_vars.borrow_mut().insert(name.to_string());
                Ok((quote! { String::from(#s) }, false))
            }
            // TRANSPILER-STRING-CONCAT-001 FIX: Handle immutable string literals too
            (ExprKind::Literal(Literal::String(_)), None) => {
                self.string_vars.borrow_mut().insert(name.to_string());
                Ok((self.transpile_expr(value)?, false))
            }
            (ExprKind::List(_), Some(type_ann)) if matches!(&type_ann.kind, TypeKind::List(_)) => {
                let list_tokens = self.transpile_expr(value)?;
                Ok((quote! { #list_tokens.to_vec() }, false))
            }
            (ExprKind::List(elements), None) if elements.is_empty() => {
                Ok((self.transpile_expr(value)?, true))
            }
            (ExprKind::Call { .. }, _) => {
                self.string_vars.borrow_mut().insert(name.to_string());
                Ok((self.transpile_expr(value)?, false))
            }
            _ => Ok((self.transpile_let_value(name, value)?, false)),
        }
    }

    /// Generate type annotation tokens
    fn generate_type_tokens(
        &self,
        type_annotation: Option<&Type>,
        needs_vec_type_hint: bool,
    ) -> Result<TokenStream> {
        if let Some(type_ann) = type_annotation {
            let type_part = self.transpile_type(type_ann)?;
            Ok(quote! { : #type_part })
        } else if needs_vec_type_hint {
            self.infer_vec_type_hint()
        } else {
            Ok(quote! {})
        }
    }

    /// Infer Vec type hint from function return type
    fn infer_vec_type_hint(&self) -> Result<TokenStream> {
        if let Some(ret_type) = self.current_function_return_type.borrow().as_ref() {
            match &ret_type.kind {
                TypeKind::List(inner_type) => {
                    let inner_tokens = self.transpile_type(inner_type)?;
                    Ok(quote! { : Vec<#inner_tokens> })
                }
                TypeKind::Generic { base, params } if base == "Vec" && params.len() == 1 => {
                    let inner_tokens = self.transpile_type(&params[0])?;
                    Ok(quote! { : Vec<#inner_tokens> })
                }
                _ => Ok(quote! { : Vec<_> }),
            }
        } else {
            Ok(quote! { : Vec<_> })
        }
    }

    /// Transpiles let pattern bindings with optional type annotations
    pub fn transpile_let_pattern_with_type(
        &self,
        pattern: &Pattern,
        type_annotation: Option<&Type>,
        value: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let mut value_tokens = self.transpile_expr(value)?;

        if self.pattern_needs_slice(pattern) && self.value_creates_vec(value) {
            value_tokens = quote! { (#value_tokens).as_slice() };
        }

        if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
            Ok(quote! {
                let #pattern_tokens = #value_tokens;
            })
        } else {
            let body_tokens = self.transpile_expr(body)?;
            if type_annotation.is_some() {
                Ok(quote! {
                    {
                        // Type annotation would be applied here if supported
                        let #pattern_tokens = #value_tokens;
                        #body_tokens
                    }
                })
            } else {
                Ok(quote! {
                    {
                        let #pattern_tokens = #value_tokens;
                        #body_tokens
                    }
                })
            }
        }
    }

    /// Transpile let-else for simple identifier binding
    pub fn transpile_let_else(
        &self,
        name: &str,
        value: &Expr,
        body: &Expr,
        else_block: &Expr,
    ) -> Result<TokenStream> {
        let name_ident = Self::safe_ident(name); // RAWIDENT-2
        let value_tokens = self.transpile_expr(value)?;
        let else_tokens = self.transpile_expr(else_block)?;
        // GLOBALSHADOW-1: the body sees the local, not a global of the same name
        let body_tokens =
            self.with_globals_shadowed(&[name.to_string()], || self.transpile_expr(body))?;

        Ok(quote! {
            {
                let #name_ident = #value_tokens;
                if #name_ident.is_none() {
                    #else_tokens
                }
                #body_tokens
            }
        })
    }

    /// Transpile let-else with pattern matching
    pub fn transpile_let_pattern_else(
        &self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
        else_block: &Expr,
    ) -> Result<TokenStream> {
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let value_tokens = self.transpile_expr(value)?;
        let else_tokens = self.transpile_expr(else_block)?;
        let bound_vars = super::pattern_bindings::extract_pattern_bindings(pattern);
        // GLOBALSHADOW-1: the body sees the pattern's bindings, not globals
        let body_tokens =
            self.with_globals_shadowed(&bound_vars, || self.transpile_expr(body))?;

        if bound_vars.is_empty() {
            bail!("Let-else pattern must bind at least one variable");
        }

        if bound_vars.len() == 1 {
            let var = &bound_vars[0];
            let var_ident = syn::Ident::new(var, proc_macro2::Span::call_site());

            Ok(quote! {
                {
                    let #var_ident = if let #pattern_tokens = #value_tokens {
                        #var_ident
                    } else {
                        #else_tokens
                    };
                    #body_tokens
                }
            })
        } else {
            let var_idents: Vec<_> = bound_vars
                .iter()
                .map(|v| syn::Ident::new(v, proc_macro2::Span::call_site()))
                .collect();

            Ok(quote! {
                {
                    let (#(#var_idents),*) = if let #pattern_tokens = #value_tokens {
                        (#(#var_idents),*)
                    } else {
                        #else_tokens
                    };
                    #body_tokens
                }
            })
        }
    }

    /// Check if a pattern requires a slice (for list pattern matching)
    pub(super) fn pattern_needs_slice(&self, pattern: &Pattern) -> bool {
        matches!(pattern, Pattern::List(_))
    }

    /// Check if an expression creates a Vec that needs conversion to slice
    pub(super) fn value_creates_vec(&self, expr: &Expr) -> bool {
        matches!(expr.kind, ExprKind::List(_))
    }
}

/// LETVEC-1: a copy of the statement-level `let` `expr` with its non-empty
/// array literal replaced by `vec![..]`, when a later sibling in `rest` is a
/// Vec-only method call on the binding; `None` otherwise. NESTARRVEC-1: when
/// a sibling grows an element (`m[i].push(x)`), the inner literals become
/// `vec![..]` too.
fn grown_let_as_vec(expr: &Expr, rest: &[Expr]) -> Option<Expr> {
    let ExprKind::Let {
        name, value, body, ..
    } = &expr.kind
    else {
        return None;
    };
    let is_list = matches!(&value.kind, ExprKind::List(items) if !items.is_empty());
    let is_statement = matches!(body.kind, ExprKind::Literal(Literal::Unit));
    if !is_list || !is_statement {
        return None;
    }
    let grown = super::mutation_detection::is_grown_in_statements(name, rest);
    let elem_grown = super::mutation_detection::is_elem_grown_in_statements(name, rest);
    if !grown && !elem_grown {
        return None;
    }
    let inner = if elem_grown {
        inner_lists_as_vec(value)
    } else {
        (**value).clone()
    };
    let mut rewritten = expr.clone();
    if let ExprKind::Let { value, .. } = &mut rewritten.kind {
        **value = if grown { list_as_vec(&inner) } else { inner };
    }
    Some(rewritten)
}

/// NESTARRVEC-1: `value` with its inner array literals as `vec![..]`, when
/// `value` is an array literal and `body` grows an element of `name`.
fn elem_grown_value(name: &str, value: &Expr, body: &Expr) -> Option<Expr> {
    let is_list = matches!(&value.kind, ExprKind::List(items) if !items.is_empty());
    (is_list && super::mutation_detection::is_elem_grown_as_vec(name, body))
        .then(|| inner_lists_as_vec(value))
}

/// NESTARRVEC-1: a copy of the array literal `value` with each element that
/// is itself an array literal replaced by `vec![..]`.
fn inner_lists_as_vec(value: &Expr) -> Expr {
    let mut rewritten = value.clone();
    if let ExprKind::List(items) = &mut rewritten.kind {
        for item in items.iter_mut() {
            *item = list_as_vec(item);
        }
    }
    rewritten
}

/// NESTPUSHLIT-1: `value` is a list literal (array or `vec![..]`) with an
/// element that is a `vec![..]` literal. (complexity: 4)
fn has_vec_inner_literals(value: &Expr) -> bool {
    let items = match &value.kind {
        ExprKind::List(items) => items,
        ExprKind::MacroInvocation { name, args } if name == "vec" => args,
        _ => return false,
    };
    items
        .iter()
        .any(|item| matches!(&item.kind, ExprKind::MacroInvocation { name, .. } if name == "vec"))
}

/// LETVEC-1: the array literal `list` as `vec![..]`; any other expression unchanged.
fn list_as_vec(list: &Expr) -> Expr {
    let mut rewritten = list.clone();
    if let ExprKind::List(items) = &list.kind {
        rewritten.kind = ExprKind::MacroInvocation {
            name: "vec".to_string(),
            args: items.clone(),
        };
    }
    rewritten
}

/// LETVEC-1: `Vec<..>` annotation (a bare `Vec` or a generic `Vec<T>`).
fn is_vec_annotation(ty: &Type) -> bool {
    match &ty.kind {
        TypeKind::Generic { base, .. } | TypeKind::Named(base) => base == "Vec",
        _ => false,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;
    use quote::format_ident;

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            contracts: Vec::new(),
        }
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn unit_expr() -> Expr {
        make_expr(ExprKind::Literal(Literal::Unit))
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    #[test]
    fn test_generate_let_binding_immutable() {
        let name_ident = format_ident!("x");
        let value = quote! { 42 };
        let result = Transpiler::generate_let_binding(&name_ident, false, false, &value);
        assert!(result.to_string().contains("let x"));
    }

    #[test]
    fn test_generate_let_binding_mutable() {
        let name_ident = format_ident!("x");
        let value = quote! { 42 };
        let result = Transpiler::generate_let_binding(&name_ident, true, false, &value);
        assert!(result.to_string().contains("let mut x"));
    }

    #[test]
    fn test_generate_let_binding_vec_hint() {
        let name_ident = format_ident!("items");
        let value = quote! { vec![] };
        let result = Transpiler::generate_let_binding(&name_ident, false, true, &value);
        let result_str = result.to_string();
        // quote! may tokenize as "Vec < _ >" with spaces
        assert!(result_str.contains("Vec") && result_str.contains("_"));
    }

    #[test]
    fn test_require_exact_args_success() {
        let args = vec![quote! { 1 }, quote! { 2 }];
        let result = Transpiler::require_exact_args("test", &args, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_exact_args_failure() {
        let args = vec![quote! { 1 }];
        let result = Transpiler::require_exact_args("test", &args, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_require_no_args_success() {
        let args: Vec<TokenStream> = vec![];
        let result = Transpiler::require_no_args("test", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_no_args_failure() {
        let args = vec![quote! { 1 }];
        let result = Transpiler::require_no_args("test", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_transpile_let_simple() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let("x", &value, &body, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("let"));
        assert!(tokens.contains("x"));
    }

    #[test]
    fn test_transpile_let_mutable() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let("x", &value, &body, true);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("let mut"));
    }

    #[test]
    fn test_transpile_let_string_literal() {
        let transpiler = Transpiler::new();
        let value = string_expr("hello");
        let body = unit_expr();
        let result = transpiler.transpile_let("s", &value, &body, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("to_string"));
    }

    #[test]
    fn test_transpile_let_with_body() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = ident_expr("x");
        let result = transpiler.transpile_let("x", &value, &body, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_simple() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let_pattern(&pattern, &value, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_tuple() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let_pattern(&pattern, &value, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_else_simple() {
        let transpiler = Transpiler::new();
        let value = ident_expr("opt");
        let body = ident_expr("x");
        let else_block = int_expr(-1);
        let result = transpiler.transpile_let_else("x", &value, &body, &else_block);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("is_none"));
    }

    #[test]
    fn test_transpile_let_pattern_else_single() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let value = ident_expr("opt");
        let body = ident_expr("x");
        let else_block = int_expr(-1);
        let result = transpiler.transpile_let_pattern_else(&pattern, &value, &body, &else_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_else_empty_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Wildcard;
        let value = ident_expr("opt");
        let body = ident_expr("x");
        let else_block = int_expr(-1);
        let result = transpiler.transpile_let_pattern_else(&pattern, &value, &body, &else_block);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("bind at least one"));
    }

    #[test]
    fn test_pattern_needs_slice_list() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
        assert!(transpiler.pattern_needs_slice(&pattern));
    }

    #[test]
    fn test_pattern_needs_slice_non_list() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        assert!(!transpiler.pattern_needs_slice(&pattern));
    }

    #[test]
    fn test_value_creates_vec_list() {
        let transpiler = Transpiler::new();
        let expr = make_expr(ExprKind::List(vec![]));
        assert!(transpiler.value_creates_vec(&expr));
    }

    #[test]
    fn test_value_creates_vec_non_list() {
        let transpiler = Transpiler::new();
        let expr = int_expr(42);
        assert!(!transpiler.value_creates_vec(&expr));
    }

    // ===== EXTREME TDD Round 156 - Bindings Transpilation Tests =====

    #[test]
    fn test_generate_let_binding_mutable_with_vec_hint() {
        let name_ident = format_ident!("data");
        let value = quote! { vec![] };
        let result = Transpiler::generate_let_binding(&name_ident, true, true, &value);
        let result_str = result.to_string();
        assert!(result_str.contains("let mut"));
        assert!(result_str.contains("Vec"));
    }

    #[test]
    fn test_require_exact_args_singular_message() {
        let args: Vec<TokenStream> = vec![];
        let result = Transpiler::require_exact_args("method", &args, 1);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("1 argument"));
        assert!(!err.contains("arguments"));
    }

    #[test]
    fn test_require_exact_args_plural_message() {
        let args: Vec<TokenStream> = vec![];
        let result = Transpiler::require_exact_args("method", &args, 3);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("3 arguments"));
    }

    #[test]
    fn test_transpile_let_with_empty_list() {
        let transpiler = Transpiler::new();
        let value = make_expr(ExprKind::List(vec![]));
        let body = unit_expr();
        let result = transpiler.transpile_let("items", &value, &body, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Vec"));
    }

    #[test]
    fn test_transpile_let_with_block_body() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = make_expr(ExprKind::Block(vec![ident_expr("x"), int_expr(10)]));
        let result = transpiler.transpile_let("x", &value, &body, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_nested_let() {
        let transpiler = Transpiler::new();
        let value = int_expr(1);
        let inner_let = make_expr(ExprKind::Let {
            name: "y".to_string(),
            type_annotation: None,
            value: Box::new(int_expr(2)),
            body: Box::new(ident_expr("y")),
            is_mutable: false,
            else_block: None,
        });
        let result = transpiler.transpile_let("x", &value, &inner_let, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_with_body() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let value = int_expr(42);
        let body = ident_expr("x");
        let result = transpiler.transpile_let_pattern(&pattern, &value, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_list_with_unit_body() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::List(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let value = make_expr(ExprKind::List(vec![int_expr(1), int_expr(2)]));
        let body = unit_expr();
        let result = transpiler.transpile_let_pattern(&pattern, &value, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_with_type_string() {
        let transpiler = Transpiler::new();
        let value = string_expr("hello");
        let body = unit_expr();
        let type_ann = crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("String".to_string()),
            span: Span::default(),
        };
        let result =
            transpiler.transpile_let_with_type("s", Some(&type_ann), &value, &body, false, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("to_string"));
    }

    #[test]
    fn test_transpile_let_with_type_const() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let_with_type("X", None, &value, &body, false, true);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("const"));
    }

    #[test]
    fn test_transpile_let_with_type_mutable() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let_with_type("x", None, &value, &body, true, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("let mut"));
    }

    #[test]
    fn test_transpile_let_with_type_empty_list() {
        let transpiler = Transpiler::new();
        let value = make_expr(ExprKind::List(vec![]));
        let body = unit_expr();
        let result = transpiler.transpile_let_with_type("items", None, &value, &body, false, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Vec"));
    }

    #[test]
    fn test_transpile_let_with_type_and_body() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = ident_expr("x");
        let result = transpiler.transpile_let_with_type("x", None, &value, &body, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_with_type_and_unit_body() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let_pattern_with_type(&pattern, None, &value, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_with_type_and_expr_body() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let value = int_expr(42);
        let body = ident_expr("x");
        let type_ann = crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        let result =
            transpiler.transpile_let_pattern_with_type(&pattern, Some(&type_ann), &value, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_pattern_else_multiple_vars() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let value = ident_expr("opt");
        let body = ident_expr("a");
        let else_block = int_expr(-1);
        let result = transpiler.transpile_let_pattern_else(&pattern, &value, &body, &else_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_let_with_reserved_keyword() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let("type", &value, &body, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("r#type"));
    }

    #[test]
    fn test_transpile_let_with_type_reserved_keyword() {
        let transpiler = Transpiler::new();
        let value = int_expr(42);
        let body = unit_expr();
        let result = transpiler.transpile_let_with_type("fn", None, &value, &body, false, false);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("r#fn"));
    }

    #[test]
    fn test_transpile_let_pattern_list_slice_conversion() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::List(vec![Pattern::Identifier("first".to_string())]);
        let value = make_expr(ExprKind::List(vec![int_expr(1), int_expr(2)]));
        let body = ident_expr("first");
        let result = transpiler.transpile_let_pattern(&pattern, &value, &body);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Should contain slice conversion
        assert!(tokens.contains('['));
    }

    #[test]
    fn test_transpile_let_with_call_value() {
        let transpiler = Transpiler::new();
        let value = make_expr(ExprKind::Call {
            func: Box::new(ident_expr("get_string")),
            args: vec![],
        });
        let body = unit_expr();
        let result = transpiler.transpile_let_with_type("s", None, &value, &body, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_needs_slice_tuple() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Tuple(vec![Pattern::Identifier("a".to_string())]);
        assert!(!transpiler.pattern_needs_slice(&pattern));
    }

    #[test]
    fn test_pattern_needs_slice_wildcard() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Wildcard;
        assert!(!transpiler.pattern_needs_slice(&pattern));
    }

    #[test]
    fn test_value_creates_vec_call() {
        let transpiler = Transpiler::new();
        let expr = make_expr(ExprKind::Call {
            func: Box::new(ident_expr("vec")),
            args: vec![],
        });
        assert!(!transpiler.value_creates_vec(&expr));
    }

    #[test]
    fn test_value_creates_vec_identifier() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("items");
        assert!(!transpiler.value_creates_vec(&expr));
    }

    /// RHLGA-1 review F7: a keyword that cannot be a raw identifier
    /// (`self`, `Self`, `super`, `crate`) must not make `let` panic.
    #[test]
    fn test_rhlga_1_let_named_non_raw_keyword_does_not_panic() {
        let transpiler = Transpiler::new();
        for name in ["self", "Self", "super", "crate"] {
            let value = int_expr(1);
            let body = unit_expr();
            let plain = transpiler.transpile_let(name, &value, &body, false);
            assert!(plain.is_ok(), "{name}: {plain:?}");
            let typed = transpiler.transpile_let_with_type(name, None, &value, &body, false, false);
            assert!(typed.is_ok(), "{name}: {typed:?}");
        }
    }
}
