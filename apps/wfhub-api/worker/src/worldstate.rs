//! The live game state. The game's server refuses requests from Cloudflare's network and from GitHub Actions (403), so the
//! Worker does not fetch it: `scripts/push-worldstate.sh` (run on a Mac by `scripts/install-worldstate-agent.sh`) fetches the raw
//! state of each platform and `PUT`s it to `/api/ingest/worldstate/{platform}`; it is kept in D1 (table `worldstate`).
//! Reading answers from that copy (cached 30 seconds per data centre) and parses it with `wfhub_api::worldstate` and the
//! names of `/data/worldstate`. Times in the answers are computed from the clock, so an old copy is still correct about
//! what has expired; the `x-worldstate-age` header says how old it is and `x-worldstate-stale` flags a copy over 15 minutes.

use wfhub_api::routes::{Reply, error};
use wfhub_api::worldstate::{self, Names};
use worker::wasm_bindgen::JsValue;
use worker::*;

/// How long a data centre keeps the copy it read from D1.
const FRESH_SECONDS: u32 = 30;
/// A copy older than this is flagged as stale.
pub const STALE_AFTER_SECONDS: i64 = 15 * 60;
/// The raw state is about 170 KB; this is far more than it needs and far less than the Worker's limits.
const MAX_BODY: usize = 3_000_000;
/// A pushed state must be from now, give or take this.
const MAX_CLOCK_DIFFERENCE: i64 = 2 * 24 * 3600;

fn cache_key(platform: &str) -> String {
  format!("https://worldstate.invalid/{platform}")
}

fn db_error(error: impl std::fmt::Display) -> String {
  format!("database: {error}")
}

/// The raw state of a platform and when it was pushed (seconds since the epoch), `None` before the first push.
async fn stored_state(env: &Env, platform: &str) -> std::result::Result<Option<(i64, String)>, String> {
  let cache = Cache::default();
  if let Some(mut response) = cache.get(cache_key(platform), false).await.ok().flatten()
    && let Ok(text) = response.text().await
    && let Some((at, body)) = text.split_once('\n')
    && let Ok(at) = at.parse()
  {
    return Ok(Some((at, body.to_string())));
  }
  let db = env.d1("DB").map_err(db_error)?;
  let row = db
    .prepare("SELECT fetched_at, body FROM worldstate WHERE platform = ?1")
    .bind(&[JsValue::from_str(platform)])
    .map_err(db_error)?
    .first::<serde_json::Value>(None)
    .await
    .map_err(db_error)?;
  let Some(row) = row else { return Ok(None) };
  let (at, body) = (row["fetched_at"].as_i64().unwrap_or(0), row["body"].as_str().unwrap_or_default().to_string());
  if let Ok(response) = Response::ok(format!("{at}\n{body}"))
    && response.headers().set("cache-control", &format!("public, max-age={FRESH_SECONDS}")).is_ok()
  {
    let _ = cache.put(cache_key(platform), response).await;
  }
  Ok(Some((at, body)))
}

/// The answer when there is no state to parse, and the age of what there is (none here).
pub fn unavailable(reason: &str) -> (Reply, Option<i64>) {
  (Reply::json(503, serde_json::json!({ "error": "Game state unavailable", "reason": reason }).to_string(), "no-store"), None)
}

