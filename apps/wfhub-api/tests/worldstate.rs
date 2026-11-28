//! The worldState parser on real states of every platform (`tests/fixtures`) and the name files of `data/worldstate`.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use wfhub_api::worldstate::{
  Names, PLATFORMS, SECTIONS, SHARDS, collect_paths, eta, humanize, iso, ms, normalize, parse_all, parse_section, platform, shard_of, shards_needed,
};

fn api() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn raw(platform: &str) -> Value {
  let path = api().join(format!("tests/fixtures/worldstate-{platform}.json"));
  serde_json::from_str(&fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))).unwrap()
}

/// The names a state needs, loaded the way the Worker does.
fn names_for(raw: &Value) -> Names {
  let data = api().join("data/worldstate");
  let tables = fs::read_to_string(data.join("tables.json")).unwrap();
  let shards: Vec<String> = shards_needed(raw).iter().map(|shard| fs::read_to_string(data.join(format!("names/{shard}.json"))).unwrap()).collect();
  Names::new(&tables, &shards).unwrap()
}

fn now(raw: &Value) -> i64 {
  raw["Time"].as_i64().unwrap() * 1000
}

#[test]
fn platforms_and_aliases() {
  assert_eq!(PLATFORMS.len(), 4);
  assert_eq!(platform("ps5"), Some("ps4"));
  assert_eq!(platform("mob"), Some("pc"));
  assert_eq!(platform("pc"), Some("pc"));
  assert!(platform("../x").is_none());
  assert!(platform("PC").is_none());
}

#[test]
fn time_helpers() {
  assert_eq!(iso(1_794_730_453_000), "2026-11-15T08:14:13.000Z");
  assert_eq!(iso(0), "1970-01-01T00:00:00.000Z");
  assert_eq!(eta(3_725_000), "1h 2m 5s");
  assert_eq!(eta(-61_000), "-1m 1s");
  assert_eq!(ms(&serde_json::json!({"$date": {"$numberLong": "1794675600000"}})), Some(1_794_675_600_000));
  assert_eq!(humanize("/Lotus/Upgrades/Mods/WeaponRecoilModExpert"), "Weapon Recoil Mod Expert");
  assert_eq!(normalize("/Lotus/StoreItems/Upgrades/Mods/X"), "/lotus/upgrades/mods/x");
  assert!(shard_of("/lotus/x") < SHARDS);
}

#[test]
fn every_platform_parses_every_section() {
  for platform in PLATFORMS {
    let raw = raw(platform);
    let names = names_for(&raw);
    let now = now(&raw);
    for section in SECTIONS {
      let value = parse_section(&raw, section, now, &names).unwrap_or_else(|| panic!("{platform}/{section}"));
      assert!(!value.is_null(), "{platform}/{section}");
    }
    assert!(parse_section(&raw, "nope", now, &names).is_none());
    let all = parse_all(&raw, now, &names);
    assert_eq!(all.as_object().unwrap().len(), SECTIONS.len() + 2);
    assert!(all["timestamp"].as_str().unwrap().ends_with('Z'));
    assert!(!all["buildLabel"].as_str().unwrap().is_empty());
  }
}

#[test]
fn names_are_resolved() {
  let raw = raw("pc");
  let names = names_for(&raw);
  let now = now(&raw);

  let fissures = parse_section(&raw, "fissures", now, &names).unwrap();
  let fissures = fissures.as_array().unwrap();
  assert!(!fissures.is_empty());
  for fissure in fissures {
    assert!(!fissure["node"].as_str().unwrap().starts_with("SolNode"), "{fissure}");
    assert!(["Lith", "Meso", "Neo", "Axi", "Requiem", "Omnia"].contains(&fissure["tier"].as_str().unwrap()), "{fissure}");
    assert!(fissure["expiry"].as_str().unwrap().ends_with('Z'));
  }

  let sortie = parse_section(&raw, "sortie", now, &names).unwrap();
  assert_eq!(sortie["boss"], "Jackal");
  assert_eq!(sortie["variants"].as_array().unwrap().len(), 3);
  assert!(!sortie["variants"][0]["modifier"].as_str().unwrap().starts_with("SORTIE"));

  let baro = parse_section(&raw, "voidTrader", now, &names).unwrap();
  assert_eq!(baro["character"], "Baro Ki'Teer");
  assert!(baro["inventory"].as_array().unwrap().iter().all(|item| !item["item"].as_str().unwrap().starts_with("/Lotus")));

  let nightwave = parse_section(&raw, "nightwave", now, &names).unwrap();
  assert!(nightwave["activeChallenges"].as_array().unwrap().iter().all(|c| !c["title"].as_str().unwrap().is_empty()));

  let construction = parse_section(&raw, "constructionProgress", now, &names).unwrap();
  assert_eq!(construction["fomorianProgress"], "26.63");
}

