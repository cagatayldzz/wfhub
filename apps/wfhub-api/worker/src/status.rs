//! The status page: every 5 minutes (cron trigger, `wrangler.jsonc`) the Worker checks two services and stores the result in
//! D1 (`status_checks`); `GET /api/status` reads it back for `/status`.
//!
//! - `api`: this API itself. A Worker cannot fetch its own public address, so the check is what the API needs to answer:
//!   the database answers a query and the static data (`/data/...`) can be read.
//! - `warframestat`: `https://api.warframestat.us/heartbeat`, the community API the apps used before `/api/worldstate`.

use wfhub_api::routes::Reply;
use worker::wasm_bindgen::JsValue;
use worker::*;

pub const SERVICES: [(&str, &str); 2] = [("api", "WFHub API"), ("warframestat", "Warframe Status API")];

const WARFRAMESTAT_URL: &str = "https://api.warframestat.us/heartbeat";
const TIMEOUT_MS: u32 = 10_000;
const KEEP_DAYS: i64 = 90;

fn now() -> i64 {
  (Date::now().as_millis() / 1000) as i64
}

fn number(value: i64) -> JsValue {
  JsValue::from_f64(value as f64)
}

/// `Ok(())` when this API can do what it is for.
async fn check_api(env: &Env) -> std::result::Result<(), String> {
  let db = env.d1("DB").map_err(|e| format!("database: {e}"))?;
  db.prepare("SELECT 1 AS ok").first::<serde_json::Value>(None).await.map_err(|e| format!("database: {e}"))?;
  match crate::asset(env, "/data/events.json").await {
    Ok(Some(_)) => Ok(()),
    Ok(None) => Err("static data not found".into()),
    Err(e) => Err(format!("static data: {e}")),
  }
}

async fn check_warframestat() -> std::result::Result<(), String> {
  let controller = AbortController::default();
  let timeout = Delay::from(std::time::Duration::from_millis(TIMEOUT_MS as u64));
  let signal = controller.signal();
  let request = Request::new(WARFRAMESTAT_URL, Method::Get).map_err(|e| e.to_string())?;
  let fetch = Fetch::Request(request);
  let fetching = fetch.send_with_signal(&signal);
  futures_util::pin_mut!(fetching);
  match futures_util::future::select(fetching, timeout).await {
    futures_util::future::Either::Left((Ok(response), _)) if (200..300).contains(&response.status_code()) => Ok(()),
    futures_util::future::Either::Left((Ok(response), _)) => Err(format!("HTTP {}", response.status_code())),
    futures_util::future::Either::Left((Err(e), _)) => Err(e.to_string()),
    futures_util::future::Either::Right(_) => {
      controller.abort();
      Err(format!("no answer in {} s", TIMEOUT_MS / 1000))
    }
  }
}

async fn record(db: &D1Database, service: &str, at: i64, result: std::result::Result<(), String>, latency_ms: i64) -> Result<()> {
  let (ok, detail) = match result {
    Ok(()) => (1, JsValue::NULL),
    Err(reason) => (0, JsValue::from_str(&reason.chars().take(200).collect::<String>())),
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
  let start = Date::now().as_millis();
  let api = check_api(env).await;
  let api_ms = timed(start);
  let start = Date::now().as_millis();
  let warframestat = check_warframestat().await;
  let warframestat_ms = timed(start);
  let at = now();
  record(&db, "api", at, api, api_ms).await?;
  record(&db, "warframestat", at, warframestat, warframestat_ms).await?;
  db.prepare("DELETE FROM status_checks WHERE checked_at < ?1").bind(&[number(at - KEEP_DAYS * 86_400)])?.run().await?;
  Ok(())
}

/// `GET /api/status`: for each service the last check, the uptime and the average latency of the last 90 days, and the
/// uptime of each day (UTC), oldest first.
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
      })).collect::<Vec<_>>(),
    })
  });
  let body = serde_json::json!({ "services": services.collect::<Vec<_>>() }).to_string();
  Ok(Reply::json(200, body, "public, max-age=60"))
}
