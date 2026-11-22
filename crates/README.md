# WFHub core

`wfhub-core` is the shared logic of the iOS and Android apps, written in Rust and used from Swift and Kotlin through
[UniFFI](https://mozilla.github.io/uniffi-rs/). The apps fetch, draw and store preferences; everything in between is here.

```text
Swift / Kotlin UI  ──►  HTTP (URLSession / HttpURLConnection)  ──►  JSON text
                                                                      │
                  view models (records)  ◄──  wfhub-core  ◄──────────┘
```

| Module | What it does |
| --- | --- |
| `urls` | every address the apps call (world state, WFHub API, market, feedback) and image addresses |
| `routes` | the activity, cycle and Codex category lists |
| `home` | reads the Home requests into one `HomeState` (events, alerts, news, banners, Nightwave, Baro) |
| `activity` | reads any world state endpoint into an `ActivityPage` (cards, sections, fissure tiers, search) |
| `codex` | reads a category list and an item detail |
| `market` | `MarketBook` joins recent orders with the item catalogue and keeps the catalogue for an hour |
| `i18n` | 14 languages; embeds the `Localizable.strings` of the iOS app, so a text is added once for both apps |
| `time`, `text` | countdowns, translation keys, labels, file names |
| `feedback` | validation, request body and error keys of the feedback form |

The crate does no I/O: every function takes text and returns records, so it is unit tested without a network (`npm run core:test`).

## Rules

- Keep the core free of I/O and platform code. The clients own transport, storage and drawing.
- A function that a client calls is `#[uniffi::export]`; records returned to the clients are `#[derive(uniffi::Record)]`.
- Record names must not collide with SwiftUI or Compose types (`Section`, `Progress`, ... are taken; see the names in `activity.rs`).
- Changing a path or JSON shape of the WFHub API breaks released apps; see [Data and integrations](../docs/data-and-integrations.md).

## Build

```sh
npm run core:test      # unit tests
bash scripts/build-core.sh ios       # apps/wfhub-ios/Generated: WFHubCore.xcframework + Swift bindings
bash scripts/build-core.sh android   # apps/wfhub-android/app/src/main: jniLibs + Kotlin bindings (needs cargo-ndk)
```

`scripts/ios.sh` and `scripts/android.sh` run the matching core build first. The generated bindings and libraries are not committed.
