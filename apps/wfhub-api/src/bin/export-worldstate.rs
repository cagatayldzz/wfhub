//! Writes the name tables of the worldState parser: `export-worldstate <worldstate sources> <catalogue sources> <output>`.
//!
//! - `<worldstate sources>` has the WFCD `warframe-worldstate-data` files (`scripts/fetch-worldstate.sh`),
//! - `<catalogue sources>` has the WFCD `warframe-items` files (`scripts/fetch-catalogue.sh`),
//! - `<output>` (`data/worldstate`) gets `tables.json` and `names/{shard}.json`.

use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::{env, fs, process};
use wfhub_api::worldstate::{SHARDS, desc_key, normalize, shard_of};

const TABLES: [&str; 8] =
  ["solNodes", "missionTypes", "factionsData", "fissureModifiers", "sortieData", "syndicatesData", "upgradeTypes", "steelPath"];

fn read(path: &Path) -> Result<Value, String> {
  let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
  serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn run(sources: &Path, items: &Path, output: &Path) -> Result<(), String> {
  let mut tables = Map::new();
  for name in TABLES {
    tables.insert(name.into(), read(&sources.join(format!("{name}.json")))?);
  }

  let mut shards: Vec<BTreeMap<String, String>> = (0..SHARDS).map(|_| BTreeMap::new()).collect();
  let mut put = |path: &str, key_extra: Option<&str>, text: &str| {
    let key = normalize(path);
    let shard = shard_of(&key) as usize;
    let key = if key_extra.is_some() { desc_key(&key) } else { key };
    shards[shard].insert(key, text.to_string());
  };

  // Items first, so a language entry (which is the wording the game shows) wins for the same path.
  let mut count = 0;
  for entry in fs::read_dir(items).map_err(|error| format!("{}: {error} (run scripts/fetch-catalogue.sh)", items.display()))? {
    let path = entry.map_err(|error| error.to_string())?.path();
    if path.extension().and_then(|e| e.to_str()) != Some("json") {
      continue;
    }
    let Value::Array(list) = read(&path)? else { continue };
    for item in list {
      if let (Some(unique), Some(name)) = (item.get("uniqueName").and_then(Value::as_str), item.get("name").and_then(Value::as_str)) {
        put(unique, None, name);
        count += 1;
      }
    }
  }

  let Value::Object(languages) = read(&sources.join("languages.json"))? else { return Err("languages.json is not an object".into()) };
  for (path, entry) in &languages {
    if let Some(value) = entry.get("value").and_then(Value::as_str) {
      put(path, None, value);
    }
    if let Some(desc) = entry.get("desc").and_then(Value::as_str) {
      put(path, Some("desc"), desc);
    }
  }

  let _ = fs::remove_dir_all(output);
  fs::create_dir_all(output.join("names")).map_err(|error| error.to_string())?;
  fs::write(output.join("tables.json"), Value::Object(tables).to_string()).map_err(|error| error.to_string())?;
  for (index, shard) in shards.iter().enumerate() {
    let map: Map<String, Value> = shard.iter().map(|(key, value)| (key.clone(), Value::String(value.clone()))).collect();
    fs::write(output.join("names").join(format!("{index}.json")), Value::Object(map).to_string()).map_err(|error| error.to_string())?;
  }
  println!("{count} items, {} language entries, {SHARDS} shards", languages.len());
  Ok(())
}

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();
  let [sources, items, output] = args.as_slice() else {
    eprintln!("usage: export-worldstate <worldstate sources> <catalogue sources> <output>");
    process::exit(2);
  };
  if let Err(message) = run(Path::new(sources), Path::new(items), Path::new(output)) {
    eprintln!("error: {message}");
    process::exit(1);
  }
}
