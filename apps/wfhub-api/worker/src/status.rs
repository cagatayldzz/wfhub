//! The status page: every 5 minutes (cron trigger, `wrangler.jsonc`) the Worker checks three things and stores the result in
//! D1 (`status_checks`); `GET /api/status` reads it back for `/status`.
//!
//! - `api`: this API itself. A Worker cannot fetch its own public address, so the check runs what `/api/worldstate/pc.json`
//!   runs: the answer must have game data (open fissures) and come from a copy that is not stale.
//! - `warframestat`: `https://api.warframestat.us/pc`, the community API the apps used before `/api/worldstate`, with the same rule.
//! - `compare`: the two answers hold the same news, events, alerts, sortie and so on (`wfhub_api::status`).

use wfhub_api::routes::Reply;
use worker::wasm_bindgen::JsValue;
use worker::*;

pub const SERVICES: [(&str, &str); 3] =
  [("api", "WFHub API"), ("warframestat", "Warframe Status API"), ("compare", "Same data in both APIs")];

const WARFRAMESTAT_URL: &str = "https://api.warframestat.us/pc";
const TIMEOUT_MS: u32 = 10_000;
const KEEP_DAYS: i64 = 90;

fn now() -> i64 {
  (Date::now().as_millis() / 1000) as i64
}

fn number(value: i64) -> JsValue {
  JsValue::from_f64(value as f64)
}

/// Fissures are always open in the game, so a state without any means the data is missing or broken (alerts and events can be
/// empty).
fn has_data(state: &serde_json::Value) -> std::result::Result<(), String> {
  match state["fissures"].as_array() {
    Some(fissures) if !fissures.is_empty() => Ok(()),
    Some(_) => Err("no fissures in the data".into()),
    None => Err("the data has no fissures".into()),
  }
}

/// `Ok(())` when this API answers with game data that is not stale: the same answer `/api/worldstate/pc.json` gives.
async fn check_api(env: &Env) -> std::result::Result<serde_json::Value, String> {
  let (reply, age) = match crate::worldstate::game_state(env, "pc", None, None).await {
    Ok(answer) => answer,
    Err(error) => return Err(error.to_string()),
  };
  if reply.status != 200 {
    return Err(format!("worldstate answered {}", reply.status));
  }
  let state = serde_json::from_str::<serde_json::Value>(&reply.body).map_err(|_| "worldstate is not JSON".to_string())?;
  has_data(&state)?;
  match age {
    Some(age) if age > crate::worldstate::STALE_AFTER_SECONDS => Err(format!("the game data is {} min old", age / 60)),
    _ => Ok(state),
  }
}

/// `Ok(())` when api.warframestat.us answers with game data (the whole request, body included, must take less than `TIMEOUT_MS`).
async fn check_warframestat() -> std::result::Result<serde_json::Value, String> {
  let controller = AbortController::default();
  let signal = controller.signal();
  let timeout = Delay::from(std::time::Duration::from_millis(TIMEOUT_MS as u64));
  let download = async {
    let request = Request::new(WARFRAMESTAT_URL, Method::Get).map_err(|e| e.to_string())?;
    let mut response = Fetch::Request(request).send_with_signal(&signal).await.map_err(|e| e.to_string())?;
    if !(200..300).contains(&response.status_code()) {
      return Err(format!("HTTP {}", response.status_code()));
    }
    let state = response.json::<serde_json::Value>().await.map_err(|_| "the answer is not JSON".to_string())?;
    has_data(&state)?;
    Ok(state)
  };
  futures_util::pin_mut!(download);
  match futures_util::future::select(download, timeout).await {
    futures_util::future::Either::Left((result, _)) => result,
    futures_util::future::Either::Right(_) => {
      controller.abort();
      Err(format!("no answer in {} s", TIMEOUT_MS / 1000))
    }
  }
}

async fn record(db: &D1Database, service: &str, at: i64, result: std::result::Result<(), String>, latency_ms: i64) -> Result<()> {
  let (ok, detail) = match result {
    Ok(()) => (1, JsValue::NULL),
    Err(reason) => (0, JsValue::from_str(&reason.chars().take(300).collect::<String>())),
  };
  db.prepare("INSERT INTO status_checks (service, checked_at, ok, latency_ms, detail) VALUES (?1, ?2, ?3, ?4, ?5)")
    .bind(&[JsValue::from_str(service), number(at), number(ok), number(latency_ms), detail])?
    .run()
    .await?;
  Ok(())
}

