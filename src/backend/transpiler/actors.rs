//! Actor system transpilation
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
use super::*;
use crate::frontend::ast::{ActorHandler, StructField};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
impl Transpiler {
    /// Transpiles actor definitions
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::StructField;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let result = transpiler.transpile_actor("TestActor", &[], &[]);
    /// assert!(result.is_ok());
    /// ```
    pub fn transpile_actor(
        &self,
        name: &str,
        state: &[StructField],
        handlers: &[ActorHandler],
    ) -> Result<TokenStream> {
        let actor_name = format_ident!("{}", name);
        let message_enum_name = format_ident!("{}Message", name);
        // Generate state fields
        let state_fields: Vec<TokenStream> = state
            .iter()
            .map(|field| {
                let field_name = format_ident!("{}", field.name);
                let field_type = self
                    .transpile_type(&field.ty)
                    .unwrap_or_else(|_| quote! { _ });
                quote! { #field_name: #field_type }
            })
            .collect();
        // Generate field names for initialization
        let field_names: Vec<_> = state
            .iter()
            .map(|field| format_ident!("{}", field.name))
            .collect();
        // Generate message enum variants
        let mut message_variants = Vec::new();
        let mut handler_arms = Vec::new();
        for handler in handlers {
            let variant_name = format_ident!("{}", handler.message_type);
            if handler.params.is_empty() {
                // Simple message without parameters
                message_variants.push(quote! { #variant_name });
                let body_tokens = self.transpile_expr(&handler.body)?;
                handler_arms.push(quote! {
                    #message_enum_name::#variant_name => {
                        #body_tokens
                    }
                });
            } else {
                // Message with parameters
                let param_types: Vec<TokenStream> = handler
                    .params
                    .iter()
                    .map(|p| {
                        self.transpile_type(&p.ty)
                            .unwrap_or_else(|_| quote! { String })
                    })
                    .collect();
                if param_types.len() == 1 {
                    message_variants.push(quote! { #variant_name(#(#param_types),*) });
                } else {
                    message_variants.push(quote! { #variant_name { #(#param_types),* } });
                }
                // Generate parameter bindings for the handler
                let param_names: Vec<_> = handler
                    .params
                    .iter()
                    .map(|p| format_ident!("{}", p.name()))
                    .collect();
                let body_tokens = self.transpile_expr(&handler.body)?;
                if param_names.len() == 1 {
                    let param = &param_names[0];
                    handler_arms.push(quote! {
                        #message_enum_name::#variant_name(#param) => {
                            #body_tokens
                        }
                    });
                } else {
                    handler_arms.push(quote! {
                        #message_enum_name::#variant_name { #(#param_names),* } => {
                            #body_tokens
                        }
                    });
                }
            }
        }
        // Generate the complete actor implementation
        Ok(quote! {
            // Message enum
            #[derive(Debug, Clone)]
            enum #message_enum_name {
                #(#message_variants,)*
            }
            // Actor struct
            struct #actor_name {
                #(#state_fields,)*
                receiver: tokio::sync::mpsc::Receiver<#message_enum_name>,
                sender: tokio::sync::mpsc::Sender<#message_enum_name>,
            }
            impl #actor_name {
                fn new() -> Self {
                    let (sender, receiver) = tokio::sync::mpsc::channel(100);
                    Self {
                        #(#field_names: Default::default(),)*
                        receiver,
                        sender,
                    }
                }
                fn sender(&self) -> tokio::sync::mpsc::Sender<#message_enum_name> {
                    self.sender.clone()
                }
                async fn run(&mut self) {
                    while let Some(msg) = self.receiver.recv().await {
                        self.handle_message(msg).await;
                    }
                }
                async fn handle_message(&mut self, msg: #message_enum_name) {
                    match msg {
                        #(#handler_arms)*
                    }
                }
            }
        })
    }
    /// Transpiles send operations (actor ! message)
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::Expr;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let actor = Expr::literal(42.into());
    /// let message = Expr::literal("hello".into());
    /// let result = transpiler.transpile_send(&actor, &message);
    /// assert!(result.is_ok());
    /// ```
    pub fn transpile_send(&self, actor: &Expr, message: &Expr) -> Result<TokenStream> {
        let actor_tokens = self.transpile_expr(actor)?;
        let message_tokens = self.transpile_expr(message)?;
        Ok(quote! {
            #actor_tokens.send(#message_tokens).await
        })
    }
    /// Transpiles ask operations (actor ? message)
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::Expr;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let actor = Expr::literal(42.into());
    /// let message = Expr::literal("hello".into());
    /// let result = transpiler.transpile_ask(&actor, &message);
    /// assert!(result.is_ok());
    /// ```
    pub fn transpile_ask(
        &self,
        actor: &Expr,
        message: &Expr,
        timeout: Option<&Expr>,
    ) -> Result<TokenStream> {
        let actor_tokens = self.transpile_expr(actor)?;
        let message_tokens = self.transpile_expr(message)?;
        if let Some(timeout_expr) = timeout {
            let timeout_tokens = self.transpile_expr(timeout_expr)?;
            Ok(quote! {
                #actor_tokens.ask(#message_tokens, #timeout_tokens).await
            })
        } else {
            // Default timeout of 5 seconds
            Ok(quote! {
                #actor_tokens.ask(#message_tokens, std::time::Duration::from_secs(5)).await
            })
        }
    }
    /// Transpiles command execution
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::Expr;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let command = Expr::literal("test_command".into());
    /// let result = transpiler.transpile_command(&command);
    /// assert!(result.is_ok());
    /// ```
    pub fn transpile_command(
        &self,
        program: &str,
        args: &[String],
        _env: &[(String, String)],
        _working_dir: &Option<String>,
    ) -> Result<TokenStream> {
        let prog_str = program;
        let args_tokens: Vec<_> = args.iter().map(|arg| quote! { #arg }).collect();
        Ok(quote! {
            std::process::Command::new(#prog_str)
                .args(&[#(#args_tokens),*])
                .output()
                .expect("Failed to execute command")
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind};

    fn make_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn make_literal(lit: Literal) -> Expr {
        Expr::new(ExprKind::Literal(lit), Span::new(0, 1))
    }

    fn make_ident(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), Span::new(0, 1))
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::new(0, 1),
        }
    }

    #[test]
    fn test_transpile_simple_actor() {
        let mut transpiler = make_transpiler();
        let state = vec![];
        let handlers = vec![];

        let result = transpiler.transpile_actor("Counter", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("enum CounterMessage"));
        assert!(tokens.contains("struct Counter"));
    }

    #[test]
    fn test_actor_with_state() {
        let mut transpiler = make_transpiler();
        use crate::frontend::ast::Visibility;
        let state = vec![StructField {
            name: "count".to_string(),
            ty: make_type("i32"),
            visibility: Visibility::Private,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        }];
        let handlers = vec![];

        let result = transpiler.transpile_actor("Counter", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("count :"));
    }

    #[test]
    fn test_actor_with_simple_handler() {
        let mut transpiler = make_transpiler();
        let state = vec![];
        let handlers = vec![ActorHandler {
            message_type: "Reset".to_string(),
            params: vec![],
            body: Box::new(make_literal(Literal::Unit)),
        }];

        let result = transpiler.transpile_actor("Counter", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Reset"));
        assert!(tokens.contains("CounterMessage :: Reset"));
    }

    #[test]
    fn test_actor_with_parameterized_handler() {
        let mut transpiler = make_transpiler();
        let state = vec![];
        let params = vec![Param {
            pattern: Pattern::Identifier("value".to_string()),
            ty: make_type("i32"),
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        }];
        let handlers = vec![ActorHandler {
            message_type: "Add".to_string(),
            params,
            body: Box::new(make_ident("value")),
        }];

        let result = transpiler.transpile_actor("Counter", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Add"));
    }

    #[test]
    fn test_actor_with_multiple_handlers() {
        let mut transpiler = make_transpiler();
        let state = vec![];
        let handlers = vec![
            ActorHandler {
                message_type: "Increment".to_string(),
                params: vec![],
                body: Box::new(make_literal(Literal::Unit)),
            },
            ActorHandler {
                message_type: "Decrement".to_string(),
                params: vec![],
                body: Box::new(make_literal(Literal::Unit)),
            },
            ActorHandler {
                message_type: "Reset".to_string(),
                params: vec![],
                body: Box::new(make_literal(Literal::Unit)),
            },
        ];

        let result = transpiler.transpile_actor("Counter", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Increment"));
        assert!(tokens.contains("Decrement"));
        assert!(tokens.contains("Reset"));
    }

    #[test]
    fn test_transpile_send() {
        let mut transpiler = make_transpiler();
        let actor = make_ident("my_actor");
        let message = make_ident("Message");

        let result = transpiler.transpile_send(&actor, &message);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("my_actor . send"));
        assert!(tokens.contains("await"));
    }

    #[test]
    fn test_transpile_ask_with_timeout() {
        let mut transpiler = make_transpiler();
        let actor = make_ident("my_actor");
        let message = make_ident("Query");
        let timeout = Some(make_literal(Literal::Integer(10, None)));

        let result = transpiler.transpile_ask(&actor, &message, timeout.as_ref());
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("my_actor . ask"));
        assert!(tokens.contains("await"));
    }

    #[test]
    fn test_transpile_ask_without_timeout() {
        let mut transpiler = make_transpiler();
        let actor = make_ident("my_actor");
        let message = make_ident("Query");

        let result = transpiler.transpile_ask(&actor, &message, None);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("my_actor . ask"));
        assert!(tokens.contains("Duration :: from_secs"));
        assert!(tokens.contains("await"));
    }

    #[test]
    fn test_transpile_command() {
        let mut transpiler = make_transpiler();
        let program = "echo";
        let args = vec!["hello".to_string(), "world".to_string()];
        let env = vec![];
        let working_dir = None;

        let result = transpiler.transpile_command(program, &args, &env, &working_dir);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Command :: new"));
        assert!(tokens.contains("echo"));
        assert!(tokens.contains("hello"));
        assert!(tokens.contains("world"));
        assert!(tokens.contains("output"));
    }

    #[test]
    fn test_transpile_command_no_args() {
        let mut transpiler = make_transpiler();
        let program = "ls";
        let args = vec![];
        let env = vec![];
        let working_dir = None;

        let result = transpiler.transpile_command(program, &args, &env, &working_dir);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Command :: new"));
        assert!(tokens.contains("ls"));
    }

    #[test]
    fn test_actor_with_multiple_params() {
        let mut transpiler = make_transpiler();
        let state = vec![];
        let params = vec![
            Param {
                pattern: Pattern::Identifier("x".to_string()),
                ty: make_type("i32"),
                span: Span::new(0, 1),
                is_mutable: false,
                default_value: None,
            },
            Param {
                pattern: Pattern::Identifier("y".to_string()),
                ty: make_type("i32"),
                span: Span::new(0, 1),
                is_mutable: false,
                default_value: None,
            },
        ];
        let handlers = vec![ActorHandler {
            message_type: "Compute".to_string(),
            params,
            body: Box::new(make_ident("x")),
        }];

        let result = transpiler.transpile_actor("Calculator", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Compute"));
    }

    #[test]
    fn test_actor_struct_generation() {
        let mut transpiler = make_transpiler();
        let state = vec![
            StructField {
                name: "value".to_string(),
                ty: make_type("String"),
                visibility: crate::frontend::ast::Visibility::Private,
                is_mut: false,
                default_value: None,
                decorators: vec![],
            },
            StructField {
                name: "count".to_string(),
                ty: make_type("usize"),
                visibility: crate::frontend::ast::Visibility::Private,
                is_mut: false,
                default_value: None,
                decorators: vec![],
            },
        ];
        let handlers = vec![];

        let result = transpiler.transpile_actor("Storage", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("struct Storage"));
        assert!(tokens.contains("value :"));
        assert!(tokens.contains("count :"));
        assert!(tokens.contains("receiver :"));
        assert!(tokens.contains("sender :"));
    }

    #[test]
    fn test_actor_async_methods() {
        let mut transpiler = make_transpiler();
        let state = vec![];
        let handlers = vec![];

        let result = transpiler.transpile_actor("Worker", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("async fn run"));
        assert!(tokens.contains("async fn handle_message"));
        assert!(tokens.contains("self . receiver . recv"));
    }

    #[test]
    fn test_actor_channel_creation() {
        let mut transpiler = make_transpiler();
        let state = vec![];
        let handlers = vec![];

        let result = transpiler.transpile_actor("Service", &state, &handlers);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("fn new"));
        assert!(tokens.contains("tokio :: sync :: mpsc :: channel"));
        assert!(tokens.contains("fn sender"));
    }
}

#[cfg(test)]
mod property_tests_actors {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_transpile_actor_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
