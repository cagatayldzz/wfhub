//! The cycles: Cetus, Cambion Drift, Earth, Orb Vallis, Zariman and Duviri.

use super::value::{array, eta, ms, text, timed};
use serde_json::{Value, json};

pub(super) fn bounty_end(raw: &Value, tag: &str) -> i64 {
  array(raw, "SyndicateMissions")
    .iter()
    .find(|mission| text(mission, "Tag") == tag)
    .and_then(|mission| mission.get("Expiry"))
    .and_then(ms)
    .unwrap_or(0)
}

/// Cetus: the bounties end with the night, a whole cycle is 150 minutes (100 of day, 50 of night).
pub(super) fn cetus_state(raw: &Value, now: i64) -> (bool, i64, i64) {
  let end = bounty_end(raw, "CetusSyndicate") / 60_000 * 60_000;
  let mut seconds = ((end - now) as f64 / 1000.0).round() as i64;
  seconds = seconds.rem_euclid(9000);
  if seconds == 0 {
    seconds = 9000;
  }
  let day = seconds > 3000;
  let remaining = if day { seconds - 3000 } else { seconds };
  let expiry = ((now + remaining * 1000) as f64 / 60_000.0).round() as i64 * 60_000;
  let start = expiry - if day { 6_000_000 } else { 3_000_000 };
  (day, start, expiry)
}

pub(super) fn cetus(raw: &Value, now: i64) -> Value {
  let (day, start, expiry) = cetus_state(raw, now);
  let mut map = timed(format!("cetusCycle{expiry}"), Some(start), Some(expiry), now);
  map.insert("isDay".into(), json!(day));
  map.insert("state".into(), json!(if day { "day" } else { "night" }));
  map.insert("timeLeft".into(), json!(eta(expiry - now)));
  map.insert("isCetus".into(), json!(true));
  Value::Object(map)
}

pub(super) fn cambion(raw: &Value, now: i64) -> Value {
  let (day, start, expiry) = cetus_state(raw, now);
  let mut map = timed(format!("cambionCycle{expiry}"), Some(start), Some(expiry), now);
  map.insert("state".into(), json!(if day { "fass" } else { "vome" }));
  map.insert("timeLeft".into(), json!(eta(expiry - now)));
  Value::Object(map)
}

pub(super) fn earth(now: i64) -> Value {
  let cycle = (now / 1000).rem_euclid(28_800);
  let day = cycle < 14_400;
  let left = 14_400 - cycle % 14_400;
  let expiry = now + left * 1000;
  let rounded = (expiry as f64 / 60_000.0).round() as i64 * 60_000;
  let mut map = timed(format!("earthCycle{rounded}"), Some(expiry - 14_400_000), Some(expiry), now);
  map.insert("isDay".into(), json!(day));
  map.insert("state".into(), json!(if day { "day" } else { "night" }));
  map.insert("timeLeft".into(), json!(eta(expiry - now)));
  Value::Object(map)
}

pub(super) fn vallis(now: i64) -> Value {
  const START: i64 = 1_770_234_408_000; // 2026-02-04T19:46:48Z
  const LOOP: i64 = 1_600_000;
  const WARM: i64 = 400_000;
  const COLD: i64 = LOOP - WARM;
  let since = (now - START).rem_euclid(LOOP);
  let to_full = LOOP - since;
  let warm = to_full > COLD;
  let to_next = if warm { to_full - COLD } else { to_full };
  let previous = (now + to_full - if warm { LOOP } else { COLD }) / 60_000 * 60_000;
  let mut map = timed(format!("vallisCycle{previous}"), Some(previous), Some(now + to_next), now);
  map.insert("isWarm".into(), json!(warm));
  map.insert("state".into(), json!(if warm { "warm" } else { "cold" }));
  map.insert("timeLeft".into(), json!(eta(to_next)));
  Value::Object(map)
}

pub(super) fn zariman(raw: &Value, now: i64) -> Value {
  const CORPUS: i64 = 1_655_182_800_000;
  const FULL: i64 = 18_000_000;
  const HALF: i64 = 9_000_000;
  let end = bounty_end(raw, "CetusSyndicate") - 5000;
  let elapsed = (end - CORPUS).rem_euclid(FULL);
  let left = FULL - elapsed;
  let corpus = left > HALF;
  // The state ends where the bounties' cycle does, but never earlier than now.
  let remaining = if corpus { left - HALF } else { left };
  let expiry = ((now + remaining.max(0)) as f64 / 60_000.0).round() as i64 * 60_000;
  let mut map = timed(format!("zarimanCycle{expiry}"), Some(expiry - HALF), Some(expiry), now);
  map.insert("isCorpus".into(), json!(corpus));
  map.insert("state".into(), json!(if corpus { "corpus" } else { "grineer" }));
  map.insert("timeLeft".into(), json!(eta(expiry - now)));
  Value::Object(map)
}

pub(super) fn duviri(raw: &Value, now: i64) -> Value {
  const CYCLE: i64 = 36_000;
  const STATE: i64 = 7200;
  const STATES: [&str; 5] = ["sorrow", "fear", "joy", "anger", "envy"];
  let delta = ((now / 1000) - 52).rem_euclid(CYCLE);
  let index = (delta / STATE) as usize;
  let until = STATE - delta % STATE;
  let expiry = (now + until * 1000) / 60_000 * 60_000;
  let activation = expiry - STATE * 1000;
  let mut map = timed(format!("duviriCycle{}{expiry}", STATES[index]), Some(activation), Some(expiry), now);
  map.insert("state".into(), json!(STATES[index]));
  let choices: Vec<Value> = array(raw, "EndlessXpSchedule")
    .first()
    .and_then(|schedule| schedule.get("CategoryChoices"))
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .map(|choice| {
      let category = text(choice, "Category");
      json!({
        "title": if category == "EXC_HARD" { "Incarnon" } else { "Warframe" },
        "categoryKey": category,
        "choices": choice.get("Choices").cloned().unwrap_or_else(|| json!([])),
      })
    })
    .collect();
  map.insert("choices".into(), json!(choices));
  Value::Object(map)
}
