# WFHub API

A Rust service for the WFHub Codex JSON dataset. It serves the existing category and item paths used by the desktop and iOS clients.

```sh
cargo run -p wfhub-api
```

The default address is `127.0.0.1:8080`. Set `WFHUB_API_ADDR` to change the bind address and `WFHUB_API_DATA_DIR` to serve a different dataset directory. For example, `/api/warframes.json` returns a category and `/api/warframes/excalibur.json` returns an item.

The local Rust server needs a JSON dataset in `api/` (or the directory set by `WFHUB_API_DATA_DIR`). The Cloudflare build instead downloads category data from [WFCD / warframe-items](https://github.com/WFCD/warframe-items) and generates the API indexes.

For Cloudflare Workers Builds, use the repository root as the root directory, `bun run cf:build` as the build command, and `bun run cf:deploy` as the deploy command. The build copies the website and generates API data in `.cloudflare-build`; the Worker serves JSON paths under `/api/`. The build requires network access to GitHub. `bun run cf:deploy` also builds and deploys locally.

Warframe and related names and assets belong to their respective rights holders, including Digital Extremes. WFHub is an independent community project.
