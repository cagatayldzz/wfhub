use crate::text::{scalar, title_of};
use crate::{CoreError, Json, codex_image_url, field_label, parse, str_of};

#[derive(Clone, Debug, uniffi::Record)]
pub struct CodexCard {
  pub slug: String,
  pub title: String,
  pub image: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct Ability {
  pub name: String,
  pub description: Option<String>,
  pub image: Option<String>,
}

/// A line of a nested block: a header (`value` is `None`) or `label: value`, indented by `depth`.
#[derive(Clone, Debug, uniffi::Record)]
pub struct DetailLine {
  pub label: String,
  pub value: Option<String>,
  pub depth: u32,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct CodexItem {
  pub title: String,
  pub image: Option<String>,
  pub description: Option<String>,
  pub stats: Vec<DetailLine>,
  pub abilities: Vec<Ability>,
  pub damage: Vec<DetailLine>,
}

const STATS: [(&str, &str); 7] = [
  ("health", "Health"),
  ("shield", "Shield"),
  ("armor", "Armor"),
  ("stamina", "Stamina"),
  ("power", "Power"),
  ("sprintSpeed", "Sprint Speed"),
  ("masteryReq", "Mastery Requirement"),
];

/// The list of a category (`category_url`). Entries without a slug can not be opened and are left out.
#[uniffi::export]
pub fn parse_category(body: String) -> Result<Vec<CodexCard>, CoreError> {
  let Json::Array(items) = parse(&body)? else { return Err(CoreError::Shape) };
  Ok(
    items
      .iter()
      .filter_map(|item| {
        Some(CodexCard {
          slug: str_of(item, "slug")?.to_string(),
          title: title_of(item),
          image: str_of(item, "imageName").and_then(|n| codex_image_url(n.into())),
        })
      })
      .collect(),
  )
}

#[uniffi::export]
pub fn filter_tiles(tiles: Vec<CodexCard>, query: String) -> Vec<CodexCard> {
  let query = query.to_lowercase();
  tiles.into_iter().filter(|tile| query.is_empty() || tile.title.to_lowercase().contains(&query)).collect()
}

/// The detail of one item (`item_url`).
#[uniffi::export]
pub fn parse_item(body: String) -> Result<CodexItem, CoreError> {
  let item = parse(&body)?;
  if !item.is_object() {
    return Err(CoreError::Shape);
  }
  let stats = STATS
    .iter()
    .filter_map(|(key, label)| {
      item.get(*key).map(|_| DetailLine {
        label: (*label).into(),
        value: scalar(&item, key).or_else(|| item.get(*key).map(|v| v.to_string())),
        depth: 0,
      })
    })
    .collect();
  let abilities = item
    .get("abilities")
    .and_then(Json::as_array)
    .map(|list| {
      list
        .iter()
        .map(|a| Ability {
          name: str_of(a, "name").unwrap_or("Ability").into(),
          description: str_of(a, "description").map(Into::into),
          image: str_of(a, "imageName").and_then(|n| codex_image_url(n.into())),
        })
        .collect()
    })
    .unwrap_or_default();
  let mut damage = Vec::new();
  if let Some(object) = item.get("damage").filter(|d| d.as_object().is_some_and(|o| !o.is_empty())) {
    lines(object, 0, &mut damage);
  }
  Ok(CodexItem {
    title: title_of(&item),
    image: str_of(&item, "imageName").and_then(|n| codex_image_url(n.into())),
    description: str_of(&item, "description").filter(|d| !d.is_empty()).map(Into::into),
    stats,
    abilities,
    damage,
  })
}

/// Flattens a nested object into indented lines, up to three levels deep. Keys are sorted; links and ids are skipped.
fn lines(value: &Json, depth: u32, out: &mut Vec<DetailLine>) {
  let Some(object) = value.as_object() else { return };
  let mut keys: Vec<&String> = object.keys().collect();
  keys.sort();
  for key in keys {
    let label = field_label(key.clone());
    let child = &object[key];
    let line = |value: String| DetailLine { label: label.clone(), value: Some(value), depth };
    let header = || DetailLine { label: label.clone(), value: None, depth };
    match child {
      Json::String(text) if !text.is_empty() && key != "link" && !key.ends_with("Key") && !key.ends_with("Id") => out.push(line(text.clone())),
      Json::Number(number) => out.push(line(number.to_string())),
      Json::Object(inner) if !inner.is_empty() && depth < 3 => {
        out.push(header());
        lines(child, depth + 1, out);
      }
      Json::Array(list) if !list.is_empty() && list.iter().all(Json::is_string) => {
        out.push(line(list.iter().filter_map(Json::as_str).collect::<Vec<_>>().join(", ")))
      }
      Json::Array(list) if !list.is_empty() && depth < 3 && list.iter().all(Json::is_object) => {
        out.push(header());
        for object in list {
          lines(object, depth + 1, out);
        }
      }
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_category_and_item() {
    let tiles = parse_category(r#"[{"slug":"ash","name":"Ash","imageName":"ash.png"},{"name":"No slug"}]"#.into()).unwrap();
    assert_eq!(tiles.len(), 1);
    assert!(tiles[0].image.as_deref().unwrap().ends_with("data/img/ash.png"));
    assert_eq!(filter_tiles(tiles, "xyz".into()).len(), 0);

    let item = parse_item(
      r#"{"name":"Serration","health":100,"description":"Hi","abilities":[{"name":"Slash"}],"damage":{"total":5,"types":{"slash":3,"tag":"x"},"ownId":"1","link":"u"}}"#.into(),
    )
    .unwrap();
    assert_eq!(item.title, "Serration");
    assert_eq!(item.stats[0].value.as_deref(), Some("100"));
    assert_eq!(item.abilities[0].name, "Slash");
    let labels: Vec<_> = item.damage.iter().map(|l| (l.label.as_str(), l.depth)).collect();
    assert_eq!(labels, [("Total", 0), ("Types", 0), ("Slash", 1), ("Tag", 1)]);
  }
}
