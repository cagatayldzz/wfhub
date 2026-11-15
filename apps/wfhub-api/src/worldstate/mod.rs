//! Reads Warframe's `worldState.php` (the raw game state) and turns it into the JSON the WFHub app shows, in the shape of
//! the WFCD `warframe-status` API (`api.warframestat.us`), so the app only changes its base URL.
//!
//! Everything here is a pure function of the raw state, the current time (`now`, in milliseconds) and a [`Names`]
//! table (node, mission and item names, made by `export-worldstate` into `data/worldstate`). The Worker (`worker/`) only
//! fetches the raw state, loads the name tables and caches the answer.
//!
//! The raw state has paths such as `/Lotus/Types/...` and node codes such as `SolNode129`; [`collect_paths`] lists the
//! paths of one state so the Worker loads only the name shards that state needs.

mod community;
mod cycles;
mod missions;
mod names;
mod store;
mod value;

pub use names::{Names, SHARDS, collect_paths, desc_key, humanize, normalize, shard_of, shards_needed};
pub use value::{eta, iso, ms};

use serde_json::{Map, Value, json};

/// The platforms of the game.
pub const PLATFORMS: [&str; 4] = ["pc", "xb1", "ps4", "swi"];

/// The host of the game's server that has the raw state of a platform (`https://{host}/cdn/worldState.php`).
pub fn host(platform: &str) -> Option<&'static str> {
  Some(match platform {
    "pc" => "api.warframe.com",
    "xb1" => "api-xb1.warframe.com",
    "ps4" => "api-ps4.warframe.com",
    "swi" => "api-swi.warframe.com",
    _ => return None,
  })
}

/// The platform of a request path (`ps5` is `ps4`, `mob` is `pc`).
pub fn platform(key: &str) -> Option<&'static str> {
  let key = match key {
    "ps5" => "ps4",
    "mob" => "pc",
    other => other,
  };
  PLATFORMS.iter().find(|name| **name == key).copied()
}

/// Every endpoint under `/api/worldstate/{platform}/`.
pub const SECTIONS: [&str; 22] = [
  "alerts",
  "arbitration",
  "archonHunt",
  "cambionCycle",
  "cetusCycle",
  "constructionProgress",
  "dailyDeals",
  "duviriCycle",
  "earthCycle",
  "events",
  "fissures",
  "flashSales",
  "invasions",
  "news",
  "nightwave",
  "sentientOutposts",
  "simaris",
  "sortie",
  "steelPath",
  "vallisCycle",
  "voidTrader",
  "zarimanCycle",
];

/// The answer of one endpoint (`None` for a name that is not an endpoint).
pub fn parse_section(raw: &Value, section: &str, now: i64, names: &Names) -> Option<Value> {
  Some(match section {
    "alerts" => missions::alerts(raw, now, names),
    "arbitration" => missions::arbitration(),
    "archonHunt" => missions::archon_hunt(raw, now, names),
    "cambionCycle" => cycles::cambion(raw, now),
    "cetusCycle" => cycles::cetus(raw, now),
    "constructionProgress" => community::construction(raw, now),
    "dailyDeals" => store::daily_deals(raw, now, names),
    "duviriCycle" => cycles::duviri(raw, now),
    "earthCycle" => cycles::earth(now),
    "events" => community::events(raw, now, names),
    "fissures" => missions::fissures(raw, now, names),
    "flashSales" => store::flash_sales(raw, now, names),
    "invasions" => missions::invasions(raw, now, names),
    "news" => community::news(raw, now, names),
    "nightwave" => community::nightwave(raw, now, names),
    "sentientOutposts" => missions::sentient_outposts(raw, now, names),
    "simaris" => community::simaris(raw, names),
    "sortie" => missions::sortie(raw, now, names),
    "steelPath" => store::steel_path(now, names),
    "vallisCycle" => cycles::vallis(now),
    "voidTrader" => store::void_trader(raw, now, names),
    "zarimanCycle" => cycles::zariman(raw, now),
    _ => return None,
  })
}

/// All endpoints in one object.
pub fn parse_all(raw: &Value, now: i64, names: &Names) -> Value {
  let mut all = Map::new();
  all.insert("timestamp".into(), json!(iso(now)));
  for section in SECTIONS {
    if let Some(value) = parse_section(raw, section, now, names) {
      all.insert(section.into(), value);
    }
  }
  Value::Object(all)
}
