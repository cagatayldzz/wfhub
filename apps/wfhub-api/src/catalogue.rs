//! Makes the static Codex catalogue from the WFCD `warframe-items` files (`catalogue/fetch.sh` downloads them):
//! for each category a list file and the items in bucket files named by the first character of the slug, so the Worker
//! never has to read more than a small file.
//!
//! ```text
//! catalogue/{category}.json        [{"name","imageName","slug"}, ...]
//! catalogue/{category}/{c}.json    {"<slug>": {...item...}, ...}   (slugs that start with c)
//! ```

use crate::routes::bucket_name;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

/// Where WFCD keeps its images; `imageName` of an item is a link into it.
pub const IMAGE_BASE: &str = "https://raw.githubusercontent.com/WFCD/warframe-items/master/data/img/";

/// The fields of an item that are kept.
const FIELDS: [&str; 14] = [
  "uniqueName",
  "name",
  "description",
  "health",
  "shield",
  "armor",
  "stamina",
  "power",
  "masteryReq",
  "sprintSpeed",
  "abilities",
  "sprint",
  "wikiaUrl",
  "releaseDate",
];

pub struct Catalogue {
  pub list: Vec<Value>,
  /// First character of the slug -> slug -> item.
  pub buckets: BTreeMap<char, Map<String, Value>>,
}

/// `Héllo Wörld!` -> `hello-world`.
pub fn slug(name: &str) -> String {
  let plain: String = name.nfkd().filter(|c| !('\u{300}'..='\u{36f}').contains(c)).collect::<String>().to_lowercase();
  plain.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit())).filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
}

/// Like JavaScript's `encodeURIComponent`.
fn encode_component(text: &str) -> String {
  let mut out = String::new();
  for byte in text.bytes() {
    if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
      out.push(byte as char);
    } else {
      out.push_str(&format!("%{byte:02X}"));
    }
  }
  out
}

/// Builds the catalogue of one category out of its WFCD items. An item without a name, or whose slug is taken, is left out.
pub fn make(items: &[Value]) -> Catalogue {
  let mut catalogue = Catalogue { list: Vec::new(), buckets: BTreeMap::new() };
  for item in items {
    let Some(name) = item.get("name").and_then(Value::as_str) else { continue };
    let slug = slug(name);
    let Some(bucket) = bucket_name(&slug) else { continue };
    if catalogue.buckets.get(&bucket).is_some_and(|items| items.contains_key(&slug)) {
      continue;
    }
    let image = match item.get("imageName").and_then(Value::as_str) {
      Some(image) if !image.is_empty() => format!("{IMAGE_BASE}{}", encode_component(image)),
      _ => String::new(),
    };
    let mut detail = Map::new();
    for field in FIELDS {
      let value = match item.get(field) {
        Some(value) if !value.is_null() => value.clone(),
        _ if field == "abilities" => json!([]),
        _ => Value::Null,
      };
      detail.insert(field.into(), value);
    }
    detail.insert("imageName".into(), json!(image));
    detail.insert("slug".into(), json!(slug));
    catalogue.list.push(json!({ "name": name, "imageName": image, "slug": slug }));
    catalogue.buckets.entry(bucket).or_default().insert(slug, Value::Object(detail));
  }
  catalogue
}
