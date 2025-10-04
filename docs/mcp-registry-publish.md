# MCP Registry Publishing Guide

This guide explains how to publish the Ruchy MCP server to the Model Context Protocol Registry.

## Overview

The Ruchy MCP server exposes Ruchy's code analysis, scoring, linting, formatting, and transpilation capabilities as MCP tools that can be used by Claude and other MCP clients.

**Current Status**:
- ✅ Implementation complete and tested locally
- ✅ Published to crates.io (v3.67.0 with `mcp` feature)
- ⏳ MCP Registry publication pending (requires test fixes)

## Automated Publishing Process

Publishing is automated via GitHub Actions. When you push a tag matching `v*`, the workflow will:

1. Build and test with `--features mcp`
2. Publish to crates.io (if not already published)
3. Publish to MCP Registry using GitHub OIDC authentication
4. Verify the publication

### Workflow File

`.github/workflows/publish-mcp.yml`

## Prerequisites

### One-Time Setup

1. **CARGO_TOKEN Secret**
   - Get your token from https://crates.io/me
   - Add to GitHub: Settings → Secrets → Actions → New repository secret
   - Name: `CARGO_TOKEN`
   - Value: Your crates.io API token

2. **GitHub OIDC Permissions**
   - Already configured in the workflow:
     ```yaml
     permissions:
       id-token: write
       contents: read
     ```

## Publishing Steps

### Automated (Recommended)

1. **Ensure tests pass locally**:
   ```bash
   cargo test --features mcp
   cargo build --features mcp --bin ruchy
   ./target/debug/ruchy mcp --verbose  # Verify it works
   ```

2. **Update version** in `Cargo.toml` and `server.json`:
   ```bash
   # Update both files to match, e.g., 3.68.0
   vim Cargo.toml server.json
   ```

3. **Commit and tag**:
   ```bash
   git add Cargo.toml server.json
   git commit -m "[MCP] Release v3.68.0"
   git push origin main

   git tag v3.68.0
   git push origin v3.68.0
   ```

4. **Monitor workflow**:
   ```bash
   gh run list --workflow=publish-mcp.yml --limit 1
   gh run watch <run-id>
   ```

### Manual Publishing

If automated publishing fails, you can publish manually:

1. **Install MCP Publisher CLI**:
   ```bash
   curl -L "https://github.com/modelcontextprotocol/publisher/releases/latest/download/mcp-publisher-linux-x64.tar.gz" | tar xz
   chmod +x mcp-publisher
   ```

2. **Publish to crates.io**:
   ```bash
   cargo publish --features mcp
   ```

3. **Login to MCP Registry**:
   ```bash
   # Using GitHub OIDC
   ./mcp-publisher login github-oidc

   # OR using Personal Access Token
   ./mcp-publisher login github-pat --token YOUR_TOKEN
   ```

4. **Publish to MCP Registry**:
   ```bash
   ./mcp-publisher publish
   ```

5. **Verify**:
   ```bash
   curl "https://registry.modelcontextprotocol.io/v0/servers?search=ruchy-mcp" | jq '.'
   ```

## Server Configuration

### server.json

The MCP server metadata is defined in `server.json`:

```json
{
  "$schema": "https://modelcontextprotocol.io/schemas/server.json",
  "identifier": "io.github.paiml.ruchy-mcp",
  "name": "Ruchy MCP Server",
  "version": "3.67.0",
  "description": "Model Context Protocol server for Ruchy - A systems scripting language...",
  "deployment": {
    "cargo": {
      "package": "ruchy",
      "features": ["mcp"],
      "command": "ruchy",
      "args": ["mcp"]
    }
  },
  "capabilities": {
    "tools": [
      {
        "name": "ruchy-score",
        "description": "Analyze code quality with unified 0.0-1.0 scoring system"
      },
      // ... 6 more tools
    ]
  }
}
```

**Important**: Keep `version` in `server.json` synchronized with `Cargo.toml`.

### Available Tools

The Ruchy MCP server provides 7 tools:

1. **ruchy-score**: Analyze code quality with unified scoring
2. **ruchy-lint**: Real-time code linting with auto-fix suggestions
3. **ruchy-format**: Format Ruchy source code
4. **ruchy-analyze**: Comprehensive code analysis with AST and metrics
5. **ruchy-eval**: Evaluate Ruchy expressions with type safety
6. **ruchy-transpile**: Transpile Ruchy code to Rust
7. **ruchy-type-check**: Type check Ruchy expressions

## Installation for Users

Once published, users can install the Ruchy MCP server:

### Install from crates.io

```bash
cargo install ruchy --features mcp
```

