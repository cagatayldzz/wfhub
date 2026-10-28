use super::*;

pub(super) fn settings_path() -> Option<std::path::PathBuf> {
  std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config/wfhub/settings.json"))
}

pub(super) fn platform_code(value: &str) -> &'static str {
  match value {
    "xb1" => "xb1",
    "swi" => "swi",
    "ps4" => "ps4",
    _ => "pc",
  }
}

pub(super) fn platform_label(value: &str) -> &'static str {
  match value {
    "xb1" => "Xbox",
    "swi" => "Nintendo Switch",
    "ps4" => "PlayStation",
    _ => "PC",
  }
}

pub(super) fn activity_routes() -> Vec<(&'static str, &'static str)> {
  let order = [
    "alerts",
    "voidTrader",
    "fissures",
    "cycles",
    "dailyDeals",
    "nightwave",
    "events",
    "invasions",
    "arbitration",
    "archonHunt",
    "steelPath",
    "sentientOutposts",
    "simaris",
    "constructionProgress",
    "news",
    "flashSales",
    "sortie",
  ];
  order
    .into_iter()
    .filter_map(|endpoint| {
      if endpoint == "cycles" { Some(("Cycles", endpoint)) } else { wfhub_core::ACTIVITIES.iter().copied().find(|(_, route)| *route == endpoint) }
    })
    .collect()
}

pub(super) fn activity_image(endpoint: &str) -> String {
  let name = match endpoint {
    "voidTrader" => "baro-ki-teer",
    "dailyDeals" => "void-trader",
    "archonHunt" => "archon-hunt",
    "steelPath" => "steel-path",
    "sentientOutposts" => "sentient-outpost",
    "constructionProgress" => "construction-progress",
    "flashSales" => "market",
    _ => endpoint,
  };
  format!("https://cdn.cagatayldzz.com/constants/wfhub/activities/{name}.png")
}

pub(super) fn title_case(value: &str) -> String {
  value
    .replace('-', " ")
    .split_whitespace()
    .map(|part| {
      let mut chars = part.chars();
      chars.next().map(|first| first.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
    })
    .collect::<Vec<_>>()
    .join(" ")
}

pub(super) fn label(item: &Value) -> String {
  for key in ["item", "name", "title", "node", "message", "description"] {
    if let Some(value) = item.get(key).and_then(Value::as_str).filter(|value| !value.is_empty()) {
      return value.to_string();
    }
  }
  "Details".into()
}

pub(super) fn fetch(url: &str) -> Result<Value, String> {
  let agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(30))).build().new_agent();
  let response = agent.get(url).header("Accept", "application/json").call().map_err(|error| error.to_string())?;
  serde_json::from_reader(response.into_body().into_reader()).map_err(|error| error.to_string())
}

pub(super) fn countdown(value: Option<&Value>) -> String {
  let Some(seconds) = remaining_seconds(value) else { return String::new() };
  format!("{}d {}h {}m", seconds / 86400, seconds % 86400 / 3600, seconds % 3600 / 60)
}

pub(super) fn timer_text(value: Option<&Value>) -> String {
  let Some(seconds) = remaining_seconds(value).filter(|seconds| *seconds > 0) else { return String::new() };
  if seconds >= 3600 {
    format!("{}:{:02}:{:02}", seconds / 3600, seconds % 3600 / 60, seconds % 60)
  } else {
    format!("{}:{:02}", seconds / 60, seconds % 60)
  }
}

pub(super) fn remaining_seconds(value: Option<&Value>) -> Option<i64> {
  let date = value.and_then(Value::as_str)?;
  let parse = || -> Option<i64> {
    let year: i64 = date.get(0..4)?.parse().ok()?;
    let month: i64 = date.get(5..7)?.parse().ok()?;
    let day: i64 = date.get(8..10)?.parse().ok()?;
    let hour: i64 = date.get(11..13)?.parse().ok()?;
    let minute: i64 = date.get(14..16)?.parse().ok()?;
    let second: i64 = date.get(17..19)?.parse().ok()?;
    if !date.ends_with('Z') || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
      return None;
    }
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted_month + 2) / 5 + day - 1;
    let days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    Some(days * 86400 + hour * 3600 + minute * 60 + second)
  };
  let expiry = parse()?;
  let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
  Some((expiry - now).max(0))
}

pub(super) fn image_path(url: &str) -> std::path::PathBuf {
  use std::hash::{Hash, Hasher};
  let mut hash = std::collections::hash_map::DefaultHasher::new();
  url.hash(&mut hash);
  std::env::temp_dir().join("wfhub-images").join(format!("{:016x}.png", hash.finish()))
}

pub(super) fn cache_image(url: Option<&str>) {
  let Some(url) = url.filter(|url| url.starts_with("https://")) else { return };
  let path = image_path(url);
  if path.exists() {
    return;
  }
  let agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(5))).build().new_agent();
  if let Ok(mut response) = agent.get(url).call()
    && let Ok(bytes) = response.body_mut().read_to_vec()
  {
    if let Some(parent) = path.parent() {
      let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, bytes);
  }
}
