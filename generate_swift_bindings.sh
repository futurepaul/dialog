#!/bin/bash

# Script to generate Swift bindings for dialog_client

set -e

echo "🔨 Building dialog_client library..."
cargo build --package dialog_client --release

echo "📝 Generating Swift bindings..."
cd dialog_client
cargo run --features=cli --bin uniffi-bindgen generate \
    src/dialog_client.udl \
    --lib-file ../target/release/libdialog_client.dylib \
    --language swift \
    --out-dir ../dialog_ios/Sources/DialogClient
cd ..

echo "✅ Swift bindings generated successfully!"
echo "📁 Output location: dialog_ios/Sources/DialogClient/"