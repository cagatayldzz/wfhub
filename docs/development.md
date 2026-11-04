# Development

## Prerequisites

Install Rust through `rustup`. Desktop development on macOS also needs Xcode and its command-line tools. iOS development requires Xcode, XcodeGen, and an iOS 17 or later simulator or device. Node.js is required for the web scripts; npm is required for Cloudflare deployment.

For an Apple Silicon iOS build:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

For an Intel simulator, add `x86_64-apple-ios` as well.

## Run the clients

From the repository root, start the desktop app:

```sh
cargo run -p wfhub
```

The app opens in a phone-proportioned window. You can also run `bash scripts/dev-phone.sh`.

To generate the iOS project and open it in Xcode:

```sh
cd apps/wfhub-mobile
xcodegen generate
open WFHub.xcodeproj
```

Select the `WFHub` scheme and a simulator or device. Physical devices require a valid signing team. The Xcode build invokes `scripts/build-rust.sh` to build the Rust static library.

Alternatively, from the repository root, `bash apps/wfhub-mobile/scripts/simulator.sh` generates the project, builds the app, and launches the default iPhone 15 simulator. Set `WFHUB_SIMULATOR` to another device name or UDID when needed. The simulator script requires a matching available simulator.

## Website and API

Run `bash scripts/dev-web.sh` from the repository root, then open `http://localhost:8000` for the landing page. API requests use locally generated data when available, otherwise they use the deployed API. Run `bash scripts/cf-build.sh` to generate local data; `bash scripts/cf-deploy.sh` builds and deploys the website and API to Cloudflare.

The privacy and terms pages are drafts and require review before publication. See the [API README](../apps/wfhub-web/api/README.md) for configuration and dataset details.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

`bash scripts/test.sh` runs these checks in sequence. The current Rust tests cover the shared route catalogue and platform validation; they do not replace UI and device testing. When changing UI behavior, exercise the affected screens and navigation in the relevant client.

## Generated files and release

`target/`, the iOS `build/` directory, and generated `.xcodeproj` files are ignored by Git. Edit `apps/wfhub-mobile/project.yml` and regenerate the project instead of editing the generated project as the source of truth.

`bash apps/wfhub-mobile/scripts/archive.sh` generates an iOS archive and increments `CURRENT_PROJECT_VERSION` in `project.yml` by default. Invoke the script directly with `--no-bump` to retain the current number. Archiving requires an appropriately configured signing team.

If Xcode reports a missing Rust target, install the target named in the error. If the simulator script cannot find iPhone 15, select an available simulator with `WFHUB_SIMULATOR`. Network-backed screens require access to the services listed in [Data and integrations](data-and-integrations.md).
