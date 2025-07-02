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
            dependencies: ["DialogClientFFI"],
            linkerSettings: [
                .linkedLibrary("dialog_client"),
                .unsafeFlags(["-L", "./Sources/DialogClient"])
            ]
        ),
        .systemLibrary(
            name: "DialogClientFFI",
            path: "./Sources/DialogClient"
        ),
        .testTarget(
            name: "DialogClientTests",
            dependencies: ["DialogClient"]
        ),
    ]
)
