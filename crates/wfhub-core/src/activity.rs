use crate::text::{scalar, title_of};
use crate::{CoreError, Json, challenge_slug, field_label, is_web_link, num_of, parse, public_url, show, str_of};

#[derive(Clone, Debug, uniffi::Record)]
pub struct InfoField {
  pub label: String,
  pub value: String,
}

/// A sub row of an entry: a fissure variant, a Nightwave challenge or a Baro item.
#[derive(Clone, Debug, uniffi::Record)]
pub struct SubRow {
  pub title: String,
  pub detail: String,
  pub trailing: Option<String>,
  /// Daily, Weekly or Elite Weekly (Nightwave).
  pub kind: Option<String>,
  pub image: Option<String>,
}

/// One card of an activity screen.
#[derive(Clone, Debug, uniffi::Record)]
pub struct ActivityEntry {
  pub title: String,
  pub detail: Option<String>,
  /// Fissure tier, upper case.
  pub badge: Option<String>,
  /// Show a countdown when the date is still in the future (`expiry_unix`).
  pub expiry: Option<String>,
  pub fields: Vec<InfoField>,
  pub sections: Vec<SubRow>,
  /// Extra lines (mission type, rewards).
  pub notes: Vec<String>,
  pub date: Option<String>,
  pub image: Option<String>,
  pub link: Option<String>,
  /// Lower case text the search matches besides the title.
  pub search: String,
  pub tier: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct BuildProgress {
  pub title: String,
  pub faction: String,
  /// 0...100, `None` when the server did not send a valid number.
  pub percent: Option<f64>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct PageHero {
  pub season: Option<i64>,
  pub expiry: Option<String>,
  pub image: String,
}

/// What an activity screen shows. `tiers` is only filled for fissures; `searchable` says whether to show a search field.
#[derive(Clone, Debug, uniffi::Record)]
pub struct ActivityPage {
  pub entries: Vec<ActivityEntry>,
  pub hero: Option<PageHero>,
  pub progress: Vec<BuildProgress>,
  pub tiers: Vec<String>,
  pub searchable: bool,
}

const FIELD_KEYS: [&str; 9] = ["node", "enemy", "faction", "target", "item", "discount", "price", "fomorianProgress", "razorbackProgress"];
const DETAIL_KEYS: [&str; 7] = ["missionType", "faction", "location", "state", "desc", "modifier", "description"];

fn owned(value: &Json, key: &str) -> Option<String> {
  str_of(value, key).map(str::to_string)
}

fn primary_title(endpoint: &str, value: &Json) -> String {
  match endpoint {
    "alerts" if value.get("mission").is_some_and(Json::is_object) => {
      value["mission"].get("node").and_then(Json::as_str).unwrap_or("Alert").to_string()
    }
    "sortie" | "archonHunt" => owned(value, "boss").unwrap_or_else(|| "Mission".into()),
    "nightwave" => "Nightwave".into(),
    "voidTrader" => "Baro Ki'Teer".into(),
    _ => title_of(value),
  }
}

fn sections(value: &Json) -> Vec<SubRow> {
  let list = |key: &str| value.get(key).and_then(Json::as_array).cloned().unwrap_or_default();
  let plain = |title: String, detail: String, trailing: Option<String>| SubRow { title, detail, trailing, kind: None, image: None };
  let mut out = Vec::new();
  for (index, variant) in list("variants").iter().enumerate() {
    let title = owned(variant, "node").or_else(|| owned(variant, "nodeKey")).unwrap_or_else(|| "Mission".into());
    let detail = owned(variant, "modifier").or_else(|| owned(variant, "missionType")).unwrap_or_default();
    out.push(plain(title, detail, Some((index + 1).to_string())));
  }
  for challenge in list("activeChallenges") {
    let title = owned(&challenge, "title").unwrap_or_else(|| "Challenge".into());
    let kind = if challenge.get("isDaily").and_then(Json::as_bool) == Some(true) {
      "Daily"
    } else if challenge.get("isElite").and_then(Json::as_bool) == Some(true) {
      "Elite Weekly"
    } else {
      "Weekly"
    };
    out.push(SubRow {
      image: Some(public_url(format!("nightwave/{}.png", challenge_slug(title.clone())))),
      detail: owned(&challenge, "desc").unwrap_or_default(),
      trailing: num_of(&challenge, "reputation").map(|points| (points as i64).to_string()),
      kind: Some(kind.into()),
      title,
    });
  }
  for entry in list("inventory") {
    let ducats = entry.get("ducats").and_then(Json::as_i64).unwrap_or(0);
    out.push(plain(owned(&entry, "item").unwrap_or_else(|| "Item".into()), String::new(), Some(format!("{ducats} ◈"))));
  }
  out
}

fn entry(endpoint: &str, value: &Json) -> ActivityEntry {
  let title = primary_title(endpoint, value);
  let detail = DETAIL_KEYS.iter().filter_map(|key| str_of(value, key)).find(|text| !text.is_empty() && *text != title).map(str::to_string);
  let fields = FIELD_KEYS
    .iter()
    .filter_map(|key| {
      let shown = value.get(*key).and_then(show)?;
      (shown != title && Some(&shown) != detail.as_ref()).then(|| InfoField { label: field_label((*key).into()), value: shown })
    })
    .collect();
  let mut notes = Vec::new();
  if let Some(mission) = value.get("mission").filter(|m| m.is_object()) {
    notes.extend(owned(mission, "type"));
    let items: Vec<&str> =
      mission.pointer("/reward/items").and_then(Json::as_array).map(|a| a.iter().filter_map(Json::as_str).collect()).unwrap_or_default();
    if !items.is_empty() {
      notes.push(items.join(", "));
    }
  }
  ActivityEntry {
    badge: owned(value, "tier").map(|tier| tier.to_uppercase()),
    tier: owned(value, "tier"),
    expiry: owned(value, "expiry"),
    fields,
    sections: sections(value),
    notes,
    date: owned(value, "date"),
    image: owned(value, "imageLink"),
    link: owned(value, "link").filter(|link| is_web_link(link.clone())),
    search: scalar(value, "item").unwrap_or_default().to_lowercase(),
    title,
    detail,
  }
}

fn percent(value: &Json, key: &str) -> Option<f64> {
  num_of(value, key).filter(|n| n.is_finite() && (0.0..=100.0).contains(n))
}

/// Reads the answer of `activity_url`.
#[uniffi::export]
pub fn parse_activity(endpoint: String, body: String) -> Result<ActivityPage, CoreError> {
  let mut page = ActivityPage { entries: vec![], hero: None, progress: vec![], tiers: vec![], searchable: false };
  match parse(&body)? {
    Json::Array(mut items) => {
      match endpoint.as_str() {
        "news" => items.sort_by(|a, b| str_of(b, "date").unwrap_or("").cmp(str_of(a, "date").unwrap_or(""))),
        "flashSales" => items.sort_by(|a, b| str_of(a, "expiry").unwrap_or("").cmp(str_of(b, "expiry").unwrap_or(""))),
        _ => {}
      }
      page.entries = items.iter().filter(|item| item.is_object()).map(|item| entry(&endpoint, item)).collect();
      if endpoint == "fissures" {
        page.tiers = ["All", "Lith", "Meso", "Neo", "Axi", "Requiem", "Omnia"].map(String::from).to_vec();
      }
      page.searchable = endpoint != "fissures" && endpoint != "news";
    }
    value @ Json::Object(_) => {
      match endpoint.as_str() {
        "nightwave" => {
          page.hero = Some(PageHero {
            season: num_of(&value, "season").map(|s| s as i64),
            expiry: owned(&value, "expiry"),
            image: public_url("sections/nightwave.jpg".into()),
          })
        }
        "constructionProgress" => {
          page.progress = vec![
            BuildProgress { title: "Balor Fomorian".into(), faction: "Grineer".into(), percent: percent(&value, "fomorianProgress") },
            BuildProgress { title: "Razorback Armada".into(), faction: "Corpus".into(), percent: percent(&value, "razorbackProgress") },
          ];
        }
        _ => {}
      }
      if page.progress.is_empty() {
        page.entries = vec![entry(&endpoint, &value)];
      }
    }
    _ => return Err(CoreError::Shape),
  }
  Ok(page)
}

/// Applies the search text, or the tier of fissures ("All" keeps everything).
#[uniffi::export]
pub fn filter_entries(entries: Vec<ActivityEntry>, query: String, tier: String) -> Vec<ActivityEntry> {
  let query = query.to_lowercase();
  entries
    .into_iter()
    .filter(|entry| {
      if tier != "All" && !tier.is_empty() {
        return entry.tier.as_deref().is_some_and(|t| t.eq_ignore_ascii_case(&tier));
      }
      query.is_empty() || entry.title.to_lowercase().contains(&query) || entry.search.contains(&query)
    })
    .collect()
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct CycleState {
  pub state: String,
  pub expiry: Option<String>,
}

/// One cycle (`cetusCycle` and the like); the state is capitalised, translate it with `translate_data`.
#[uniffi::export]
pub fn parse_cycle(body: String) -> Result<CycleState, CoreError> {
  let value = parse(&body)?;
  if !value.is_object() {
    return Err(CoreError::Shape);
  }
  let state = owned(&value, "state").map(|s| crate::routes::capitalized(&s)).unwrap_or_else(|| "–".into());
  Ok(CycleState { state, expiry: owned(&value, "expiry") })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fissures_filter_by_tier() {
    let page = parse_activity(
      "fissures".into(),
      r#"[{"node":"A","tier":"Lith","missionType":"Capture"},{"node":"B","tier":"Axi","missionType":"Spy"}]"#.into(),
    )
    .unwrap();
    assert_eq!(page.tiers.len(), 7);
    assert!(!page.searchable);
    assert_eq!(page.entries[0].badge.as_deref(), Some("LITH"));
    assert_eq!(page.entries[0].detail.as_deref(), Some("Capture"));
    assert_eq!(filter_entries(page.entries.clone(), String::new(), "axi".into()).len(), 1);
    assert_eq!(filter_entries(page.entries, String::new(), "All".into()).len(), 2);
  }

  #[test]
  fn news_is_newest_first_and_links_are_checked() {
    let page = parse_activity(
      "news".into(),
      r#"[{"date":"1","message":"a"},{"date":"2","message":"b","link":"https://x"},{"date":"0","message":"c","link":"file:///"}]"#.into(),
    )
    .unwrap();
    assert_eq!(page.entries.iter().map(|e| e.title.as_str()).collect::<Vec<_>>(), ["b", "a", "c"]);
    assert_eq!(page.entries[0].link.as_deref(), Some("https://x"));
    assert_eq!(page.entries[2].link, None);
  }

  #[test]
  fn nightwave_and_construction() {
    let page = parse_activity(
      "nightwave".into(),
      r#"{"season":5,"expiry":"2030-01-01T00:00:00Z","activeChallenges":[{"title":"Gorgon's Fury II","desc":"d","reputation":4500,"isElite":true}]}"#
        .into(),
    )
    .unwrap();
    assert_eq!(page.hero.unwrap().season, Some(5));
    let section = &page.entries[0].sections[0];
    assert_eq!(section.kind.as_deref(), Some("Elite Weekly"));
    assert_eq!(section.trailing.as_deref(), Some("4500"));
    assert!(section.image.as_deref().unwrap().ends_with("nightwave/gorgons-fury.png"));

    let page = parse_activity("constructionProgress".into(), r#"{"fomorianProgress":"56.5","razorbackProgress":"abc"}"#.into()).unwrap();
    assert_eq!(page.progress[0].percent, Some(56.5));
    assert_eq!(page.progress[1].percent, None);
  }

  #[test]
  fn rejects_wrong_shapes() {
    assert!(parse_activity("alerts".into(), "3".into()).is_err());
    assert!(parse_activity("alerts".into(), "not json".into()).is_err());
  }
}
