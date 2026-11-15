//! Anonymous feedback: stored in D1 (`DB`) with a daily-keyed rate limit, and listed for the owner (Cloudflare Access).

use wfhub_api::feedback::{self, Feedback};
use wfhub_api::routes::{Reply, error};
use worker::wasm_bindgen::JsValue;
use worker::*;

/// Rate limit: the client may save `RATE_LIMIT` things per `RATE_WINDOW_SECONDS`; `true` when this one is let through.
async fn consume_rate_limit(db: &D1Database, key: &str, now: i64) -> Result<bool> {
  let window_start = now - feedback::RATE_WINDOW_SECONDS;
  let number = |value: i64| JsValue::from_f64(value as f64);
  let row = db
    .prepare(
      "INSERT INTO feedback_rate_limits (client_key, window_start, count) VALUES (?, ?, 1)
       ON CONFLICT(client_key) DO UPDATE SET
         count = CASE WHEN window_start <= ? THEN 1 ELSE count + 1 END,
         window_start = CASE WHEN window_start <= ? THEN excluded.window_start ELSE window_start END
       WHERE window_start <= ? OR count < ?
       RETURNING count",
    )
    .bind(&[JsValue::from_str(key), number(now), number(window_start), number(window_start), number(window_start), number(feedback::RATE_LIMIT)])?
    .first::<serde_json::Value>(None)
    .await?;
  Ok(row.is_some())
}

async fn save(db: &D1Database, feedback: &Feedback) -> Result<()> {
  db.prepare("INSERT INTO feedback (id, category, title, message) VALUES (lower(hex(randomblob(16))), ?, ?, ?)")
    .bind(&[JsValue::from_str(&feedback.category), JsValue::from_str(&feedback.title), JsValue::from_str(&feedback.message)])?
    .run()
    .await?;
  Ok(())
}

pub async fn submit(mut req: Request, env: &Env) -> Result<Reply> {
  let headers = req.headers();
  let content_type = headers.get("content-type")?;
  let content_length = headers.get("content-length")?;
  if let Err((status, message)) = feedback::check_headers(content_type.as_deref(), content_length.as_deref()) {
    return Ok(error(status, message));
  }
  let Some(ip) = headers.get("cf-connecting-ip")? else { return Ok(error(403, "Request unavailable")) };
  let feedback = match feedback::parse(&req.bytes().await?) {
    Ok(feedback) => feedback,
    Err((status, message)) => return Ok(error(status, message)),
  };
  let Ok(secret) = env.secret("FEEDBACK_RATE_LIMIT_SECRET") else { return Ok(error(503, "Service unavailable")) };
  let Ok(db) = env.d1("DB") else { return Ok(error(503, "Service unavailable")) };

  // The key changes every day, so the database never holds an address.
  let now = (Date::now().as_millis() / 1000) as i64;
  let iso = worker::js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default();
  let key = feedback::client_key(&secret.to_string(), iso.get(..10).unwrap_or_default(), &ip);
  if !consume_rate_limit(&db, &key, now).await? {
    return Ok(error(429, "Too many requests"));
  }
  save(&db, &feedback).await?;
  // A cleanup failure must not report a saved submission as failed.
  if let Ok(statement) =
    db.prepare("DELETE FROM feedback_rate_limits WHERE window_start < ?").bind(&[JsValue::from_f64((now - feedback::RATE_RETENTION_SECONDS) as f64)])
  {
    let _ = statement.run().await;
  }
  Ok(Reply::json(201, "{\"ok\":true}".into(), "no-store"))
}

/// `GET /api/admin/feedback`: the saved feedback, newest first. Only for the owner (Cloudflare Access).
pub async fn list_feedback(req: &Request, env: &Env) -> Result<Reply> {
  if !crate::access::is_authorized(req, env).await {
    return Ok(Reply { status: 403, body: "Forbidden\n".into(), cache: "no-store", content_type: "text/plain; charset=utf-8" });
  }
  let Ok(db) = env.d1("DB") else { return Ok(error(503, "Service unavailable")) };
  let rows = db
    .prepare("SELECT id, category, title, message, created_at FROM feedback ORDER BY created_at DESC LIMIT 500")
    .all()
    .await?
    .results::<serde_json::Value>()?;
  Ok(Reply::json(200, serde_json::Value::Array(rows).to_string(), "no-store"))
}
