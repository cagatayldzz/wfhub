//! Shared logic of the WFHub clients (iOS and Android).
//!
//! The rule of this crate: the clients fetch (HTTP is theirs), render and store preferences; everything in between is here: where to
//! fetch (`urls`), how to read the answer (`home`, `activity`, `codex`, `market`), text (`text`, `i18n`, `time`) and feedback validation.
//! Nothing in here does I/O, so every function is deterministic and unit tested.

mod activity;
mod codex;
mod feedback;
mod home;
mod i18n;
mod market;
mod routes;
mod text;
mod time;
mod urls;

pub use activity::*;
pub use codex::*;
pub use feedback::*;
pub use home::*;
pub use i18n::*;
pub use market::*;
pub use routes::*;
pub use text::*;
pub use time::*;
pub use urls::*;

uniffi::setup_scaffolding!();

/// Failure of reading an API answer. The clients show `message` next to a retry button.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
  #[error("The server answer is not readable")]
  Parse,
  #[error("The server answer has an unexpected shape")]
  Shape,
}

pub(crate) type Json = serde_json::Value;

pub(crate) fn parse(body: &str) -> Result<Json, CoreError> {
  serde_json::from_str(body).map_err(|_| CoreError::Parse)
}

pub(crate) fn str_of<'a>(value: &'a Json, key: &str) -> Option<&'a str> {
  value.get(key).and_then(Json::as_str)
}

/// Number or numeric string, the world state API uses both.
pub(crate) fn num_of(value: &Json, key: &str) -> Option<f64> {
  match value.get(key)? {
    Json::Number(n) => n.as_f64(),
    Json::String(s) => s.parse().ok(),
    _ => None,
  }
}

/// How a scalar JSON value is shown: no quotes, integers without a fraction.
pub(crate) fn show(value: &Json) -> Option<String> {
  match value {
    Json::String(s) => Some(s.clone()),
    Json::Number(n) => Some(n.to_string()),
    Json::Bool(b) => Some(b.to_string()),
    _ => None,
  }
}