/// `GET /api/worldstate/{platform}[/{section}].json`: the parsed state and the age of the copy it comes from (seconds).
pub async fn game_state(env: &Env, platform: &str, section: Option<&str>) -> Result<(Reply, Option<i64>)> {
  let now = (Date::now().as_millis() / 1000) as i64;
  let (pushed_at, text) = match stored_state(env, platform).await {
    Ok(Some(state)) => state,
    Ok(None) => return Ok(unavailable(&format!("no state has been pushed for {platform} yet"))),
    Err(reason) => return Ok(unavailable(&reason)),
  };
  let Ok(state) = serde_json::from_str::<serde_json::Value>(&text) else { return Ok(unavailable("stored state is not JSON")) };
  let Some(tables) = crate::asset(env, "/data/worldstate/tables.json").await? else { return Ok(unavailable("name tables are missing")) };
  let mut shards = Vec::new();
  for shard in worldstate::shards_needed(&state) {
    if let Some(text) = crate::asset(env, &format!("/data/worldstate/names/{shard}.json")).await? {
      shards.push(text);
    }
  }
  let Ok(names) = Names::new(&tables, &shards) else { return Ok(unavailable("name tables are not valid")) };
  let body = match section {
    Some(section) => worldstate::parse_section(&state, section, now * 1000, &names),
    None => Some(worldstate::parse_all(&state, now * 1000, &names)),
  };
  let Some(body) = body else { return Ok((wfhub_api::routes::not_found(), None)) };
  let age = (now - pushed_at).max(0);
  let cache = if age > STALE_AFTER_SECONDS { "public, max-age=10" } else { "public, max-age=30" };
  Ok((Reply::json(200, body.to_string(), cache), Some(age)))
}

/// Compares two secrets without stopping at the first difference.
fn same(a: &[u8], b: &[u8]) -> bool {
  let mut difference = a.len() ^ b.len();
  for (x, y) in a.iter().zip(b.iter()) {
    difference |= usize::from(x ^ y);
  }
  difference == 0
}

/// `PUT /api/ingest/worldstate/{platform}`: stores the raw state (the body, as the game sent it). Needs
/// `Authorization: Bearer <WORLDSTATE_PUSH_TOKEN>` (a secret of the Worker). An older state never replaces a newer one.
pub async fn ingest(mut req: Request, env: &Env, platform: &str) -> Result<Reply> {
  let Ok(secret) = env.secret("WORLDSTATE_PUSH_TOKEN") else { return Ok(error(503, "Service unavailable")) };
  let secret = secret.to_string();
  let header = req.headers().get("authorization")?.unwrap_or_default();
  if secret.is_empty() || !same(header.strip_prefix("Bearer ").unwrap_or("").as_bytes(), secret.as_bytes()) {
    return Ok(error(401, "Unauthorized"));
  }
  if req.headers().get("content-length")?.and_then(|length| length.parse::<usize>().ok()).is_some_and(|length| length > MAX_BODY) {
    return Ok(error(413, "Too large"));
  }
  let bytes = req.bytes().await?;
  if bytes.len() > MAX_BODY {
    return Ok(error(413, "Too large"));
  }
  let Ok(state) = serde_json::from_slice::<serde_json::Value>(&bytes) else { return Ok(error(400, "Not JSON")) };
  let Some(time) = state.get("Time").and_then(serde_json::Value::as_i64) else { return Ok(error(400, "Not a worldState")) };
  let now = (Date::now().as_millis() / 1000) as i64;
  if (time - now).abs() > MAX_CLOCK_DIFFERENCE {
    return Ok(error(400, "State is not from now"));
  }
  let Ok(db) = env.d1("DB") else { return Ok(error(503, "Service unavailable")) };
  // The state is stored compact (one line), which also keeps the cached copy's `{at}\n{body}` format unambiguous.
  let body = state.to_string();
  let number = |value: i64| JsValue::from_f64(value as f64);
  db.prepare(
    "INSERT INTO worldstate (platform, state_time, fetched_at, body) VALUES (?1, ?2, ?3, ?4)
     ON CONFLICT(platform) DO UPDATE SET state_time = excluded.state_time, fetched_at = excluded.fetched_at, body = excluded.body
     WHERE excluded.state_time >= worldstate.state_time",
  )
  .bind(&[JsValue::from_str(platform), number(time), number(now), JsValue::from_str(&body)])?
  .run()
  .await?;
  Ok(Reply::json(200, serde_json::json!({ "ok": true, "time": time }).to_string(), "no-store"))
}
