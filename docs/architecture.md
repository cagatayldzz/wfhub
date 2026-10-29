# Architecture

WFHub has two native clients in a Rust workspace. They share a route catalogue, but each owns its UI, state, network requests, and platform-specific behavior.

```text
                  crates/core
                 /           \
     apps/wfhub               crates/ffi
       Rust + GPUI                |
                         apps/wfhub-mobile
                            SwiftUI + C bridge
```

## Shared code

`crates/core/src/lib.rs` defines activity routes, Codex categories, and platform validation. `crates/ffi` exposes the catalogue to iOS through `wfhub_catalogue` and `wfhub_free`. The catalogue is returned as an owned UTF-8 string; callers must free it with `wfhub_free`. The corresponding C declarations live in `apps/wfhub-mobile/WFHub/wfhub.h`.

## Desktop client

`apps/wfhub/src/main.rs` creates the GPUI application, window, and embedded navigation icon assets. The UI is under `apps/wfhub/src/ui`:

- `mod.rs` owns application state, page navigation, and asynchronous loading.
- `screens.rs` renders the screens and reusable card components.
- `navigation.rs` renders the bottom tab bar and scrollable content.
- `support.rs` contains route, date, network, image cache, and settings helpers.

The root state tracks the selected tab, each tab's current page and navigation history, and a request identifier that prevents an older response from replacing the current page's data. Selecting an item opens a detail page; the back button follows the current tab's history.

## Codex API

`apps/wfhub-api` is a Rust workspace application that serves the included Codex JSON snapshot at `/api/` paths. See its [README](../apps/wfhub-api/README.md) for configuration and data provenance. The legacy dataset generator has not yet been migrated.

## iOS client

`apps/wfhub-mobile/WFHub/WFHubApp.swift` creates the SwiftUI tab view and reads the shared catalogue through the bridge. `HomeView.swift`, `MarketView.swift`, `OtherScreens.swift`, and `DataScreen.swift` implement the main screens. `DetailComponents.swift` and `Design.swift` provide shared detail views and visual components. Localized strings are stored in language-specific `.lproj` directories.

`project.yml` is the source for the generated Xcode project. An Xcode pre-build script compiles `wfhub-ffi` as a static library for the selected iOS platform and architecture.

## Boundaries

The two clients do not currently share a networking layer or UI implementation. A change to one client does not automatically update the other. Keep the shared route catalogue and C interface compatible with both consumers when changing them.
