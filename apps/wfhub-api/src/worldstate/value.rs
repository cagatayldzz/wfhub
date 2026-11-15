//! Reading the raw state (Mongo-style dates and ids) and the fields every timed thing has.

use super::names::Names;
use serde_json::{Map, Value, json};

/// `{"$date":{"$numberLong":"1794675600000"}}` (or a plain number) -> milliseconds.
pub fn ms(value: &Value) -> Option<i64> {
  match value {
    Value::Number(number) => number.as_i64(),
    Value::Object(map) => {
      let date = map.get("$date")?;
      match date {
        Value::Object(inner) => match inner.get("$numberLong")? {
          Value::String(text) => text.parse().ok(),
          Value::Number(number) => number.as_i64(),
          _ => None,
        },
        Value::Number(number) => number.as_i64(),
        _ => None,
      }
    }
    _ => None,
  }
}

pub(super) fn oid(value: &Value) -> String {
  value.get("_id").and_then(|id| id.get("$oid")).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Milliseconds since the epoch as `2026-11-15T08:00:00.000Z`.
pub fn iso(milliseconds: i64) -> String {
  let seconds = milliseconds.div_euclid(1000);
  let millis = milliseconds.rem_euclid(1000);
  let days = seconds.div_euclid(86_400);
  let rest = seconds.rem_euclid(86_400);
  // civil_from_days (Howard Hinnant)
  let z = days + 719_468;
  let era = z.div_euclid(146_097);
  let doe = z.rem_euclid(146_097);
  let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
  let year = yoe + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let day = doy - (153 * mp + 2) / 5 + 1;
  let month = if mp < 10 { mp + 3 } else { mp - 9 };
  let year = if month <= 2 { year + 1 } else { year };
  format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}Z", rest / 3600, rest % 3600 / 60, rest % 60)
}

/// `3725000` -> `1h 2m 5s`; negative times get a `-`.
pub fn eta(delta_ms: i64) -> String {
  let sign = if delta_ms < 0 { "-" } else { "" };
  let mut seconds = delta_ms.abs() / 1000;
  let mut parts = Vec::new();
  for (unit, size) in [("d", 86_400), ("h", 3600), ("m", 60)] {
    if seconds >= size {
      parts.push(format!("{}{unit}", seconds / size));
      seconds %= size;
    }
  }
  parts.push(format!("{seconds}s"));
  format!("{sign}{}", parts.join(" "))
}

/// An object that starts with the fields every timed thing has. Nothing in it depends on the clock (no `eta`, `expired`,
/// `active`): the client counts down from `activation` and `expiry`, so the answer is the same for as long as the game's
/// state is.
pub(super) fn timed(id: String, activation: Option<i64>, expiry: Option<i64>) -> Map<String, Value> {
  let mut map = Map::new();
  map.insert("id".into(), json!(id));
  if let Some(activation) = activation {
    map.insert("activation".into(), json!(iso(activation)));
  }
  if let Some(expiry) = expiry {
    map.insert("expiry".into(), json!(iso(expiry)));
  }
  map
}

pub(super) fn array<'a>(raw: &'a Value, key: &str) -> &'a [Value] {
  raw.get(key).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

pub(super) fn text<'a>(value: &'a Value, key: &str) -> &'a str {
  value.get(key).and_then(Value::as_str).unwrap_or("")
}

pub(super) fn number(value: &Value, key: &str) -> i64 {
  value.get(key).and_then(Value::as_i64).unwrap_or(0)
}

pub(super) fn percent(value: f64) -> String {
  format!("{value:.2}")
}

/// `{credits, items, countedItems}` of the raw state -> `{credits, items, countedItems, itemString, asString}`.
pub(super) fn reward(raw: &Value, names: &Names) -> Value {
  let credits = number(raw, "credits");
  let items: Vec<String> =
    raw.get("items").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(|path| names.text(path)).collect();
  let counted: Vec<Value> = raw
    .get("countedItems")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .map(|entry| json!({ "count": number(entry, "ItemCount"), "type": names.text(text(entry, "ItemType")), "key": text(entry, "ItemType") }))
    .collect();
  let mut shown: Vec<String> = counted.iter().map(|entry| format!("{} {}", entry["count"], entry["type"].as_str().unwrap_or(""))).collect();
  shown.extend(items.iter().cloned());
  let item_string = shown.join(" + ");
  let as_string = if credits > 0 {
    if item_string.is_empty() { format!("{credits}cr") } else { format!("{credits}cr + {item_string}") }
  } else {
    item_string.clone()
  };
  json!({ "credits": credits, "items": items, "countedItems": counted, "itemString": item_string, "asString": as_string })
}