/// The cron: checks every service, stores the results, and drops what is older than `KEEP_DAYS`.
pub async fn run_checks(env: &Env) -> Result<()> {
  let db = env.d1("DB")?;
  let timed = |start: u64| (Date::now().as_millis() - start) as i64;
  // The first answer may have to be made (parsing the state), which no user waits for: the time shown is that of the next one, from the cache.
  let api = check_api(env).await;
  let start = Date::now().as_millis();
  let _ = crate::worldstate::game_state(env, "pc", None, None).await;
  let api_ms = timed(start);
  let start = Date::now().as_millis();
  let warframestat = check_warframestat().await;
  let warframestat_ms = timed(start);
  let compared = match (&api, &warframestat) {
    (Ok(mine), Ok(theirs)) => match wfhub_api::status::differences(mine, theirs, now()) {
      differences if differences.is_empty() => Ok(()),
      differences => Err(differences.join("; ")),
    },
    (Err(_), _) => Err("cannot compare: the WFHub API has no data".into()),
    (_, Err(_)) => Err("cannot compare: the Warframe Status API has no data".into()),
  };
  let at = now();
  record(&db, "api", at, api.map(|_| ()), api_ms).await?;
  record(&db, "warframestat", at, warframestat.map(|_| ()), warframestat_ms).await?;
  record(&db, "compare", at, compared, 0).await?;
  db.prepare("DELETE FROM status_checks WHERE checked_at < ?1").bind(&[number(at - KEEP_DAYS * 86_400)])?.run().await?;
  Ok(())
}

/// `GET /api/status`: for each service the last check, the uptime and the average latency of the last 90 days, and the
/// uptime and number of failed checks (5 minutes each) of each day (UTC), oldest first.
pub async fn report(env: &Env) -> Result<Reply> {
  let db = env.d1("DB")?;
  let since = now() - KEEP_DAYS * 86_400;
  let days = db
    .prepare(
      "SELECT service, date(checked_at, 'unixepoch') AS day, COUNT(*) AS checks, SUM(ok) AS up, CAST(AVG(latency_ms) AS INTEGER) AS latency_ms
       FROM status_checks WHERE checked_at >= ?1 GROUP BY service, day ORDER BY day",
    )
    .bind(&[number(since)])?
    .all()
    .await?
    .results::<serde_json::Value>()?;
  let last = db
    .prepare(
      "SELECT service, checked_at, ok, latency_ms, detail FROM status_checks
       WHERE id IN (SELECT MAX(id) FROM status_checks GROUP BY service)",
    )
    .all()
    .await?
    .results::<serde_json::Value>()?;
  let services = SERVICES.iter().map(|(id, name)| {
    let mine = |row: &&serde_json::Value| row["service"] == *id;
    let days: Vec<_> = days.iter().filter(mine).collect();
    let checks: i64 = days.iter().map(|d| d["checks"].as_i64().unwrap_or(0)).sum();
    let up: i64 = days.iter().map(|d| d["up"].as_i64().unwrap_or(0)).sum();
    let latency: i64 = days.iter().map(|d| d["latency_ms"].as_i64().unwrap_or(0) * d["checks"].as_i64().unwrap_or(0)).sum();
    let last = last.iter().find(mine);
    serde_json::json!({
      "id": id,
      "name": name,
      "status": last.map(|row| if row["ok"].as_i64() == Some(1) { "up" } else { "down" }),
      "checkedAt": last.map(|row| row["checked_at"].clone()),
      "latencyMs": last.map(|row| row["latency_ms"].clone()),
      "detail": last.map(|row| row["detail"].clone()),
      "uptime": if checks > 0 { Some((up as f64 / checks as f64 * 10_000.0).round() / 100.0) } else { None },
      "averageLatencyMs": if checks > 0 { Some(latency / checks) } else { None },
      "days": days.iter().map(|d| serde_json::json!({
        "date": d["day"],
        "uptime": (d["up"].as_f64().unwrap_or(0.0) / d["checks"].as_f64().unwrap_or(1.0) * 10_000.0).round() / 100.0,
        "downChecks": d["checks"].as_i64().unwrap_or(0) - d["up"].as_i64().unwrap_or(0),
      })).collect::<Vec<_>>(),
    })
  });
  let body = serde_json::json!({ "services": services.collect::<Vec<_>>() }).to_string();
  Ok(Reply::json(200, body, "public, max-age=60"))
}
