//! The status page's comparison: does the WFHub API (`/api/worldstate/pc.json`) show the same game state as
//! api.warframestat.us (`/pc`)? Both are made from the game's `worldState.php`, so the same things must be in both. What is
//! compared is *which* things (the `id` of each item), not how they are written: the two name and shape the fields differently.

use serde_json::Value;
use std::collections::BTreeSet;

/// The sections compared, those with an `id` per item (or one for the section).
pub const COMPARED: [&str; 9] = ["news", "events", "alerts", "sortie", "archonHunt", "voidTrader", "nightwave", "invasions", "fissures"];

/// Two copies are made up to ~5 minutes apart (each side caches 5 minutes), so an item that starts or ends within this many
/// seconds of now can be in one copy and not in the other. Those are left out.
const EDGE_SECONDS: i64 = 600;
/// The game's own clocks (`timestamp`) of the two copies may differ by this much.
const MAX_CLOCK_GAP_SECONDS: i64 = 15 * 60;

/// Seconds since the epoch of `YYYY-MM-DDTHH:MM:SS...Z`.
pub fn parse_iso(text: &str) -> Option<i64> {
  let number = |from: usize, to: usize| text.get(from..to)?.parse::<i64>().ok();
  let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
  let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
  // days_from_civil (Howard Hinnant)
  let year = if month <= 2 { year - 1 } else { year };
  let era = year.div_euclid(400);
  let year_of_era = year - era * 400;
  let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
  let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
  let days = era * 146_097 + day_of_era - 719_468;
  Some(days * 86_400 + hour * 3600 + minute * 60 + second)
}

/// What names an item: its `id`, except in `nightwave`, whose ids are written differently in the two (`nightwave18` and
/// `nightwave1804464000000`): the season is named by when it starts.
fn key(section: &str, item: &Value) -> Option<String> {
  let field = if section == "nightwave" { "activation" } else { "id" };
  item.get(field).and_then(Value::as_str).map(str::to_string)
}

fn ids(name: &str, section: &Value, now: i64) -> BTreeSet<String> {
  let items: Vec<&Value> = match section {
    Value::Array(items) => items.iter().collect(),
    Value::Object(_) => vec![section],
    _ => vec![],
  };
  items
    .into_iter()
    // api.warframestat.us also lists the invasions that are over (`completed`); this API only the running ones.
    .filter(|item| item.get("completed").and_then(Value::as_bool) != Some(true))
    .filter(|item| {
      let near = |key: &str| item.get(key).and_then(Value::as_str).and_then(parse_iso).is_some_and(|at| (at - now).abs() < EDGE_SECONDS);
      !(near("activation") || near("expiry"))
    })
    .filter_map(|item| key(name, item))
    .collect()
}

/// What differs between the two states, one line each (empty when they agree). `now` is in seconds.
pub fn differences(mine: &Value, theirs: &Value, now: i64) -> Vec<String> {
  let mut found = Vec::new();
  if let (Some(a), Some(b)) = (mine["timestamp"].as_str().and_then(parse_iso), theirs["timestamp"].as_str().and_then(parse_iso))
    && (a - b).abs() > MAX_CLOCK_GAP_SECONDS
  {
    found.push(format!("timestamp: {} min apart", (a - b).abs() / 60));
  }
  for section in COMPARED {
    let (a, b) = (&mine[section], &theirs[section]);
    if a.is_null() || b.is_null() {
      found.push(format!("{section}: missing in {}", if a.is_null() { "WFHub" } else { "Warframe Status" }));
      continue;
    }
    let (a, b) = (ids(section, a, now), ids(section, b, now));
    let (only_a, only_b) = (a.difference(&b).count(), b.difference(&a).count());
    if only_a + only_b > 0 {
      found.push(format!("{section}: {only_a} only in WFHub, {only_b} only in Warframe Status"));
    }
  }
  found
}
