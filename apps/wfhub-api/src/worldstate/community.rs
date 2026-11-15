//! Events, news, Nightwave, Simaris and the relay construction progress.

use super::names::{Names, humanize};
use super::value::{array, eta, iso, ms, number, oid, percent, reward, text, timed};
use serde_json::{Map, Value, json};

pub(super) fn events(raw: &Value, now: i64, names: &Names) -> Value {
  let list = array(raw, "Goals")
    .iter()
    .filter(|goal| goal.get("Expiry").and_then(ms).is_none_or(|expiry| expiry >= now))
    .map(|goal| {
      let (activation, expiry) = (goal.get("Activation").and_then(ms), goal.get("Expiry").and_then(ms));
      let mut map = timed(oid(goal), activation, expiry, now);
      let description = names.lookup(text(goal, "Desc")).map(str::to_string).unwrap_or_else(|| humanize(text(goal, "Desc")));
      map.insert("description".into(), json!(description));
      map.insert("tooltip".into(), json!(names.lookup(text(goal, "ToolTip")).unwrap_or("")));
      map.insert("tag".into(), json!(text(goal, "Tag")));
      map.insert("node".into(), json!(if text(goal, "Node").is_empty() { Value::Null } else { json!(names.node(text(goal, "Node"))) }));
      map.insert("faction".into(), json!(if text(goal, "Faction").is_empty() { Value::Null } else { json!(names.faction(text(goal, "Faction"))) }));
      map.insert("scoreLocator".into(), json!(names.lookup(text(goal, "ScoreLocTag")).unwrap_or("")));
      map.insert("maximumScore".into(), json!(number(goal, "Goal")));
      map.insert("currentScore".into(), json!(number(goal, "Count")));
      map.insert("health".into(), json!(goal.get("HealthPct").and_then(Value::as_f64).map(|h| (h * 10_000.0).round() / 100.0)));
      map.insert("isPersonal".into(), json!(goal.get("Personal").and_then(Value::as_bool).unwrap_or(false)));
      map.insert("isCommunity".into(), json!(goal.get("Community").and_then(Value::as_bool).unwrap_or(false)));
      let rewards: Vec<Value> = goal.get("Reward").map(|r| vec![reward(r, names)]).unwrap_or_default();
      map.insert("rewards".into(), json!(rewards));
      Value::Object(map)
    })
    .collect();
  Value::Array(list)
}

pub(super) fn news(raw: &Value, now: i64, names: &Names) -> Value {
  let mut list: Vec<(i64, Value)> = array(raw, "Events")
    .iter()
    .filter_map(|event| {
      let messages = event.get("Messages").and_then(Value::as_array)?;
      let mut translations = Map::new();
      for message in messages {
        let (language, path) = (text(message, "LanguageCode"), text(message, "Message"));
        if language == "en" {
          let value = if path.starts_with("/Lotus/") { names.text(path) } else { path.to_string() };
          translations.insert("en".into(), json!(value));
        }
      }
      let english = translations.get("en")?.as_str()?.to_string();
      let date = event.get("Date").and_then(ms).unwrap_or(0);
      let mut map = timed(oid(event), None, None, now);
      map.remove("active");
      map.insert("message".into(), json!(english));
      map.insert("link".into(), json!(text(event, "Prop")));
      map.insert(
        "imageLink".into(),
        json!(if text(event, "ImageUrl").is_empty() { "https://i.imgur.com/5ZXFzOw.png" } else { text(event, "ImageUrl") }),
      );
      map.insert("priority".into(), json!(event.get("Priority").and_then(Value::as_bool).unwrap_or(false)));
      map.insert("date".into(), json!(iso(date)));
      map.insert("eta".into(), json!(eta(date - now)));
      map.insert("update".into(), json!(english.to_ascii_lowercase().contains("update") || english.to_ascii_lowercase().contains("hotfix")));
      map.insert("primeAccess".into(), json!(english.to_ascii_lowercase().contains("prime access")));
      map.insert("stream".into(), json!(english.to_ascii_lowercase().contains("devstream") || english.to_ascii_lowercase().contains("prime time")));
      map.insert("translations".into(), Value::Object(translations));
      map.insert("asString".into(), json!(format!("{} - {}", eta(date - now), english)));
      Some((date, Value::Object(map)))
    })
    .collect();
  list.sort_by_key(|entry| std::cmp::Reverse(entry.0));
  Value::Array(list.into_iter().map(|(_, value)| value).collect())
}

pub(super) fn nightwave(raw: &Value, now: i64, names: &Names) -> Value {
  let Some(season) = raw.get("SeasonInfo") else { return json!({}) };
  let (activation, expiry) = (season.get("Activation").and_then(ms), season.get("Expiry").and_then(ms));
  let mut map = timed(format!("nightwave{}", number(season, "Season")), activation, expiry, now);
  let challenges: Vec<Value> = season
    .get("ActiveChallenges")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .filter(|challenge| challenge.get("Expiry").and_then(ms).is_none_or(|e| e > now))
    .map(|challenge| {
      let path = text(challenge, "Challenge");
      let daily = challenge.get("Daily").and_then(Value::as_bool).unwrap_or(false);
      let elite = path.contains("WeeklyHard");
      let (a, e) = (challenge.get("Activation").and_then(ms), challenge.get("Expiry").and_then(ms));
      let mut entry = timed(oid(challenge), a, e, now);
      let title = names.lookup(path).map(str::to_string).unwrap_or_else(|| {
        let last = path.rsplit('/').next().unwrap_or(path);
        humanize(
          last.strip_prefix("Season").unwrap_or(last).trim_start_matches("Daily").trim_start_matches("WeeklyHard").trim_start_matches("Weekly"),
        )
      });
      entry.insert("isDaily".into(), json!(daily));
      entry.insert("isElite".into(), json!(elite));
      entry.insert("title".into(), json!(title));
      entry.insert("desc".into(), json!(names.desc(path).unwrap_or("")));
      entry.insert(
        "reputation".into(),
        json!(if daily {
          1000
        } else if elite {
          7000
        } else {
          4500
        }),
      );
      Value::Object(entry)
    })
    .collect();
  map.insert("season".into(), json!(number(season, "Season")));
  map.insert("tag".into(), json!(text(season, "AffiliationTag")));
  map.insert("phase".into(), json!(number(season, "Phase")));
  map.insert("params".into(), json!(text(season, "Params")));
  map.insert("possibleChallenges".into(), json!([]));
  map.insert("activeChallenges".into(), json!(challenges));
  Value::Object(map)
}

pub(super) fn simaris(raw: &Value, names: &Names) -> Value {
  let target = raw.get("LibraryInfo").map(|info| text(info, "LastCompletedTargetType")).unwrap_or("");
  json!({
    "target": if target.is_empty() { "N/A".to_string() } else { names.text(target) },
    "isTargetActive": false,
    "asString": if target.is_empty() { "No active target".to_string() } else { format!("Last completed: {}", names.text(target)) },
  })
}

pub(super) fn construction(raw: &Value, now: i64) -> Value {
  let progress = raw.get("ProjectPct").and_then(Value::as_array).cloned().unwrap_or_default();
  let at = |index: usize| percent(progress.get(index).and_then(Value::as_f64).unwrap_or(0.0));
  json!({
    "id": format!("constructionprogress{}", now / 60_000),
    "fomorianProgress": at(0),
    "razorbackProgress": at(1),
    "unknownProgress": at(2),
  })
}
