//! Trade: Baro Ki'Teer, daily deals, flash sales and the Steel Path honors.

use super::names::Names;
use super::value::{array, eta, iso, ms, number, oid, text, timed};
use serde_json::{Value, json};

pub(super) fn void_trader(raw: &Value, now: i64, names: &Names) -> Value {
  let Some(trader) = array(raw, "VoidTraders").first() else { return json!({}) };
  let (activation, expiry) = (trader.get("Activation").and_then(ms), trader.get("Expiry").and_then(ms));
  let mut map = timed(oid(trader), activation, expiry, now);
  let node = text(trader, "Node");
  let character = if text(trader, "Character").contains("Baro") { "Baro Ki'Teer" } else { text(trader, "Character") };
  let inventory: Vec<Value> = trader
    .get("Manifest")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .map(
      |entry| json!({ "item": names.text(text(entry, "ItemType")), "ducats": number(entry, "PrimePrice"), "credits": number(entry, "RegularPrice") }),
    )
    .collect();
  let started = activation.is_some_and(|a| a <= now);
  map.insert("character".into(), json!(character));
  map.insert("location".into(), json!(names.node(node)));
  map.insert("nodeKey".into(), json!(node));
  map.insert("inventory".into(), json!(if started { inventory } else { Vec::new() }));
  map.insert("startString".into(), json!(activation.map(|a| eta(a - now)).unwrap_or_default()));
  map.insert("endString".into(), json!(expiry.map(|e| eta(e - now)).unwrap_or_default()));
  Value::Object(map)
}

pub(super) fn daily_deals(raw: &Value, now: i64, names: &Names) -> Value {
  let list = array(raw, "DailyDeals")
    .iter()
    .map(|deal| {
      let (activation, expiry) = (deal.get("Activation").and_then(ms), deal.get("Expiry").and_then(ms));
      let mut map = timed(format!("dailyDeal{}", expiry.unwrap_or(0)), activation, expiry, now);
      map.insert("item".into(), json!(names.text(text(deal, "StoreItem"))));
      map.insert("sku".into(), json!(text(deal, "StoreItem")));
      map.insert("originalPrice".into(), json!(number(deal, "OriginalPrice")));
      map.insert("salePrice".into(), json!(number(deal, "SalePrice")));
      map.insert("discount".into(), json!(number(deal, "Discount")));
      map.insert("total".into(), json!(number(deal, "AmountTotal")));
      map.insert("sold".into(), json!(number(deal, "AmountSold")));
      Value::Object(map)
    })
    .collect();
  Value::Array(list)
}

pub(super) fn flash_sales(raw: &Value, now: i64, names: &Names) -> Value {
  let list = array(raw, "FlashSales")
    .iter()
    .filter_map(|sale| {
      let (activation, expiry) = (sale.get("StartDate").and_then(ms), sale.get("EndDate").and_then(ms));
      if expiry.is_some_and(|e| e < now) || activation.is_some_and(|a| a > now) {
        return None;
      }
      let mut map = timed(format!("{}{}", text(sale, "TypeName"), expiry.unwrap_or(0)), activation, expiry, now);
      map.insert("item".into(), json!(names.text(text(sale, "TypeName"))));
      map.insert("discount".into(), json!(number(sale, "Discount")));
      map.insert("regularOverride".into(), json!(number(sale, "RegularOverride")));
      map.insert("premiumOverride".into(), json!(number(sale, "PremiumOverride")));
      map.insert("isShownInMarket".into(), json!(sale.get("ShowInMarket").and_then(Value::as_bool).unwrap_or(false)));
      map.insert("isFeatured".into(), json!(sale.get("Featured").and_then(Value::as_bool).unwrap_or(false)));
      map.insert("isPopular".into(), json!(sale.get("Popular").and_then(Value::as_bool).unwrap_or(false)));
      Some(Value::Object(map))
    })
    .collect();
  Value::Array(list)
}

pub(super) fn steel_path(now: i64, names: &Names) -> Value {
  const START: i64 = 1_605_484_800_000; // 2020-11-16T00:00:00Z, a Monday
  const WEEK: i64 = 604_800_000;
  let steel = names.tables.get("steelPath").cloned().unwrap_or(Value::Null);
  let rotation = steel.get("rotation").and_then(Value::as_array).cloned().unwrap_or_default();
  let evergreens = steel.get("evergreen").cloned().unwrap_or_else(|| json!([]));
  let weeks = (now - START).div_euclid(WEEK);
  let index = if rotation.is_empty() { 0 } else { weeks.rem_euclid(rotation.len() as i64) as usize };
  let activation = START + weeks * WEEK;
  let expiry = activation + WEEK - 1000;
  let day = now.div_euclid(86_400_000) * 86_400_000;
  json!({
    "currentReward": rotation.get(index).cloned().unwrap_or(Value::Null),
    "activation": iso(activation),
    "expiry": iso(expiry),
    "remaining": eta(expiry - now),
    "rotation": rotation,
    "evergreens": evergreens,
    "incursions": { "id": format!("spi:{day}"), "activation": iso(day), "expiry": iso(day + 86_399_000) },
  })
}
