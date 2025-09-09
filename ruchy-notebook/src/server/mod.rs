use anyhow::Result;
use axum::{
    Router,
    response::Html,
    routing::get,
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;

pub async fn start_server(port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .layer(CorsLayer::permissive());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 Notebook server running at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../assets/index.html"))
}

async fn health_handler() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_endpoint() {
        let response = health_handler().await;
        assert_eq!(response, "OK");
    }
}