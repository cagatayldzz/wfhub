//! The live game state: fetched from the game's server, cached (the last good copy is kept for when the server is
//! down) and parsed by `wfhub_api::worldstate` with the names of `/data/worldstate`.

use wfhub_api::routes::{Reply, error};
use wfhub_api::worldstate::{self, Names};
use worker::*;

/// How long the raw state is used without asking the game again, and how long the last good copy is kept.
const FRESH_SECONDS: u32 = 30;
const LAST_GOOD_SECONDS: u32 = 7 * 24 * 3600;

fn cache_key(kind: &str, platform: &str) -> String {
  format!("https://worldstate.invalid/{kind}/{platform}")
}

async fn cached(cache: &Cache, kind: &str, platform: &str) -> Option<String> {
  let mut response = cache.get(cache_key(kind, platform), false).await.ok()??;
  response.text().await.ok()
}

async fn keep(cache: &Cache, kind: &str, platform: &str, seconds: u32, body: &str) {
  let Ok(response) = Response::ok(body) else { return };
  if response.headers().set("cache-control", &format!("public, max-age={seconds}")).is_err() {
    return;
  }
  let _ = cache.put(cache_key(kind, platform), response).await;
}

/// The raw state of the game, as text, and whether it is an old copy (the game did not answer). Cached 30 seconds; the
/// last good copy is kept a week.
async fn raw_state(platform: &str, upstream: &str) -> std::result::Result<(String, bool), String> {
  let cache = Cache::default();
  if let Some(text) = cached(&cache, "fresh", platform).await {
    return Ok((text, false));
  }
  let fetched = fetch_upstream(upstream).await;
  match fetched {
    Ok(text) => {
      keep(&cache, "fresh", platform, FRESH_SECONDS, &text).await;
      keep(&cache, "last", platform, LAST_GOOD_SECONDS, &text).await;
      Ok((text, false))
    }
    Err(reason) => {
      console_error!("worldstate {platform}: {reason}");
      cached(&cache, "last", platform).await.map(|text| (text, true)).ok_or(reason)
    }
  }
}

/// The raw state from the game's server; the error says what went wrong.
async fn fetch_upstream(upstream: &str) -> std::result::Result<String, String> {
  let url = Url::parse(upstream).map_err(|error| format!("bad address: {error}"))?;
  let headers = Headers::new();
  headers.set("user-agent", "Mozilla/5.0 (compatible; WFHub/1.0)").map_err(|error| error.to_string())?;
  headers.set("accept", "application/json, text/plain, */*").map_err(|error| error.to_string())?;
  let mut init = RequestInit::new();
  init.with_method(Method::Get).with_headers(headers);
  let request = Request::new_with_init(url.as_str(), &init).map_err(|error| error.to_string())?;
  let mut response = Fetch::Request(request).send().await.map_err(|error| format!("request failed: {error}"))?;
  if response.status_code() != 200 {
    return Err(format!("upstream answered {}", response.status_code()));
  }
  let text = response.text().await.map_err(|error| format!("body unreadable: {error}"))?;
  match serde_json::from_str::<serde_json::Value>(&text) {
    Ok(state) if state.get("Time").is_some() => Ok(text),
    Ok(_) => Err("upstream answer has no Time".into()),
    Err(error) => Err(format!("upstream answer is not JSON: {error} ({} bytes)", text.len())),
  }
}

pub fn unavailable(reason: &str) -> (Reply, bool) {
  (Reply::json(503, serde_json::json!({ "error": "Game state unavailable", "reason": reason }).to_string(), "no-store"), false)
}

/// `GET /api/worldstate/{platform}[/{section}].json`: the parsed state and whether it is a stale copy.
pub async fn game_state(env: &Env, platform: &str, upstream: &str, section: Option<&str>) -> Result<(Reply, bool)> {
  let (text, stale) = match raw_state(platform, upstream).await {
    Ok(raw) => raw,
    Err(reason) => return Ok(unavailable(&reason)),
  };
  let Ok(state) = serde_json::from_str::<serde_json::Value>(&text) else { return Ok(unavailable("stored state is not JSON")) };
  let Some(tables) = crate::asset(env, "/data/worldstate/tables.json").await? else { return Ok((error(503, "Names unavailable"), false)) };
  let mut shards = Vec::new();
  for shard in worldstate::shards_needed(&state) {
    if let Some(text) = crate::asset(env, &format!("/data/worldstate/names/{shard}.json")).await? {
      shards.push(text);
    }
  }
  let Ok(names) = Names::new(&tables, &shards) else { return Ok((error(503, "Names unavailable"), false)) };
  let now = Date::now().as_millis() as i64;
  let body = match section {
    Some(section) => worldstate::parse_section(&state, section, now, &names),
    None => Some(worldstate::parse_all(&state, now, &names)),
  };
  let Some(body) = body else { return Ok((wfhub_api::routes::not_found(), false)) };
  let cache = if stale { "public, max-age=10" } else { "public, max-age=30" };
  Ok((Reply::json(200, body.to_string(), cache), stale))
}
