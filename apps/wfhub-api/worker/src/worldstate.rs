//! The live game state. The Worker fetches it itself when a copy is older than 5 minutes: the game's server answers 403 to
//! Cloudflare's addresses, so the request goes through a proxy (secret `WORLDSTATE_PROXY`, `http://user:password@host:port`)
//! that `tunnel.js` reaches with a CONNECT tunnel. The raw state is kept in D1 (table `worldstate`), so all data centres share
//! one fetch, and in the Cache API (5 minutes) in front of it. When the proxy or the game fails the last copy is still
//! answered, flagged by `x-worldstate-stale`. The state is parsed with `wfhub_api::worldstate` and the names of
//! `/data/worldstate`. Nothing in an answer depends on the clock (the client counts down from `activation` and `expiry`), so the parsed
//! answer is cached too (1 minute); the `x-worldstate-age` header says how old the copy it comes from is.

use wfhub_api::routes::Reply;
use wfhub_api::worldstate::{self, Names};
use worker::js_sys;
use worker::wasm_bindgen::{self, JsValue, prelude::wasm_bindgen};
use worker::*;

/// How long a copy is used before the game is asked again.
const FRESH_SECONDS: i64 = 300;
/// How long a parsed answer is reused: nothing in it depends on the clock, and parsing (the 170 KB state, the name tables) is
/// what makes a request slow.
const ANSWER_SECONDS: i64 = 60;
/// A copy older than this is flagged as stale.
pub const STALE_AFTER_SECONDS: i64 = 30 * 60;

#[wasm_bindgen(module = "/src/tunnel.js")]
extern "C" {
  #[wasm_bindgen(catch)]
  async fn fetch_via_proxy(proxy: &str, host: &str, path: &str) -> std::result::Result<JsValue, JsValue>;
}

fn cache_key(platform: &str) -> String {
  format!("https://worldstate.invalid/{platform}")
}

fn db_error(error: impl std::fmt::Display) -> String {
  format!("database: {error}")
}

fn js_error(error: JsValue) -> String {
  error.as_string().or_else(|| js_sys::Reflect::get(&error, &"message".into()).ok().and_then(|message| message.as_string())).unwrap_or_else(|| "unknown error".into())
}

fn now() -> i64 {
  (Date::now().as_millis() / 1000) as i64
}

async fn cache_copy(platform: &str, at: i64, body: &str) {
  let left = FRESH_SECONDS - (now() - at);
  if left <= 0 {
    return;
  }
  if let Ok(response) = Response::ok(format!("{at}\n{body}"))
    && response.headers().set("cache-control", &format!("public, max-age={left}")).is_ok()
  {
    let _ = Cache::default().put(cache_key(platform), response).await;
  }
}

/// Asks the game (through the proxy) for the raw state of a platform and keeps it in D1.
async fn refresh(env: &Env, platform: &str) -> std::result::Result<(i64, String), String> {
  let proxy = env.secret("WORLDSTATE_PROXY").map_err(|_| "the proxy is not configured (secret WORLDSTATE_PROXY)".to_string())?.to_string();
  let host = match platform {
    "pc" => "api",
    "xb1" => "api-xb1",
    "ps4" => "api-ps4",
    "swi" => "api-swi",
    _ => return Err(format!("unknown platform {platform}")),
  };
  let text = fetch_via_proxy(&proxy, &format!("{host}.warframe.com"), "/cdn/worldState.php").await.map_err(js_error)?.as_string().unwrap_or_default();
  let state = serde_json::from_str::<serde_json::Value>(&text).map_err(|_| "the game's answer is not JSON".to_string())?;
  let time = state.get("Time").and_then(serde_json::Value::as_i64).ok_or("the game's answer is not a worldState")?;
  // Stored compact (one line), which also keeps the cached copy's `{at}\n{body}` format unambiguous.
  let body = state.to_string();
  let at = now();
  let db = env.d1("DB").map_err(db_error)?;
  let number = |value: i64| JsValue::from_f64(value as f64);
  db.prepare(
    "INSERT INTO worldstate (platform, state_time, fetched_at, body) VALUES (?1, ?2, ?3, ?4)
     ON CONFLICT(platform) DO UPDATE SET state_time = excluded.state_time, fetched_at = excluded.fetched_at, body = excluded.body",
  )
  .bind(&[JsValue::from_str(platform), number(time), number(at), JsValue::from_str(&body)])
  .map_err(db_error)?
  .run()
  .await
  .map_err(db_error)?;
  Ok((at, body))
}

