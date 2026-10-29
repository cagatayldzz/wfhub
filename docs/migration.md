# Migration and coverage

WFHub contains a SwiftUI iOS client and a Rust/GPUI desktop client. They share a route catalogue through `crates/core`; iOS consumes it through the C bridge in `crates/ffi`. Their screens and networking remain separate implementations.

## Current coverage

Both clients have Home, Activities, Market, Codex, and Settings navigation. They display network-backed world-state, Codex, and market data. The desktop client has a phone-proportioned window, per-tab navigation history, and locally saved settings. The iOS client includes localized strings and native navigation.

## Remaining work

- Verify visual and functional parity screen by screen, including nested activity and Codex details.
- Bring desktop language selection, input handling, and settings behavior closer to iOS.
- Improve desktop activity and item-specific rendering, filtering, and cycle summaries where generic views remain.
- Verify refresh behavior, links, loading and error states, and accessibility in both clients.
- Add UI and integration coverage for navigation and network-backed screens.
- Design and implement an Android client if one is needed; it is not currently included.

This document describes the broad direction, not a guarantee that every feature is identical across clients. Check the relevant implementation and test on the target platform when planning a change.
