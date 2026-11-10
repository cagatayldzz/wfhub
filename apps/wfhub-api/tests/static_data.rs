//! Checks the static files of the site that the API serves.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use wfhub_api::catalogue::slug;
use wfhub_api::routes::{CATALOGUE_CATEGORIES, MASTERY_CATEGORIES, bucket_name};

fn web() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../wfhub-web")
}

fn json(path: PathBuf) -> Value {
  serde_json::from_str(&fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display())))
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn slugs_match_the_published_ones() {
  assert_eq!(slug("Excalibur Prime"), "excalibur-prime");
  assert_eq!(slug("Héllo, Wörld!"), "hello-world");
  assert_eq!(slug("  --A  B-- "), "a-b");
  assert_eq!(slug("!!!"), "");
}

#[test]
fn events_and_openapi_are_json() {
  assert!(json(web().join("data/events.json")).is_array());
  assert!(json(web().join("swagger/openapi.json")).get("paths").is_some());
}

#[test]
fn every_mastery_file_exists() {
  for name in MASTERY_CATEGORIES {
    json(web().join(format!("data/mastery/{name}.json")));
  }
}

#[test]
fn the_catalogue_lists_match_their_buckets() {
  for category in CATALOGUE_CATEGORIES {
    let list = json(web().join(format!("data/catalogue/{category}.json")));
    let list = list.as_array().unwrap_or_else(|| panic!("{category} is not a list"));
    assert!(!list.is_empty(), "{category} is empty");
    for entry in list {
      let slug = entry["slug"].as_str().unwrap();
      let bucket = json(web().join(format!("data/catalogue/{category}/{}.json", bucket_name(slug).unwrap())));
      assert_eq!(bucket[slug]["slug"], slug, "{category}/{slug}");
    }
  }
}
