# Binary Build Story Specification

## Overview

This document defines the binary release strategy for Ruchy, focusing on reliable Linux distribution with a clear path to multi-platform support.

## Goals

1. **Immediate**: Linux x86_64 static binaries that work on any Linux distribution
2. **Near-term**: Linux ARM64 support for cloud/embedded deployments  
3. **Future**: macOS and Windows native binaries

## Design Principles

1. **Zero Dependencies**: Static binaries that work everywhere
2. **Small Size**: Optimize for minimal download size (<10MB)
3. **Security First**: Checksums and signatures on all releases
4. **Simple Installation**: One-line install script
5. **Reproducible Builds**: Deterministic compilation

## Implementation Strategy

### Phase 1: Linux x86_64 (RUCHY-0718)

#### Build Configuration

```toml
# Cargo.toml profile for optimized release builds
[profile.release-dist]
inherits = "release"
strip = true           # Remove debug symbols
lto = "fat"           # Link-time optimization
codegen-units = 1     # Single codegen unit for better optimization
panic = "abort"       # Smaller panic handler
opt-level = "z"       # Optimize for size
```

#### GitHub Actions Workflow

```yaml
name: Binary Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-linux:
    runs-on: ubuntu-22.04
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-musl    # Static binary
          - aarch64-unknown-linux-musl   # ARM64 static
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install musl tools
        run: |
          sudo apt-get update
          sudo apt-get install -y musl-tools
          
      - name: Install cross-compilation tools
        if: matrix.target == 'aarch64-unknown-linux-musl'
        run: |
          cargo install cross --locked
          
      - name: Build optimized binary
        run: |
          if [[ "${{ matrix.target }}" == "aarch64-unknown-linux-musl" ]]; then
            cross build --release --target ${{ matrix.target }} --profile release-dist
          else
            cargo build --release --target ${{ matrix.target }} --profile release-dist
          fi
          
      - name: Package binary
        run: |
          cd target/${{ matrix.target }}/release-dist
          tar czf ruchy-${{ github.ref_name }}-${{ matrix.target }}.tar.gz ruchy
          mv *.tar.gz ../../../
          
      - name: Generate checksums
        run: |
          sha256sum ruchy-*.tar.gz > SHA256SUMS
          
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ruchy-${{ matrix.target }}
          path: |
            ruchy-*.tar.gz
            SHA256SUMS
```

#### Installation Script

```bash
#!/usr/bin/env bash
# install.sh - Install Ruchy binary

set -euo pipefail

REPO="paiml/ruchy"
INSTALL_DIR="${HOME}/.local/bin"

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        TARGET="x86_64-unknown-linux-musl"
        ;;
    aarch64|arm64)
        TARGET="aarch64-unknown-linux-musl"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Get latest version
VERSION=$(curl -s https://api.github.com/repos/${REPO}/releases/latest | grep '"tag_name"' | cut -d'"' -f4)

# Download binary
echo "Downloading Ruchy ${VERSION} for ${TARGET}..."
curl -L -o ruchy.tar.gz \
    "https://github.com/${REPO}/releases/download/${VERSION}/ruchy-${VERSION}-${TARGET}.tar.gz"

# Verify checksum
curl -L -o SHA256SUMS \
    "https://github.com/${REPO}/releases/download/${VERSION}/SHA256SUMS"
sha256sum -c SHA256SUMS --ignore-missing

# Extract and install
tar xzf ruchy.tar.gz
mkdir -p "$INSTALL_DIR"
mv ruchy "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/ruchy"

# Clean up
rm -f ruchy.tar.gz SHA256SUMS

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Add $INSTALL_DIR to your PATH:"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
fi

echo "Ruchy ${VERSION} installed successfully!"
```

### Phase 2: Multi-Platform Support (RUCHY-0719)

#### Cargo-dist Integration

```toml
# Cargo.toml
[workspace.metadata.dist]
cargo-dist-version = "0.28.0"
ci = ["github"]
installers = ["shell", "powershell"]
targets = [
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

### Phase 3: Package Managers (RUCHY-0720)

#### Homebrew Formula
```ruby
class Ruchy < Formula
  desc "Ruchy programming language"
  homepage "https://github.com/paiml/ruchy"
  version "0.9.10"
  
  if OS.mac?
    url "https://github.com/paiml/ruchy/releases/download/v#{version}/ruchy-v#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "..."
  elsif OS.linux?
    url "https://github.com/paiml/ruchy/releases/download/v#{version}/ruchy-v#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "..."
  end
  
  def install
    bin.install "ruchy"
  end
end
```

## Binary Size Optimization Techniques

1. **Strip Symbols**: `-C strip=symbols`
2. **LTO**: `-C lto=fat`
3. **Single Codegen Unit**: `-C codegen-units=1`
4. **Abort on Panic**: `-C panic=abort`
5. **Size Optimization**: `-C opt-level=z`
6. **Remove Unused Dependencies**: Regular `cargo tree` audits
7. **Feature Flags**: Conditional compilation for optional features

## Security Considerations

1. **Checksums**: SHA256 for all artifacts
2. **Signatures**: GPG signing for releases (future)
3. **SBOM**: Generate Software Bill of Materials
4. **Reproducible Builds**: Document exact build environment
5. **Supply Chain**: Pin all dependencies with lock files

## Testing Strategy

1. **Smoke Tests**: Basic functionality on each platform
2. **Compatibility Tests**: Test on multiple Linux distributions
3. **Performance Tests**: Ensure optimizations don't break functionality
4. **Size Monitoring**: Track binary size over time

## Success Metrics

1. **Binary Size**: < 10MB compressed
2. **Platform Coverage**: Linux x86_64 + ARM64
3. **Installation Success Rate**: > 99%
4. **Download Speed**: < 5 seconds on average connection
5. **Compatibility**: Works on Linux kernels 3.2+

## Release Checklist

- [ ] Version bump in Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Build binaries locally
- [ ] Test installation script
- [ ] Create git tag
- [ ] Push tag to trigger CI
- [ ] Verify GitHub release
- [ ] Test download and installation
- [ ] Update documentation
- [ ] Announce release

## Future Enhancements

1. **Delta Updates**: Only download changed portions
2. **Self-Update**: Built-in update command
3. **Telemetry**: Optional usage statistics
4. **Plugins**: Dynamic loading of extensions
5. **Container Images**: Docker/OCI images

## References

- [Cargo Dist Documentation](https://github.com/axodotdev/cargo-dist)
- [Rust Binary Size Optimization](https://github.com/johnthagen/min-sized-rust)
- [Linux Static Linking Guide](https://doc.rust-lang.org/rustc/platform-support.html)