/// The raw state of a platform and when it was fetched (seconds since the epoch): from the cache, else from D1 when it is
/// fresh, else from the game. If the game cannot be reached the last copy is returned; the error only when there is none.
async fn stored_state(env: &Env, platform: &str) -> std::result::Result<(i64, String), String> {
  if let Some(mut response) = Cache::default().get(cache_key(platform), false).await.ok().flatten()
    && let Ok(text) = response.text().await
    && let Some((at, body)) = text.split_once('\n')
    && let Ok(at) = at.parse()
  {
    return Ok((at, body.to_string()));
  }
  let db = env.d1("DB").map_err(db_error)?;
  let row = db
    .prepare("SELECT fetched_at, body FROM worldstate WHERE platform = ?1")
    .bind(&[JsValue::from_str(platform)])
    .map_err(db_error)?
    .first::<serde_json::Value>(None)
    .await
    .map_err(db_error)?;
  let last = row.map(|row| (row["fetched_at"].as_i64().unwrap_or(0), row["body"].as_str().unwrap_or_default().to_string()));
  if let Some((at, body)) = &last
    && now() - at < FRESH_SECONDS
  {
    cache_copy(platform, *at, body).await;
    return Ok(last.unwrap());
  }
  match refresh(env, platform).await {
    Ok((at, body)) => {
      cache_copy(platform, at, &body).await;
      Ok((at, body))
    }
    Err(reason) => {
      console_error!("worldstate refresh failed ({platform}): {reason}");
      last.ok_or(reason)
    }
  }
}

/// The answer when there is no state to parse, and the age of what there is (none here).
pub fn unavailable(reason: &str) -> (Reply, Option<i64>) {
  (Reply::json(503, serde_json::json!({ "error": "Game state unavailable", "reason": reason }).to_string(), "no-store"), None)
}

fn answer_key(platform: &str, section: Option<&str>) -> String {
  format!("https://worldstate.invalid/answer/{platform}/{}", section.unwrap_or("all"))
}

fn answer_cache_control(age: i64) -> &'static str {
  if age > STALE_AFTER_SECONDS { "public, max-age=10" } else { "public, max-age=60" }
}

/// `GET /api/worldstate/{platform}[/{section}].json`: the parsed state and the age of the copy it comes from (seconds).
pub async fn game_state(env: &Env, platform: &str, section: Option<&str>) -> Result<(Reply, Option<i64>)> {
  let now = now();
  let key = answer_key(platform, section);
  if let Some(mut response) = Cache::default().get(key.clone(), false).await.ok().flatten()
    && let Ok(text) = response.text().await
    && let Some((fetched_at, body)) = text.split_once('\n')
    && let Ok(fetched_at) = fetched_at.parse::<i64>()
  {
    let age = (now - fetched_at).max(0);
    return Ok((Reply::json(200, body.to_string(), answer_cache_control(age)), Some(age)));
  }
  let (fetched_at, text) = match stored_state(env, platform).await {
    Ok(state) => state,
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
  let age = (now - fetched_at).max(0);
  let body = body.to_string();
  if let Ok(response) = Response::ok(format!("{fetched_at}\n{body}"))
    && response.headers().set("cache-control", &format!("public, max-age={ANSWER_SECONDS}")).is_ok()
  {
    let _ = Cache::default().put(key, response).await;
  }
  Ok((Reply::json(200, body, answer_cache_control(age)), Some(age)))
}
