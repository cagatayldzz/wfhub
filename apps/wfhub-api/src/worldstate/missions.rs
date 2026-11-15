//! Missions: fissures, sortie, archon hunt, alerts, invasions, arbitration and sentient outposts.

use super::names::{Names, title};
use super::value::{array, iso, ms, number, oid, percent, reward, text, timed};
use serde_json::{Value, json};

pub(super) fn fissures(raw: &Value, now: i64, names: &Names) -> Value {
  let tiers = |key: &str| -> (String, i64) {
    let entry = names.entry("fissureModifiers", key);
    (
      entry.and_then(|e| e.get("value")).and_then(Value::as_str).unwrap_or(key).to_string(),
      entry.and_then(|e| e.get("num")).and_then(Value::as_i64).unwrap_or(0),
    )
  };
  let mut list = Vec::new();
  for (key, storm) in [("ActiveMissions", false), ("VoidStorms", true)] {
    for mission in array(raw, key) {
      let (activation, expiry) = (mission.get("Activation").and_then(ms), mission.get("Expiry").and_then(ms));
      if expiry.is_some_and(|e| e < now) {
        continue;
      }
      let node = text(mission, "Node");
      let tier_key = if storm { text(mission, "ActiveMissionTier") } else { text(mission, "Modifier") };
      let (tier, tier_num) = tiers(tier_key);
      let mission_key = if storm { "MT_VOID_FLOOD" } else { text(mission, "MissionType") };
      let mut map = timed(oid(mission), activation, expiry);
      map.insert("node".into(), json!(names.node(node)));
      map.insert("nodeKey".into(), json!(node));
      map.insert("missionType".into(), json!(if storm { "Void Storm".to_string() } else { names.mission_type(mission_key) }));
      map.insert("missionKey".into(), json!(mission_key));
      map.insert("enemy".into(), json!(names.node_enemy(node).unwrap_or_default()));
      map.insert("tier".into(), json!(tier));
      map.insert("tierNum".into(), json!(tier_num));
      map.insert("isStorm".into(), json!(storm));
      map.insert("isHard".into(), json!(mission.get("Hard").and_then(Value::as_bool).unwrap_or(false)));
      list.push(Value::Object(map));
    }
  }
  list.sort_by_key(|fissure| (fissure["tierNum"].as_i64().unwrap_or(0), fissure["expiry"].as_str().unwrap_or("").to_string()));
  Value::Array(list)
}

pub(super) fn sortie(raw: &Value, names: &Names) -> Value {
  let Some(sortie) = array(raw, "Sorties").first() else { return json!({}) };
  let (activation, expiry) = (sortie.get("Activation").and_then(ms), sortie.get("Expiry").and_then(ms));
  let mut map = timed(oid(sortie), activation, expiry);
  let boss_key = text(sortie, "Boss");
  let (boss, faction) = names.boss(boss_key).unwrap_or_else(|| (title(boss_key.strip_prefix("SORTIE_BOSS_").unwrap_or(boss_key)), String::new()));
  let variants: Vec<Value> = sortie
    .get("Variants")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .map(|variant| {
      let (node, modifier) = (text(variant, "node"), text(variant, "modifierType"));
      json!({
        "node": names.node(node),
        "nodeKey": node,
        "missionType": names.mission_type(text(variant, "missionType")),
        "modifier": names.sortie_text("modifierTypes", modifier).unwrap_or_else(|| title(modifier.strip_prefix("SORTIE_MODIFIER_").unwrap_or(modifier))),
        "modifierDescription": names.sortie_text("modifierDescriptions", modifier).unwrap_or_default(),
      })
    })
    .collect();
  map.insert("rewardPool".into(), json!("Sortie Rewards"));
  map.insert("boss".into(), json!(boss));
  map.insert("faction".into(), json!(faction));
  map.insert("variants".into(), json!(variants));
  Value::Object(map)
}

pub(super) fn archon_hunt(raw: &Value, names: &Names) -> Value {
  let Some(hunt) = array(raw, "LiteSorties").first() else { return json!({}) };
  let (activation, expiry) = (hunt.get("Activation").and_then(ms), hunt.get("Expiry").and_then(ms));
  let mut map = timed(oid(hunt), activation, expiry);
  let boss_key = text(hunt, "Boss");
  let (boss, faction) = names.boss(boss_key).unwrap_or_else(|| (title(boss_key.strip_prefix("SORTIE_BOSS_").unwrap_or(boss_key)), String::new()));
  let missions: Vec<Value> = hunt
    .get("Missions")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .map(|mission| {
      let node = text(mission, "node");
      json!({ "type": names.mission_type(text(mission, "missionType")), "missionType": names.mission_type(text(mission, "missionType")), "node": names.node(node), "nodeKey": node })
    })
    .collect();
  map.insert("rewardPool".into(), json!("Archon Hunt Rewards"));
  map.insert("boss".into(), json!(boss));
  map.insert("faction".into(), json!(faction));
  map.insert("variants".into(), json!(missions));
  map.insert("missions".into(), json!(missions));
  Value::Object(map)
}

