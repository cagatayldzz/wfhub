# WFHub

WFHub is an application for browsing Warframe world state, activities, Codex content, and recent market orders. This repository contains a desktop client built with Rust and GPUI, an iOS client built with SwiftUI, and shared Rust code used by both clients.

## Project structure

```text
apps/
  wfhub/          GPUI desktop application and UI source
  wfhub-mobile/   SwiftUI iOS application and XcodeGen configuration
  wfhub-web/      Website, legal pages, and Codex API
crates/
  core/           Shared activity and Codex route catalogue
  ffi/            C interface used by the iOS client
docs/             Migration and App Store notes
```

The desktop UI lives in `apps/wfhub/src/ui`. The iOS app reads the route catalogue from a Rust static library through a C bridge and makes network requests in Swift. The desktop client makes network requests in Rust.

## Requirements

- Rust and Cargo; `rustup` is needed to install iOS targets.
- Xcode and its command-line tools to run the desktop application on macOS.
- Xcode, XcodeGen, and an iOS 17 or later device or simulator for the iOS application.
- Bun or npm to use the scripts in `package.json`.

## Desktop application

From the repository root:

```sh
cargo run -p wfhub
```

The application opens in a phone-proportioned window. You can also use `bun run dev:phone`.

## Website

The landing page and legal pages are static files in `apps/wfhub-web`. To preview them locally:

```sh
bun run dev:web
```

Open `http://localhost:8000` for the landing page and `/hub/` for the web companion. API requests use locally generated data when available, otherwise they use the deployed API. Run `bun run cf:build` to generate local data. The privacy and terms pages are drafts and require review before publication. The `cf:deploy` script deploys this directory to Cloudflare.

## Codex API

The website and API are deployed together with Cloudflare Workers. See the [API README](apps/wfhub-web/api/README.md) for configuration and dataset details.

## iOS application

On an Apple Silicon Mac, install the Rust targets for devices and simulators:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

For an Intel simulator, also run `rustup target add x86_64-apple-ios`. Then generate and open the Xcode project:

```sh
cd apps/wfhub-mobile
xcodegen generate
open WFHub.xcodeproj
```

In Xcode, select the `WFHub` scheme and a simulator or device, then run the app. Select a valid signing team for a physical device. The Xcode build script automatically builds the Rust library for the selected platform.

To build and launch the app on an iPhone 15 simulator from the repository root:

```sh
bun run mobile:simulator
```

Set the `WFHUB_SIMULATOR` environment variable to a device name or UDID to select a different simulator. See the [App Store notes](docs/app-store.md) for archiving and distribution. By default, `bun run mobile:archive` increments the build number before creating an archive.

## Checks

Run formatting, Clippy, and tests from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

You can also run all three checks with `bun run check`.

## Status

The desktop and iOS clients are under active development and do not yet have full screen or behavior parity. There is no Android client. See the [documentation index](docs/README.md) for architecture, development, integrations, and migration notes.
