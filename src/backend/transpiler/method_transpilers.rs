//! Collection and String Method Transpilation
//!
//! This module handles transpilation of method calls on collections and strings:
//! - Iterator methods: map, filter, reduce
//! - HashMap/HashSet methods: contains_key, items, update
//! - Set operations: union, intersection, difference
//! - String methods: to_upper, strip, split, etc.
//! - Advanced collection methods: slice, concat, flatten, unique, join
//!
//! **EXTREME TDD Round 65**: Extracted from statements.rs for modularization.

use super::Transpiler;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Handle iterator operations: map, filter, reduce
    ///
    /// DEFECT-023 FIX: Check if receiver already has `.iter()` to avoid double iteration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"[1, 2, 3].map(|x| x * 2)"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("map"));
    /// ```
    /// Complexity: 5 (within Toyota Way limits)
    pub fn transpile_iterator_methods(
        &self,
        obj_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        // Check if receiver already ends with .iter() or .into_iter()
        let obj_str = obj_tokens.to_string();
        let already_iter = obj_str.ends_with(". iter ()")
            || obj_str.ends_with(". into_iter ()")
            || obj_str.contains(". iter ( )");

        match method {
            "map" => {
                // vec.map(f) -> vec.iter().map(f).collect::<Vec<_>>()
                // DEFECT-023: Skip .iter() if receiver is already an iterator
                if already_iter {
                    Ok(quote! { #obj_tokens.map(#(#arg_tokens),*).collect::<Vec<_>>() })
                } else {
                    Ok(quote! { #obj_tokens.iter().map(#(#arg_tokens),*).collect::<Vec<_>>() })
                }
            }
            "filter" => {
                // TRANSPILER-ITERATOR-001 FIX: Wrap user closure to handle reference
                // filter's closure signature is FnMut(&Item) - it ALWAYS receives a reference
                // We wrap the user's closure to dereference: |__x| user_closure(*__x)
                if already_iter {
                    Ok(quote! { #obj_tokens.filter(#(#arg_tokens),*).collect::<Vec<_>>() })
                } else {
                    // Wrap closure: |__x| { let __f = user_closure; __f(*__x) }
                    let user_closure = &arg_tokens[0];
                    Ok(
                        quote! { #obj_tokens.into_iter().filter(|__x| { let __f = #user_closure; __f(*__x) }).collect::<Vec<_>>() },
                    )
                }
            }
            "reduce" => {
                // vec.reduce(f) -> vec.into_iter().reduce(f)
                if already_iter {
                    Ok(quote! { #obj_tokens.reduce(#(#arg_tokens),*) })
                } else {
                    Ok(quote! { #obj_tokens.into_iter().reduce(#(#arg_tokens),*) })
                }
            }
            _ => unreachable!("Non-iterator method passed to transpile_iterator_methods"),
        }
    }

    /// Handle HashMap/HashSet methods: `contains_key`, items, etc.
    ///
    /// TRANSPILER-002 FIX: Removed "get" case - was causing .`cloned()` on all `get()` methods
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"dict.items()"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("iter"));
    /// ```
    /// Complexity: 4 (within Toyota Way limits)
    pub fn transpile_map_set_methods(
        &self,
        obj_tokens: &TokenStream,
        method_ident: &proc_macro2::Ident,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "contains_key" | "keys" | "values" | "entry" | "contains" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            "items" => {
                // HashMap.items() -> iterator of (K, V) tuples (not references)
                Ok(quote! { #obj_tokens.iter().map(|(k, v)| (k.clone(), v.clone())) })
            }
            "update" => {
                // Python dict.update(other) -> Rust HashMap.extend(other)
                Ok(quote! { #obj_tokens.extend(#(#arg_tokens),*) })
            }
            // TRANSPILER-007: "add" removed - was causing user-defined add() to become insert()
            // For HashSet.add(), we need proper type inference instead of hardcoded renaming
            _ => unreachable!(
                "Non-map/set method {} passed to transpile_map_set_methods",
                method
            ),
        }
    }

    /// Handle `HashSet` set operations: union, intersection, difference, `symmetric_difference`
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("a.union(b)");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("union"));
    /// ```
    /// Complexity: 3 (within Toyota Way limits)
    pub fn transpile_set_operations(
        &self,
        obj_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        Self::require_exact_args(method, arg_tokens, 1)?;
        let other = &arg_tokens[0];
        let method_ident = format_ident!("{}", method);
        Ok(quote! {
            {
                use std::collections::HashSet;
                #obj_tokens.#method_ident(&#other).cloned().collect::<HashSet<_>>()
            }
        })
    }

    /// Handle string methods: Python-style and Rust-style
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#""hello".to_upper()"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("to_uppercase"));
    /// ```
    /// Complexity: 8 (within Toyota Way limits)
    pub fn transpile_string_methods(
        &self,
        obj_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "to_s" | "to_string" => {
                // DEFECT-003 FIX: Always emit .to_string() method call
                // This converts any value to String (integers, floats, etc.)
                Ok(quote! { #obj_tokens.to_string() })
            }
            "to_upper" | "upper" => {
                let rust_method = format_ident!("to_uppercase");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "to_lower" | "lower" => {
                let rust_method = format_ident!("to_lowercase");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "strip" => Ok(quote! { #obj_tokens.trim().to_string() }),
            "lstrip" => Ok(quote! { #obj_tokens.trim_start() }),
            "rstrip" => Ok(quote! { #obj_tokens.trim_end() }),
            "startswith" => Ok(quote! { #obj_tokens.starts_with(#(#arg_tokens),*) }),
            "endswith" => Ok(quote! { #obj_tokens.ends_with(#(#arg_tokens),*) }),
            "split" => {
                // DEFECT-002 FIX: Convert iterator to Vec<String>
                // .split() returns std::str::Split iterator, but Ruchy expects Vec<String>
                Ok(
                    quote! { #obj_tokens.split(#(#arg_tokens),*).map(|s| s.to_string()).collect::<Vec<String>>() },
                )
            }
            "replace" => Ok(quote! { #obj_tokens.replace(#(#arg_tokens),*) }),
            "length" => {
                // Map Ruchy's length() to Rust's len()
                let rust_method = format_ident!("len");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "substring" => {
                // string.substring(start, end) -> string.chars().skip(start).take(end-start).collect()
                Self::require_exact_args("substring", arg_tokens, 2)?;
                let start = &arg_tokens[0];
                let end = &arg_tokens[1];
                Ok(quote! {
                    #obj_tokens.chars()
                        .skip(#start as usize)
                        .take((#end as usize).saturating_sub(#start as usize))
                        .collect::<String>()
                })
            }
            _ => unreachable!(
                "Non-string method {} passed to transpile_string_methods",
                method
            ),
        }
    }

    /// Handle advanced collection methods: slice, concat, flatten, unique, join
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("[1, 2, 3].slice(0, 2)");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("to_vec"));
    /// ```
    /// Complexity: 6 (within Toyota Way limits)
    pub fn transpile_advanced_collection_methods(
        &self,
        obj_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "slice" => {
                // vec.slice(start, end) -> vec[start..end].to_vec()
                Self::require_exact_args("slice", arg_tokens, 2)?;
                let start = &arg_tokens[0];
                let end = &arg_tokens[1];
                Ok(quote! { #obj_tokens[#start as usize..#end as usize].to_vec() })
            }
            "concat" => {
                // vec.concat(other) -> [vec, other].concat()
                Self::require_exact_args("concat", arg_tokens, 1)?;
                let other = &arg_tokens[0];
                Ok(quote! { [#obj_tokens, #other].concat() })
            }
            "flatten" => {
                // vec.flatten() -> vec.into_iter().flatten().collect()
                Self::require_no_args("flatten", arg_tokens)?;
                Ok(quote! { #obj_tokens.into_iter().flatten().collect::<Vec<_>>() })
            }
            "unique" => {
                // vec.unique() -> vec.into_iter().collect::<HashSet<_>>().into_iter().collect()
                Self::require_no_args("unique", arg_tokens)?;
                Ok(quote! {
                    {
                        use std::collections::HashSet;
                        #obj_tokens.into_iter().collect::<HashSet<_>>().into_iter().collect::<Vec<_>>()
                    }
                })
            }
            "join" => {
                // vec.join(separator) -> vec.join(separator) (for Vec<String>)
                Self::require_exact_args("join", arg_tokens, 1)?;
                let separator = &arg_tokens[0];
                Ok(quote! { #obj_tokens.join(&#separator) })
            }
            _ => unreachable!(
                "Non-advanced-collection method passed to transpile_advanced_collection_methods"
            ),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // transpile_iterator_methods tests
    // ========================================================================

    #[test]
    fn test_iterator_map() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec };
        let arg_tokens = vec![quote! { |x| x * 2 }];
        let result = transpiler.transpile_iterator_methods(&obj_tokens, "map", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("iter"));
        assert!(tokens_str.contains("map"));
        assert!(tokens_str.contains("collect"));
    }

    #[test]
    fn test_iterator_map_already_iter() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec . iter () };
        let arg_tokens = vec![quote! { |x| x * 2 }];
        let result = transpiler.transpile_iterator_methods(&obj_tokens, "map", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        // Should not add another .iter()
        assert!(tokens_str.contains("map"));
    }

    #[test]
    fn test_iterator_filter() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec };
        let arg_tokens = vec![quote! { |x| x > 0 }];
        let result = transpiler.transpile_iterator_methods(&obj_tokens, "filter", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("into_iter"));
        assert!(tokens_str.contains("filter"));
    }

    #[test]
    fn test_iterator_reduce() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec };
        let arg_tokens = vec![quote! { |a, b| a + b }];
        let result = transpiler.transpile_iterator_methods(&obj_tokens, "reduce", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("into_iter"));
        assert!(tokens_str.contains("reduce"));
    }

    // ========================================================================
    // transpile_map_set_methods tests
    // ========================================================================

    #[test]
    fn test_map_contains_key() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { map };
        let method_ident = format_ident!("contains_key");
        let arg_tokens = vec![quote! { "key" }];
        let result =
            transpiler.transpile_map_set_methods(&obj_tokens, &method_ident, "contains_key", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("contains_key"));
    }

    #[test]
    fn test_map_items() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { map };
        let method_ident = format_ident!("items");
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_map_set_methods(&obj_tokens, &method_ident, "items", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("iter"));
        assert!(tokens_str.contains("clone"));
    }

    #[test]
    fn test_map_update() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { map };
        let method_ident = format_ident!("update");
        let arg_tokens = vec![quote! { other }];
        let result = transpiler.transpile_map_set_methods(&obj_tokens, &method_ident, "update", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("extend"));
    }

    #[test]
    fn test_map_keys() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { map };
        let method_ident = format_ident!("keys");
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_map_set_methods(&obj_tokens, &method_ident, "keys", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("keys"));
    }

    #[test]
    fn test_map_values() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { map };
        let method_ident = format_ident!("values");
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_map_set_methods(&obj_tokens, &method_ident, "values", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("values"));
    }

    // ========================================================================
    // transpile_set_operations tests
    // ========================================================================

    #[test]
    fn test_set_union() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { set_a };
        let arg_tokens = vec![quote! { set_b }];
        let result = transpiler.transpile_set_operations(&obj_tokens, "union", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("union"));
        assert!(tokens_str.contains("HashSet"));
    }

    #[test]
    fn test_set_intersection() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { set_a };
        let arg_tokens = vec![quote! { set_b }];
        let result = transpiler.transpile_set_operations(&obj_tokens, "intersection", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("intersection"));
    }

    #[test]
    fn test_set_difference() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { set_a };
        let arg_tokens = vec![quote! { set_b }];
        let result = transpiler.transpile_set_operations(&obj_tokens, "difference", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("difference"));
    }

    // ========================================================================
    // transpile_string_methods tests
    // ========================================================================

    #[test]
    fn test_string_to_s() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { value };
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_string_methods(&obj_tokens, "to_s", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("to_string"));
    }

    #[test]
    fn test_string_to_upper() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_string_methods(&obj_tokens, "to_upper", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("to_uppercase"));
    }

    #[test]
    fn test_string_to_lower() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_string_methods(&obj_tokens, "to_lower", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("to_lowercase"));
    }

    #[test]
    fn test_string_strip() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_string_methods(&obj_tokens, "strip", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("trim"));
    }

    #[test]
    fn test_string_split() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens = vec![quote! { "," }];
        let result = transpiler.transpile_string_methods(&obj_tokens, "split", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("split"));
        assert!(tokens_str.contains("collect"));
    }

    #[test]
    fn test_string_startswith() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens = vec![quote! { "prefix" }];
        let result = transpiler.transpile_string_methods(&obj_tokens, "startswith", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("starts_with"));
    }

    #[test]
    fn test_string_endswith() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens = vec![quote! { "suffix" }];
        let result = transpiler.transpile_string_methods(&obj_tokens, "endswith", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("ends_with"));
    }

    #[test]
    fn test_string_replace() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens = vec![quote! { "old" }, quote! { "new" }];
        let result = transpiler.transpile_string_methods(&obj_tokens, "replace", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("replace"));
    }

    #[test]
    fn test_string_length() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_string_methods(&obj_tokens, "length", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("len"));
    }

    #[test]
    fn test_string_substring() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { s };
        let arg_tokens = vec![quote! { 0 }, quote! { 5 }];
        let result = transpiler.transpile_string_methods(&obj_tokens, "substring", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("chars"));
        assert!(tokens_str.contains("skip"));
        assert!(tokens_str.contains("take"));
    }

    // ========================================================================
    // transpile_advanced_collection_methods tests
    // ========================================================================

    #[test]
    fn test_collection_slice() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec };
        let arg_tokens = vec![quote! { 0 }, quote! { 2 }];
        let result = transpiler.transpile_advanced_collection_methods(&obj_tokens, "slice", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("to_vec"));
    }

    #[test]
    fn test_collection_concat() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec1 };
        let arg_tokens = vec![quote! { vec2 }];
        let result = transpiler.transpile_advanced_collection_methods(&obj_tokens, "concat", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("concat"));
    }

    #[test]
    fn test_collection_flatten() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { nested };
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_advanced_collection_methods(&obj_tokens, "flatten", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("flatten"));
    }

    #[test]
    fn test_collection_unique() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec };
        let arg_tokens: Vec<TokenStream> = vec![];
        let result = transpiler.transpile_advanced_collection_methods(&obj_tokens, "unique", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("HashSet"));
    }

    #[test]
    fn test_collection_join() {
        let transpiler = Transpiler::new();
        let obj_tokens = quote! { vec };
        let arg_tokens = vec![quote! { ", " }];
        let result = transpiler.transpile_advanced_collection_methods(&obj_tokens, "join", &arg_tokens);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("join"));
    }
}
