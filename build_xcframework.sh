#!/bin/bash

# Script to build XCFramework for iOS

set -e

echo "🏗️  Building XCFramework for dialog_client..."

# Clean previous builds
rm -rf target/universal
rm -rf target/aarch64-apple-ios
rm -rf target/aarch64-apple-ios-sim
rm -rf target/x86_64-apple-ios
rm -rf dialog_ios/DialogClientFFI.xcframework

# Add iOS targets if not already added
echo "📱 Adding iOS targets..."
rustup target add aarch64-apple-ios || true
rustup target add aarch64-apple-ios-sim || true
rustup target add x86_64-apple-ios || true

# Build for iOS device (arm64)
echo "🔨 Building for iOS device (arm64)..."
cargo build --package dialog_client --target aarch64-apple-ios --release

# Build for iOS simulator (arm64)
echo "🔨 Building for iOS simulator (arm64)..."
cargo build --package dialog_client --target aarch64-apple-ios-sim --release

# Build for iOS simulator (x86_64)
echo "🔨 Building for iOS simulator (x86_64)..."
cargo build --package dialog_client --target x86_64-apple-ios --release

# Create universal library for simulator
echo "🔗 Creating universal library for simulator..."
mkdir -p target/universal/release
lipo -create \
    target/aarch64-apple-ios-sim/release/libdialog_client.a \
    target/x86_64-apple-ios/release/libdialog_client.a \
    -output target/universal/release/libdialog_client.a

# Generate Swift bindings header
echo "📝 Generating module map and headers..."
cd dialog_client
cargo run --features=cli --bin uniffi-bindgen generate \
    src/dialog_client.udl \
    --lib-file ../target/aarch64-apple-ios/release/libdialog_client.a \
    --language swift \
    --out-dir ../target/swift-bindings
cd ..

# Create module.modulemap
mkdir -p target/DialogClientFFI.xcframework
cat > target/module.modulemap << 'EOF'
module DialogClientFFI {
    header "dialogclientFFI.h"
    export *
}
EOF

# Create XCFramework
echo "📦 Creating XCFramework..."
xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libdialog_client.a \
    -headers target/swift-bindings \
    -library target/universal/release/libdialog_client.a \
    -headers target/swift-bindings \
    -output dialog_ios/DialogClientFFI.xcframework

# Copy Swift files
echo "📄 Copying Swift files..."
cp target/swift-bindings/*.swift dialog_ios/Sources/DialogClient/

echo "✅ XCFramework built successfully!"
echo "📁 Output: dialog_ios/DialogClientFFI.xcframework"
echo ""
echo "🎉 Ready to use in your iOS app!"
echo "1. Add dialog_ios as a local Swift Package in Xcode"
echo "2. Import DialogClient in your Swift files"