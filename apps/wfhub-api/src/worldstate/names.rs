//! The names the raw state needs: nodes, mission types, factions (`tables.json`) and the names of paths (items, texts,
//! challenges), which are split into shards by the hash of the path so a request loads only the shards it needs.

use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

/// The name files are split into this many shards, by the hash of the (normalised) path.
pub const SHARDS: u32 = 32;

/// The key under which a path is stored: lowercase, and without `/StoreItems` (the store has its own path for an item).
pub fn normalize(path: &str) -> String {
  let lower = path.to_ascii_lowercase();
  match lower.strip_prefix("/lotus/storeitems/") {
    Some(rest) => format!("/lotus/{rest}"),
    None => lower,
  }
}

/// The shard (FNV-1a) of a normalised path.
pub fn shard_of(normalized: &str) -> u32 {
  let mut hash: u32 = 0x811c_9dc5;
  for byte in normalized.bytes() {
    hash ^= u32::from(byte);
    hash = hash.wrapping_mul(0x0100_0193);
  }
  hash % SHARDS
}

/// The key of a description next to a name (`{path}|desc`).
pub fn desc_key(normalized: &str) -> String {
  format!("{normalized}|desc")
}

fn walk(value: &Value, found: &mut BTreeSet<String>) {
  match value {
    Value::String(text) if text.starts_with("/Lotus/") => {
      found.insert(normalize(text));
    }
    Value::Array(items) => items.iter().for_each(|item| walk(item, found)),
    Value::Object(map) => map.values().for_each(|item| walk(item, found)),
    _ => {}
  }
}

/// All the `/Lotus/...` paths of a raw state, normalised.
pub fn collect_paths(raw: &Value) -> BTreeSet<String> {
  let mut found = BTreeSet::new();
  walk(raw, &mut found);
  found
}

/// The shards a raw state needs.
pub fn shards_needed(raw: &Value) -> BTreeSet<u32> {
  collect_paths(raw).iter().map(|path| shard_of(path)).collect()
}

/// The name tables: `tables` (nodes, mission types, factions, ...) and the path names of the shards loaded.
pub struct Names {
  pub(super) tables: Value,
  names: HashMap<String, String>,
}

impl Names {
  /// `tables` is the text of `data/worldstate/tables.json`, `shards` the texts of the shard files (objects of key to name).
  pub fn new(tables: &str, shards: &[String]) -> Result<Names, String> {
    let tables: Value = serde_json::from_str(tables).map_err(|error| format!("tables: {error}"))?;
    let mut names = HashMap::new();
    for shard in shards {
      let Value::Object(map) = serde_json::from_str::<Value>(shard).map_err(|error| format!("names: {error}"))? else {
        return Err("names: not an object".into());
      };
      for (key, value) in map {
        if let Value::String(text) = value {
          names.insert(key, text);
        }
      }
    }
    Ok(Names { tables, names })
  }

  pub(super) fn entry(&self, table: &str, key: &str) -> Option<&Value> {
    self.tables.get(table)?.get(key)
  }

  fn value(&self, table: &str, key: &str) -> Option<String> {
    self.entry(table, key)?.get("value")?.as_str().map(str::to_string)
  }

  /// The name of a solar node (`Galatea (Neptune)`), or its code.
  pub fn node(&self, code: &str) -> String {
    self.value("solNodes", code).unwrap_or_else(|| code.to_string())
  }

  pub(super) fn node_enemy(&self, code: &str) -> Option<String> {
    self.entry("solNodes", code)?.get("enemy")?.as_str().map(str::to_string)
  }

  pub(super) fn node_type(&self, code: &str) -> Option<String> {
    self.entry("solNodes", code)?.get("type")?.as_str().map(str::to_string)
  }

  /// `MT_EXCAVATE` -> `Excavation`.
  pub fn mission_type(&self, key: &str) -> String {
    self.value("missionTypes", key).unwrap_or_else(|| title(key.strip_prefix("MT_").unwrap_or(key)))
  }

  /// `FC_GRINEER` -> `Grineer`.
  pub fn faction(&self, key: &str) -> String {
    self.value("factionsData", key).unwrap_or_else(|| title(key.strip_prefix("FC_").unwrap_or(key)))
  }

  /// The name of a path (an item, a language key, a challenge), or its last part made readable.
  pub fn text(&self, path: &str) -> String {
    self.names.get(&normalize(path)).cloned().unwrap_or_else(|| humanize(path))
  }

  /// The name of a path, when there is one.
  pub fn lookup(&self, path: &str) -> Option<&str> {
    self.names.get(&normalize(path)).map(String::as_str)
  }

  /// The description next to a name (challenges, events).
  pub fn desc(&self, path: &str) -> Option<&str> {
    self.names.get(&desc_key(&normalize(path))).map(String::as_str)
  }

  pub(super) fn sortie_text(&self, table: &str, key: &str) -> Option<String> {
    self.tables.get("sortieData")?.get(table)?.get(key)?.as_str().map(str::to_string)
  }

  pub(super) fn boss(&self, key: &str) -> Option<(String, String)> {
    let boss = self.tables.get("sortieData")?.get("bosses")?.get(key)?;
    Some((boss.get("name")?.as_str()?.to_string(), boss.get("faction").and_then(Value::as_str).unwrap_or("").to_string()))
  }
}

/// `SORTIE_BOSS_KELA` -> `Sortie Boss Kela`, `Research7Target` -> `Research 7 Target`.
pub(super) fn title(text: &str) -> String {
  text
    .split(['_', ' '])
    .filter(|word| !word.is_empty())
    .map(|word| {
      let lower = word.to_ascii_lowercase();
      let mut chars = lower.chars();
      chars.next().map(|first| first.to_ascii_uppercase().to_string() + chars.as_str()).unwrap_or_default()
    })
    .collect::<Vec<_>>()
    .join(" ")
}

/// The last part of a path as words: `/Lotus/Types/Items/MiscItems/Forma` -> `Forma`, `WeaponRecoilModExpert` ->
/// `Weapon Recoil Mod Expert`.
pub fn humanize(path: &str) -> String {
  let last = path.rsplit('/').next().unwrap_or(path);
  let mut out = String::new();
  let mut previous: Option<char> = None;
  for c in last.chars() {
    if let Some(p) = previous
      && ((c.is_ascii_uppercase() && (p.is_ascii_lowercase() || p.is_ascii_digit())) || (c.is_ascii_digit() && p.is_ascii_alphabetic()))
    {
      out.push(' ');
    }
    out.push(if c == '_' { ' ' } else { c });
    previous = Some(c);
  }
  out
}
