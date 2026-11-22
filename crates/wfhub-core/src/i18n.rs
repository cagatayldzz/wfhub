//! Translations. The strings are the iOS `Localizable.strings` files: one source for both apps, so a key is added once.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::PLATFORMS;

const FILES: [(&str, &str); 14] = [
  ("de", include_str!("../../../apps/wfhub-ios/WFHub/Resources/de.lproj/Localizable.strings")),
  ("en", include_str!("../../../apps/wfhub-ios/WFHub/Resources/en.lproj/Localizable.strings")),
  ("es", include_str!("../../../apps/wfhub-ios/WFHub/Resources/es.lproj/Localizable.strings")),
  ("fr", include_str!("../../../apps/wfhub-ios/WFHub/Resources/fr.lproj/Localizable.strings")),
  ("it", include_str!("../../../apps/wfhub-ios/WFHub/Resources/it.lproj/Localizable.strings")),
  ("ja", include_str!("../../../apps/wfhub-ios/WFHub/Resources/ja.lproj/Localizable.strings")),
  ("ko", include_str!("../../../apps/wfhub-ios/WFHub/Resources/ko.lproj/Localizable.strings")),
  ("pl", include_str!("../../../apps/wfhub-ios/WFHub/Resources/pl.lproj/Localizable.strings")),
  ("pt", include_str!("../../../apps/wfhub-ios/WFHub/Resources/pt.lproj/Localizable.strings")),
  ("ru", include_str!("../../../apps/wfhub-ios/WFHub/Resources/ru.lproj/Localizable.strings")),
  ("th", include_str!("../../../apps/wfhub-ios/WFHub/Resources/th.lproj/Localizable.strings")),
  ("tr", include_str!("../../../apps/wfhub-ios/WFHub/Resources/tr.lproj/Localizable.strings")),
  ("uk", include_str!("../../../apps/wfhub-ios/WFHub/Resources/uk.lproj/Localizable.strings")),
  ("zh", include_str!("../../../apps/wfhub-ios/WFHub/Resources/zh.lproj/Localizable.strings")),
];

/// Language name in its own language, for the language picker.
const NAMES: [(&str, &str); 14] = [
  ("de", "Deutsch"),
  ("en", "English"),
  ("es", "Español"),
  ("fr", "Français"),
  ("it", "Italiano"),
  ("ja", "日本語"),
  ("ko", "한국어"),
  ("pl", "Polski"),
  ("pt", "Português"),
  ("ru", "Русский"),
  ("th", "ไทย"),
  ("tr", "Türkçe"),
  ("uk", "Українська"),
  ("zh", "简体中文"),
];

type Table = HashMap<String, String>;

fn tables() -> &'static HashMap<&'static str, Table> {
  static TABLES: OnceLock<HashMap<&'static str, Table>> = OnceLock::new();
  TABLES.get_or_init(|| FILES.iter().map(|(code, text)| (*code, parse_strings(text))).collect())
}

/// Reads `"key" = "value";` lines (these files have no comments and only `\"`, `\\` and `\n` escapes).
fn parse_strings(text: &str) -> Table {
  text
    .lines()
    .filter_map(|line| {
      let (key, value) = line.trim().strip_suffix(';')?.split_once(" = ")?;
      Some((unquote(key)?, unquote(value)?))
    })
    .collect()
}

fn unquote(text: &str) -> Option<String> {
  let inner = text.strip_prefix('"')?.strip_suffix('"')?;
  let mut out = String::with_capacity(inner.len());
  let mut chars = inner.chars();
  while let Some(c) = chars.next() {
    if c != '\\' {
      out.push(c);
      continue;
    }
    match chars.next()? {
      'n' => out.push('\n'),
      other => out.push(other),
    }
  }
  Some(out)
}

fn lookup(language: &str, key: &str) -> Option<&'static str> {
  tables().get(language).and_then(|table| table.get(key)).map(String::as_str)
}

/// The text of `key` in `language`; falls back to English, then to the key itself.
#[uniffi::export]
pub fn translate(language: String, key: String) -> String {
  lookup(&language, &key).or_else(|| lookup("en", &key)).map(str::to_string).unwrap_or(key)
}

/// `translate` with `%@`, `%d`, `%lld` and `%s` filled from `arguments` in order.
#[uniffi::export]
pub fn translate_format(language: String, key: String, arguments: Vec<String>) -> String {
  let text = translate(language, key);
  let mut arguments = arguments.into_iter();
  let mut out = String::with_capacity(text.len());
  let mut rest = text.as_str();
  while let Some(position) = rest.find('%') {
    out.push_str(&rest[..position]);
    rest = &rest[position + 1..];
    if let Some(tail) = rest.strip_prefix('%') {
      out.push('%');
      rest = tail;
      continue;
    }
    match ["lld", "@", "d", "s"].iter().find(|token| rest.starts_with(**token)) {
      Some(token) => {
        out.push_str(&arguments.next().unwrap_or_default());
        rest = &rest[token.len()..];
      }
      None => out.push('%'),
    }
  }
  out.push_str(rest);
  out
}

/// Translates display text that comes from the API (English) through its `translation_key`.
/// Returns the text unchanged when there is no translation.
#[uniffi::export]
pub fn translate_data(language: String, text: String) -> String {
  let Some(key) = crate::translation_key(text.clone()) else { return text };
  lookup(&language, &key).or_else(|| lookup("en", &key)).map(str::to_string).unwrap_or(text)
}

/// The first supported language of the user's preferred languages (`["tr-TR", "en"]`), English when none is.
#[uniffi::export]
pub fn resolve_language(preferred: Vec<String>) -> String {
  preferred
    .iter()
    .find_map(|identifier| NAMES.iter().find(|(code, _)| identifier.starts_with(code)).map(|(code, _)| (*code).to_string()))
    .unwrap_or_else(|| "en".into())
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct Choice {
  pub code: String,
  pub title: String,
}

#[uniffi::export]
pub fn languages() -> Vec<Choice> {
  NAMES.iter().map(|(code, title)| Choice { code: (*code).into(), title: (*title).into() }).collect()
}

#[uniffi::export]
pub fn platforms() -> Vec<Choice> {
  PLATFORMS.iter().map(|(code, title)| Choice { code: (*code).into(), title: (*title).into() }).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn translates_with_fallback() {
    assert_eq!(translate("tr".into(), "retry".into()), "Tekrar dene");
    assert_eq!(translate("xx".into(), "retry".into()), "Retry");
    assert_eq!(translate("tr".into(), "doesNotExist".into()), "doesNotExist");
  }

  #[test]
  fn formats_arguments() {
    assert_eq!(translate_format("en".into(), "seasonNumber".into(), vec!["7".into()]), "Season 7");
    assert_eq!(translate_format("en".into(), "masteryPoints".into(), vec!["6,000".into()]), "6,000 mastery");
  }

  #[test]
  fn resolves_language() {
    assert_eq!(resolve_language(vec!["tr-TR".into(), "en".into()]), "tr");
    assert_eq!(resolve_language(vec!["sv".into()]), "en");
  }

  #[test]
  fn every_language_is_loaded() {
    for (code, _) in FILES {
      assert!(tables()[code].len() > 100, "{code} has too few keys");
    }
  }
}
