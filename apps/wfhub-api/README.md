# WFHub API

A Rust service for the WFHub Codex JSON dataset. It serves the existing category and item paths used by the desktop and iOS clients.

```sh
cargo run -p wfhub-api
```

The default address is `127.0.0.1:8080`. Set `WFHUB_API_ADDR` to change the bind address and `WFHUB_API_DATA_DIR` to serve a different dataset directory. For example, `/api/warframes.json` returns a category and `/api/warframes/excalibur.json` returns an item.

The `api/` directory is a snapshot migrated from the previous WFHub API. It is based on data from [WFCD / warframe-items](https://github.com/WFCD/warframe-items), and its image URLs point to the WFCD CDN. The former TypeScript data generator has not yet been migrated; the snapshot is not automatically updated.

Warframe and related names and assets belong to their respective rights holders, including Digital Extremes. WFHub is an independent community project.
