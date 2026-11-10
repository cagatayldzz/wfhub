# Architecture

WFHub has a SwiftUI iOS app, a plain HTML/CSS website and a Rust API whose output is static. The site and the API are one Cloudflare Worker deployment served from `https://wfhub.cagatayldzz.com/` (the API under `/api/`, images from `apps/wfhub-web/public`). `scripts/build-site.sh` merges `apps/wfhub-web` and `apps/wfhub-api/data` into `dist/site`, which Cloudflare serves.

## Website and API

`apps/wfhub-web` is the static website (home, legal pages, Swagger page). `apps/wfhub-api` is the Rust API and owns its static data in `data/` (events, mastery, catalogue): a tool that makes the Codex catalogue files, and a Cloudflare Worker (Rust, WASM) that is compiled once, committed, and only serves those files and stores feedback. See the [API README](../apps/wfhub-api/README.md) for configuration and data provenance.

## iOS client

`apps/wfhub-mobile/WFHub/WFHubApp.swift` creates the SwiftUI tab view and defines the mobile catalogue. `Screens/` contains the main views, `Components/` contains shared detail views and visual components, and `Core/` contains shared services. Localized strings are in language-specific `.lproj` directories under `Resources/`.

`project.yml` is the source for the generated Xcode project.
