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

# Create the Swift Package structure if it doesn't exist
if [ ! -f "dialog_ios/Package.swift" ]; then
    echo "📦 Creating Swift Package structure..."
    mkdir -p dialog_ios/Sources/DialogClient
    mkdir -p dialog_ios/Tests/DialogClientTests
    
    cat > dialog_ios/Package.swift << 'EOF'
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "DialogClient",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "DialogClient",
            targets: ["DialogClient", "DialogClientFFI"]
        ),
    ],
    targets: [
        .target(
            name: "DialogClient",
            dependencies: ["DialogClientFFI"]
        ),
        .binaryTarget(
            name: "DialogClientFFI",
            path: "./DialogClientFFI.xcframework"
        ),
        .testTarget(
            name: "DialogClientTests",
            dependencies: ["DialogClient"]
        ),
    ]
)
EOF

    echo "✅ Swift Package structure created!"
fi

echo ""
echo "📱 Next steps:"
echo "1. Build the XCFramework: ./build_xcframework.sh"
echo "2. Add the Swift package to your iOS app in Xcode"
echo "3. Import DialogClient in your Swift code"