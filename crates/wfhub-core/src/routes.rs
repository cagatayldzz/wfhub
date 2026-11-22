/// A screen the user can open: `title` is English display text (translate it with `translate_data`), `endpoint` the API name.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct Route {
  pub title: String,
  pub endpoint: String,
}

fn route(title: &str, endpoint: &str) -> Route {
  Route { title: title.into(), endpoint: endpoint.into() }
}

/// Every world state route the app knows.
const ACTIVITIES: &[(&str, &str)] = &[
  ("Alerts", "alerts"),
  ("Baro Ki'Teer", "voidTrader"),
  ("Fissures", "fissures"),
  ("Nightwave", "nightwave"),
  ("Events", "events"),
  ("Invasions", "invasions"),
  ("Arbitration", "arbitration"),
  ("Archon Hunt", "archonHunt"),
  ("Steel Path", "steelPath"),
  ("Sentient Outposts", "sentientOutposts"),
  ("Simaris", "simaris"),
  ("Construction Progress", "constructionProgress"),
  ("News", "news"),
  ("Market", "flashSales"),
  ("Sortie", "sortie"),
  ("Daily Deals", "dailyDeals"),
  ("Earth Cycle", "earthCycle"),
  ("Cetus Cycle", "cetusCycle"),
  ("Vallis Cycle", "vallisCycle"),
  ("Cambion Cycle", "cambionCycle"),
  ("Duviri Cycle", "duviriCycle"),
  ("Zariman Cycle", "zarimanCycle"),
];

const CATEGORIES: &[&str] = &[
  "arcanes",
  "arch-gun",
  "arch-melee",
  "archwing",
  "enemy",
  "fish",
  "gear",
  "glyphs",
  "melee",
  "misc",
  "mods",
  "node",
  "pets",
  "primary",
  "quests",
  "railjack",
  "relics",
  "resources",
  "secondary",
  "sentinels",
  "sentinelweapons",
  "sigils",
  "skins",
  "warframes",
];

/// Order of the Activities list. `cycles` is the one entry that is a screen of its own instead of an endpoint.
const ACTIVITY_ORDER: &[&str] = &[
  "alerts",
  "voidTrader",
  "fissures",
  "cycles",
  "dailyDeals",
  "nightwave",
  "events",
  "invasions",
  "arbitration",
  "archonHunt",
  "steelPath",
  "sentientOutposts",
  "simaris",
  "constructionProgress",
  "news",
  "flashSales",
  "sortie",
];

const CYCLES: &[(&str, &str)] = &[
  ("Plains of Eidolon", "cetusCycle"),
  ("Orb Vallis", "vallisCycle"),
  ("Cambion Drift", "cambionCycle"),
  ("Zariman Ten Zero", "zarimanCycle"),
  ("Earth Forests", "earthCycle"),
  ("Duviri", "duviriCycle"),
];

/// Looks an activity route up by its endpoint; unknown endpoints get `fallback` as title.
#[uniffi::export]
pub fn activity_route(endpoint: String, fallback: String) -> Route {
  ACTIVITIES.iter().find(|(_, e)| *e == endpoint).map(|(t, e)| route(t, e)).unwrap_or(Route { title: fallback, endpoint })
}

/// The Activities tab, in display order.
#[uniffi::export]
pub fn activity_routes() -> Vec<Route> {
  ACTIVITY_ORDER
    .iter()
    .filter_map(|endpoint| {
      if *endpoint == "cycles" { Some(route("Cycles", "cycles")) } else { ACTIVITIES.iter().find(|(_, e)| e == endpoint).map(|(t, e)| route(t, e)) }
    })
    .collect()
}

/// The six cycles of the Cycles screen.
#[uniffi::export]
pub fn cycle_routes() -> Vec<Route> {
  CYCLES.iter().map(|(t, e)| route(t, e)).collect()
}

/// Codex categories, sorted by title.
#[uniffi::export]
pub fn category_routes() -> Vec<Route> {
  let mut routes: Vec<Route> = CATEGORIES.iter().map(|c| Route { title: capitalized(&c.replace('-', " ")), endpoint: (*c).into() }).collect();
  routes.sort_by_key(|r| r.title.to_lowercase());
  routes
}

/// "arch gun" -> "Arch Gun" (every word starts upper case, the rest is lower case).
pub(crate) fn capitalized(text: &str) -> String {
  text
    .split(' ')
    .map(|word| {
      let mut chars = word.chars();
      chars.next().map(|first| first.to_uppercase().chain(chars.flat_map(char::to_lowercase)).collect::<String>()).unwrap_or_default()
    })
    .collect::<Vec<_>>()
    .join(" ")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn activities_list_has_cycles_screen() {
    let routes = activity_routes();
    assert_eq!(routes.len(), ACTIVITY_ORDER.len());
    assert_eq!(routes[3], route("Cycles", "cycles"));
    assert_eq!(routes[1].title, "Baro Ki'Teer");
  }

  #[test]
  fn categories_are_sorted_and_titled() {
    let routes = category_routes();
    assert_eq!(routes.len(), 24);
    assert!(routes.iter().any(|r| r.title == "Arch Gun" && r.endpoint == "arch-gun"));
    assert!(routes.windows(2).all(|w| w[0].title.to_lowercase() <= w[1].title.to_lowercase()));
  }
}
