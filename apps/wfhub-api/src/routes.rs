//! The routes of the API, as plain functions, so the Worker (compiled once, `worker/`) is only glue: it fetches static
//! files from the site's assets and hands their text to these. Paths here are the full request paths (with `/api`).
//!
//! | Route | Answer | Built from |
//! | --- | --- | --- |
//! | `GET /api/` | the Swagger page | `swagger/index.html` |
//! | `GET /api/swagger/openapi.json` | the OpenAPI description | `swagger/openapi.json` |
//! | `GET /api/health` | `"ok"` | |
//! | `GET /api/events.json` | the banners | `data/events.json` |
//! | `GET /api/mastery/{category}.json` | a mastery checklist snapshot | `data/mastery/{category}.json` |
//! | `GET /api/{category}.json` | the items of a Codex category | `data/catalogue/{category}.json` |
//! | `GET /api/{category}/{slug}.json` | one item | `data/catalogue/{category}/{first character}.json` |
//! | `POST /api/feedback` | stores feedback (D1) | `feedback` |

use serde_json::Value;

/// The Codex categories (the lowercase names of the WFCD files).
pub const CATALOGUE_CATEGORIES: [&str; 24] = [
  "arcanes",
  "arch-gun",
  "arch-melee",
  "archwing",
  "enemy",
  "fish",
  "gear",
  "glyphs",
  "melee",
  "misc",
  "mods",
  "node",
  "pets",
  "primary",
  "quests",
  "railjack",
  "relics",
  "resources",
  "secondary",
  "sentinels",
  "sentinelweapons",
  "sigils",
  "skins",
  "warframes",
];

/// The mastery snapshots (`data/mastery/{name}.json`).
pub const MASTERY_CATEGORIES: [&str; 25] = [
  "warframes",
  "primary",
  "secondary",
  "melee",
  "kitgun",
  "zaw",
  "amp",
  "sentinel",
  "moa",
  "hound",
  "beast",
  "robotic",
  "archwing",
  "archgun",
  "archmelee",
  "necramech",
  "railjack",
  "k-drive",
  "railjack-intrinsic",
  "drifter-intrinsic",
  "starchart",
  "steel-path",
  "founders",
  "china-founders",
  "updated",
];

/// What a route answers.
#[derive(Debug)]
pub struct Reply {
  pub status: u16,
  pub body: String,
  pub cache: &'static str,
  pub content_type: &'static str,
}

impl Reply {
  pub fn json(status: u16, body: String, cache: &'static str) -> Reply {
    Reply { status, body, cache, content_type: "application/json; charset=utf-8" }
  }
}

pub fn not_found() -> Reply {
  Reply { status: 404, body: "Not found\n".into(), cache: "no-store", content_type: "text/plain; charset=utf-8" }
}

pub fn method_not_allowed() -> Reply {
  Reply { status: 405, body: "Method not allowed\n".into(), cache: "no-store", content_type: "text/plain; charset=utf-8" }
}

/// `{"error": message}`, for the feedback route.
pub fn error(status: u16, message: &str) -> Reply {
  Reply::json(status, serde_json::json!({ "error": message }).to_string(), "no-store")
}

/// What a request asks for.
#[derive(Debug, PartialEq)]
pub enum Route {
  Health,
  /// Send the client to this path.
  Redirect(&'static str),
  /// A static file of the site, answered as it is.
  File {
    path: String,
    content_type: &'static str,
    cache: &'static str,
  },
  /// One item out of the bucket file that holds it.
  Item {
    bucket: String,
    slug: String,
  },
  Feedback,
}

const JSON: &str = "application/json; charset=utf-8";
const SHORT: &str = "public, max-age=300";

/// The first character of a slug names the file its item is in: `data/catalogue/{category}/{character}.json`.
pub fn bucket_name(slug: &str) -> Option<char> {
  slug.chars().next().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn is_slug(text: &str) -> bool {
  !text.is_empty() && text.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Reads a request: the method (`GET` stands for `HEAD` too) and a path such as `/api/warframes/excalibur.json`.
pub fn route(method: &str, path: &str) -> Result<Route, Reply> {
  if path == "/api/feedback" {
    return if method == "POST" { Ok(Route::Feedback) } else { Err(method_not_allowed()) };
  }
  if method != "GET" {
    return Err(method_not_allowed());
  }
  let file = |path: &str, content_type, cache| Ok(Route::File { path: path.into(), content_type, cache });
  match path {
    "/api" => return Ok(Route::Redirect("/api/")),
    "/api/swagger" => return Ok(Route::Redirect("/api/swagger/")),
    "/api/" | "/api/swagger/" | "/api/swagger/index.html" => return file("/swagger/index.html", "text/html; charset=utf-8", "no-cache"),
    "/api/swagger/openapi.json" => return file("/swagger/openapi.json", JSON, "no-cache"),
    "/api/swagger/init.js" => return file("/swagger/init.js", "text/javascript; charset=utf-8", "no-cache"),
    "/api/health" => return Ok(Route::Health),
    "/api/events.json" => return file("/data/events.json", JSON, SHORT),
    _ => {}
  }
  let rest = path.strip_prefix("/api/").ok_or_else(not_found)?;
  let rest = rest.strip_suffix(".json").ok_or_else(not_found)?;
  match rest.split('/').collect::<Vec<_>>().as_slice() {
    ["mastery", name] if MASTERY_CATEGORIES.contains(name) => file(&format!("/data/mastery/{name}.json"), JSON, SHORT),
    [category] if CATALOGUE_CATEGORIES.contains(category) => file(&format!("/data/catalogue/{category}.json"), JSON, SHORT),
    [category, slug] if CATALOGUE_CATEGORIES.contains(category) && is_slug(slug) => {
      let bucket = bucket_name(slug).ok_or_else(not_found)?;
      Ok(Route::Item { bucket: format!("/data/catalogue/{category}/{bucket}.json"), slug: slug.to_string() })
    }
    _ => Err(not_found()),
  }
}

/// The answer for one item: `bucket` is the text of the bucket file (an object of slug to item).
pub fn item_reply(bucket: Option<&str>, slug: &str) -> Reply {
  let item = bucket.and_then(|text| serde_json::from_str::<Value>(text).ok()).and_then(|mut bucket| bucket.get_mut(slug).map(Value::take));
  match item {
    Some(item) => Reply::json(200, item.to_string(), SHORT),
    None => not_found(),
  }
}
