# MCP Registry Publishing Guide

This document describes how to publish the Ruchy MCP Server to the Model Context Protocol Registry.

## Automated Publishing (Recommended)

The repository is configured with GitHub Actions for automated publishing. Publishing happens automatically when:

1. **Version Tags**: Push a version tag (e.g., `v3.67.0`)
   ```bash
   git tag v3.67.0
   git push origin v3.67.0
   ```

2. **Manual Workflow Dispatch**: Trigger the workflow manually from GitHub Actions UI

### Prerequisites

Ensure the following GitHub secrets are configured:

- `CARGO_TOKEN`: Token for publishing to crates.io (get from https://crates.io/me)

### Workflow Steps

The automated workflow performs:

1. **Build & Test**: Builds the project with MCP features enabled
2. **Publish to crates.io**: Publishes the Rust crate (optional, may already exist)
3. **Publish to MCP Registry**: Uses GitHub OIDC authentication to publish

## Manual Publishing

If you need to publish manually:

### Prerequisites

1. Install the MCP Publisher CLI:
   ```bash
   curl -L "https://github.com/modelcontextprotocol/publisher/releases/latest/download/mcp-publisher-linux-x64.tar.gz" | tar xz
   chmod +x mcp-publisher
   ```

2. Ensure `server.json` is up to date with the correct version

### Steps

1. **Build and test locally**:
   ```bash
   cargo build --features mcp
   cargo test --features mcp
   ```

2. **Publish to crates.io** (if not already published):
   ```bash
   cargo publish --features mcp
   ```

3. **Login to MCP Registry**:
   ```bash
   # Using GitHub OIDC (requires GitHub CLI and proper permissions)
   ./mcp-publisher login github-oidc

   # OR using GitHub Personal Access Token
   ./mcp-publisher login github-pat --token YOUR_TOKEN
   ```

4. **Publish to MCP Registry**:
   ```bash
   ./mcp-publisher publish
   ```

5. **Verify publication**:
   ```bash
   curl "https://registry.modelcontextprotocol.io/v0/servers?search=ruchy-mcp" | jq '.'
   ```

## Configuration Files

### server.json

The `server.json` file describes the MCP server for the registry:

- **identifier**: `io.github.paiml.ruchy-mcp` (must be unique in registry)
- **deployment.cargo**: Specifies installation via `cargo install ruchy --features mcp`
- **capabilities.tools**: Lists all 7 available MCP tools

### Updating server.json

When adding new tools or changing capabilities:

1. Update `server.json` with new tool descriptions
2. Ensure version matches `Cargo.toml`
3. Validate with:
   ```bash
   node -e "console.log(JSON.parse(require('fs').readFileSync('server.json')))"
   ```

## MCP Tools Provided

The Ruchy MCP Server provides the following tools:

1. **ruchy-score**: Analyze code quality with unified 0.0-1.0 scoring system
2. **ruchy-lint**: Real-time code linting with auto-fix suggestions
3. **ruchy-format**: Format Ruchy source code with configurable style
4. **ruchy-analyze**: Comprehensive code analysis with AST, metrics, and insights
5. **ruchy-eval**: Evaluate Ruchy expressions with type safety
6. **ruchy-transpile**: Transpile Ruchy code to Rust
7. **ruchy-type-check**: Type check Ruchy expressions

## Troubleshooting

### Authentication Issues

If GitHub OIDC fails:
- Ensure repository has `id-token: write` permission in workflow
- Try using a GitHub Personal Access Token instead

### Publication Fails

If publication fails:
- Verify `server.json` is valid JSON
- Ensure identifier is unique and follows `io.github.username.project` format
- Check that deployment configuration is correct
- Verify crates.io package exists and is accessible

### Verification Fails

If the server doesn't appear in registry:
- Wait 1-2 minutes for registry to update
- Check registry directly: https://registry.modelcontextprotocol.io/
- Verify the identifier matches what was published

## Resources

- [MCP Publishing Guide](https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/publish-server.md)
- [GitHub Actions Guide](https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/github-actions.md)
- [MCP Registry](https://registry.modelcontextprotocol.io/)