pub(super) fn alerts(raw: &Value, now: i64, names: &Names) -> Value {
  let list = array(raw, "Alerts")
    .iter()
    .filter_map(|alert| {
      let (activation, expiry) = (alert.get("Activation").and_then(ms), alert.get("Expiry").and_then(ms));
      if expiry.is_some_and(|e| e < now) {
        return None;
      }
      let info = alert.get("MissionInfo")?;
      let node = text(info, "location");
      let mut map = timed(oid(alert), activation, expiry);
      let reward = reward(info.get("missionReward").unwrap_or(&Value::Null), names);
      let mission = json!({
        "description": names.lookup(text(info, "descText")).unwrap_or(""),
        "node": names.node(node),
        "nodeKey": node,
        "type": names.mission_type(text(info, "missionType")),
        "typeKey": text(info, "missionType"),
        "faction": names.faction(text(info, "faction")),
        "factionKey": text(info, "faction"),
        "reward": reward,
        "minEnemyLevel": number(info, "minEnemyLevel"),
        "maxEnemyLevel": number(info, "maxEnemyLevel"),
        "maxWaveNum": number(info, "maxWaveNum"),
        "nightmare": info.get("nightmare").and_then(Value::as_bool).unwrap_or(false),
        "archwingRequired": info.get("archwingRequired").and_then(Value::as_bool).unwrap_or(false),
      });
      map.insert("mission".into(), mission);
      map.insert("tag".into(), json!(text(alert, "Tag")));
      map.insert("forceUnlock".into(), json!(alert.get("ForceUnlock").and_then(Value::as_bool).unwrap_or(false)));
      Some(Value::Object(map))
    })
    .collect();
  Value::Array(list)
}

pub(super) fn invasions(raw: &Value, names: &Names) -> Value {
  let list = array(raw, "Invasions")
    .iter()
    .filter(|invasion| !invasion.get("Completed").and_then(Value::as_bool).unwrap_or(false))
    .map(|invasion| {
      let activation = invasion.get("Activation").and_then(ms);
      let node = text(invasion, "Node");
      let (attacker, defender) = (text(invasion, "Faction"), text(invasion, "DefenderFaction"));
      let (count, goal) = (number(invasion, "Count"), number(invasion, "Goal").max(1));
      let versus_infestation = attacker == "FC_INFESTATION";
      let completion = if versus_infestation { (count.max(0) as f64 / goal as f64) * 100.0 } else { (1.0 + count as f64 / goal as f64) * 50.0 };
      let mut map = timed(oid(invasion), activation, None);
      map.insert("node".into(), json!(names.node(node)));
      map.insert("nodeKey".into(), json!(node));
      map.insert("desc".into(), json!(names.lookup(text(invasion, "LocTag")).unwrap_or("")));
      map.insert("attackingFaction".into(), json!(names.faction(attacker)));
      map.insert("defendingFaction".into(), json!(names.faction(defender)));
      map.insert("attacker".into(), json!({ "faction": names.faction(attacker), "factionKey": attacker, "reward": reward(invasion.get("AttackerReward").unwrap_or(&Value::Null), names) }));
      map.insert("defender".into(), json!({ "faction": names.faction(defender), "factionKey": defender, "reward": reward(invasion.get("DefenderReward").unwrap_or(&Value::Null), names) }));
      map.insert("vsInfestation".into(), json!(versus_infestation));
      map.insert("count".into(), json!(count));
      map.insert("requiredRuns".into(), json!(goal));
      map.insert("completion".into(), json!(percent(completion.clamp(0.0, 100.0))));
      map.insert("completed".into(), json!(false));
      Value::Object(map)
    })
    .collect();
  Value::Array(list)
}

/// The arbitration is not in the raw state (WFCD reads a separate schedule), so this is the "unknown" placeholder
/// `warframe-status` also answers when it has none.
pub(super) fn arbitration() -> Value {
  json!({
    "id": "SolNode000",
    "node": "SolNode000",
    "nodeKey": "SolNode000",
    "activation": iso(0),
    "expiry": iso(8_640_000_000_000_000),
    "enemy": "Tenno",
    "type": "Unknown",
    "typeKey": "Unknown",
    "archwing": false,
    "sharkwing": false,
    "active": false,
    "expired": true
  })
}

pub(super) fn sentient_outposts(raw: &Value, now: i64, names: &Names) -> Value {
  let tmp: Value = raw.get("Tmp").and_then(Value::as_str).and_then(|text| serde_json::from_str(text).ok()).unwrap_or(Value::Null);
  let id = tmp.get("sfn").and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
  let mut map = timed(format!("sentientoutposts{}", now / 1_800_000), None, None);
  match id {
    Some(id) => {
      let code = format!("CrewBattleNode{id}");
      map.insert("active".into(), json!(true));
      map.insert(
        "mission".into(),
        json!({ "node": names.node(&code), "nodeKey": code, "faction": "Sentient", "type": names.node_type(&code).unwrap_or_default() }),
      );
    }
    None => {
      map.insert("active".into(), json!(false));
      map.insert("mission".into(), Value::Null);
    }
  }
  Value::Object(map)
}
