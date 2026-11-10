//! Anonymous feedback: what is accepted (`parse`) and the keyed, daily-changing hash that rate limits a client without
//! keeping its address (`client_key`). Saving it to D1 is the Worker's job.

use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;

pub const MAX_BODY_BYTES: usize = 8192;
/// At most `RATE_LIMIT` submissions per client in this many seconds.
pub const RATE_WINDOW_SECONDS: i64 = 60 * 60;
pub const RATE_LIMIT: i64 = 5;
/// Rate-limit rows older than this are deleted.
pub const RATE_RETENTION_SECONDS: i64 = 48 * 60 * 60;

const CATEGORIES: [&str; 3] = ["bug", "suggestion", "other"];
const FIELDS: [&str; 3] = ["title", "category", "message"];

#[derive(Debug, PartialEq)]
pub struct Feedback {
  pub title: String,
  pub category: String,
  pub message: String,
}

/// A refusal: the HTTP status and the message sent back.
pub type Refusal = (u16, &'static str);

/// Checks the headers that come before the body.
pub fn check_headers(content_type: Option<&str>, content_length: Option<&str>) -> Result<(), Refusal> {
  let content_type = content_type.unwrap_or("").to_ascii_lowercase();
  let json = content_type.strip_prefix("application/json").is_some_and(|rest| rest.is_empty() || rest.trim_start().starts_with(';'));
  if !json {
    return Err((415, "Unsupported content type"));
  }
  if content_length.and_then(|length| length.trim().parse::<usize>().ok()).unwrap_or(0) > MAX_BODY_BYTES {
    return Err((413, "Request too large"));
  }
  Ok(())
}

fn is_control(c: char) -> bool {
  matches!(c, '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{7f}')
}

/// Reads and validates a request body.
pub fn parse(body: &[u8]) -> Result<Feedback, Refusal> {
  if body.len() > MAX_BODY_BYTES {
    return Err((413, "Request too large"));
  }
  let value: Value = serde_json::from_slice(body).map_err(|_| (400, "Invalid JSON"))?;
  const INVALID: Refusal = (400, "Invalid feedback");
  let object = value.as_object().ok_or(INVALID)?;
  if object.keys().any(|key| !FIELDS.contains(&key.as_str())) {
    return Err(INVALID);
  }
  let text = |name: &str| object.get(name).and_then(Value::as_str).ok_or(INVALID);
  let (title, message, category) = (text("title")?.trim(), text("message")?.trim(), text("category")?);
  if !CATEGORIES.contains(&category) {
    return Err(INVALID);
  }
  let (title_length, message_length) = (title.chars().count(), message.chars().count());
  if !(3..=120).contains(&title_length) || !(10..=4000).contains(&message_length) || title.chars().chain(message.chars()).any(is_control) {
    return Err(INVALID);
  }
  Ok(Feedback { title: title.into(), category: category.into(), message: message.into() })
}

/// The rate-limit key of a client: HMAC-SHA256 of `day:ip`, in hex. It changes every day, so the database never holds an
/// address, and the same client cannot be followed from one day to the next.
pub fn client_key(secret: &str, day: &str, ip: &str) -> String {
  let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC takes a key of any length");
  mac.update(format!("{day}:{ip}").as_bytes());
  mac.finalize().into_bytes().iter().map(|byte| format!("{byte:02x}")).collect()
}
