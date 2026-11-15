//! The WFHub API as a Cloudflare Worker, in front of the website: requests to `/api/...` come here, the rest is served
//! from the static assets. It is Rust compiled to WASM (`npm run api:build`; the result in `dist/` is
//! committed, `cf:deploy` checks it is fresh): it holds no data. Everything it answers comes from static files of the site (`/data/...`, `/swagger/...`), so
//! changing the API is changing those files and deploying the site. The one thing it writes is feedback (D1, `DB`).
//!
//! The live game state (`/api/worldstate/...`) is the one thing that is not a file: it is fetched from the game, cached
//! (the last good copy is kept for when the game's server is down) and parsed by `wfhub_api::worldstate`.
//!
//! The routes are in `wfhub_api::routes`, the feedback rules in `wfhub_api::feedback`; this file only fetches and stores.

mod access;
mod feedback;
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
    Route::Redirect(_) | Route::WorldState { .. } => unreachable!("redirects and the game state are answered before"),
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
  if let Ok(Route::WorldState { platform, url: upstream, section }) = route(&method, url.path()) {
    let (reply, stale) = match worldstate::game_state(&env, platform, upstream, section).await {
      Ok(answer) => answer,
      Err(error) => worldstate::unavailable(&error.to_string()),
    };
    let response = respond(reply, head, true)?;
    if stale {
      response.headers().set("x-worldstate-stale", "1")?;
    }
    return Ok(response);
  }
  let public = !url.path().starts_with("/api/admin/");
  respond(answer(req, &env, &url, &method).await?, head, public)
}
