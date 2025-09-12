// Test if SharedSession compiles correctly
use std::sync::{Arc, Mutex};

mod shared_session {
    include!("src/wasm/shared_session.rs");
}

fn main() {
    let session = Arc::new(Mutex::new(shared_session::SharedSession::new()));
    let mut guard = session.lock().unwrap();
    // This should work if create_checkpoint exists
    let _ = guard.create_checkpoint("test");
    println!("SharedSession has create_checkpoint method!");
}