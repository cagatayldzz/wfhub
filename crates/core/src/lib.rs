//! Shared application routes and endpoint construction.

pub const CATEGORIES: &[&str] = &[
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

pub const ACTIVITIES: &[(&str, &str)] = &[
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

pub fn valid_platform(value: &str) -> bool {
  matches!(value, "pc" | "ps4" | "xb1" | "swi")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn platform_is_limited_to_supported_values() {
    assert!(valid_platform("pc"));
    assert!(!valid_platform("../pc"));
  }

  #[test]
  fn routes_are_unique() {
    let mut routes: Vec<_> = ACTIVITIES.iter().map(|(_, endpoint)| endpoint).collect();
    routes.sort_unstable();
    routes.dedup();
    assert_eq!(routes.len(), ACTIVITIES.len());
  }
}
