#!/usr/bin/env bash
# install.sh - Install Ruchy binary
# Usage: curl -sSL https://raw.githubusercontent.com/paiml/ruchy/main/install.sh | bash

set -euo pipefail

# Configuration
REPO="paiml/ruchy"
INSTALL_DIR="${RUCHY_INSTALL_DIR:-${HOME}/.local/bin}"
GITHUB_API="https://api.github.com"
GITHUB_REPO="https://github.com/${REPO}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check dependencies
check_deps() {
    local deps=("curl" "tar" "sha256sum")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            error "Required command '$dep' not found. Please install it first."
        fi
    done
}

# Detect system architecture
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "x86_64-unknown-linux-gnu"
            ;;
        aarch64|arm64)
            echo "aarch64-unknown-linux-gnu"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac
}

# Detect operating system
detect_os() {
    local os=$(uname -s)
    case "$os" in
        Linux)
            echo "linux"
            ;;
        Darwin)
            warn "macOS binaries not yet available. Please install using: cargo install ruchy"
            exit 0
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac
}

# Get latest version from GitHub
get_latest_version() {
    local version
    version=$(curl -s "${GITHUB_API}/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    
    if [[ -z "$version" ]]; then
        error "Failed to fetch latest version. Please check your internet connection."
    fi
    
    echo "$version"
}

# Download and install binary
install_ruchy() {
    local version="${1:-$(get_latest_version)}"
    local target=$(detect_arch)
    local os=$(detect_os)
    
    info "Installing Ruchy ${version} for ${target}..."
    
    # Create temp directory
    local temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT
    
    cd "$temp_dir"
    
    # Download binary archive
    local archive_name="ruchy-${version}-${target}.tar.gz"
    local download_url="${GITHUB_REPO}/releases/download/${version}/${archive_name}"
    
    info "Downloading from: ${download_url}"
    if ! curl -L -o "${archive_name}" "${download_url}"; then
        error "Failed to download binary. The release might not exist yet."
    fi
    
    # Download checksums
    local checksum_url="${GITHUB_REPO}/releases/download/${version}/SHA256SUMS"
    if curl -L -o "SHA256SUMS" "${checksum_url}" 2>/dev/null; then
        info "Verifying checksum..."
        if ! sha256sum -c SHA256SUMS --ignore-missing 2>/dev/null; then
            warn "Checksum verification failed. Proceeding anyway..."
        else
            info "Checksum verified successfully"
        fi
    else
        warn "No checksum file found. Skipping verification."
    fi
    
    # Extract binary
    info "Extracting binary..."
    tar xzf "${archive_name}"
    
    # Install binary
    mkdir -p "$INSTALL_DIR"
    mv ruchy "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/ruchy"
    
    info "Ruchy ${version} installed to ${INSTALL_DIR}/ruchy"
    
    # Verify installation
    if "$INSTALL_DIR/ruchy" --version &>/dev/null; then
        info "Installation verified successfully"
    else
        warn "Installation completed but verification failed"
    fi
    
    # Check PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "Add $INSTALL_DIR to your PATH to use ruchy:"
        echo ""
        echo "  For bash:"
        echo "    echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
        echo "    source ~/.bashrc"
        echo ""
        echo "  For zsh:"
        echo "    echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.zshrc"
        echo "    source ~/.zshrc"
        echo ""
        echo "  For fish:"
        echo "    fish_add_path $INSTALL_DIR"
        echo ""
    else
        info "ruchy is ready to use!"
    fi
}

# Main execution
main() {
    info "Ruchy Binary Installer"
    
    # Parse arguments
    local version=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                version="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [--version VERSION]"
                echo "  --version VERSION  Install specific version (e.g., v0.9.10)"
                echo "  --help            Show this help message"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done
    
    check_deps
    install_ruchy "$version"
    
    echo ""
    echo "Get started with:"
    echo "  ruchy --help"
    echo "  ruchy repl"
    echo ""
    echo "Documentation: https://github.com/paiml/ruchy"
}

# Run main function
main "$@"