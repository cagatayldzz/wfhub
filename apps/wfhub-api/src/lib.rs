//! The WFHub API. Everything it serves is static: the files under `apps/wfhub-api/data` (events, mastery snapshots and
//! the Codex catalogue made by `catalogue`). The Cloudflare Worker (`worker/`) is Rust compiled to WASM (`npm run api:build`) and mostly glues those
//! files to the `routes`; `feedback` is the one thing it writes (to D1).

pub mod feedback;
pub mod routes;
pub mod worldstate;

#[cfg(feature = "generate")]
pub mod catalogue;