### Configure in Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "ruchy": {
      "command": "ruchy",
      "args": ["mcp"]
    }
  }
}
```

### Test Installation

```bash
ruchy mcp --verbose
```

Expected output:
```
🚀 Starting Ruchy MCP Server: ruchy-mcp
   Registered 7 tools:
   - ruchy-score: Analyze code quality...
   - ruchy-lint: Real-time code linting...
   ...
   Transport: stdio
✅ MCP server running
```

## Troubleshooting

### Build Failures

**Issue**: Tests fail with type mismatches (e.g., `Rc<String>` vs `Rc<str>`)

**Solution**: This is a known issue being tracked. Tests need to be updated to use:
- `Rc::from("string")` instead of `Rc::new("string".to_string())`
- `Rc::from(&vec![...])` instead of `Rc::new(vec![...])`

**Workaround**: Publish to crates.io manually without running full test suite if integration tests pass.

### Authentication Issues

**Issue**: GitHub OIDC fails

**Solution**:
- Ensure workflow has `id-token: write` permission
- Try using a GitHub Personal Access Token instead
- Verify repository settings allow GitHub Actions

### Publication Verification Fails

**Issue**: Server doesn't appear in registry

**Solution**:
- Wait 1-2 minutes for registry to update
- Check identifier is unique: `io.github.paiml.ruchy-mcp`
- Verify crates.io package exists and is accessible
- Check registry directly: https://registry.modelcontextprotocol.io/

### Version Mismatch

**Issue**: `server.json` and `Cargo.toml` versions don't match

**Solution**: Use validation script before publishing:
```bash
./scripts/validate-mcp-config.sh
```

## Version Management

Follow semantic versioning:

- **Major (X.0.0)**: Breaking changes to MCP tool interfaces
- **Minor (x.Y.0)**: New tools or capabilities added
- **Patch (x.y.Z)**: Bug fixes and improvements

Always update both `Cargo.toml` and `server.json` together.

## Dependency Requirements

The MCP feature requires:

```toml
[dependencies]
pmcp = { version = "1.3.0", features = ["full"], optional = true }
tokio = { version = "1.47", features = ["full"], optional = true }
tokio-util = { version = "0.7", features = ["rt"], optional = true }
async-trait = { version = "0.1", optional = true }
tower-lsp = { version = "0.20", optional = true }

[features]
mcp = ["tokio", "tokio-util", "pmcp", "async-trait", "tower-lsp"]
```

## Testing Before Publishing

### Local Testing

```bash
# Build with MCP features
cargo build --features mcp --bin ruchy

# Run tests
cargo test --features mcp

# Test MCP server startup
./target/debug/ruchy mcp --verbose

# Validate configuration
./scripts/validate-mcp-config.sh
```

### Integration Testing

Test the full installation flow in a clean environment:

```bash
# In a new directory
cargo install --path /path/to/ruchy --features mcp
ruchy mcp --verbose
```

## Schema and Registry Resources

- **MCP Registry**: https://registry.modelcontextprotocol.io/
- **Server Schema**: https://modelcontextprotocol.io/schemas/server.json
- **Publishing Guide**: https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/publish-server.md
- **GitHub Actions Guide**: https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/github-actions.md

## Current Implementation Status

### ✅ Completed

- MCP server command implementation (`src/bin/handlers/mod.rs`)
- All 7 tools registered and functional
- Stdio transport configured
- Feature-gated compilation
- Published to crates.io with `mcp` feature
- Automated GitHub Actions workflow
- Configuration files (`server.json`, validation script)

### ⏳ Pending

- Fix remaining test type mismatches
- Complete MCP Registry publication
- End-to-end verification in Claude Desktop

### Known Issues

1. **Test Type Mismatches**: Some tests use old `Rc<Vec<T>>` and `Rc<String>` patterns
   - Tracking: Multiple test files need updates
   - Impact: Blocks CI/CD workflow
   - Workaround: Manual crates.io publication works

2. **Registry Publication**: Workflow blocked by test failures
   - Status: Tests need fixing before automated publication completes
   - Workaround: Manual MCP publisher CLI can be used

## Next Steps for Maintainers

1. **Fix Test Types**: Update all tests to use correct `Rc<[T]>` and `Rc<str>` types
2. **Re-run Workflow**: Push new tag to trigger automated publication
3. **Verify Registry**: Confirm server appears in MCP registry search
4. **Update Docs**: Add MCP server section to main README
5. **Test in Claude**: Verify end-to-end functionality in Claude Desktop

## Support

For issues or questions:

- **GitHub Issues**: https://github.com/paiml/ruchy/issues
- **Discussions**: https://github.com/paiml/ruchy/discussions
- **MCP Registry**: https://registry.modelcontextprotocol.io/

---

Last Updated: 2025-10-04
Status: Implementation complete, awaiting test fixes for full automation
