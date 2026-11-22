use crate::{Json, routes::capitalized, show, str_of};

/// "Baro Ki'Teer" -> "baroKiteer": the key under which API text is translated (see `translate_data`).
#[uniffi::export]
pub fn translation_key(text: String) -> Option<String> {
  let cleaned: String = text.chars().filter(|c| *c != '\'' && *c != '’').collect();
  let mut words = cleaned.split(|c: char| !(c.is_ascii_alphanumeric())).filter(|w| !w.is_empty());
  let first = words.next()?;
  let mut key = first.to_lowercase();
  for word in words {
    let mut chars = word.chars();
    if let Some(head) = chars.next() {
      key.extend(head.to_uppercase());
      key.extend(chars.flat_map(char::to_lowercase));
    }
  }
  Some(key)
}

/// "sprintSpeed" -> "Sprint Speed"
#[uniffi::export]
pub fn field_label(key: String) -> String {
  let mut spaced = String::new();
  let mut previous: Option<char> = None;
  for c in key.chars() {
    if c.is_ascii_uppercase() && previous.is_some_and(|p| p.is_ascii_lowercase()) {
      spaced.push(' ');
    }
    spaced.push(c);
    previous = Some(c);
  }
  capitalized(&spaced)
}

const TITLE_KEYS: [&str; 7] = ["name", "title", "node", "message", "description", "type", "id"];

pub(crate) fn title_of(item: &Json) -> String {
  TITLE_KEYS.iter().filter_map(|key| str_of(item, key)).find(|text| !text.is_empty()).unwrap_or("Details").to_string()
}

/// File name of a Nightwave challenge icon: no apostrophes, hyphenated, a trailing roman numeral dropped.
#[uniffi::export]
pub fn challenge_slug(title: String) -> String {
  let hyphenated = title.replace('\'', "").split_whitespace().collect::<Vec<_>>().join("-");
  let roman = |c: char| "IVXLCDMivxlcdm".contains(c);
  let slug = match hyphenated.rsplit_once('-') {
    Some((head, last)) if !last.is_empty() && last.chars().all(roman) => head.to_string(),
    None if !hyphenated.is_empty() && hyphenated.chars().all(roman) => String::new(),
    _ => hyphenated,
  };
  slug.to_lowercase()
}

pub(crate) fn scalar(item: &Json, key: &str) -> Option<String> {
  item.get(key).and_then(show)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn keys_and_labels() {
    assert_eq!(translation_key("Baro Ki'Teer".into()).as_deref(), Some("baroKiteer"));
    assert_eq!(translation_key("---".into()), None);
    assert_eq!(field_label("sprintSpeed".into()), "Sprint Speed");
    assert_eq!(field_label("fomorianProgress".into()), "Fomorian Progress");
  }

  #[test]
  fn slugs() {
    assert_eq!(challenge_slug("Dagger Hunter".into()), "dagger-hunter");
    assert_eq!(challenge_slug("Eximus Bane - IV".into()), "eximus-bane--");
    assert_eq!(challenge_slug("Gorgon's Fury II".into()), "gorgons-fury");
  }
}
