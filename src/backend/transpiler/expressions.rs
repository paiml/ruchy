//! Expression transpilation methods

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::needless_pass_by_value)] // TokenStream by value is intentional for quote! macro

use super::Transpiler;
use crate::frontend::ast::{BinaryOp::{self, NullCoalesce}, Expr, ExprKind, Literal, StringPart, UnaryOp};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles literal values
    pub fn transpile_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::Integer(i) => Self::transpile_integer(*i),
            Literal::Float(f) => quote! { #f },
            Literal::Unit => quote! { () },
            _ => Self::transpile_simple_literal(lit),
        }
    }

    fn transpile_simple_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::String(s) => quote! { #s },
            Literal::Bool(b) => quote! { #b },
            Literal::Char(c) => quote! { #c },
            _ => unreachable!(),
        }
    }

    fn transpile_integer(i: i64) -> TokenStream {
        // Integer literals in Rust need proper type handling
        // Use i32 for values that fit, i64 otherwise
        if let Ok(i32_val) = i32::try_from(i) {
            // Use i32 suffix for clarity and to match struct field types
            let literal = proc_macro2::Literal::i32_suffixed(i32_val);
            quote! { #literal }
        } else {
            // For large integers, we need i64 suffix
            let literal = proc_macro2::Literal::i64_suffixed(i);
            quote! { #literal }
        }
    }

    /// Transpiles string interpolation
    ///
    /// # Errors
    /// Returns an error if expression transpilation fails
    pub fn transpile_string_interpolation(&self, parts: &[StringPart]) -> Result<TokenStream> {
        if parts.is_empty() {
            return Ok(quote! { "" });
        }

        let mut format_string = String::new();
        let mut args = Vec::new();

        for part in parts {
            match part {
                StringPart::Text(s) => {
                    // Escape any format specifiers in literal parts
                    format_string.push_str(&s.replace('{', "{{").replace('}', "}}"));
                }
                StringPart::Expr(expr) => {
                    format_string.push_str("{}");
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
                StringPart::ExprWithFormat { expr, format_spec } => {
                    // Include the format specifier in the format string
                    format_string.push('{');
                    format_string.push_str(format_spec);
                    format_string.push('}');
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
            }
        }

        Ok(quote! {
            format!(#format_string #(, #args)*)
        })
    }

    /// Transpiles binary operations
    pub fn transpile_binary(&self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<TokenStream> {
        // Special handling for string concatenation
        // Only treat as string concatenation if at least one operand is definitely a string
        if op == BinaryOp::Add && (Self::is_definitely_string(left) || Self::is_definitely_string(right)) {
            return self.transpile_string_concatenation(left, right);
        }

        // Transpile operands with precedence-aware parentheses
        let left_tokens = self.transpile_expr_with_precedence(left, op, true)?;
        let right_tokens = self.transpile_expr_with_precedence(right, op, false)?;

        Ok(Self::transpile_binary_op(left_tokens, op, right_tokens))
    }

    /// Transpile expression with precedence-aware parentheses
    /// 
    /// Adds parentheses around sub-expressions when needed to preserve precedence
    fn transpile_expr_with_precedence(&self, expr: &Expr, parent_op: BinaryOp, is_left_operand: bool) -> Result<TokenStream> {
        let tokens = self.transpile_expr(expr)?;
        
        // Check if we need parentheses
        if let ExprKind::Binary { op: child_op, .. } = &expr.kind {
            let parent_prec = Self::get_operator_precedence(parent_op);
            let child_prec = Self::get_operator_precedence(*child_op);
            
            // Add parentheses if child has lower precedence
            // For right operands, also add parentheses if precedence is equal and parent is right-associative  
            let needs_parens = child_prec < parent_prec ||
                (!is_left_operand && child_prec == parent_prec && Self::is_right_associative(parent_op));
            
            if needs_parens {
                return Ok(quote! { (#tokens) });
            }
        }
        
        Ok(tokens)
    }

    /// Get operator precedence (higher number = higher precedence)
    fn get_operator_precedence(op: BinaryOp) -> i32 {
        match op {
            BinaryOp::Or => 10,
            BinaryOp::And => 20,
            BinaryOp::Equal | BinaryOp::NotEqual => 30,
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => 40,
            BinaryOp::Add | BinaryOp::Subtract => 50,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 60,
            BinaryOp::Power => 70,
            _ => 0, // Default for other operators
        }
    }
    
    /// Check if operator is right-associative
    fn is_right_associative(op: BinaryOp) -> bool {
        matches!(op, BinaryOp::Power) // Only power is right-associative in most languages
    }

    fn transpile_binary_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{
            Add, And, BitwiseAnd, BitwiseOr, BitwiseXor, Divide, Equal, Greater, GreaterEqual,
            LeftShift, Less, LessEqual, Modulo, Multiply, NotEqual, Or, Power,
            Subtract,
        };
        match op {
            // Arithmetic operations
            Add | Subtract | Multiply | Divide | Modulo | Power => {
                Self::transpile_arithmetic_op(left, op, right)
            }
            // Comparison operations
            Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual => {
                Self::transpile_comparison_op(left, op, right)
            }
            // Logical operations
            And | Or | NullCoalesce => Self::transpile_logical_op(left, op, right),
            // Bitwise operations
            BitwiseAnd | BitwiseOr | BitwiseXor | LeftShift => {
                Self::transpile_bitwise_op(left, op, right)
            }
        }
    }

    fn transpile_arithmetic_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{Add, Divide, Modulo, Multiply, Power, Subtract};
        match op {
            Add | Subtract | Multiply | Divide | Modulo => {
                Self::transpile_basic_arithmetic(left, op, right)
            }
            Power => quote! { #left.pow(#right) },
            _ => unreachable!(),
        }
    }

    fn transpile_basic_arithmetic(
        left: TokenStream,
        op: BinaryOp,
        right: TokenStream,
    ) -> TokenStream {
        // Reduce complexity by splitting into smaller functions
        match op {
            BinaryOp::Add => quote! { #left + #right },
            BinaryOp::Subtract => quote! { #left - #right },
            BinaryOp::Multiply => quote! { #left * #right },
            _ => Self::transpile_division_mod(left, op, right),
        }
    }

    fn transpile_division_mod(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Divide => quote! { #left / #right },
            BinaryOp::Modulo => quote! { #left % #right },
            _ => unreachable!(),
        }
    }

    fn transpile_comparison_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual};
        match op {
            Equal | NotEqual => Self::transpile_equality(left, op, right),
            Less | LessEqual | Greater | GreaterEqual => Self::transpile_ordering(left, op, right),
            _ => unreachable!(),
        }
    }

    fn transpile_equality(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Equal => quote! { #left == #right },
            BinaryOp::NotEqual => quote! { #left != #right },
            _ => unreachable!(),
        }
    }

    fn transpile_ordering(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Less => quote! { #left < #right },
            BinaryOp::LessEqual => quote! { #left <= #right },
            _ => Self::transpile_greater_ops(left, op, right),
        }
    }

    fn transpile_greater_ops(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Greater => quote! { #left > #right },
            BinaryOp::GreaterEqual => quote! { #left >= #right },
            _ => unreachable!(),
        }
    }

    fn transpile_logical_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::And => quote! { #left && #right },
            BinaryOp::Or => quote! { #left || #right },
            _ => unreachable!(),
        }
    }

    fn transpile_bitwise_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{BitwiseAnd, BitwiseOr, BitwiseXor};
        match op {
            BitwiseAnd => quote! { #left & #right },
            BitwiseOr => quote! { #left | #right },
            BitwiseXor => quote! { #left ^ #right },
            _ => Self::transpile_shift_ops(left, op, right),
        }
    }

    fn transpile_shift_ops(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::LeftShift => quote! { #left << #right },
            _ => unreachable!(),
        }
    }

    /// Transpiles unary operations  
    pub fn transpile_unary(&self, op: UnaryOp, operand: &Expr) -> Result<TokenStream> {
        let operand_tokens = self.transpile_expr(operand)?;

        Ok(match op {
            UnaryOp::Not | UnaryOp::BitwiseNot => quote! { !#operand_tokens },
            UnaryOp::Negate => quote! { -#operand_tokens },
            UnaryOp::Reference => quote! { &#operand_tokens },
        })
    }


    /// Transpiles await expressions
    pub fn transpile_await(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! { #expr_tokens.await })
    }

    /// Transpiles async blocks
    pub fn transpile_async_block(&self, body: &Expr) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;
        Ok(quote! { async { #body_tokens } })
    }

    /// Transpiles throw expressions (panic in Rust)
    pub fn transpile_throw(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! {
            panic!(#expr_tokens)
        })
    }

    /// Transpiles field access
    pub fn transpile_field_access(&self, object: &Expr, field: &str) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        let field_ident = format_ident!("{}", field);
        Ok(quote! { #obj_tokens.#field_ident })
    }

    /// Transpiles index access (array[index])
    pub fn transpile_index_access(&self, object: &Expr, index: &Expr) -> Result<TokenStream> {
        use crate::frontend::ast::{ExprKind, Literal};
        
        let obj_tokens = self.transpile_expr(object)?;
        let index_tokens = self.transpile_expr(index)?;
        
        // Smart index access: HashMap.get() for string keys, array indexing for numeric
        match &index.kind {
            // String literal keys use HashMap.get()
            ExprKind::Literal(Literal::String(_)) => {
                Ok(quote! { 
                    #obj_tokens.get(#index_tokens)
                        .cloned()
                        .unwrap_or_else(|| panic!("Key not found"))
                })
            }
            // Numeric and other keys use array indexing
            _ => {
                Ok(quote! { #obj_tokens[#index_tokens as usize] })
            }
        }
    }

    /// Transpiles slice access (array[start:end])
    pub fn transpile_slice(&self, object: &Expr, start: Option<&Expr>, end: Option<&Expr>) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        
        match (start, end) {
            (None, None) => {
                // Full slice [..]
                Ok(quote! { &#obj_tokens[..] })
            }
            (None, Some(end)) => {
                // Slice from beginning [..end]
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[..#end_tokens as usize] })
            }
            (Some(start), None) => {
                // Slice to end [start..]
                let start_tokens = self.transpile_expr(start)?;
                Ok(quote! { &#obj_tokens[#start_tokens as usize..] })
            }
            (Some(start), Some(end)) => {
                // Full range slice [start..end]
                let start_tokens = self.transpile_expr(start)?;
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[#start_tokens as usize..#end_tokens as usize] })
            }
        }
    }

    /// Transpiles assignment
    pub fn transpile_assign(&self, target: &Expr, value: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        let value_tokens = self.transpile_expr(value)?;
        Ok(quote! { #target_tokens = #value_tokens })
    }

    /// Transpiles compound assignment
    pub fn transpile_compound_assign(
        &self,
        target: &Expr,
        op: BinaryOp,
        value: &Expr,
    ) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        let value_tokens = self.transpile_expr(value)?;
        let op_tokens = Self::get_compound_op_token(op)?;

        Ok(quote! { #target_tokens #op_tokens #value_tokens })
    }

    fn get_compound_op_token(op: BinaryOp) -> Result<TokenStream> {
        use BinaryOp::{Add, Divide, Modulo, Multiply, Subtract};
        match op {
            Add | Subtract | Multiply => Ok(Self::get_basic_compound_token(op)),
            Divide | Modulo => Ok(Self::get_division_compound_token(op)),
            _ => {
                use anyhow::bail;
                bail!("Invalid operator for compound assignment: {:?}", op)
            }
        }
    }

    fn get_basic_compound_token(op: BinaryOp) -> TokenStream {
        match op {
            BinaryOp::Add => quote! { += },
            BinaryOp::Subtract => quote! { -= },
            BinaryOp::Multiply => quote! { *= },
            _ => unreachable!(),
        }
    }

    fn get_division_compound_token(op: BinaryOp) -> TokenStream {
        match op {
            BinaryOp::Divide => quote! { /= },
            BinaryOp::Modulo => quote! { %= },
            _ => unreachable!(),
        }
    }

    /// Transpiles pre-increment
    pub fn transpile_pre_increment(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! { { #target_tokens += 1; #target_tokens } })
    }

    /// Transpiles post-increment
    pub fn transpile_post_increment(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            {
                let _tmp = #target_tokens;
                #target_tokens += 1;
                _tmp
            }
        })
    }

    /// Transpiles pre-decrement
    pub fn transpile_pre_decrement(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! { { #target_tokens -= 1; #target_tokens } })
    }

    /// Transpiles post-decrement
    pub fn transpile_post_decrement(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            {
                let _tmp = #target_tokens;
                #target_tokens -= 1;
                _tmp
            }
        })
    }

    /// Transpiles list literals
    pub fn transpile_list(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;
        Ok(quote! { vec![#(#element_tokens),*] })
    }

    /// Transpiles tuple literals
    pub fn transpile_tuple(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;
        Ok(quote! { (#(#element_tokens),*) })
    }

    /// Transpiles range expressions
    pub fn transpile_range(
        &self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
    ) -> Result<TokenStream> {
        let start_tokens = self.transpile_expr(start)?;
        let end_tokens = self.transpile_expr(end)?;

        if inclusive {
            Ok(quote! { #start_tokens..=#end_tokens })
        } else {
            Ok(quote! { #start_tokens..#end_tokens })
        }
    }

    /// Transpiles object literals
    pub fn transpile_object_literal(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<TokenStream> {
        let field_tokens = self.collect_hashmap_field_tokens(fields)?;
        Ok(quote! {
            {
                let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                #(#field_tokens)*
                map
            }
        })
    }

    fn collect_hashmap_field_tokens(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<Vec<TokenStream>> {
        use crate::frontend::ast::ObjectField;
        let mut field_tokens = Vec::new();

        for field in fields {
            let token = match field {
                ObjectField::KeyValue { key, value } => {
                    let value_tokens = self.transpile_expr(value)?;
                    quote! { map.insert(#key.to_string(), (#value_tokens).to_string()); }
                }
                ObjectField::Spread { expr } => {
                    let expr_tokens = self.transpile_expr(expr)?;
                    // For spread syntax, merge the other map into this one
                    quote! { 
                        for (k, v) in #expr_tokens {
                            map.insert(k, v);
                        }
                    }
                }
            };
            field_tokens.push(token);
        }

        Ok(field_tokens)
    }


    /// Transpiles struct literals
    pub fn transpile_struct_literal(
        &self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> Result<TokenStream> {
        let struct_name = format_ident!("{}", name);
        let mut field_tokens = Vec::new();

        for (field_name, value) in fields {
            let field_ident = format_ident!("{}", field_name);
            let value_tokens = match &value.kind {
                // Convert string literals to String for struct fields
                ExprKind::Literal(Literal::String(s)) => {
                    quote! { #s.to_string() }
                }
                _ => self.transpile_expr(value)?,
            };
            field_tokens.push(quote! { #field_ident: #value_tokens });
        }

        Ok(quote! {
            #struct_name {
                #(#field_tokens,)*
            }
        })
    }

    /// Check if an expression is definitely a string (conservative detection)
    fn is_definitely_string(expr: &Expr) -> bool {
        match &expr.kind {
            // String literals are definitely strings
            ExprKind::Literal(Literal::String(_)) => true,
            // String interpolation is definitely strings
            ExprKind::StringInterpolation { .. } => true,
            // Binary expressions with + that involve strings are string concatenations
            ExprKind::Binary { op: BinaryOp::Add, left, right } => {
                Self::is_definitely_string(left) || Self::is_definitely_string(right)
            },
            // Method calls on strings that return strings
            ExprKind::MethodCall { receiver, method, .. } => {
                matches!(method.as_str(), "to_string" | "trim" | "to_uppercase" | "to_lowercase") ||
                Self::is_definitely_string(receiver)
            },
            // Variables could be strings, but we can't be sure without type info
            // For now, be conservative and don't assume variables are strings
            ExprKind::Identifier(_) => false,
            // Function calls are NOT definitely strings - they could return any type
            ExprKind::Call { .. } => false,
            // Other expressions are not strings
            _ => false,
        }
    }


    /// Transpile string concatenation using proper Rust string operations
    fn transpile_string_concatenation(&self, left: &Expr, right: &Expr) -> Result<TokenStream> {
        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;
        
        // Use format! with proper string handling - convert both to strings to avoid type mismatches
        // This avoids the String + String issue in Rust by using format! exclusively
        Ok(quote! { format!("{}{}", #left_tokens, #right_tokens) })
    }
}

#[cfg(test)]
#[allow(clippy::single_char_pattern)]
mod tests {
    use super::*;
    use crate::{Parser, frontend::ast::ExprKind};

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
    }

    #[test]
    fn test_transpile_integer_literal() {
        let transpiler = create_transpiler();
        let code = "42";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("42"));
    }

    #[test]
    fn test_transpile_float_literal() {
        let transpiler = create_transpiler();
        let code = "3.14";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("3.14"));
    }

    #[test]
    fn test_transpile_string_literal() {
        let transpiler = create_transpiler();
        let code = "\"hello\"";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("hello"));
    }

    #[test]
    fn test_transpile_boolean_literal() {
        let transpiler = create_transpiler();
        let code = "true";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("true"));
    }

    #[test]
    fn test_transpile_unit_literal() {
        let transpiler = create_transpiler();
        let code = "()";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("()"));
    }

    #[test]
    fn test_transpile_binary_addition() {
        let transpiler = create_transpiler();
        let code = "5 + 3";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("5") && rust_str.contains("3"));
        assert!(rust_str.contains("+"));
    }

    #[test]
    fn test_transpile_binary_subtraction() {
        let transpiler = create_transpiler();
        let code = "10 - 4";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("10") && rust_str.contains("4"));
        assert!(rust_str.contains("-"));
    }

    #[test]
    fn test_transpile_binary_multiplication() {
        let transpiler = create_transpiler();
        let code = "6 * 7";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("6") && rust_str.contains("7"));
        assert!(rust_str.contains("*"));
    }

    #[test]
    fn test_transpile_binary_division() {
        let transpiler = create_transpiler();
        let code = "15 / 3";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("15") && rust_str.contains("3"));
        assert!(rust_str.contains("/"));
    }

    #[test]
    fn test_transpile_binary_modulo() {
        let transpiler = create_transpiler();
        let code = "10 % 3";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("10") && rust_str.contains("3"));
        assert!(rust_str.contains("%"));
    }

    // Note: String concatenation test removed due to parser limitations with string + operator

    #[test]
    fn test_transpile_comparison_operators() {
        let operators = vec!["<", ">", "<=", ">=", "==", "!="];
        
        for op in operators {
            let transpiler = create_transpiler();
            let code = format!("5 {op} 3");
            let mut parser = Parser::new(&code);
            let ast = parser.parse().unwrap();
            
            let result = transpiler.transpile(&ast).unwrap();
            let rust_str = result.to_string();
            
            assert!(rust_str.contains("5") && rust_str.contains("3"), 
                   "Failed for operator {op}: {rust_str}");
        }
    }

    #[test]
    fn test_transpile_logical_operators() {
        let operators = vec!["&&", "||"];
        
        for op in operators {
            let transpiler = create_transpiler();
            let code = format!("true {op} false");
            let mut parser = Parser::new(&code);
            let ast = parser.parse().unwrap();
            
            let result = transpiler.transpile(&ast).unwrap();
            let rust_str = result.to_string();
            
            assert!(rust_str.contains("true") && rust_str.contains("false"),
                   "Failed for operator {op}: {rust_str}");
        }
    }

    #[test]
    fn test_transpile_unary_operators() {
        let test_cases = vec![
            ("!true", "true"),
            ("-5", "5"),
        ];
        
        for (code, expected) in test_cases {
            let transpiler = create_transpiler();
            let mut parser = Parser::new(code);
            let ast = parser.parse().unwrap();
            
            let result = transpiler.transpile(&ast).unwrap();
            let rust_str = result.to_string();
            
            assert!(rust_str.contains(expected),
                   "Failed for {code}: {rust_str}");
        }
    }

    #[test]
    fn test_transpile_identifier() {
        let transpiler = create_transpiler();
        let code = "variable_name";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("variable_name"));
    }

    #[test]
    fn test_transpile_function_call() {
        let transpiler = create_transpiler();
        let code = "func_name(arg1, arg2)";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("func_name"));
        assert!(rust_str.contains("arg1"));
        assert!(rust_str.contains("arg2"));
    }

    #[test]
    fn test_transpile_function_call_no_args() {
        let transpiler = create_transpiler();
        let code = "func_name()";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("func_name"));
        assert!(rust_str.contains("()"));
    }

    #[test]
    fn test_transpile_list() {
        let transpiler = create_transpiler();
        let code = "[1, 2, 3]";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("vec!") || rust_str.contains("["));
        assert!(rust_str.contains("1") && rust_str.contains("2") && rust_str.contains("3"));
    }

    #[test]
    fn test_transpile_empty_list() {
        let transpiler = create_transpiler();
        let code = "[]";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("vec!") || rust_str.contains("[]"));
    }

    #[test]
    fn test_transpile_range() {
        let transpiler = create_transpiler();
        let code = "1..10";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("1") && rust_str.contains("10"));
        assert!(rust_str.contains("..") || rust_str.contains("range"));
    }

    #[test]
    fn test_transpile_inclusive_range() {
        let transpiler = create_transpiler();
        let code = "1..=10";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("1") && rust_str.contains("10"));
        assert!(rust_str.contains("..=") || rust_str.contains("inclusive"));
    }

    #[test]
    fn test_transpile_block_expression() {
        let transpiler = create_transpiler();
        let code = "{ let x = 5; x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("{"));
        assert!(rust_str.contains("}"));
        assert!(rust_str.contains("let"));
        assert!(rust_str.contains("x"));
        assert!(rust_str.contains("5"));
    }

    #[test]
    fn test_definitely_string_detection() {
        // String literals should be definitely strings
        let code1 = "\"hello\"";
        let mut parser1 = Parser::new(code1);
        let ast1 = parser1.parse().unwrap();
        if let ExprKind::Block(exprs) = &ast1.kind {
            if let Some(expr) = exprs.first() {
                assert!(Transpiler::is_definitely_string(expr));
            }
        }
        
        // Numbers should not be definitely strings
        let code2 = "42";
        let mut parser2 = Parser::new(code2);
        let ast2 = parser2.parse().unwrap();
        if let ExprKind::Block(exprs) = &ast2.kind {
            if let Some(expr) = exprs.first() {
                assert!(!Transpiler::is_definitely_string(expr));
            }
        }
    }

    #[test]
    fn test_complex_nested_expressions() {
        let transpiler = create_transpiler();
        let code = "(5 + 3) * (10 - 2)";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // Should handle nested arithmetic with parentheses
        assert!(rust_str.contains("5") && rust_str.contains("3"));
        assert!(rust_str.contains("10") && rust_str.contains("2"));
        assert!(rust_str.contains("+") && rust_str.contains("-") && rust_str.contains("*"));
    }

    #[test]
    fn test_integer_literal_size_handling() {
        // Small integers
        assert_eq!(
            Transpiler::transpile_integer(42).to_string(),
            "42i32"
        );
        
        // Large integers  
        #[allow(clippy::unreadable_literal)]
        let large_int = 9223372036854775807;
        assert_eq!(
            Transpiler::transpile_integer(large_int).to_string(),
            "9223372036854775807i64"
        );
        
        // Negative integers
        assert_eq!(
            Transpiler::transpile_integer(-42).to_string(),
            "- 42i32"
        );
    }

    #[test]
    fn test_method_call_transpilation() {
        let transpiler = create_transpiler();
        let code = "obj.method(arg)";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("obj"));
        assert!(rust_str.contains("method"));
        assert!(rust_str.contains("arg"));
    }

    #[test]
    fn test_string_interpolation_transpilation() {
        let transpiler = create_transpiler();
        let code = "f\"Hello {name}\"";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // String interpolation should use format!
        assert!(rust_str.contains("format!") || rust_str.contains("Hello"));
        assert!(rust_str.contains("name"));
    }
}