#[test]
fn cycles_are_consistent() {
  let raw = raw("pc");
  let names = names_for(&raw);
  let now = now(&raw);
  let cycle = |name: &str| parse_section(&raw, name, now, &names).unwrap();
  let (cetus, cambion) = (cycle("cetusCycle"), cycle("cambionCycle"));
  assert_eq!(cetus["expiry"], cambion["expiry"]);
  assert_eq!(cetus["isDay"].as_bool().unwrap(), cambion["state"] == "fass");
  for name in ["cetusCycle", "cambionCycle", "vallisCycle", "earthCycle", "zarimanCycle", "duviriCycle"] {
    let value = cycle(name);
    assert!(value["state"].is_string(), "{name}");
    assert!(value.get("expired").is_none() && value.get("eta").is_none() && value.get("timeLeft").is_none(), "{name}");
  }
  let earth = cycle("earthCycle");
  assert!(["day", "night"].contains(&earth["state"].as_str().unwrap()));
}

#[test]
fn collected_paths_cover_the_shards() {
  let raw = raw("pc");
  assert!(collect_paths(&raw).len() > 100);
  assert!(shards_needed(&raw).iter().all(|shard| *shard < SHARDS));
}

#[test]
fn name_files_are_complete() {
  let data = api().join("data/worldstate");
  let tables: Value = serde_json::from_str(&fs::read_to_string(data.join("tables.json")).unwrap()).unwrap();
  for table in ["solNodes", "missionTypes", "factionsData", "fissureModifiers", "sortieData", "syndicatesData", "upgradeTypes", "steelPath"] {
    assert!(!tables[table].is_null(), "{table}");
  }
  for shard in 0..SHARDS {
    let names: Value = serde_json::from_str(&fs::read_to_string(data.join(format!("names/{shard}.json"))).unwrap()).unwrap();
    for key in names.as_object().unwrap().keys() {
      assert_eq!(shard_of(key.strip_suffix("|desc").unwrap_or(key)), shard, "{key}");
    }
  }
}

#[test]
fn status_comparison_finds_what_differs() {
  use wfhub_api::status::{differences, parse_iso};
  assert_eq!(parse_iso("2026-11-23T10:35:13.000Z"), Some(1_795_430_113));
  let now = 1_795_430_113;
  let mine = serde_json::json!({
    "timestamp": "2026-11-23T10:35:13.000Z",
    "news": [{ "id": "a" }, { "id": "b" }], "events": [], "alerts": [{ "id": "x", "expiry": "2026-11-23T10:36:00.000Z" }],
    "sortie": { "id": "s1" }, "archonHunt": { "id": "h" }, "voidTrader": { "id": "v" }, "nightwave": { "id": "n" },
    "invasions": [{ "id": "i1" }], "fissures": [{ "id": "f1" }],
  });
  assert!(differences(&mine, &mine, now).is_empty());
  let mut theirs = mine.clone();
  // An alert that ends right now is not held against either side.
  theirs["alerts"] = serde_json::json!([]);
  assert!(differences(&mine, &theirs, now).is_empty());
  theirs["news"] = serde_json::json!([{ "id": "a" }]);
  theirs["sortie"] = serde_json::json!({ "id": "s2" });
  theirs["fissures"] = serde_json::Value::Null;
  let found = differences(&mine, &theirs, now);
  assert_eq!(found, ["news: 1 only in WFHub, 0 only in Warframe Status", "sortie: 1 only in WFHub, 1 only in Warframe Status", "fissures: missing in Warframe Status"]);
  theirs["timestamp"] = serde_json::json!("2026-11-23T09:00:00.000Z");
  assert_eq!(differences(&mine, &theirs, now), ["timestamp: 95 min apart, Warframe Status is behind"]);
}

#[test]
fn status_comparison_ignores_how_the_two_write_the_same_thing() {
  use wfhub_api::status::{COMPARED, differences};
  let now = 1_795_430_113;
  let state = |nightwave: &str, invasions: serde_json::Value| {
    let mut state = serde_json::json!({ "timestamp": "2026-11-23T10:35:13.000Z" });
    for name in COMPARED {
      state[name] = serde_json::json!([]);
    }
    state["nightwave"] = serde_json::json!({ "id": nightwave, "activation": "2026-08-12T15:30:00.000Z" });
    state["invasions"] = invasions;
    state
  };
  // The same season under two ids, and an invasion that is over only in one of them.
  let mine = state("nightwave18", serde_json::json!([{ "id": "i1" }]));
  let theirs = state("nightwave1804464000000", serde_json::json!([{ "id": "i1" }, { "id": "i2", "completed": true }]));
  assert_eq!(differences(&mine, &theirs, now), Vec::<String>::new());
  // A running invasion that is missing is still found.
  let theirs = state("nightwave1804464000000", serde_json::json!([{ "id": "i1" }, { "id": "i3", "completed": false }]));
  assert_eq!(differences(&mine, &theirs, now), ["invasions: 0 only in WFHub, 1 only in Warframe Status"]);
}
