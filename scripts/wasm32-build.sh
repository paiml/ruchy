#!/usr/bin/env bash
# PMAT-136 -- the wasm32 build gate, as a standalone declared gate.
#
# This is the exact command scripts/pre-release-gate.sh stage 2 runs (and the one
# the release workflow's Build WASM job runs): getrandom 0.3 needs its wasm_js
# backend selected both as a dependency feature and as a cfg flag, so RUSTFLAGS
# carries the cfg here the same way release.yml sets it (PMAT-129/130).
#
# The target's absence is a FAIL, never a silent skip: an unmeasured gate must
# say which of the two states it is in.
#
# CARGO_TARGET_DIR is honoured -- it is passed through to cargo untouched.
# Exit code is cargo's.

set -u -o pipefail

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
TARGET=wasm32-unknown-unknown

has_target() {
    rustup target list --installed 2> /dev/null | grep -qx "$TARGET"
}

if ! has_target; then
    echo "installing the $TARGET target ..."
    rustup target add "$TARGET" || echo "rustup target add $TARGET did not succeed"
fi

if ! has_target; then
    echo "FAIL $TARGET unavailable -- the wasm build cannot be measured"
    exit 1
fi

cd "$ROOT" || exit 2
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --lib \
    --target "$TARGET" --no-default-features --features wasm-compile
