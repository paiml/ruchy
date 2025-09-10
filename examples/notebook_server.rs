//! Example: Run Ruchy Notebook Server
//! 
//! This example demonstrates how to programmatically start a Ruchy notebook server
//! for interactive data science workflows.
//! 
//! Run with: cargo run --example notebook_server

#[cfg(feature = "notebook")]
use ruchy_notebook::server::start_server;

#[cfg(not(feature = "notebook"))]
fn main() {
    eprintln!("This example requires the 'notebook' feature.");
    eprintln!("Run with: cargo run --example notebook_server --features notebook");
}

#[cfg(feature = "notebook")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ruchy Notebook Server Example\n");
    println!("📊 Starting interactive notebook server for data science...");
    println!("🌐 Server will be available at: http://127.0.0.1:8888\n");
    
    // Configuration options
    let port = 8888;
    
    println!("Features:");
    println!("  ✓ Interactive code execution");
    println!("  ✓ Data visualization support");
    println!("  ✓ DataFrame operations");
    println!("  ✓ WebAssembly compilation");
    println!("  ✓ Real-time collaboration ready\n");
    
    println!("📝 Instructions:");
    println!("  1. Open your browser to http://127.0.0.1:{}", port);
    println!("  2. Create a new notebook or open existing .ruchy files");
    println!("  3. Use Shift+Enter to execute cells");
    println!("  4. Press Ctrl+C in terminal to stop the server\n");
    
    // Start the server
    println!("🔥 Server starting on port {}...", port);
    start_server(port).await?;
    
    Ok(())
}