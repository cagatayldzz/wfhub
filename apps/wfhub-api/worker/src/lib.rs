//! The WFHub API as a Cloudflare Worker, in front of the website: requests to `/api/...` come here, the rest is served
//! from the static assets. It is Rust compiled to WASM (`npm run api:build`; the result in `dist/` is
//! committed, `cf:deploy` checks it is fresh): it holds no data. Everything it answers comes from static files of the site (`/data/...`, `/swagger/...`), so
//! changing the API is changing those files and deploying the site. It writes feedback and the status checks (D1, `DB`; the checks run from a cron trigger every 5 minutes, see `status.rs`).
//!
//! The live game state (`/api/worldstate/...`) is the one thing that is not a file: the Worker fetches it from the game
//! through a proxy when its copy (D1 + a 5 minute cache) is old, and parses it with `wfhub_api::worldstate` (see `worldstate.rs`).
//!
//! The routes are in `wfhub_api::routes`, the feedback rules in `wfhub_api::feedback`; this file only fetches and stores.

mod access;
mod feedback;
mod status;
mod worldstate;

use wfhub_api::routes::{Reply, Route, error, item_reply, route};
use worker::*;

/// A file from the assets; `None` when there is none.
pub(crate) async fn asset(env: &Env, path: &str) -> Result<Option<String>> {
  let mut response = env.assets("ASSETS")?.fetch(format!("https://assets.invalid{path}"), None).await?;
  if response.status_code() != 200 {
    return Ok(None);
  }
  Ok(Some(response.text().await?))
}

fn respond(reply: Reply, head: bool, public: bool) -> Result<Response> {
  let headers = Headers::new();
  headers.set("content-type", reply.content_type)?;
  headers.set("cache-control", reply.cache)?;
  if public {
    headers.set("access-control-allow-origin", "*")?;
  }
  headers.set("x-content-type-options", "nosniff")?;
  let response = if head { Response::empty()? } else { Response::ok(reply.body)? };
  Ok(response.with_status(reply.status).with_headers(headers))
}

async fn answer(req: Request, env: &Env, url: &Url, method: &str) -> Result<Reply> {
  if url.path() == "/api/admin/feedback" {
    return if method == "GET" { feedback::list_feedback(&req, env).await } else { Ok(wfhub_api::routes::method_not_allowed()) };
  }
  let route = match route(method, url.path()) {
    Ok(route) => route,
    Err(reply) => return Ok(reply),
  };
  Ok(match route {
    Route::Health => Reply::json(200, "\"ok\"".into(), "no-store"),
    Route::Feedback => match feedback::submit(req, env).await {
      Ok(reply) => reply,
      Err(_) => error(503, "Service unavailable"),
    },
    Route::Status => status::report(env).await?,
    Route::Redirect(_) | Route::WorldState { .. } => unreachable!("redirects and the game state are answered before"),
    Route::File { path, content_type, cache } => match asset(env, &path).await? {
      Some(body) => Reply { status: 200, body, cache, content_type },
      None => wfhub_api::routes::not_found(),
    },
    Route::Item { bucket, slug } => item_reply(asset(env, &bucket).await?.as_deref(), &slug),
  })
}

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
  // Every minute: keep the game state at most ~4 minutes old, so requests never wait for the game. Every 5th minute: the status
  // checks (they read the state too, so they are made after it was refreshed).
  worldstate::keep_fresh(&env, 240).await;
  if (event.schedule() as u64 / 60_000) % 5 == 0
    && let Err(error) = status::run_checks(&env).await
  {
    console_error!("status checks failed: {error}");
  }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
  let url = req.url()?;
  let head = req.method() == Method::Head;
  let method = if head { "GET".to_string() } else { req.method().to_string() };
  if let Ok(Route::Redirect(to)) = route(&method, url.path()) {
    return Response::redirect_with_status(url.join(to)?, 308);
  }
  if let Ok(Route::WorldState { platform, section }) = route(&method, url.path()) {
    let (reply, age) = match worldstate::game_state(&env, platform, section, Some(&ctx)).await {
      Ok(answer) => answer,
      Err(error) => worldstate::unavailable(&error.to_string()),
    };
    let response = respond(reply, head, true)?;
    if let Some(age) = age {
      response.headers().set("x-worldstate-age", &age.to_string())?;
      if age > worldstate::STALE_AFTER_SECONDS {
        response.headers().set("x-worldstate-stale", "1")?;
      }
    }
    return Ok(response);
  }
  let public = !url.path().starts_with("/api/admin/");
  respond(answer(req, &env, &url, &method).await?, head, public)
}
