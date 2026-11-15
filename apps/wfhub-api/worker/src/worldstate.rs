//! The live game state. The game's server answers 403 to Cloudflare's addresses, so the Worker reaches it through a proxy
//! (`WORLDSTATE_PROXY`, a secret: `http://user:password@host:port`) whose addresses it accepts: a `CONNECT` tunnel to the
//! game's server over the Worker's TCP sockets (`tunnel.rs`). The raw state (about 27 KB gzipped on the wire) is cached
//! for 5 minutes per platform in each data centre (Cache API), and the last good copy is kept for a day for when the proxy
//! or the game fails. It is parsed per request by `wfhub_api::worldstate` with the names of `/data/worldstate`; times in
//! the answers come from the clock, so a copy a few minutes old is still right about what has expired.
//! The `x-worldstate-age` header says how old the copy is; `x-worldstate-stale` flags a copy over 15 minutes.

use crate::tunnel;
use wfhub_api::routes::Reply;
use wfhub_api::worldstate::{self, Names};
use worker::*;

/// How long a data centre uses the raw state before asking the game again (through the proxy, which is metered).
const FRESH_SECONDS: u32 = 300;
/// How long the last good copy is kept, for when the game or the proxy does not answer.
const LAST_GOOD_SECONDS: u32 = 24 * 3600;
/// A copy older than this is flagged as stale.
pub const STALE_AFTER_SECONDS: i64 = 15 * 60;

fn cache_key(kind: &str, platform: &str) -> String {
  format!("https://worldstate.invalid/{kind}/{platform}")
}

/// A copy in the cache: `{fetched_at}\n{raw state}`.
async fn cached(cache: &Cache, kind: &str, platform: &str) -> Option<(i64, String)> {
  let mut response = cache.get(cache_key(kind, platform), false).await.ok().flatten()?;
  let text = response.text().await.ok()?;
  let (at, body) = text.split_once('\n')?;
  Some((at.parse().ok()?, body.to_string()))
}

async fn keep(cache: &Cache, kind: &str, platform: &str, seconds: u32, fetched_at: i64, body: &str) {
  let Ok(response) = Response::ok(format!("{fetched_at}\n{body}")) else { return };
  if response.headers().set("cache-control", &format!("public, max-age={seconds}")).is_err() {
    return;
  }
  let _ = cache.put(cache_key(kind, platform), response).await;
}

/// The raw state of a platform from the game, through the proxy; the error says what went wrong.
async fn fetch_state(env: &Env, platform: &str) -> std::result::Result<String, String> {
  let proxy = env.secret("WORLDSTATE_PROXY").map_err(|_| "no proxy is configured (secret WORLDSTATE_PROXY)".to_string())?.to_string();
  let host = worldstate::host(platform).ok_or("unknown platform")?;
  let text = tunnel::get(&proxy, host, "/cdn/worldState.php").await?;
  match serde_json::from_str::<serde_json::Value>(&text) {
    // Stored in one line: the cached copy is `{at}\n{body}`.
    Ok(state) if state.get("Time").is_some() => Ok(state.to_string()),
    Ok(_) => Err("the game's answer has no Time".into()),
    Err(error) => Err(format!("the game's answer is not JSON: {error} ({} bytes)", text.len())),
  }
}

/// The raw state of a platform and when it was fetched (seconds since the epoch).
async fn raw_state(env: &Env, platform: &str, now: i64) -> std::result::Result<(i64, String), String> {
  let cache = Cache::default();
  if let Some(copy) = cached(&cache, "fresh", platform).await {
    return Ok(copy);
  }
  match fetch_state(env, platform).await {
    Ok(body) => {
      keep(&cache, "fresh", platform, FRESH_SECONDS, now, &body).await;
      keep(&cache, "last", platform, LAST_GOOD_SECONDS, now, &body).await;
      Ok((now, body))
    }
    Err(reason) => {
      console_error!("worldstate {platform}: {reason}");
      cached(&cache, "last", platform).await.ok_or(reason)
    }
  }
}

/// The answer when there is no state to parse, and the age of what there is (none here).
pub fn unavailable(reason: &str) -> (Reply, Option<i64>) {
  (Reply::json(503, serde_json::json!({ "error": "Game state unavailable", "reason": reason }).to_string(), "no-store"), None)
}

/// `GET /api/worldstate/{platform}[/{section}].json`: the parsed state and the age of the copy it comes from (seconds).
pub async fn game_state(env: &Env, platform: &str, section: Option<&str>) -> Result<(Reply, Option<i64>)> {
  let now = (Date::now().as_millis() / 1000) as i64;
  let (fetched_at, text) = match raw_state(env, platform, now).await {
    Ok(copy) => copy,
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
  let cache = if age > STALE_AFTER_SECONDS { "public, max-age=10" } else { "public, max-age=60" };
  Ok((Reply::json(200, body.to_string(), cache), Some(age)))
}
