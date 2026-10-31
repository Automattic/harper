// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "HarperMac",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "HarperMac",
            path: "Sources/HarperMac"
        )
    ]
)
