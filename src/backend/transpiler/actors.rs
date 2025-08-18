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
                    .map(|p| format_ident!("{}", p.name))
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
                        #(#state_fields: Default::default(),)*
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
    pub fn transpile_send(&self, actor: &Expr, message: &Expr) -> Result<TokenStream> {
        let actor_tokens = self.transpile_expr(actor)?;
        let message_tokens = self.transpile_expr(message)?;

        Ok(quote! {
            #actor_tokens.send(#message_tokens).await
        })
    }

    /// Transpiles ask operations (actor ? message)
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
}
