//! Network and Data Format Built-in Function Transpilation
//!
//! This module handles transpilation of network and data format functions:
//! - JSON: `json_parse`, `json_stringify`, `json_pretty`, `json_read`, `json_write`,
//!   `json_validate`, `json_type`, `json_merge`, `json_get`, `json_set`
//! - HTTP: `http_get`, `http_post`, `http_put`, `http_delete`
//!
//! **EXTREME TDD Round 62**: Extracted from statements.rs for modularization.

#![allow(clippy::doc_markdown)]

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Transpile JSON functions (json_*)
    ///
    /// Layer 2 of three-layer builtin pattern (proven from env/fs/path functions)
    /// Phase 4: `STDLIB_ACCESS_PLAN` - JSON Module (10 functions)
    /// Complexity: 10 (within Toyota Way limits)
    pub fn try_transpile_json_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "json_parse" => {
                if args.len() != 1 {
                    bail!("json_parse() expects 1 argument");
                }
                let json_str = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    serde_json::from_str::<serde_json::Value>(&#json_str)
                        .expect("JSON parse error")
                }))
            }
            "json_stringify" => {
                if args.len() != 1 {
                    bail!("json_stringify() expects 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        let value = #value;
                        // Convert value to JSON and stringify
                        serde_json::to_string(&value).unwrap_or_else(|_| String::from("null"))
                    }
                }))
            }
            "json_pretty" => {
                if args.len() != 1 {
                    bail!("json_pretty() expects 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        let value = #value;
                        serde_json::to_string_pretty(&value).unwrap_or_else(|_| String::from("null"))
                    }
                }))
            }
            "json_read" => {
                if args.len() != 1 {
                    bail!("json_read() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        let content = std::fs::read_to_string(#path).expect("Failed to read file");
                        serde_json::from_str::<serde_json::Value>(&content).expect("JSON parse error")
                    }
                }))
            }
            "json_write" => {
                if args.len() != 2 {
                    bail!("json_write() expects 2 arguments");
                }
                let path = self.transpile_expr(&args[0])?;
                let value = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    {
                        let json_str = serde_json::to_string_pretty(&#value).expect("JSON stringify error");
                        std::fs::write(#path, json_str).expect("Failed to write file");
                        true
                    }
                }))
            }
            "json_validate" => {
                if args.len() != 1 {
                    bail!("json_validate() expects 1 argument");
                }
                let json_str = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    serde_json::from_str::<serde_json::Value>(&#json_str).is_ok()
                }))
            }
            "json_type" => {
                if args.len() != 1 {
                    bail!("json_type() expects 1 argument");
                }
                let json_str = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        match serde_json::from_str::<serde_json::Value>(&#json_str) {
                            Ok(serde_json::Value::Null) => "null",
                            Ok(serde_json::Value::Bool(_)) => "boolean",
                            Ok(serde_json::Value::Number(_)) => "number",
                            Ok(serde_json::Value::String(_)) => "string",
                            Ok(serde_json::Value::Array(_)) => "array",
                            Ok(serde_json::Value::Object(_)) => "object",
                            Err(_) => "invalid",
                        }.to_string()
                    }
                }))
            }
            "json_merge" => {
                if args.len() != 2 {
                    bail!("json_merge() expects 2 arguments");
                }
                let obj1 = self.transpile_expr(&args[0])?;
                let obj2 = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    {
                        // Deep merge two JSON objects
                        fn merge_json(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
                            match (a, b) {
                                (serde_json::Value::Object(mut a_map), serde_json::Value::Object(b_map)) => {
                                    for (k, v) in b_map {
                                        if let Some(a_val) = a_map.get_mut(&k) {
                                            *a_val = merge_json(a_val.clone(), v);
                                        } else {
                                            a_map.insert(k, v);
                                        }
                                    }
                                    serde_json::Value::Object(a_map)
                                },
                                (_, b_val) => b_val,
                            }
                        }
                        merge_json(#obj1, #obj2)
                    }
                }))
            }
            "json_get" => {
                if args.len() != 2 {
                    bail!("json_get() expects 2 arguments");
                }
                let obj = self.transpile_expr(&args[0])?;
                let path = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    {
                        let parts: Vec<&str> = #path.split('.').collect();
                        let mut current = &#obj;
                        for part in parts {
                            if let serde_json::Value::Object(map) = current {
                                current = map.get(part).unwrap_or(&serde_json::Value::Null);
                            } else {
                                current = &serde_json::Value::Null;
                                break;
                            }
                        }
                        current.clone()
                    }
                }))
            }
            "json_set" => {
                if args.len() != 3 {
                    bail!("json_set() expects 3 arguments");
                }
                let obj = self.transpile_expr(&args[0])?;
                let path = self.transpile_expr(&args[1])?;
                let value = self.transpile_expr(&args[2])?;
                Ok(Some(quote! {
                    {
                        fn set_json_path(obj: serde_json::Value, path: &str, value: serde_json::Value) -> serde_json::Value {
                            let mut result = obj.clone();
                            let parts: Vec<&str> = path.split('.').collect();
                            if let serde_json::Value::Object(ref mut map) = result {
                                if parts.len() == 1 {
                                    map.insert(parts[0].to_string(), value);
                                } else if !parts.is_empty() {
                                    // Nested path setting
                                    let first = parts[0];
                                    let rest = parts[1..].join(".");
                                    if let Some(nested) = map.get(first).cloned() {
                                        let updated = set_json_path(nested, &rest, value);
                                        map.insert(first.to_string(), updated);
                                    }
                                }
                            }
                            result
                        }
                        set_json_path(#obj, &#path, serde_json::json!(#value))
                    }
                }))
            }
            _ => Ok(None),
        }
    }

    /// Transpile HTTP builtin functions (STDLIB-PHASE-5)
    ///
    /// Wraps `ruchy::stdlib::http` module functions for compilation
    /// Complexity: 5 (within Toyota Way limits)
    pub fn try_transpile_http_function(
        &self,
        name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match name {
            "http_get" => {
                if args.len() != 1 {
                    bail!("http_get() expects 1 argument");
                }
                let url = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        let response = reqwest::blocking::get(&#url).expect("HTTP GET failed");
                        if !response.status().is_success() {
                            panic!("HTTP GET failed with status {}", response.status());
                        }
                        response.text().expect("Failed to read response body")
                    }
                }))
            }
            "http_post" => {
                if args.len() != 2 {
                    bail!("http_post() expects 2 arguments");
                }
                let url = self.transpile_expr(&args[0])?;
                let body = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    {
                        let client = reqwest::blocking::Client::new();
                        let response = client.post(&#url)
                            .header("content-type", "application/json")
                            .body((#body).to_string())
                            .send()
                            .expect("HTTP POST failed");
                        if !response.status().is_success() {
                            panic!("HTTP POST failed with status {}", response.status());
                        }
                        response.text().expect("Failed to read response body")
                    }
                }))
            }
            "http_put" => {
                if args.len() != 2 {
                    bail!("http_put() expects 2 arguments");
                }
                let url = self.transpile_expr(&args[0])?;
                let body = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    {
                        let client = reqwest::blocking::Client::new();
                        let response = client.put(&#url)
                            .header("content-type", "application/json")
                            .body((#body).to_string())
                            .send()
                            .expect("HTTP PUT failed");
                        if !response.status().is_success() {
                            panic!("HTTP PUT failed with status {}", response.status());
                        }
                        response.text().expect("Failed to read response body")
                    }
                }))
            }
            "http_delete" => {
                if args.len() != 1 {
                    bail!("http_delete() expects 1 argument");
                }
                let url = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        let client = reqwest::blocking::Client::new();
                        let response = client.delete(&#url)
                            .send()
                            .expect("HTTP DELETE failed");
                        if !response.status().is_success() {
                            panic!("HTTP DELETE failed with status {}", response.status());
                        }
                        response.text().expect("Failed to read response body")
                    }
                }))
            }
            _ => Ok(None),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // JSON function tests
    // ========================================================================

    #[test]
    fn test_json_parse() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr(r#"{"key": "value"}"#)];
        let result = transpiler.try_transpile_json_function("json_parse", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("serde_json"));
        assert!(tokens_str.contains("from_str"));
    }

    #[test]
    fn test_json_parse_wrong_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_json_function("json_parse", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_stringify() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("obj")];
        let result = transpiler.try_transpile_json_function("json_stringify", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("to_string"));
    }

    #[test]
    fn test_json_pretty() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("data")];
        let result = transpiler.try_transpile_json_function("json_pretty", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("to_string_pretty"));
    }

    #[test]
    fn test_json_read() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/data.json")];
        let result = transpiler.try_transpile_json_function("json_read", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("read_to_string"));
    }

    #[test]
    fn test_json_write() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/out.json"), ident_expr("data")];
        let result = transpiler.try_transpile_json_function("json_write", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("fs :: write"));
    }

    #[test]
    fn test_json_write_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/out.json")];
        let result = transpiler.try_transpile_json_function("json_write", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_validate() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr(r#"{"valid": true}"#)];
        let result = transpiler.try_transpile_json_function("json_validate", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("is_ok"));
    }

    #[test]
    fn test_json_type() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr(r#"{"object": true}"#)];
        let result = transpiler.try_transpile_json_function("json_type", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Value :: Null"));
        assert!(tokens_str.contains("Value :: Bool"));
        assert!(tokens_str.contains("Value :: Object"));
    }

    #[test]
    fn test_json_merge() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("obj1"), ident_expr("obj2")];
        let result = transpiler.try_transpile_json_function("json_merge", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("merge_json"));
    }

    #[test]
    fn test_json_merge_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("obj1")];
        let result = transpiler.try_transpile_json_function("json_merge", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_get() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("obj"), string_expr("key.nested")];
        let result = transpiler.try_transpile_json_function("json_get", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("split"));
        assert!(tokens_str.contains("clone"));
    }

    #[test]
    fn test_json_set() {
        let transpiler = Transpiler::new();
        let args = vec![
            ident_expr("obj"),
            string_expr("key"),
            string_expr("new_value"),
        ];
        let result = transpiler.try_transpile_json_function("json_set", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("set_json_path"));
    }

    #[test]
    fn test_json_set_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("obj"), string_expr("key")];
        let result = transpiler.try_transpile_json_function("json_set", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("test")];
        let result = transpiler.try_transpile_json_function("json_unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // HTTP function tests
    // ========================================================================

    #[test]
    fn test_http_get() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("https://api.example.com")];
        let result = transpiler.try_transpile_http_function("http_get", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("reqwest"));
        assert!(tokens_str.contains("get"));
    }

    #[test]
    fn test_http_get_wrong_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_http_function("http_get", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_post() {
        let transpiler = Transpiler::new();
        let args = vec![
            string_expr("https://api.example.com"),
            string_expr(r#"{"key": "value"}"#),
        ];
        let result = transpiler.try_transpile_http_function("http_post", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("post"));
        assert!(tokens_str.contains("content-type"));
    }

    #[test]
    fn test_http_post_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("https://api.example.com")];
        let result = transpiler.try_transpile_http_function("http_post", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_put() {
        let transpiler = Transpiler::new();
        let args = vec![
            string_expr("https://api.example.com/resource"),
            string_expr(r#"{"updated": true}"#),
        ];
        let result = transpiler.try_transpile_http_function("http_put", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("put"));
    }

    #[test]
    fn test_http_delete() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("https://api.example.com/resource/123")];
        let result = transpiler.try_transpile_http_function("http_delete", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("delete"));
    }

    #[test]
    fn test_http_delete_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("https://api.example.com"), string_expr("extra")];
        let result = transpiler.try_transpile_http_function("http_delete", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("test")];
        let result = transpiler.try_transpile_http_function("http_unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_http_get_with_identifier() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("url")];
        let result = transpiler.try_transpile_http_function("http_get", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("url"));
    }

    #[test]
    fn test_http_post_with_identifiers() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("endpoint"), ident_expr("body")];
        let result = transpiler.try_transpile_http_function("http_post", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("endpoint"));
        assert!(tokens_str.contains("body"));
    }
}
