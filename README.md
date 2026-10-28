# WFHub native migration

A native iOS migration of the Expo WFHub application. The workspace follows Kintsu's split between a shared Rust core, a C bridge, and a SwiftUI application. See [the migration plan](docs/migration.md) for current coverage and remaining work.

## Run

Install Xcode, XcodeGen and Rust, then install the iOS Rust targets:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cd apps/wfhub-mobile
xcodegen generate
open WFHub.xcodeproj
```

Select a signing team for a physical device. The Xcode build script compiles the Rust static library. Run Rust checks with `cargo test --workspace` and `cargo clippy --workspace --all-targets`.
