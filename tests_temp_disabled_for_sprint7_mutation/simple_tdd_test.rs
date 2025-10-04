//! Simple TDD test to prove the 404 issue

#[test]
fn test_route_in_source_passes() {
    // This should PASS - route exists in source
    let server_code =
        std::fs::read_to_string("ruchy-notebook/src/server/mod.rs").expect("Server file not found");

    assert!(server_code.contains("/api/execute"), "Route not in source");
    println!("✅ Route exists in source code");
}

#[test]
fn test_cli_imports_server_fails() {
    // This should FAIL - CLI doesn't import ruchy-notebook server
    let files_to_check = vec![
        "src/main.rs",
        "src/cli.rs",
        "src/lib.rs",
        "src/frontend/cli.rs",
    ];

    let mut found_import = false;
    for file in files_to_check {
        if let Ok(content) = std::fs::read_to_string(file) {
            if content.contains("ruchy_notebook::server") {
                found_import = true;
                println!("✅ Found import in {file}");
                break;
            }
        }
    }

    assert!(
        found_import,
        "❌ FAILING TEST: CLI doesn't import ruchy_notebook::server - this causes 404!"
    );
}
