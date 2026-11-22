use std::collections::HashMap;
use std::sync::Mutex;

use crate::{CoreError, Json, parse, str_of};

const CATALOGUE_TTL: i64 = 3600;
const ROWS_PER_TYPE: usize = 35;

#[derive(Clone, Debug, uniffi::Record)]
pub struct OrderRow {
  pub name: String,
  /// warframe.market page of the item.
  pub link: Option<String>,
  pub icon: Option<String>,
  pub user: String,
  pub quantity: i64,
  pub platinum: i64,
}

#[derive(Clone, Debug, Default, uniffi::Record)]
pub struct MarketRows {
  pub sell: Vec<OrderRow>,
  pub buy: Vec<OrderRow>,
}

#[derive(Clone)]
struct Item {
  slug: String,
  name: String,
  icon: Option<String>,
}

#[derive(Default)]
struct Catalogue {
  items: HashMap<String, Item>,
  loaded_at: Option<i64>,
}

/// Recent market orders joined with the item catalogue. The catalogue is big and rarely changes, so it is kept here and the clients
/// download it again only when `needs_catalogue` says so.
#[derive(uniffi::Object, Default)]
pub struct MarketBook {
  catalogue: Mutex<Catalogue>,
}

#[uniffi::export]
impl MarketBook {
  #[uniffi::constructor]
  pub fn new() -> Self {
    Self::default()
  }

  pub fn needs_catalogue(&self, now_unix: i64) -> bool {
    self.catalogue.lock().map(|c| c.loaded_at.is_none_or(|at| now_unix - at >= CATALOGUE_TTL)).unwrap_or(true)
  }

  /// Reads the answer of `market_items_url`.
  pub fn set_catalogue(&self, body: String, now_unix: i64) -> Result<(), CoreError> {
    let parsed = parse(&body)?;
    let items: HashMap<String, Item> = data(&parsed)?
      .iter()
      .filter_map(|item| {
        let id = str_of(item, "id")?.to_string();
        let english = item.pointer("/i18n/en");
        let name = english.and_then(|e| str_of(e, "name")).unwrap_or("Item").to_string();
        let icon = english.and_then(|e| str_of(e, "icon")).map(|icon| format!("https://warframe.market/static/assets/{icon}"));
        Some((id, Item { slug: str_of(item, "slug").unwrap_or("").to_string(), name, icon }))
      })
      .collect();
    let mut catalogue = self.catalogue.lock().map_err(|_| CoreError::Shape)?;
    *catalogue = Catalogue { items, loaded_at: Some(now_unix) };
    Ok(())
  }

  /// Reads the answer of `market_orders_url`: the first 35 sell and 35 buy orders.
  pub fn rows(&self, orders_body: String) -> Result<MarketRows, CoreError> {
    let parsed = parse(&orders_body)?;
    let orders = data(&parsed)?;
    let catalogue = self.catalogue.lock().map_err(|_| CoreError::Shape)?;
    let rows_of = |kind: &str| -> Vec<OrderRow> {
      orders
        .iter()
        .filter(|order| str_of(order, "type") == Some(kind))
        .take(ROWS_PER_TYPE)
        .map(|order| {
          let item = str_of(order, "itemId").and_then(|id| catalogue.items.get(id));
          OrderRow {
            name: item.map(|i| i.name.clone()).unwrap_or_else(|| "Item".into()),
            link: item.filter(|i| !i.slug.is_empty()).map(|i| format!("https://warframe.market/items/{}", i.slug)),
            icon: item.and_then(|i| i.icon.clone()),
            user: order.pointer("/user/ingameName").and_then(Json::as_str).unwrap_or("–").to_string(),
            quantity: order.get("quantity").and_then(Json::as_i64).unwrap_or(0),
            platinum: order.get("platinum").and_then(Json::as_i64).unwrap_or(0),
          }
        })
        .collect()
    };
    Ok(MarketRows { sell: rows_of("sell"), buy: rows_of("buy") })
  }
}

fn data(value: &Json) -> Result<&Vec<Json>, CoreError> {
  value.get("data").and_then(Json::as_array).ok_or(CoreError::Shape)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn joins_orders_with_the_catalogue() {
    let book = MarketBook::new();
    assert!(book.needs_catalogue(0));
    book.set_catalogue(r#"{"data":[{"id":"a","slug":"ash_prime","i18n":{"en":{"name":"Ash Prime","icon":"i/a.png"}}}]}"#.into(), 100).unwrap();
    assert!(!book.needs_catalogue(200));
    assert!(book.needs_catalogue(100 + CATALOGUE_TTL));
    let rows = book
      .rows(r#"{"data":[{"type":"sell","itemId":"a","platinum":10,"quantity":2,"user":{"ingameName":"X"}},{"type":"buy","itemId":"zz"}]}"#.into())
      .unwrap();
    assert_eq!(rows.sell[0].name, "Ash Prime");
    assert_eq!(rows.sell[0].link.as_deref(), Some("https://warframe.market/items/ash_prime"));
    assert_eq!((rows.sell[0].platinum, rows.sell[0].quantity), (10, 2));
    assert_eq!((rows.buy[0].name.as_str(), rows.buy[0].user.as_str()), ("Item", "–"));
  }

  #[test]
  fn rejects_wrong_shapes() {
    assert!(MarketBook::new().rows("{}".into()).is_err());
  }
}
