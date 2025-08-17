# Publishing to crates.io

## Prerequisites

1. Create a crates.io account at https://crates.io
2. Generate an API token:
   - Go to https://crates.io/settings/tokens
   - Click "New Token"
   - Name it "ruchy-publish"
   - Copy the token

## Publishing Steps

### 1. Login to crates.io
```bash
cargo login <YOUR_CRATES_IO_TOKEN>
```

### 2. Publish the ruchy library crate first
```bash
cargo publish --package ruchy
```

Wait for the crate to be indexed (1-2 minutes), then verify at:
https://crates.io/crates/ruchy

### 3. Publish the ruchy-cli crate
```bash
cargo publish --package ruchy-cli
```

Verify at:
https://crates.io/crates/ruchy-cli

## Installation Test

After publishing, test installation:

```bash
cargo install ruchy-cli
ruchy --version
```

## GitHub Secrets Setup (for automated releases)

To enable automatic crates.io publishing in GitHub Actions:

1. Go to https://github.com/paiml/ruchy/settings/secrets/actions
2. Click "New repository secret"
3. Name: `CRATES_TOKEN`
4. Value: Your crates.io API token
5. Click "Add secret"

## Binary Installation

Pre-built binaries are available from the GitHub release:
https://github.com/paiml/ruchy/releases/latest

### Linux/macOS:
```bash
# Download for your platform
curl -LO https://github.com/paiml/ruchy/releases/download/v0.1.0/ruchy-linux-amd64
# or
curl -LO https://github.com/paiml/ruchy/releases/download/v0.1.0/ruchy-darwin-amd64

# Make executable
chmod +x ruchy-*

# Move to PATH
sudo mv ruchy-* /usr/local/bin/ruchy
```

### Windows:
Download `ruchy-windows-amd64.exe` from the releases page.

## Verification

After installation, verify:
```bash
ruchy --version
ruchy --help
```

Start the REPL:
```bash
ruchy
```