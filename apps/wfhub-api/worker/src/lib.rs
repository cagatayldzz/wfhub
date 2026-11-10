//! The WFHub API as a Cloudflare Worker, in front of the website: requests to `/api/...` come here, the rest is served
//! from the static assets. It is compiled **once** (`bun run api:build` puts the result in `dist/`, which is committed):
//! it holds no data. Everything it answers comes from static files of the site (`/data/...`, `/swagger/...`), so
//! changing the API is changing those files and deploying the site. The one thing it writes is feedback (D1, `DB`).
//!
//! The routes are in `wfhub_api::routes`, the feedback rules in `wfhub_api::feedback`; this file only fetches and stores.

use wfhub_api::feedback::{self, Feedback};
use wfhub_api::routes::{Reply, Route, error, item_reply, route};
use worker::wasm_bindgen::JsValue;
use worker::*;

/// A file from the assets; `None` when there is none.
async fn asset(env: &Env, path: &str) -> Result<Option<String>> {
  let mut response = env.assets("ASSETS")?.fetch(format!("https://assets.invalid{path}"), None).await?;
  if response.status_code() != 200 {
    return Ok(None);
  }
  Ok(Some(response.text().await?))
}

fn respond(reply: Reply, head: bool) -> Result<Response> {
  let headers = Headers::new();
  headers.set("content-type", reply.content_type)?;
  headers.set("cache-control", reply.cache)?;
  headers.set("access-control-allow-origin", "*")?;
  headers.set("x-content-type-options", "nosniff")?;
  let response = if head { Response::empty()? } else { Response::ok(reply.body)? };
  Ok(response.with_status(reply.status).with_headers(headers))
}

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
    .bind(&[
      JsValue::from_str(key),
      number(now),
      number(window_start),
      number(window_start),
      number(window_start),
      number(feedback::RATE_LIMIT),
    ])?
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

async fn submit(mut req: Request, env: &Env) -> Result<Reply> {
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
  if let Ok(statement) = db
    .prepare("DELETE FROM feedback_rate_limits WHERE window_start < ?")
    .bind(&[JsValue::from_f64((now - feedback::RATE_RETENTION_SECONDS) as f64)])
  {
    let _ = statement.run().await;
  }
  Ok(Reply::json(201, "{\"ok\":true}".into(), "no-store"))
}

async fn answer(req: Request, env: &Env, url: &Url, method: &str) -> Result<Reply> {
  let route = match route(method, url.path()) {
    Ok(route) => route,
    Err(reply) => return Ok(reply),
  };
  Ok(match route {
    Route::Health => Reply::json(200, "\"ok\"".into(), "no-store"),
    Route::Feedback => match submit(req, env).await {
      Ok(reply) => reply,
      Err(_) => error(503, "Service unavailable"),
    },
    Route::Redirect(_) => unreachable!("redirects are answered before"),
    Route::File { path, content_type, cache } => match asset(env, &path).await? {
      Some(body) => Reply { status: 200, body, cache, content_type },
      None => wfhub_api::routes::not_found(),
    },
    Route::Item { bucket, slug } => item_reply(asset(env, &bucket).await?.as_deref(), &slug),
  })
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let url = req.url()?;
  let head = req.method() == Method::Head;
  let method = if head { "GET".to_string() } else { req.method().to_string() };
  if let Ok(Route::Redirect(to)) = route(&method, url.path()) {
    return Response::redirect_with_status(url.join(to)?, 308);
  }
  respond(answer(req, &env, &url, &method).await?, head)
}
