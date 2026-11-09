# Architecture

WFHub has a SwiftUI iOS app and a TypeScript website and API.

## Codex API

`apps/wfhub-web` contains the website and Cloudflare Worker serving Codex data at `/api/` paths. See the [API README](../apps/wfhub-web/api/README.md) for configuration and data provenance.

## iOS client

`apps/wfhub-mobile/WFHub/WFHubApp.swift` creates the SwiftUI tab view and defines the mobile catalogue. `Screens/` contains the main views, `Components/` contains shared detail views and visual components, and `Core/` contains shared services. Localized strings are in language-specific `.lproj` directories under `Resources/`.

`project.yml` is the source for the generated Xcode project.
