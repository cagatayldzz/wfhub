# WFHub

WFHub is a Warframe companion for browsing world state, activities, Codex content, and recent market orders. It includes a Rust/GPUI desktop app, a SwiftUI iOS app, and a website with an API.

## Quick start

```sh
cargo run -p wfhub    # Desktop app
bash scripts/dev-web.sh       # Website at http://localhost:8000
```

For prerequisites, iOS setup, and checks, see the [development guide](docs/development.md). The [documentation index](docs/README.md) covers architecture, integrations, and release notes.

The desktop and iOS clients are under active development and do not yet have full feature parity. There is no Android client.

## License

Project code is available under the [MIT License](LICENSE). Third-party assets and trademarks have separate rights; see [asset notices](docs/assets.md).
