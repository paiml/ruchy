#!/bin/bash
# Deploy WASM binaries to interactive.paiml.com
#
# Usage:
#   ./scripts/deploy-wasm.sh [--build] [--deploy] [--all]
#
# Options:
#   --build   Build WASM package with wasm-pack
#   --deploy  Deploy to ../interactive.paiml.com/wasm/ruchy/
#   --all     Build and deploy (default)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEPLOY_TARGET="../interactive.paiml.com/wasm/ruchy"

# Parse arguments
DO_BUILD=false
DO_DEPLOY=false

if [ $# -eq 0 ]; then
    # Default: build and deploy
    DO_BUILD=true
    DO_DEPLOY=true
else
    for arg in "$@"; do
        case "$arg" in
            --build) DO_BUILD=true ;;
            --deploy) DO_DEPLOY=true ;;
            --all)
                DO_BUILD=true
                DO_DEPLOY=true
                ;;
            *)
                echo "Unknown option: $arg"
                echo "Usage: $0 [--build] [--deploy] [--all]"
                exit 1
                ;;
        esac
    done
fi

# Build WASM
if [ "$DO_BUILD" = true ]; then
    echo "🔨 Building WASM package with wasm-pack..."
    cd "$PROJECT_ROOT"

    wasm-pack build \
        --target web \
        --no-default-features \
        --features wasm-compile

    if [ ! -f "pkg/ruchy_bg.wasm" ]; then
        echo "❌ Build failed - no WASM output found"
        exit 1
    fi

    WASM_SIZE=$(du -h pkg/ruchy_bg.wasm | cut -f1)
    echo "✅ WASM built successfully: $WASM_SIZE"
fi

# Deploy WASM
if [ "$DO_DEPLOY" = true ]; then
    echo "🚀 Deploying WASM binaries..."

    # Check if deployment target exists
    if [ ! -d "$DEPLOY_TARGET" ]; then
        echo "❌ Deployment target not found: $DEPLOY_TARGET"
        echo "   Expected: $(cd "$PROJECT_ROOT" && readlink -f "$DEPLOY_TARGET")"
        exit 1
    fi

    # Copy WASM binaries
    cp -v pkg/ruchy_bg.wasm "$DEPLOY_TARGET/"
    cp -v pkg/ruchy.js "$DEPLOY_TARGET/"
    cp -v pkg/ruchy_bg.wasm.d.ts "$DEPLOY_TARGET/"

    echo "✅ WASM deployed to: $DEPLOY_TARGET"
    echo ""
    echo "📦 Deployed files:"
    ls -lh "$DEPLOY_TARGET"/{ruchy_bg.wasm,ruchy.js,ruchy_bg.wasm.d.ts}
fi

echo ""
echo "✅ WASM deployment complete!"
