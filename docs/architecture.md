# Architecture

WFHub has independent Rust desktop and SwiftUI iOS clients. Each owns its route catalogue, UI, state, and network requests.

```text
    crates/core → apps/wfhub (Rust + GPUI Fast)
    apps/wfhub-mobile (SwiftUI)
```

## Desktop core

`crates/core/src/lib.rs` defines the desktop activity routes, Codex categories, and platform validation.

## Desktop client

`apps/wfhub/src/main.rs` creates the GPUI Fast application, window, and embedded navigation icon assets. The UI is under `apps/wfhub/src/ui`:

- `mod.rs` owns application state, page navigation, and asynchronous loading.
- `screens.rs` renders the screens and reusable card components.
- `navigation.rs` renders the bottom tab bar and scrollable content.
- `support.rs` contains route, date, network, image cache, and settings helpers.

The root state tracks the selected tab, each tab's current page and navigation history, and a request identifier that prevents an older response from replacing the current page's data. Selecting an item opens a detail page; the back button follows the current tab's history.

## Codex API

`apps/wfhub-web` contains the website and Cloudflare Worker serving Codex data at `/api/` paths. See the [API README](../apps/wfhub-web/api/README.md) for configuration and data provenance.

## iOS client

`apps/wfhub-mobile/WFHub/WFHubApp.swift` creates the SwiftUI tab view and defines the mobile catalogue. `Screens/` contains the main views, `Components/` contains shared detail views and visual components, and `Core/` contains shared services. Localized strings are in language-specific `.lproj` directories under `Resources/`.

`project.yml` is the source for the generated Xcode project. The iOS app does not depend on the Rust toolchain.

## Boundaries

The clients do not share a catalogue, networking layer, or UI implementation. A change to one client does not automatically update the other; update both catalogues when a change applies to both apps.
