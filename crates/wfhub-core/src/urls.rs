//! Every address the clients call. Changing a path of the WFHub API breaks released apps, see `docs/data-and-integrations.md`.

const SITE: &str = "https://wfhub.cagatayldzz.com";
const WORLD_STATE: &str = "https://api.warframestat.us";
const MARKET: &str = "https://api.warframe.market/v2";

/// One request of the Home screen. `key` identifies the answer in `parse_home`.
#[derive(Clone, Debug, uniffi::Record)]
pub struct HomeRequest {
  pub key: String,
  pub url: String,
}

/// Supported platform codes. Unknown codes fall back to `pc` so a stale preference can not build a broken address.
fn platform_code(platform: &str) -> &str {
  if PLATFORMS.iter().any(|(code, _)| *code == platform) { platform } else { "pc" }
}

pub(crate) const PLATFORMS: &[(&str, &str)] = &[("pc", "PC"), ("xb1", "Xbox"), ("swi", "Nintendo Switch"), ("ps4", "PlayStation")];

#[uniffi::export]
pub fn activity_url(platform: String, endpoint: String) -> String {
  format!("{WORLD_STATE}/{}/{endpoint}/", platform_code(&platform))
}

#[uniffi::export]
pub fn category_url(category: String) -> String {
  format!("{SITE}/api/{category}.json")
}

#[uniffi::export]
pub fn item_url(category: String, slug: String) -> String {
  format!("{SITE}/api/{category}/{slug}.json")
}

#[uniffi::export]
pub fn feedback_url() -> String {
  format!("{SITE}/api/feedback")
}

#[uniffi::export]
pub fn market_orders_url() -> String {
  format!("{MARKET}/orders/recent")
}

#[uniffi::export]
pub fn market_items_url() -> String {
  format!("{MARKET}/items")
}

/// The requests the Home screen needs for `platform`.
#[uniffi::export]
pub fn home_requests(platform: String) -> Vec<HomeRequest> {
  ["nightwave", "voidTrader", "events", "alerts", "news"]
    .iter()
    .map(|key| HomeRequest { key: (*key).into(), url: activity_url(platform.clone(), (*key).into()) })
    .chain([HomeRequest { key: "banners".into(), url: format!("{SITE}/api/events.json") }])
    .collect()
}

/// Artwork of a WFHub section, activity or codex category on the site.
#[uniffi::export]
pub fn public_url(path: String) -> String {
  format!("{SITE}/public/{path}")
}

/// Icon of an activity in the Activities list.
#[uniffi::export]
pub fn activity_image_url(endpoint: String) -> String {
  let name = match endpoint.as_str() {
    "voidTrader" => "baro-ki-teer",
    "dailyDeals" => "void-trader",
    "archonHunt" => "archon-hunt",
    "steelPath" => "steel-path",
    "sentientOutposts" => "sentient-outpost",
    "constructionProgress" => "construction-progress",
    "flashSales" => "market",
    other => other,
  };
  public_url(format!("activities/{name}.png"))
}

#[uniffi::export]
pub fn category_image_url(category: String) -> String {
  public_url(format!("codex/{}.png", if category == "misc" { "gear" } else { &category }))
}

/// Image of a Codex entry: absolute addresses stay, file names live in the WFCD image repository.
#[uniffi::export]
pub fn codex_image_url(value: String) -> Option<String> {
  if value.is_empty() {
    None
  } else if value.contains("://") {
    Some(value)
  } else {
    Some(format!("https://raw.githubusercontent.com/WFCD/warframe-items/master/data/img/{}", percent_encode_path(&value)))
  }
}

pub(crate) fn percent_encode_path(value: &str) -> String {
  let mut out = String::new();
  for byte in value.bytes() {
    match byte {
      b'A'..=b'Z'
      | b'a'..=b'z'
      | b'0'..=b'9'
      | b'-'
      | b'.'
      | b'_'
      | b'~'
      | b'/'
      | b'!'
      | b'$'
      | b'&'
      | b'\''
      | b'('
      | b')'
      | b'*'
      | b'+'
      | b','
      | b';'
      | b'='
      | b':'
      | b'@' => out.push(byte as char),
      _ => out.push_str(&format!("%{byte:02X}")),
    }
  }
  out
}

/// Only web links are opened by the apps.
#[uniffi::export]
pub fn is_web_link(link: String) -> bool {
  let lower = link.to_ascii_lowercase();
  lower.starts_with("https://") || lower.starts_with("http://")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builds_addresses() {
    assert_eq!(activity_url("xb1".into(), "fissures".into()), "https://api.warframestat.us/xb1/fissures/");
    assert_eq!(activity_url("../x".into(), "alerts".into()), "https://api.warframestat.us/pc/alerts/");
    assert_eq!(item_url("mods".into(), "serration".into()), "https://wfhub.cagatayldzz.com/api/mods/serration.json");
    assert_eq!(home_requests("pc".into()).len(), 6);
  }

  #[test]
  fn codex_images() {
    assert_eq!(codex_image_url("https://x.y/a.png".into()).as_deref(), Some("https://x.y/a.png"));
    assert_eq!(codex_image_url("a b.png".into()).as_deref(), Some("https://raw.githubusercontent.com/WFCD/warframe-items/master/data/img/a%20b.png"));
    assert_eq!(codex_image_url(String::new()), None);
  }
}
