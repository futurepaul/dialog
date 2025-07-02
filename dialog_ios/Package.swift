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
