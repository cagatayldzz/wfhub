//! Writes the static catalogue: `export-catalogue <sources> <output>` reads `<sources>/<Category>.json` (WFCD files, see
//! `catalogue/fetch.sh`) and writes `<output>/<category>.json` and `<output>/<category>/<c>.json`.

use serde_json::Value;
use std::path::Path;
use std::{env, fs, process};
use wfhub_api::catalogue::make;
use wfhub_api::routes::CATALOGUE_CATEGORIES;

/// The WFCD file name of a category (`arch-gun` -> `Arch-Gun`, `sentinelweapons` -> `SentinelWeapons`).
fn source_name(category: &str) -> String {
  match category {
    "sentinelweapons" => "SentinelWeapons".into(),
    _ => category.split('-').map(|part| part[..1].to_uppercase() + &part[1..]).collect::<Vec<_>>().join("-"),
  }
}

fn run(sources: &Path, output: &Path) -> Result<(), String> {
  for category in CATALOGUE_CATEGORIES {
    let file = sources.join(format!("{}.json", source_name(category)));
    let text = fs::read_to_string(&file).map_err(|error| format!("{}: {error} (run catalogue/fetch.sh)", file.display()))?;
    let items: Vec<Value> = serde_json::from_str(&text).map_err(|error| format!("{}: {error}", file.display()))?;
    let catalogue = make(&items);
    let folder = output.join(category);
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    fs::write(output.join(format!("{category}.json")), Value::Array(catalogue.list.clone()).to_string()).map_err(|error| error.to_string())?;
    for (bucket, items) in &catalogue.buckets {
      fs::write(folder.join(format!("{bucket}.json")), Value::Object(items.clone()).to_string()).map_err(|error| error.to_string())?;
    }
    println!("{category}: {} items", catalogue.list.len());
  }
  Ok(())
}

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();
  let [sources, output] = args.as_slice() else {
    eprintln!("usage: export-catalogue <sources> <output>");
    process::exit(2);
  };
  if let Err(message) = run(Path::new(sources), Path::new(output)) {
    eprintln!("error: {message}");
    process::exit(1);
  }
}
