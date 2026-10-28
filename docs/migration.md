# Migration plan and current coverage

Source: `wfhub-mobile` Expo application. The destination is a Rust workspace with a native SwiftUI iOS client, following Kintsu's core → C FFI → platform UI direction. Network requests currently live in Swift; the Rust core owns the route catalogue. The source application remains the reference for behavior and design.

## Implemented foundation

- Five native tabs and navigation, platform and Tenno name preferences.
- Activity endpoints, six cycle endpoints, Codex categories and item navigation.
- Recent market buy/sell orders, joined with the market item catalogue.
- Async loading, errors and refresh for data screens.
- XcodeGen configuration and Rust static-library build script.

## Remaining migration work

- Replace generic activity and item rendering with the source app's specific cards, nested data, countdowns, filters, images and links; group cycles as in the source.
- Match the original home dashboard, theme, assets and layouts; migrate all translations and language selection.
- Match market behavior, API headers, refresh timing, error handling and item imagery against live responses.
- Add accessibility, offline behavior and UI/integration tests; verify every screen against the source on device.
- Android is not implemented: Kintsu supplies an iOS architecture, not an Android client. A native Android frontend and Rust bridge remain to be designed.

This is a migration foundation, not feature or visual parity with the Expo app.
