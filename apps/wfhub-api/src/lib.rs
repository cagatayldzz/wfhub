//! The WFHub API. Everything it serves is static: the files under `apps/wfhub-web/data` (events, mastery snapshots and
//! the Codex catalogue made by `catalogue`). The Cloudflare Worker (`worker/`) is compiled once and only glues those
//! files to the `routes`; `feedback` is the one thing it writes (to D1).

pub mod feedback;
pub mod routes;

#[cfg(feature = "generate")]
pub mod catalogue;
