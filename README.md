# WFHub

WFHub is a Warframe companion for exploring world state, activities, Codex content, and recent market orders. The project includes a Rust desktop app, a SwiftUI iOS app, and a website with an API.

## Getting started

Install [Rust](https://rustup.rs/) and [Bun](https://bun.sh/), then run a client from the repository root:

```sh
cargo run -p wfhub       # Desktop app
bun run dev:web          # Website: http://localhost:8000
```

For iOS setup, prerequisites, and development checks, see the [development guide](docs/development.md). More information is available in the [documentation](docs/README.md).

The desktop and iOS apps are under active development and do not yet have full feature parity.

## License

The project is licensed under the [MIT License](LICENSE). Third-party assets and trademarks have separate rights; see the [asset notices](docs/assets.md).
