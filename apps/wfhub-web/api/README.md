# WFHub API

A Rust service for the WFHub Codex JSON dataset. It serves the existing category and item paths used by the desktop and iOS clients.

```sh
cargo run -p wfhub-api
```

The default address is `127.0.0.1:8080`. Set `WFHUB_API_ADDR` to change the bind address and `WFHUB_API_DATA_DIR` to serve a different dataset directory. For example, `/api/warframes.json` returns a category and `/api/warframes/excalibur.json` returns an item.

Swagger documentation is available at `/api/`.

The `/api/events.json` endpoint serves the event banners maintained in `api/data/events.json`. Update that file and redeploy to change the banners.

The local Rust server needs a JSON dataset in `api/` (or the directory set by `WFHUB_API_DATA_DIR`). The Cloudflare build instead downloads category data from [WFCD / warframe-items](https://github.com/WFCD/warframe-items) and generates the API indexes.

For Cloudflare Workers Builds, use the repository root as the root directory, `bun scripts/cloudflare.ts build` as the build command, and `bun scripts/cloudflare.ts deploy` as the deploy command. The build copies the website and generates API data in `.cloudflare-build`; the Worker serves JSON paths under `/api/`. The build requires network access to GitHub. `bun scripts/cloudflare.ts deploy` also builds and deploys locally.

Warframe and related names and assets belong to their respective rights holders, including Digital Extremes. WFHub is an independent community project.

`GET /api/mastery/warframes.json` returns the main Warframe checklist as an array of `{ "name", "image", "level", "mastery" }`. It is a manually maintained snapshot of the [Warframe Wiki checklist](https://wiki.warframe.com/w/Mastery_Rank_Checklist), checked against the main Warframe table on 2026-11-07 (wiki page last edited 2026-10-23); Founder exclusives are in a separate wiki section and are not included. Image URLs point to the thumbnails shown in the wiki checklist. Update `api/data/mastery-warframes.json` and redeploy when the checklist changes. Wiki content is available under [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/); additional terms apply.

The same snapshot format is available for `primary`, `secondary`, `melee`, `kitgun`, `zaw`, `amp`, `sentinel`, `moa`, `hound`, `beast`, `robotic`, `archwing`, `archgun`, `archmelee`, `necramech`, `railjack`, and `k-drive`, at `/api/mastery/{category}.json`. The `image` field is null for entries without an image in the wiki checklist (Plexus and the five K-Drives). Checklist rows are maintained in `api/data/mastery-*.json`, independently of the Codex catalogue.

Additional mastery endpoints: `railjack-intrinsic`, `drifter-intrinsic`, `starchart`, `steel-path`, `founders`, and `china-founders`. These include stable `id` and optional `group`; `level` is null for missions and founder items. `/api/mastery/updated.json` reports our snapshot date and the source page update date. Snapshot dates are maintained alongside the JSON data and must be updated when the data is refreshed.
