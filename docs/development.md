# Development

## Prerequisites

Install Node.js (for Wrangler) and Rust with the `wasm32-unknown-unknown` target for web and API development. iOS development requires Xcode, XcodeGen, and an iOS 17 or later simulator or device.

## Run the clients

To generate the iOS project and open it in Xcode:

```sh
cd apps/wfhub-mobile
xcodegen generate
open WFHub.xcodeproj
```

Select the `WFHub` scheme and a simulator or device. Physical devices require a valid signing team.

Alternatively, from the repository root, `bash scripts/mobile.sh simulator` generates the project, builds the app, and launches the default iPhone 15 simulator. Set `WFHUB_SIMULATOR` to another device name or UDID when needed. The simulator script requires a matching available simulator.

## Website and API

The website is plain HTML and CSS in `apps/wfhub-web` (images in `public/`); `scripts/build-site.sh` merges it and the API data (`apps/wfhub-api/data`) into `dist/site` (the scripts below run it). Run `npm run dev:web` from the repository root and open `http://localhost:8000`: Wrangler serves the site and the API together. `npm run cf:deploy` applies pending D1 migrations and uploads the site and the committed Worker to Cloudflare; deploying never compiles Rust. Run `npm run api:build` (and commit `apps/wfhub-api/worker/dist`) only when the Rust code of the Worker or `routes.rs`/`feedback.rs` changes. Run `npm run catalogue:fetch` and `npm run catalogue:export` to refresh the Codex catalogue files in `apps/wfhub-api/data/catalogue`.

The privacy and terms pages are drafts and require review before publication. See the [API README](../apps/wfhub-api/README.md) for configuration and dataset details.

## Checks

```sh
npm run api:test
npm run format:rust
```

API tests cover routes, HTTP methods, feedback validation, and the static data files; they do not replace UI and device testing. When changing UI behavior, exercise the affected screens and navigation in the relevant client.

## Generated files and release

The iOS `build/` directory and generated `.xcodeproj` files are ignored by Git. Edit `apps/wfhub-mobile/project.yml` and regenerate the project instead of editing the generated project as the source of truth.

`bash scripts/mobile.sh archive` generates an iOS archive and increments `CURRENT_PROJECT_VERSION` in `project.yml` by default. Invoke the script directly with `--no-bump` to retain the current number. Archiving requires an appropriately configured signing team.

If the simulator script cannot find iPhone 15, select an available simulator with `WFHUB_SIMULATOR`. Network-backed screens require access to the services listed in [Data and integrations](data-and-integrations.md).

The web and API scripts are npm scripts (`package.json`); they only need Node.js for `npx wrangler`.
