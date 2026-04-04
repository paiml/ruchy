#![allow(clippy::approx_constant)]
// Code formatter for Ruchy
// Toyota Way: Consistent code style prevents defects
use crate::frontend::ast::{Expr, ExprKind};
use crate::quality::formatter_config::FormatterConfig;
use anyhow::Result;

pub struct Formatter {
    config: FormatterConfig,
    source: Option<String>,
}

impl Formatter {
    /// Create a new formatter with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::formatter::Formatter;
    ///
    /// let instance = Formatter::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self::with_config(FormatterConfig::default())
    }

    /// Create a new formatter with custom configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::formatter::Formatter;
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::default();
    /// let instance = Formatter::with_config(config);
    /// ```
    pub fn with_config(config: FormatterConfig) -> Self {
        Self {
            config,
            source: None,
        }
    }

    /// Set the original source text for preserve-original-text ignore directives
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::formatter::Formatter;
    ///
    /// let mut formatter = Formatter::new();
    /// formatter.set_source("let x = 1 + 2");
    /// ```
    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = Some(source.into());
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::formatter::Formatter;
    ///
    /// let mut instance = Formatter::new();
    /// let result = instance.format();
    /// // Verify behavior
    /// ```
    pub fn format(&self, ast: &Expr) -> Result<String> {
        // Check if the top-level expression should be ignored
        if self.should_ignore(ast) {
            if let Some(original) = self.get_original_text(ast) {
                return Ok(original);
            }
        }

        // Handle top-level blocks specially (don't add braces)
        if let ExprKind::Block(exprs) = &ast.kind {
            let mut result = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    result.push('\n');
                }
                result.push_str(&self.format_expr(expr, 0));
            }
            Ok(result)
        } else {
            // Single expression at top level
            Ok(self.format_expr(ast, 0))
        }
    }
    fn format_type(ty_kind: &crate::frontend::ast::TypeKind) -> String {
        use crate::frontend::ast::TypeKind;

        match ty_kind {
            TypeKind::Named(name) => name.clone(),
            TypeKind::Generic { base, params } => {
                let params_str = params
                    .iter()
                    .map(|t| Self::format_type(&t.kind))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{base}<{params_str}>")
            }
            TypeKind::Function { params, ret } => {
                let params_str = params
                    .iter()
                    .map(|t| Self::format_type(&t.kind))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", params_str, Self::format_type(&ret.kind))
            }
            TypeKind::Tuple(types) => {
                format!(
                    "({})",
                    types
                        .iter()
                        .map(|t| Self::format_type(&t.kind))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TypeKind::Array { elem_type, size } => {
                format!("[{}; {}]", Self::format_type(&elem_type.kind), size)
            }
            _ => format!("{ty_kind:?}"),
        }
    }
    /// Check if an expression should be ignored based on leading comments
    fn should_ignore(&self, expr: &Expr) -> bool {
        expr.leading_comments.iter().any(|comment| {
            use crate::frontend::ast::CommentKind;
            match &comment.kind {
                CommentKind::Line(text) => {
                    let trimmed = text.trim();
                    trimmed == "ruchy-fmt-ignore" || trimmed == "ruchy-fmt-ignore-next"
                }
                _ => false,
            }
        })
    }

    /// Get original text from span (for ignore directives)
    fn get_original_text(&self, expr: &Expr) -> Option<String> {
        self.source.as_ref().map(|src| {
            // Calculate span including leading comments
            let start = if expr.leading_comments.is_empty() {
                expr.span.start
            } else {
                expr.leading_comments[0].span.start
            };

            // Find the true end by recursing through the AST to find the rightmost span
            let mut end = Self::find_rightmost_span_end(expr);

            // WORKAROUND for incomplete parser spans: scan forward to find actual end
            // For expressions like functions and blocks, the span often doesn't include closing braces
            // We need to find the TRUE end of the expression by scanning forward
            let bytes = src.as_bytes();

            // Track brace depth to find matching closing brace
            let mut brace_depth = 0;
            let mut in_expression = false;

            // Check if this is a block expression (starts with {)
            // Skip past leading comments first
            let mut scan_pos = start;
            // Skip comment lines
            while scan_pos < bytes.len() {
                // Skip whitespace
                while scan_pos < bytes.len()
                    && (bytes[scan_pos] == b' ' || bytes[scan_pos] == b'\t')
                {
                    scan_pos += 1;
                }
                // Check for comment
                if scan_pos + 1 < bytes.len()
                    && bytes[scan_pos] == b'/'
                    && bytes[scan_pos + 1] == b'/'
                {
                    // Skip to end of line
                    while scan_pos < bytes.len() && bytes[scan_pos] != b'\n' {
                        scan_pos += 1;
                    }
                    if scan_pos < bytes.len() {
                        scan_pos += 1; // skip newline
                    }
                } else {
                    break;
                }
            }
            // Now skip final whitespace before the actual expression
            while scan_pos < bytes.len()
                && (bytes[scan_pos] == b' ' || bytes[scan_pos] == b'\t' || bytes[scan_pos] == b'\n')
            {
                scan_pos += 1;
            }
            // Check if we found a block
            if scan_pos < bytes.len() && bytes[scan_pos] == b'{' {
                brace_depth = 1;
                in_expression = true;
                scan_pos += 1;
            }

            if in_expression {
                // Scan forward to find matching closing brace
                while scan_pos < bytes.len() && brace_depth > 0 {
                    if bytes[scan_pos] == b'{' {
                        brace_depth += 1;
                    } else if bytes[scan_pos] == b'}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // Found the matching closing brace
                            end = scan_pos + 1;
                            break;
                        }
                    }
                    scan_pos += 1;
                }
            } else {
                // Not a block - scan to end of line
                while end < bytes.len() {
                    if bytes[end] == b'\n' {
                        break;
                    }
                    end += 1;
                }
            }

            let start = start.min(src.len());
            let end = end.min(src.len());
            src[start..end].to_string()
        })
    }

    /// Recursively find the rightmost (maximum) span end in an expression tree
    fn find_rightmost_span_end(expr: &Expr) -> usize {
        use ExprKind::{Binary, Block, Function, Let};
        let mut max_end = expr.span.end;

        match &expr.kind {
            Let { value, body, .. } => {
                max_end = max_end.max(Self::find_rightmost_span_end(value));
                max_end = max_end.max(Self::find_rightmost_span_end(body));
            }
            Binary { left, right, .. } => {
                max_end = max_end.max(Self::find_rightmost_span_end(left));
                max_end = max_end.max(Self::find_rightmost_span_end(right));
            }
            Function { body, .. } => {
                // Function body is the rightmost part
                max_end = max_end.max(Self::find_rightmost_span_end(body));
            }
            Block(exprs) => {
                // Last expression in block is the rightmost
                if let Some(last) = exprs.last() {
                    max_end = max_end.max(Self::find_rightmost_span_end(last));
                }
            }
            _ => {
                // For other expression types, use the expr.span.end
                // This is a simplified version - full implementation would need to recurse into all expression types
            }
        }

        max_end
    }

    fn format_expr(&self, expr: &Expr, indent: usize) -> String {
        // Check for ignore directives FIRST
        if self.should_ignore(expr) {
            // If we have original source, preserve it exactly
            if let Some(original) = self.get_original_text(expr) {
                return original;
            }
            // Otherwise, fall through to normal formatting
        }

        let indent_str = if self.config.use_tabs {
            "\t".repeat(indent)
        } else {
            " ".repeat(indent * self.config.indent_width)
        };

        // Format leading comments
        let mut result = String::new();
        for comment in &expr.leading_comments {
            result.push_str(&self.format_comment(comment, indent));
            result.push('\n');
        }

        // Format the expression itself
        let expr_str = self.format_expr_kind(&expr.kind, &indent_str, indent);

        // Append the formatted expression
        result.push_str(&expr_str);

        // Append trailing comment if present
        if let Some(trailing) = &expr.trailing_comment {
            result.push(' ');
            result.push_str(&self.format_comment(trailing, 0)); // No indent for trailing
        }

        result
    }

    // --- Extracted helpers for format_expr (CB-200 reduction) ---

    fn format_expr_kind(&self, kind: &ExprKind, indent_str: &str, indent: usize) -> String {
        match kind {
            ExprKind::Literal(lit) => self.format_literal_expr(lit),
            ExprKind::Identifier(name) => name.clone(),
            ExprKind::Let {
                name, value, body, ..
            } => self.format_let_expr(name, value, body, indent),
            ExprKind::Block(exprs) => self.format_block_expr(exprs, indent_str, indent),
            ExprKind::Function {
                name,
                params,
                return_type,
                body,
                ..
            } => self.format_function_expr(name, params, return_type.as_ref(), body, indent),
            ExprKind::Lambda { params, body } => self.format_lambda_expr(params, body, indent),
            ExprKind::AsyncLambda { params, body } => {
                format!(
                    "async |{}| {}",
                    params.join(", "),
                    self.format_expr(body, indent)
                )
            }
            // Control flow
            ExprKind::If { .. }
            | ExprKind::IfLet { .. }
            | ExprKind::Match { .. }
            | ExprKind::For { .. }
            | ExprKind::While { .. }
            | ExprKind::WhileLet { .. }
            | ExprKind::Loop { .. }
            | ExprKind::Break { .. }
            | ExprKind::Continue { .. }
            | ExprKind::Return { .. }
            | ExprKind::TryCatch { .. }
            | ExprKind::Throw { .. } => self.format_control_flow(kind, indent),
            // Operators and access
            ExprKind::Binary { .. }
            | ExprKind::Unary { .. }
            | ExprKind::CompoundAssign { .. }
            | ExprKind::Ternary { .. }
            | ExprKind::Range { .. }
            | ExprKind::Assign { .. }
            | ExprKind::IndexAccess { .. }
            | ExprKind::FieldAccess { .. }
            | ExprKind::OptionalFieldAccess { .. }
            | ExprKind::Slice { .. }
            | ExprKind::TypeCast { .. }
            | ExprKind::Try { .. }
            | ExprKind::Spread { .. }
            | ExprKind::PreIncrement { .. }
            | ExprKind::PostIncrement { .. }
            | ExprKind::PreDecrement { .. }
            | ExprKind::PostDecrement { .. }
            | ExprKind::Pipeline { .. } => self.format_operators(kind, indent),
            // Call expressions
            ExprKind::Call { func, args } => self.format_call_expr(func, args, indent),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
                ..
            } => self.format_method_call_expr(receiver, method, args, indent),
            ExprKind::OptionalMethodCall {
                receiver,
                method,
                args,
            } => self.format_optional_method_call(receiver, method, args, indent),
            // Collections and literals
            ExprKind::List(items) => self.format_collection("[", items, "]", indent),
            ExprKind::Tuple(items) => self.format_collection("(", items, ")", indent),
            ExprKind::ObjectLiteral { fields } => self.format_object_literal(fields, indent),
            ExprKind::StructLiteral { name, fields, base } => {
                self.format_struct_literal(name, fields, base.as_ref(), indent)
            }
            ExprKind::ArrayInit { value, size } => {
                format!(
                    "[{}; {}]",
                    self.format_expr(value, indent),
                    self.format_expr(size, indent)
                )
            }
            ExprKind::VecRepeat { value, count } => {
                format!(
                    "vec![{}; {}]",
                    self.format_expr(value, indent),
                    self.format_expr(count, indent)
                )
            }
            ExprKind::Set(elems) => format!(
                "{{{}}}",
                elems
                    .iter()
                    .map(|e| self.format_expr(e, indent))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            // Result/Option wrappers
            ExprKind::Ok { value } => format!("Ok({})", self.format_expr(value, indent)),
            ExprKind::Err { error } => format!("Err({})", self.format_expr(error, indent)),
            ExprKind::Some { value } => format!("Some({})", self.format_expr(value, indent)),
            ExprKind::None => "None".to_string(),
            // Declarations
            ExprKind::Struct { .. }
            | ExprKind::TupleStruct { .. }
            | ExprKind::Enum { .. }
            | ExprKind::Trait { .. }
            | ExprKind::Impl { .. }
            | ExprKind::Class { .. }
            | ExprKind::TypeAlias { .. }
            | ExprKind::Extension { .. } => self.format_declarations(kind, indent),
            // Module system
            ExprKind::Module { .. }
            | ExprKind::ModuleDeclaration { .. }
            | ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. }
            | ExprKind::Export { .. }
            | ExprKind::ExportList { .. }
            | ExprKind::ExportDefault { .. }
            | ExprKind::ReExport { .. }
            | ExprKind::QualifiedName { .. } => self.format_module_system(kind, indent),
            // String interpolation
            ExprKind::StringInterpolation { parts } => {
                self.format_string_interpolation(parts, indent)
            }
            ExprKind::LetPattern {
                pattern,
                value,
                body,
                ..
            } => {
                format!(
                    "let {} = {} in {}",
                    self.format_pattern(pattern),
                    self.format_expr(value, indent),
                    self.format_expr(body, indent)
                )
            }
            // Comprehensions
            ExprKind::ListComprehension { element, clauses } => {
                self.format_list_comprehension(element, clauses, indent)
            }
            ExprKind::DictComprehension {
                key,
                value,
                clauses,
            } => self.format_dict_comprehension(key, value, clauses, indent),
            ExprKind::SetComprehension { element, clauses } => {
                self.format_set_comprehension(element, clauses, indent)
            }
            // Actor and effects
            ExprKind::Actor { .. }
            | ExprKind::Effect { .. }
            | ExprKind::Handle { .. }
            | ExprKind::Send { .. }
            | ExprKind::Spawn { .. }
            | ExprKind::ActorSend { .. }
            | ExprKind::ActorQuery { .. }
            | ExprKind::Ask { .. } => self.format_actor_effects(kind, indent),
            // Async
            ExprKind::Await { expr } => format!("await {}", self.format_expr(expr, indent)),
            ExprKind::AsyncBlock { body } => format!("async {}", self.format_expr(body, indent)),
            // Macros and commands
            ExprKind::Macro { name, args } => self.format_macro_def(name, args, indent),
            ExprKind::MacroInvocation { name, args } => {
                self.format_macro_invocation(name, args, indent)
            }
            ExprKind::Command { program, args, .. } => self.format_command(program, args),
            // Data
            ExprKind::DataFrame { columns } => self.format_dataframe(columns, indent),
            ExprKind::DataFrameOperation { source, operation } => {
                format!("{}.{:?}", self.format_expr(source, indent), operation)
            }
            ExprKind::Lazy { expr } => format!("lazy {}", self.format_expr(expr, indent)),
            // Ruchy 5.0 Sovereign Platform expressions
            ExprKind::Yield { value } => match value {
                Some(v) => format!("yield {}", self.format_expr(v, indent)),
                None => "yield".to_string(),
            },
            ExprKind::Signal { initial_value } => {
                format!("signal({})", self.format_expr(initial_value, indent))
            }
            ExprKind::InfraBlock { body } => {
                let pad = "    ".repeat(indent + 1);
                let close_pad = "    ".repeat(indent);
                let body_str = body
                    .iter()
                    .map(|e| format!("{pad}{}", self.format_expr(e, indent + 1)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("infra {{\n{body_str}\n{close_pad}}}")
            }
        }
    }

    fn format_control_flow(&self, kind: &ExprKind, indent: usize) -> String {
        match kind {
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.format_if_expr(condition, then_branch, else_branch.as_ref(), indent),
            ExprKind::IfLet {
                pattern,
                expr,
                then_branch,
                else_branch,
            } => self.format_if_let_expr(pattern, expr, then_branch, else_branch.as_ref(), indent),
            ExprKind::Match { expr, arms } => self.format_match_expr(expr, arms, indent),
            ExprKind::For {
                var,
                pattern,
                iter,
                body,
                ..
            } => self.format_for_expr(var, pattern.as_ref(), iter, body, indent),
            ExprKind::While {
                condition, body, ..
            } => {
                format!(
                    "while {} {}",
                    self.format_expr(condition, indent),
                    self.format_expr(body, indent)
                )
            }
            ExprKind::WhileLet {
                pattern,
                expr,
                body,
                ..
            } => {
                format!(
                    "while let {} = {} {}",
                    self.format_pattern(pattern),
                    self.format_expr(expr, indent),
                    self.format_expr(body, indent)
                )
            }
            ExprKind::Loop { body, .. } => {
                format!(
                    "loop {{\n{}\n{}}}",
                    self.format_expr(body, indent + 1),
                    " ".repeat(indent * self.config.indent_width)
                )
            }
            ExprKind::Break { value, .. } => {
                self.format_return_like("break", value.as_ref(), indent)
            }
            ExprKind::Continue { .. } => "continue".to_string(),
            ExprKind::Return { value } => self.format_return(value.as_ref(), indent),
            ExprKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => self.format_try_catch(try_block, catch_clauses, finally_block.as_ref(), indent),
            ExprKind::Throw { expr } => format!("throw {}", self.format_expr(expr, indent)),
            _ => unreachable!(),
        }
    }

    fn format_operators(&self, kind: &ExprKind, indent: usize) -> String {
        match kind {
            ExprKind::Binary { left, op, right } => {
                format!(
                    "{} {} {}",
                    self.format_expr(left, indent),
                    op,
                    self.format_expr(right, indent)
                )
            }
            ExprKind::Unary { op, operand } => {
                format!("{}{}", op, self.format_expr(operand, indent))
            }
            ExprKind::CompoundAssign { target, op, value } => {
                format!(
                    "{} {}= {}",
                    self.format_expr(target, indent),
                    op,
                    self.format_expr(value, indent)
                )
            }
            ExprKind::Ternary {
                condition,
                true_expr,
                false_expr,
            } => {
                format!(
                    "{} ? {} : {}",
                    self.format_expr(condition, indent),
                    self.format_expr(true_expr, indent),
                    self.format_expr(false_expr, indent)
                )
            }
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => {
                let op = if *inclusive { "..=" } else { ".." };
                format!(
                    "{}{}{}",
                    self.format_expr(start, indent),
                    op,
                    self.format_expr(end, indent)
                )
            }
            ExprKind::Assign { target, value } => self.format_assign(target, value, indent),
            ExprKind::IndexAccess { object, index } => {
                self.format_index_access(object, index, indent)
            }
            ExprKind::FieldAccess { object, field } => {
                format!("{}.{}", self.format_expr(object, indent), field)
            }
            ExprKind::OptionalFieldAccess { object, field } => {
                format!("{}?.{}", self.format_expr(object, indent), field)
            }
            ExprKind::Slice { object, start, end } => {
                self.format_slice(object, start.as_ref(), end.as_ref(), indent)
            }
            ExprKind::TypeCast { expr, target_type } => {
                format!("{} as {}", self.format_expr(expr, indent), target_type)
            }
            ExprKind::Try { expr } => format!("{}?", self.format_expr(expr, indent)),
            ExprKind::Spread { expr } => format!("...{}", self.format_expr(expr, indent)),
            ExprKind::PreIncrement { target } => format!("++{}", self.format_expr(target, indent)),
            ExprKind::PostIncrement { target } => format!("{}++", self.format_expr(target, indent)),
            ExprKind::PreDecrement { target } => format!("--{}", self.format_expr(target, indent)),
            ExprKind::PostDecrement { target } => format!("{}--", self.format_expr(target, indent)),
            ExprKind::Pipeline { expr, stages } => self.format_pipeline(expr, stages, indent),
            _ => unreachable!(),
        }
    }

    fn format_declarations(&self, kind: &ExprKind, indent: usize) -> String {
        match kind {
            ExprKind::Struct {
                name,
                type_params,
                fields,
                is_pub,
                ..
            } => self.format_struct_decl(name, type_params, fields, *is_pub),
            ExprKind::TupleStruct {
                name,
                type_params,
                fields,
                is_pub,
                ..
            } => self.format_tuple_struct_decl(name, type_params, fields, *is_pub),
            ExprKind::Enum {
                name,
                type_params,
                variants,
                is_pub,
            } => self.format_enum_decl(name, type_params, variants, *is_pub),
            ExprKind::Trait {
                name,
                type_params,
                methods,
                is_pub,
                ..
            } => self.format_trait_decl(name, type_params, methods, *is_pub),
            ExprKind::Impl {
                type_params,
                trait_name,
                for_type,
                methods,
                ..
            } => self.format_impl_decl(type_params, trait_name.as_ref(), for_type, methods),
            ExprKind::Class {
                name,
                type_params,
                fields,
                ..
            } => self.format_class_decl(name, type_params, fields),
            ExprKind::TypeAlias { name, target_type } => {
                format!("type {} = {}", name, Self::format_type(&target_type.kind))
            }
            ExprKind::Extension {
                target_type,
                methods,
            } => self.format_extension(target_type, methods, indent),
            _ => unreachable!(),
        }
    }

    fn format_module_system(&self, kind: &ExprKind, indent: usize) -> String {
        match kind {
            ExprKind::Module { name, body } => {
                format!("mod {} {}", name, self.format_expr(body, indent))
            }
            ExprKind::ModuleDeclaration { name } => format!("mod {name};"),
            ExprKind::Import { module, items } => self.format_import(module, items.as_ref()),
            ExprKind::ImportAll { module, .. } => format!("import {module}::*"),
            ExprKind::ImportDefault { module, name } => {
                format!("import default {name} from {module}")
            }
            ExprKind::Export { expr, is_default } => {
                let prefix = if *is_default {
                    "export default "
                } else {
                    "export "
                };
                format!("{}{}", prefix, self.format_expr(expr, indent))
            }
            ExprKind::ExportList { names } => format!("export {{ {} }}", names.join(", ")),
            ExprKind::ExportDefault { expr } => {
                format!("export default {}", self.format_expr(expr, indent))
            }
            ExprKind::ReExport { items, module } => {
                format!("export {{ {} }} from {}", items.join(", "), module)
            }
            ExprKind::QualifiedName { module, name } => format!("{module}::{name}"),
            _ => unreachable!(),
        }
    }

    fn format_actor_effects(&self, kind: &ExprKind, indent: usize) -> String {
        match kind {
            ExprKind::Actor {
                name,
                state,
                handlers,
            } => self.format_actor(name, state, handlers),
            ExprKind::Effect { name, operations } => self.format_effect(name, operations),
            ExprKind::Handle { expr, handlers } => self.format_handle(expr, handlers, indent),
            ExprKind::Send { actor, message } => {
                format!(
                    "send({}, {})",
                    self.format_expr(actor, indent),
                    self.format_expr(message, indent)
                )
            }
            ExprKind::Spawn { actor } => format!("spawn {}", self.format_expr(actor, indent)),
            ExprKind::ActorSend { actor, message } => {
                format!(
                    "{} <- {}",
                    self.format_expr(actor, indent),
                    self.format_expr(message, indent)
                )
            }
            ExprKind::ActorQuery { actor, message } => {
                format!(
                    "{} <? {}",
                    self.format_expr(actor, indent),
                    self.format_expr(message, indent)
                )
            }
            ExprKind::Ask { actor, message, .. } => {
                format!(
                    "ask {} {}",
                    self.format_expr(actor, indent),
                    self.format_expr(message, indent)
                )
            }
            _ => unreachable!(),
        }
    }

    fn format_literal_expr(&self, lit: &crate::frontend::ast::Literal) -> String {
        match lit {
            crate::frontend::ast::Literal::Integer(n, _) => n.to_string(),
            crate::frontend::ast::Literal::Float(f) => f.to_string(),
            crate::frontend::ast::Literal::String(s) => {
                format!("\"{}\"", s.replace('"', "\\\""))
            }
            crate::frontend::ast::Literal::Bool(b) => b.to_string(),
            crate::frontend::ast::Literal::Char(c) => format!("'{c}'"),
            crate::frontend::ast::Literal::Byte(b) => format!("b'{}'", *b as char),
            crate::frontend::ast::Literal::Unit => "()".to_string(),
            crate::frontend::ast::Literal::Null => "null".to_string(),
            crate::frontend::ast::Literal::Atom(s) => format!(":{s}"),
        }
    }

    fn format_let_expr(&self, name: &str, value: &Expr, body: &Expr, indent: usize) -> String {
        // FIX: QUALITY-FORMATTER-002 (GitHub Issue #64)
        let is_sequential_statement = matches!(
            body.kind,
            ExprKind::Literal(crate::frontend::ast::Literal::Unit)
                | ExprKind::Block(_)
                | ExprKind::Call { .. }
                | ExprKind::MethodCall { .. }
                | ExprKind::Let { .. }
        );

        if is_sequential_statement {
            let mut result = format!("let {} = {}", name, self.format_expr(value, indent));
            if let ExprKind::Block(body_exprs) = &body.kind {
                let indent_str = if self.config.use_tabs {
                    "\t".repeat(indent)
                } else {
                    " ".repeat(indent * self.config.indent_width)
                };
                for expr in body_exprs {
                    result.push('\n');
                    result.push_str(&indent_str);
                    result.push_str(&self.format_expr(expr, indent));
                }
            } else if !matches!(
                body.kind,
                ExprKind::Literal(crate::frontend::ast::Literal::Unit)
            ) {
                // FIX: CRITICAL-FMT-DATA-LOSS (GitHub Issue #64)
                let indent_str = if self.config.use_tabs {
                    "\t".repeat(indent)
                } else {
                    " ".repeat(indent * self.config.indent_width)
                };
                result.push('\n');
                result.push_str(&indent_str);
                result.push_str(&self.format_expr(body, indent));
            }
            result
        } else {
            format!(
                "let {} = {} in {}",
                name,
                self.format_expr(value, indent),
                self.format_expr(body, indent)
            )
        }
    }

    fn format_block_expr(&self, exprs: &[Expr], indent_str: &str, indent: usize) -> String {
        let mut result = String::from("{\n");
        let inner_indent_str = if self.config.use_tabs {
            "\t".repeat(indent + 1)
        } else {
            " ".repeat((indent + 1) * self.config.indent_width)
        };
        for expr in exprs {
            result.push_str(&format!(
                "{}{}\n",
                inner_indent_str,
                self.format_expr(expr, indent + 1)
            ));
        }
        result.push_str(&format!("{indent_str}}}"));
        result
    }

    fn format_function_expr(
        &self,
        name: &str,
        params: &[crate::frontend::ast::Param],
        return_type: Option<&crate::frontend::ast::Type>,
        body: &Expr,
        indent: usize,
    ) -> String {
        let mut result = format!("fun {name}");
        result.push('(');
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            if let crate::frontend::ast::Pattern::Identifier(param_name) = &param.pattern {
                result.push_str(param_name);
                if let crate::frontend::ast::TypeKind::Named(type_name) = &param.ty.kind {
                    if type_name != "Any" {
                        result.push_str(": ");
                        result.push_str(type_name);
                    }
                } else {
                    result.push_str(": ");
                    result.push_str(&Self::format_type(&param.ty.kind));
                }
            }
        }
        result.push(')');
        if let Some(ret_ty) = return_type {
            result.push_str(" -> ");
            result.push_str(&Self::format_type(&ret_ty.kind));
        }
        result.push(' ');
        result.push_str(&self.format_expr(body, indent));
        result
    }

    fn format_if_expr(
        &self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Box<Expr>>,
        indent: usize,
    ) -> String {
        let mut result = "if ".to_string();
        result.push_str(&self.format_expr(condition, indent));
        result.push(' ');
        result.push_str(&self.format_expr(then_branch, indent));
        if let Some(else_expr) = else_branch {
            result.push_str(" else ");
            result.push_str(&self.format_expr(else_expr, indent));
        }
        result
    }

    fn format_call_expr(&self, func: &Expr, args: &[Expr], indent: usize) -> String {
        let mut result = self.format_expr(func, indent);
        result.push('(');
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&self.format_expr(arg, indent));
        }
        result.push(')');
        result
    }

    fn format_method_call_expr(
        &self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
        indent: usize,
    ) -> String {
        let mut result = self.format_expr(receiver, indent);
        result.push('.');
        result.push_str(method);
        result.push('(');
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&self.format_expr(arg, indent));
        }
        result.push(')');
        result
    }

    fn format_for_expr(
        &self,
        var: &str,
        pattern: Option<&crate::frontend::ast::Pattern>,
        iter: &Expr,
        body: &Expr,
        indent: usize,
    ) -> String {
        let mut result = "for ".to_string();
        if let Some(pat) = pattern {
            if let crate::frontend::ast::Pattern::Identifier(name) = pat {
                result.push_str(name);
            } else {
                result.push_str(&format!("{pat:?}"));
            }
        } else {
            result.push_str(var);
        }
        result.push_str(" in ");
        result.push_str(&self.format_expr(iter, indent));
        result.push(' ');
        result.push_str(&self.format_expr(body, indent));
        result
    }

    fn format_index_access(&self, object: &Expr, index: &Expr, indent: usize) -> String {
        format!(
            "{}[{}]",
            self.format_expr(object, indent),
            self.format_expr(index, indent)
        )
    }

    fn format_assign(&self, target: &Expr, value: &Expr, indent: usize) -> String {
        format!(
            "{} = {}",
            self.format_expr(target, indent),
            self.format_expr(value, indent)
        )
    }

    fn format_return(&self, value: Option<&Box<Expr>>, indent: usize) -> String {
        if let Some(val) = value {
            format!("return {}", self.format_expr(val, indent))
        } else {
            "return".to_string()
        }
    }

    fn format_return_like(
        &self,
        keyword: &str,
        value: Option<&Box<Expr>>,
        indent: usize,
    ) -> String {
        if let Some(val) = value {
            format!("{} {}", keyword, self.format_expr(val, indent))
        } else {
            keyword.to_string()
        }
    }

    fn format_collection(&self, open: &str, items: &[Expr], close: &str, indent: usize) -> String {
        let formatted: Vec<String> = items
            .iter()
            .map(|item| self.format_expr(item, indent))
            .collect();
        format!("{}{}{}", open, formatted.join(", "), close)
    }

    fn format_match_expr(
        &self,
        scrutinee: &Expr,
        arms: &[crate::frontend::ast::MatchArm],
        indent: usize,
    ) -> String {
        let mut result = format!("match {} {{\n", self.format_expr(scrutinee, indent));
        for arm in arms {
            let pattern_str = format!("{:?}", arm.pattern);
            result.push_str(&format!(
                "{}  {} => {},\n",
                " ".repeat(indent * self.config.indent_width),
                pattern_str,
                self.format_expr(&arm.body, indent + 1)
            ));
        }
        result.push_str(&format!(
            "{}}}",
            " ".repeat(indent * self.config.indent_width)
        ));
        result
    }

    fn format_lambda_expr(
        &self,
        params: &[crate::frontend::ast::Param],
        body: &Expr,
        indent: usize,
    ) -> String {
        let params_str = params
            .iter()
            .map(|p| self.format_pattern(&p.pattern))
            .collect::<Vec<_>>()
            .join(", ");
        format!("|{}| {}", params_str, self.format_expr(body, indent))
    }

    fn format_object_literal(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
        indent: usize,
    ) -> String {
        if fields.is_empty() {
            "{}".to_string()
        } else {
            let fields_str = fields
                .iter()
                .map(|f| match f {
                    crate::frontend::ast::ObjectField::KeyValue { key, value } => {
                        format!("{}: {}", key, self.format_expr(value, indent))
                    }
                    crate::frontend::ast::ObjectField::Spread { expr } => {
                        format!("...{}", self.format_expr(expr, indent))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {fields_str} }}")
        }
    }

    fn format_struct_literal(
        &self,
        name: &str,
        fields: &[(String, Expr)],
        base: Option<&Box<Expr>>,
        indent: usize,
    ) -> String {
        let fields_str = fields
            .iter()
            .map(|(key, val)| format!("{}: {}", key, self.format_expr(val, indent)))
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(base_expr) = base {
            format!(
                "{} {{ {}, ..{} }}",
                name,
                fields_str,
                self.format_expr(base_expr, indent)
            )
        } else {
            format!("{name} {{ {fields_str} }}")
        }
    }

    fn format_try_catch(
        &self,
        try_block: &Expr,
        catch_clauses: &[crate::frontend::ast::CatchClause],
        finally_block: Option<&Box<Expr>>,
        indent: usize,
    ) -> String {
        let mut result = format!("try {}", self.format_expr(try_block, indent));
        for catch_clause in catch_clauses {
            result.push_str(&format!(
                " catch ({}) {}",
                self.format_pattern(&catch_clause.pattern),
                self.format_expr(&catch_clause.body, indent)
            ));
        }
        if let Some(finally) = finally_block {
            result.push_str(&format!(" finally {}", self.format_expr(finally, indent)));
        }
        result
    }

    fn format_if_let_expr(
        &self,
        pattern: &crate::frontend::ast::Pattern,
        expr: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Box<Expr>>,
        indent: usize,
    ) -> String {
        let mut result = format!(
            "if let {} = {} {}",
            self.format_pattern(pattern),
            self.format_expr(expr, indent),
            self.format_expr(then_branch, indent)
        );
        if let Some(else_expr) = else_branch {
            result.push_str(&format!(" else {}", self.format_expr(else_expr, indent)));
        }
        result
    }

    fn format_slice(
        &self,
        object: &Expr,
        start: Option<&Box<Expr>>,
        end: Option<&Box<Expr>>,
        indent: usize,
    ) -> String {
        let start_str = start.map_or(String::new(), |e| self.format_expr(e, indent));
        let end_str = end.map_or(String::new(), |e| self.format_expr(e, indent));
        format!(
            "{}[{}..{}]",
            self.format_expr(object, indent),
            start_str,
            end_str
        )
    }

    fn format_type_params_str(type_params: &[String]) -> String {
        if type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", type_params.join(", "))
        }
    }

    fn format_struct_decl(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[crate::frontend::ast::StructField],
        is_pub: bool,
    ) -> String {
        let pub_str = if is_pub { "pub " } else { "" };
        let type_params_str = Self::format_type_params_str(type_params);
        let fields_str = fields
            .iter()
            .map(|f| format!("{}: {}", f.name, Self::format_type(&f.ty.kind)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{pub_str}struct {name}{type_params_str} {{ {fields_str} }}")
    }

    fn format_tuple_struct_decl(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[crate::frontend::ast::Type],
        is_pub: bool,
    ) -> String {
        let pub_str = if is_pub { "pub " } else { "" };
        let type_params_str = Self::format_type_params_str(type_params);
        let fields_str = fields
            .iter()
            .map(|ty| Self::format_type(&ty.kind))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{pub_str}struct {name}{type_params_str}({fields_str})")
    }

    fn format_enum_decl(
        &self,
        name: &str,
        type_params: &[String],
        variants: &[crate::frontend::ast::EnumVariant],
        is_pub: bool,
    ) -> String {
        let pub_str = if is_pub { "pub " } else { "" };
        let type_params_str = Self::format_type_params_str(type_params);
        let variants_str = variants
            .iter()
            .map(|v| self.format_enum_variant(v))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{pub_str}enum {name}{type_params_str} {{ {variants_str} }}")
    }

    fn format_trait_decl(
        &self,
        name: &str,
        type_params: &[String],
        methods: &[crate::frontend::ast::TraitMethod],
        is_pub: bool,
    ) -> String {
        let pub_str = if is_pub { "pub " } else { "" };
        let type_params_str = Self::format_type_params_str(type_params);
        let methods_str = methods
            .iter()
            .map(|m| self.format_trait_method(m))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{pub_str}trait {name}{type_params_str} {{ {methods_str} }}")
    }

    fn format_impl_decl(
        &self,
        type_params: &[String],
        trait_name: Option<&String>,
        for_type: &str,
        methods: &[crate::frontend::ast::ImplMethod],
    ) -> String {
        let type_params_str = Self::format_type_params_str(type_params);
        let trait_part = trait_name.map_or(String::new(), |t| format!("{t} for "));
        let methods_str = methods
            .iter()
            .map(|m| self.format_impl_method(m))
            .collect::<Vec<_>>()
            .join(" ");
        format!("impl{type_params_str} {trait_part}{for_type} {{ {methods_str} }}")
    }

    fn format_class_decl(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[crate::frontend::ast::StructField],
    ) -> String {
        let type_params_str = Self::format_type_params_str(type_params);
        let fields_str = fields
            .iter()
            .map(|f| format!("{}: {}", f.name, Self::format_type(&f.ty.kind)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("class {name}{type_params_str} {{ {fields_str} }}")
    }

    fn format_import(&self, module: &str, items: Option<&Vec<String>>) -> String {
        if let Some(item_list) = items {
            format!("import {}::{{{}}}", module, item_list.join(", "))
        } else {
            format!("import {module}")
        }
    }

    fn format_string_interpolation(
        &self,
        parts: &[crate::frontend::ast::StringPart],
        indent: usize,
    ) -> String {
        let parts_str = parts
            .iter()
            .map(|part| match part {
                crate::frontend::ast::StringPart::Text(s) => s.clone(),
                crate::frontend::ast::StringPart::Expr(e) => {
                    format!("{{{}}}", self.format_expr(e, indent))
                }
                crate::frontend::ast::StringPart::ExprWithFormat { expr, format_spec } => {
                    format!("{{{}:{}}}", self.format_expr(expr, indent), format_spec)
                }
            })
            .collect::<String>();
        format!("f\"{parts_str}\"")
    }

    fn format_actor(
        &self,
        name: &str,
        state: &[crate::frontend::ast::StructField],
        handlers: &[crate::frontend::ast::ActorHandler],
    ) -> String {
        let state_str = state
            .iter()
            .map(|f| format!("{}: {}", f.name, Self::format_type(&f.ty.kind)))
            .collect::<Vec<_>>()
            .join(", ");
        let handlers_str = handlers
            .iter()
            .map(|h| format!("handle {}", h.message_type))
            .collect::<Vec<_>>()
            .join(" ");
        format!("actor {name} {{ {state_str} {handlers_str} }}")
    }

    fn format_effect(
        &self,
        name: &str,
        operations: &[crate::frontend::ast::EffectOperation],
    ) -> String {
        let ops_str = operations
            .iter()
            .map(|op| {
                let params_str = op
                    .params
                    .iter()
                    .map(|p| format!("{:?}: {:?}", p.pattern, p.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_str = op
                    .return_type
                    .as_ref()
                    .map(|t| format!(" -> {:?}", t.kind))
                    .unwrap_or_default();
                format!("{}({}){}", op.name, params_str, ret_str)
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("effect {name} {{ {ops_str} }}")
    }

    fn format_handle(
        &self,
        expr: &Expr,
        handlers: &[crate::frontend::ast::EffectHandler],
        indent: usize,
    ) -> String {
        let expr_str = self.format_expr(expr, indent);
        let handlers_str = handlers
            .iter()
            .map(|h| {
                let params_str = if h.params.is_empty() {
                    String::new()
                } else {
                    let joined = h
                        .params
                        .iter()
                        .map(|p| format!("{p:?}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({joined})")
                };
                let body_str = self.format_expr(&h.body, indent);
                format!("{}{} => {}", h.operation, params_str, body_str)
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("handle {expr_str} with {{ {handlers_str} }}")
    }

    fn format_pipeline(
        &self,
        expr: &Expr,
        stages: &[crate::frontend::ast::PipelineStage],
        indent: usize,
    ) -> String {
        let mut result = self.format_expr(expr, indent);
        for stage in stages {
            result.push_str(" |> ");
            result.push_str(&self.format_expr(&stage.op, indent));
        }
        result
    }

    fn format_comprehension_clauses(
        &self,
        clauses: &[crate::frontend::ast::ComprehensionClause],
        indent: usize,
    ) -> String {
        clauses
            .iter()
            .map(|clause| {
                let cond = clause
                    .condition
                    .as_ref()
                    .map(|c| format!(" if {}", self.format_expr(c, indent)))
                    .unwrap_or_default();
                format!(
                    "{} in {}{}",
                    clause.variable,
                    self.format_expr(&clause.iterable, indent),
                    cond
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_list_comprehension(
        &self,
        element: &Expr,
        clauses: &[crate::frontend::ast::ComprehensionClause],
        indent: usize,
    ) -> String {
        format!(
            "[{} for {}]",
            self.format_expr(element, indent),
            self.format_comprehension_clauses(clauses, indent)
        )
    }

    fn format_dict_comprehension(
        &self,
        key: &Expr,
        value: &Expr,
        clauses: &[crate::frontend::ast::ComprehensionClause],
        indent: usize,
    ) -> String {
        format!(
            "{{{}: {} for {}}}",
            self.format_expr(key, indent),
            self.format_expr(value, indent),
            self.format_comprehension_clauses(clauses, indent)
        )
    }

    fn format_set_comprehension(
        &self,
        element: &Expr,
        clauses: &[crate::frontend::ast::ComprehensionClause],
        indent: usize,
    ) -> String {
        format!(
            "{{{} for {}}}",
            self.format_expr(element, indent),
            self.format_comprehension_clauses(clauses, indent)
        )
    }

    fn format_command(&self, program: &str, args: &[String]) -> String {
        let full_cmd = if args.is_empty() {
            program.to_string()
        } else {
            format!("{} {}", program, args.join(" "))
        };
        format!("`{full_cmd}`")
    }

    fn format_optional_method_call(
        &self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
        indent: usize,
    ) -> String {
        let args_str = args
            .iter()
            .map(|arg| self.format_expr(arg, indent))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}?.{}({})",
            self.format_expr(receiver, indent),
            method,
            args_str
        )
    }

    fn format_extension(
        &self,
        target_type: &str,
        methods: &[crate::frontend::ast::ImplMethod],
        indent: usize,
    ) -> String {
        let indent_str = " ".repeat(indent * self.config.indent_width);
        let methods_str = methods
            .iter()
            .map(|method| {
                let params_str = method
                    .params
                    .iter()
                    .map(|p| self.format_pattern(&p.pattern))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "{}    fun {}({}) {{ }}",
                    indent_str, method.name, params_str
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("extension {target_type} {{\n{methods_str}\n{indent_str}}}")
    }

    fn format_macro_def(&self, name: &str, args: &[Expr], indent: usize) -> String {
        let args_str = args
            .iter()
            .map(|arg| self.format_expr(arg, indent))
            .collect::<Vec<_>>()
            .join(", ");
        format!("macro {name}({args_str}) {{ }}")
    }

    fn format_macro_invocation(&self, name: &str, args: &[Expr], indent: usize) -> String {
        let args_str = args
            .iter()
            .map(|arg| self.format_expr(arg, indent))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{name}!({args_str})")
    }

    fn format_dataframe(
        &self,
        columns: &[crate::frontend::ast::DataFrameColumn],
        indent: usize,
    ) -> String {
        let columns_str = columns
            .iter()
            .map(|col| {
                let values_str = col
                    .values
                    .iter()
                    .map(|v| self.format_expr(v, indent))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("\"{}\" => [{}]", col.name, values_str)
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("df![{columns_str}]")
    }

    /// Format a comment (complexity: 2)
    fn format_comment(&self, comment: &crate::frontend::ast::Comment, indent: usize) -> String {
        let indent_str = if self.config.use_tabs {
            "\t".repeat(indent)
        } else {
            " ".repeat(indent * self.config.indent_width)
        };

        match &comment.kind {
            crate::frontend::ast::CommentKind::Line(text) => {
                // Line comments: text already has leading space from lexer
                format!("{indent_str}//{text}")
            }
            crate::frontend::ast::CommentKind::Doc(text) => {
                // Doc comments: text already has leading space from lexer
                format!("{indent_str}///{text}")
            }
            crate::frontend::ast::CommentKind::Block(text) => {
                // Block comments: preserve text exactly as captured
                format!("{indent_str}/*{text}*/")
            }
        }
    }

    /// Format a pattern (complexity: 10)
    fn format_pattern(&self, pattern: &crate::frontend::ast::Pattern) -> String {
        use crate::frontend::ast::Pattern;

        match pattern {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Literal(lit) => self.format_literal(lit),
            Pattern::Identifier(name) => name.clone(),
            Pattern::QualifiedName(parts) => parts.join("::"),
            Pattern::Tuple(patterns) => {
                let inner = patterns
                    .iter()
                    .map(|p| self.format_pattern(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({inner})")
            }
            Pattern::List(patterns) => {
                let inner = patterns
                    .iter()
                    .map(|p| self.format_pattern(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{inner}]")
            }
            Pattern::Struct {
                name,
                fields,
                has_rest,
            } => {
                let fields_str = fields
                    .iter()
                    .map(|f| self.format_struct_pattern_field(f))
                    .collect::<Vec<_>>()
                    .join(", ");
                if *has_rest {
                    format!("{name} {{ {fields_str}, .. }}")
                } else {
                    format!("{name} {{ {fields_str} }}")
                }
            }
            Pattern::TupleVariant { path, patterns } => {
                let path_str = path.join("::");
                let patterns_str = patterns
                    .iter()
                    .map(|p| self.format_pattern(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{path_str}({patterns_str})")
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let op = if *inclusive { "..=" } else { ".." };
                format!(
                    "{}{}{}",
                    self.format_pattern(start),
                    op,
                    self.format_pattern(end)
                )
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .map(|p| self.format_pattern(p))
                .collect::<Vec<_>>()
                .join(" | "),
            Pattern::Rest => "..".to_string(),
            Pattern::RestNamed(name) => format!("..{name}"),
            Pattern::AtBinding { name, pattern } => {
                format!("{} @ {}", name, self.format_pattern(pattern))
            }
            Pattern::WithDefault { pattern, default } => {
                format!(
                    "{} = {}",
                    self.format_pattern(pattern),
                    self.format_expr(default, 0)
                )
            }
            Pattern::Mut(pattern) => {
                format!("mut {}", self.format_pattern(pattern))
            }
            Pattern::Ok(pattern) => {
                format!("Ok({})", self.format_pattern(pattern))
            }
            Pattern::Err(pattern) => {
                format!("Err({})", self.format_pattern(pattern))
            }
            Pattern::Some(pattern) => {
                format!("Some({})", self.format_pattern(pattern))
            }
            Pattern::None => "None".to_string(),
        }
    }

    /// Format a struct pattern field (complexity: 2)
    fn format_struct_pattern_field(
        &self,
        field: &crate::frontend::ast::StructPatternField,
    ) -> String {
        if let Some(pattern) = &field.pattern {
            format!("{}: {}", field.name, self.format_pattern(pattern))
        } else {
            // Shorthand syntax: just the field name
            field.name.clone()
        }
    }

    /// Format a literal value (complexity: 7)
    fn format_literal(&self, literal: &crate::frontend::ast::Literal) -> String {
        use crate::frontend::ast::Literal;

        match literal {
            Literal::Integer(val, suffix) => {
                if let Some(suffix) = suffix {
                    format!("{val}{suffix}")
                } else {
                    val.to_string()
                }
            }
            Literal::Float(val) => val.to_string(),
            Literal::String(s) => format!("\"{s}\""),
            Literal::Bool(b) => b.to_string(),
            Literal::Char(c) => format!("'{c}'"),
            Literal::Byte(b) => format!("{b}u8"),
            crate::frontend::ast::Literal::Unit => "()".to_string(),
            crate::frontend::ast::Literal::Null => "null".to_string(),
            crate::frontend::ast::Literal::Atom(s) => format!(":{s}"),
        }
    }
    /// Format an enum variant (complexity: 3)
    fn format_enum_variant(&self, variant: &crate::frontend::ast::EnumVariant) -> String {
        use crate::frontend::ast::EnumVariantKind;

        match &variant.kind {
            EnumVariantKind::Unit => variant.name.clone(),
            EnumVariantKind::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(|t| Self::format_type(&t.kind))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", variant.name, types_str)
            }
            EnumVariantKind::Struct(fields) => {
                let fields_str = fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, Self::format_type(&f.ty.kind)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", variant.name, fields_str)
            }
        }
    }

    /// Format a trait method (complexity: 3)
    fn format_trait_method(&self, method: &crate::frontend::ast::TraitMethod) -> String {
        let params_str = method
            .params
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    self.format_pattern(&p.pattern),
                    Self::format_type(&p.ty.kind)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let return_str = method.return_type.as_ref().map_or(String::new(), |t| {
            format!(" -> {}", Self::format_type(&t.kind))
        });
        format!("fun {}({}){}; ", method.name, params_str, return_str)
    }

    /// Format an impl method (complexity: 3)
    fn format_impl_method(&self, method: &crate::frontend::ast::ImplMethod) -> String {
        let params_str = method
            .params
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    self.format_pattern(&p.pattern),
                    Self::format_type(&p.ty.kind)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let return_str = method.return_type.as_ref().map_or(String::new(), |t| {
            format!(" -> {}", Self::format_type(&t.kind))
        });
        format!(
            "fun {}({}){}  {}",
            method.name,
            params_str,
            return_str,
            self.format_expr(&method.body, 0)
        )
    }
}
impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "formatter_tests.rs"]
mod tests;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "formatter_prop_tests.rs"]
mod property_tests_formatter;

#[cfg(test)]
#[path = "formatter_tests_r164.rs"]
mod formatter_tests_r164;

#[cfg(test)]
#[path = "formatter_coverage_tests.rs"]
mod formatter_coverage_tests;

#[cfg(test)]
#[path = "formatter_expr_coverage_tests.rs"]
mod formatter_expr_coverage_tests;

#[cfg(test)]
#[path = "formatter_expr_coverage_tests_part2.rs"]
mod formatter_expr_coverage_tests_part2;
