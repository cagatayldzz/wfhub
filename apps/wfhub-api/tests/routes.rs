use wfhub_api::routes::{CATALOGUE_CATEGORIES, MASTERY_CATEGORIES, Route, item_reply, route};

#[test]
fn reads_the_routes() {
  assert_eq!(route("GET", "/api").unwrap(), Route::Redirect("/api/"));
  assert_eq!(route("GET", "/api/swagger").unwrap(), Route::Redirect("/api/swagger/"));
  assert!(matches!(route("GET", "/api/"), Ok(Route::File { .. })));
  assert!(matches!(route("GET", "/api/events.json"), Ok(Route::File { path, .. }) if path == "/data/events.json"));
  assert!(matches!(route("GET", "/api/mastery/k-drive.json"), Ok(Route::File { path, .. }) if path == "/data/mastery/k-drive.json"));
  assert!(matches!(route("GET", "/api/warframes.json"), Ok(Route::File { path, .. }) if path == "/data/catalogue/warframes.json"));
  assert_eq!(
    route("GET", "/api/warframes/excalibur.json").unwrap(),
    Route::Item { bucket: "/data/catalogue/warframes/e.json".into(), slug: "excalibur".into() }
  );
  assert_eq!(route("POST", "/api/feedback").unwrap(), Route::Feedback);
}

#[test]
fn refuses_the_rest() {
  assert_eq!(route("GET", "/api/feedback").err().unwrap().status, 405);
  assert_eq!(route("POST", "/api/events.json").err().unwrap().status, 405);
  for path in ["/api/nothing.json", "/api/mastery/nothing.json", "/api/warframes/Excalibur.json", "/api/warframes/../x.json", "/other"] {
    assert_eq!(route("GET", path).err().unwrap().status, 404, "{path}");
  }
}

#[test]
fn item_reply_picks_one_item() {
  let bucket = r#"{"ember":{"name":"Ember"},"equinox":{"name":"Equinox"}}"#;
  assert_eq!(item_reply(Some(bucket), "ember").body, r#"{"name":"Ember"}"#);
  assert_eq!(item_reply(Some(bucket), "excalibur").status, 404);
  assert_eq!(item_reply(None, "ember").status, 404);
}

#[test]
fn every_category_is_distinct() {
  for list in [&CATALOGUE_CATEGORIES[..], &MASTERY_CATEGORIES[..]] {
    let mut sorted = list.to_vec();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), list.len());
  }
}

#[test]
fn worldstate_routes() {
  use wfhub_api::worldstate::SECTIONS;
  assert_eq!(route("GET", "/api/worldstate/pc/fissures.json").unwrap(), Route::WorldState { platform: "pc", section: Some("fissures") });
  assert_eq!(route("GET", "/api/worldstate/ps5.json").unwrap(), Route::WorldState { platform: "ps4", section: None });
  for section in SECTIONS {
    assert!(matches!(route("GET", &format!("/api/worldstate/swi/{section}.json")), Ok(Route::WorldState { .. })), "{section}");
  }
  for path in [
    "/api/worldstate/ps3/fissures.json",
    "/api/worldstate/pc/nope.json",
    "/api/worldstate/pc/fissures",
    "/api/worldstate/.json",
    "/api/worldstate/pc/fissures/x.json",
  ] {
    assert_eq!(route("GET", path).unwrap_err().status, 404, "{path}");
  }
  assert_eq!(route("POST", "/api/worldstate/pc.json").unwrap_err().status, 405);
}
