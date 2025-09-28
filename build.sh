#!/bin/bash
set -euo pipefail

FEATURES="${FEATURES:-}"
echo "Features: $FEATURES"

PKG="$1"
DID_PATH="$2"

if [ -z "$PKG" ] || [ -z "$DID_PATH" ]; then
    echo "Usage: $0 <canister_name> <path_to_did_file>"
    exit 1
fi

if [ ! -f "$DID_PATH" ]; then
    echo "Error: DID file not found at $DID_PATH"
    exit 1
fi

if [ -n "${FEATURES}" ]; then
    cargo build --target wasm32-unknown-unknown --release --package "$PKG" --features "$FEATURES" --locked
else
    cargo build --target wasm32-unknown-unknown --release --package "$PKG" --locked
fi

WASM_FILE=target/wasm32-unknown-unknown/release/$PKG.wasm
ic-wasm $WASM_FILE -o $WASM_FILE shrink
ic-wasm $WASM_FILE -o $WASM_FILE metadata candid:service -f $DID_PATH -v public
gzip -nf9v $WASM_FILE
