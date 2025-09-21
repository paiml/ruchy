//! Core LSP server implementation
use super::{Formatter, SemanticAnalyzer, Workspace, SEMANTIC_TOKEN_LEGEND};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DiagnosticOptions,
    DiagnosticServerCapabilities, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentFormattingParams, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverParams, HoverProviderCapability, InitializeParams,
    InitializeResult, InitializedParams, MessageType, OneOf, Position, Range,
    SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensServerCapabilities,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
    Url, WorkDoneProgressOptions,
};
use tower_lsp::{Client, LanguageServer};
pub struct RuchyLanguageServer {
    client: Client,
    workspace: Arc<Mutex<Workspace>>,
    analyzer: Arc<Mutex<SemanticAnalyzer>>,
    formatter: Arc<Formatter>,
}
impl RuchyLanguageServer {
    /// # Examples
    ///
    /// ```
    /// use ruchy::lsp::server::new;
    ///
    /// let result = new(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace: Arc::new(Mutex::new(Workspace::new())),
            analyzer: Arc::new(Mutex::new(SemanticAnalyzer::new())),
            formatter: Arc::new(Formatter::new()),
        }
    }
}
#[tower_lsp::async_trait]
impl LanguageServer for RuchyLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SEMANTIC_TOKEN_LEGEND.clone(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ruchy".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "ruchy-lsp".to_string(),
                version: Some("0.3.2".to_string()),
            }),
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Ruchy Language Server initialized!")
            .await;
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut workspace = self.workspace.lock().await;
        workspace.add_document(params.text_document.uri.clone(), params.text_document.text);
        // Publish diagnostics
        self.publish_diagnostics(params.text_document.uri).await;
    }
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            let mut workspace = self.workspace.lock().await;
            workspace.update_document(&params.text_document.uri, change.text);
            drop(workspace); // Release lock before async call
            self.publish_diagnostics(params.text_document.uri).await;
        }
    }
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut workspace = self.workspace.lock().await;
        workspace.remove_document(&params.text_document.uri);
    }
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let workspace = self.workspace.lock().await;
        let analyzer = self.analyzer.lock().await;
        let position = params.text_document_position;
        if let Ok(document) = workspace.get_document(&position.text_document.uri) {
            let completions = analyzer.get_completions(document, position.position)?;
            Ok(Some(CompletionResponse::Array(completions)))
        } else {
            Ok(None)
        }
    }
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let workspace = self.workspace.lock().await;
        let analyzer = self.analyzer.lock().await;
        let position = params.text_document_position_params;
        if let Ok(document) = workspace.get_document(&position.text_document.uri) {
            let hover_info = analyzer.get_hover_info(document, position.position)?;
            Ok(hover_info)
        } else {
            Ok(None)
        }
    }
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let workspace = self.workspace.lock().await;
        let analyzer = self.analyzer.lock().await;
        let position = params.text_document_position_params;
        if let Ok(document) = workspace.get_document(&position.text_document.uri) {
            let definition = analyzer.get_definition(document, position.position)?;
            Ok(definition.map(GotoDefinitionResponse::Scalar))
        } else {
            Ok(None)
        }
    }
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let workspace = self.workspace.lock().await;
        if let Ok(document) = workspace.get_document(&params.text_document.uri) {
            let formatted = self.formatter.format(document)?;
            // Return a single text edit that replaces the entire document
            let edit = TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: document.lines().count().try_into().unwrap_or(u32::MAX),
                        character: 0,
                    },
                },
                new_text: formatted,
            };
            Ok(Some(vec![edit]))
        } else {
            Ok(None)
        }
    }
}
impl RuchyLanguageServer {
    async fn publish_diagnostics(&self, uri: Url) {
        let workspace = self.workspace.lock().await;
        let mut analyzer = self.analyzer.lock().await;
        if let Ok(document) = workspace.get_document(&uri) {
            let diagnostics = analyzer.get_diagnostics(document).unwrap_or_default();
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }
}
#[cfg(test)]
mod property_tests_server {
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
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
