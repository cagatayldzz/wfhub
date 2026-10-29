# WFHub API

A Rust service for the WFHub Codex JSON dataset. It serves the existing category and item paths used by the desktop and iOS clients.

```sh
cargo run -p wfhub-api
```

The default address is `127.0.0.1:8080`. Set `WFHUB_API_ADDR` to change the bind address and `WFHUB_API_DATA_DIR` to serve a different dataset directory. For example, `/api/warframes.json` returns a category and `/api/warframes/excalibur.json` returns an item.

The JSON dataset is not included. Place the category and item JSON files in `api/` (or set `WFHUB_API_DATA_DIR` for the local Rust server). The previous snapshot was based on [WFCD / warframe-items](https://github.com/WFCD/warframe-items); the former data generator has not yet been migrated.

For Cloudflare Workers Builds, use the repository root as the root directory, `bun run cf:build` as the build command, and `bun run cf:deploy` as the deploy command. The build copies the website and any `api/` data into `.cloudflare-build`; the Worker serves JSON paths under `/api/`. Without the dataset those paths return 404. `bun run cf:deploy` also builds and deploys locally.

Warframe and related names and assets belong to their respective rights holders, including Digital Extremes. WFHub is an independent community project.
