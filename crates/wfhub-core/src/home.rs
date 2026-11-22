use crate::{CoreError, Json, is_web_link, num_of, parse, str_of};

/// The answer of one `HomeRequest`; `body` is `None` when the request failed.
#[derive(Clone, Debug, uniffi::Record)]
pub struct HomeResponse {
  pub key: String,
  pub body: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NightwaveSummary {
  pub season: Option<i64>,
  pub expiry: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct TraderSummary {
  pub location: Option<String>,
  pub expiry: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct EventCard {
  pub title: String,
  pub node: String,
  /// 0...100
  pub progress: Option<f64>,
  pub expiry: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct AlertCard {
  pub title: String,
  pub rewards: String,
  pub expiry: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NewsCard {
  pub title: String,
  pub image: Option<String>,
  pub date: Option<String>,
  pub link: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct HomeBanner {
  pub title: String,
  pub link: String,
  pub image: Option<String>,
}

/// Everything the Home screen shows. `failed` lists the keys that could not be loaded (the screen offers a retry).
#[derive(Clone, Debug, Default, uniffi::Record)]
pub struct HomeState {
  pub nightwave: Option<NightwaveSummary>,
  pub trader: Option<TraderSummary>,
  pub events: Vec<EventCard>,
  pub alerts: Vec<AlertCard>,
  pub news: Vec<NewsCard>,
  pub banners: Vec<HomeBanner>,
  pub failed: Vec<String>,
}

fn owned(value: &Json, key: &str) -> Option<String> {
  str_of(value, key).map(str::to_string)
}

fn objects(body: &str) -> Result<Vec<Json>, CoreError> {
  match parse(body)? {
    Json::Array(items) => Ok(items),
    _ => Err(CoreError::Shape),
  }
}

fn object(body: &str) -> Result<Json, CoreError> {
  match parse(body)? {
    value @ Json::Object(_) => Ok(value),
    _ => Err(CoreError::Shape),
  }
}

fn event_progress(event: &Json) -> Option<f64> {
  if let (Some(current), Some(maximum)) = (event.get("currentScore").and_then(Json::as_f64), event.get("maximumScore").and_then(Json::as_f64))
    && maximum > 0.0
  {
    return Some((100.0 - current / maximum * 100.0).clamp(0.0, 100.0));
  }
  event.get("health").and_then(Json::as_f64).map(|health| health.clamp(0.0, 100.0))
}

fn alert_card(alert: &Json) -> AlertCard {
  let mission = alert.get("mission").cloned().unwrap_or(Json::Null);
  let reward = mission.get("reward").cloned().unwrap_or(Json::Null);
  let items: Vec<String> =
    reward.get("items").and_then(Json::as_array).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
  let counted: Vec<String> = reward
    .get("countedItems")
    .and_then(Json::as_array)
    .map(|a| {
      a.iter()
        .map(|c| format!("{} {}", num_of(c, "count").map(|n| n.to_string()).unwrap_or_else(|| "–".into()), str_of(c, "type").unwrap_or("")))
        .collect()
    })
    .unwrap_or_default();
  let title = if str_of(alert, "tag") == Some("LotusGift") {
    "Gifts of the Lotus".to_string()
  } else {
    owned(&mission, "node").unwrap_or_else(|| "Alert".into())
  };
  AlertCard { title, rewards: (if items.is_empty() { counted } else { items }).join(", "), expiry: owned(alert, "expiry") }
}

fn news_card(item: &Json, language: &str) -> NewsCard {
  let translations = item.get("translations");
  let translated = |code: &str| translations.and_then(|t| str_of(t, code));
  let title = translated(language).or_else(|| translated("en")).or_else(|| str_of(item, "message")).unwrap_or("News");
  NewsCard {
    title: title.to_string(),
    image: owned(item, "imageLink"),
    date: owned(item, "date"),
    link: owned(item, "link").filter(|l| is_web_link(l.clone())),
  }
}

fn banner(item: &Json, language: &str) -> Option<HomeBanner> {
  let title = owned(item, "title")?;
  let link = owned(item, "link")?.replace("{lang}", language);
  is_web_link(link.clone()).then(|| HomeBanner { title, link, image: owned(item, "imageLink") })
}

/// Reads the answers of `home_requests`. A key that is missing, failed or has an unexpected shape lands in `failed`.
#[uniffi::export]
pub fn parse_home(responses: Vec<HomeResponse>, language: String) -> HomeState {
  let mut state = HomeState::default();
  for key in ["nightwave", "voidTrader", "events", "alerts", "news", "banners"] {
    let body = responses.iter().find(|r| r.key == key).and_then(|r| r.body.as_deref());
    let read = body.map(|body| -> Result<(), CoreError> {
      match key {
        "nightwave" => {
          let value = object(body)?;
          state.nightwave = Some(NightwaveSummary { season: num_of(&value, "season").map(|s| s as i64), expiry: owned(&value, "expiry") });
        }
        "voidTrader" => {
          let value = object(body)?;
          state.trader = Some(TraderSummary { location: owned(&value, "location"), expiry: owned(&value, "expiry") });
        }
        "events" => {
          state.events = objects(body)?
            .iter()
            .map(|event| EventCard {
              title: owned(event, "description").unwrap_or_else(|| "Event".into()),
              node: owned(event, "node").or_else(|| owned(event, "victimNode")).unwrap_or_else(|| "–".into()),
              progress: event_progress(event),
              expiry: owned(event, "expiry"),
            })
            .collect()
        }
        "alerts" => state.alerts = objects(body)?.iter().map(alert_card).collect(),
        "news" => {
          let mut items = objects(body)?;
          items.sort_by(|a, b| str_of(b, "date").unwrap_or("").cmp(str_of(a, "date").unwrap_or("")));
          state.news = items.iter().take(2).map(|item| news_card(item, &language)).collect();
        }
        _ => state.banners = objects(body)?.iter().filter_map(|item| banner(item, &language)).collect(),
      }
      Ok(())
    });
    if !matches!(read, Some(Ok(()))) {
      state.failed.push(key.to_string());
    }
  }
  state
}

#[cfg(test)]
mod tests {
  use super::*;

  fn response(key: &str, body: &str) -> HomeResponse {
    HomeResponse { key: key.into(), body: Some(body.into()) }
  }

  #[test]
  fn reads_home() {
    let state = parse_home(
      vec![
        response("nightwave", r#"{"season":4,"expiry":"2030-01-01T00:00:00.000Z"}"#),
        response("events", r#"[{"description":"Op","currentScore":25,"maximumScore":100,"victimNode":"Mars"}]"#),
        response("alerts", r#"[{"tag":"LotusGift","mission":{"node":"X","reward":{"countedItems":[{"count":3,"type":"Credits"}]}}}]"#),
        response("news", r#"[{"date":"2024","message":"old"},{"date":"2025","message":"new","translations":{"tr":"yeni"},"link":"javascript:1"}]"#),
        response("banners", r#"[{"title":"Site","link":"https://x.y/{lang}/"},{"title":"Bad","link":"ftp://x"}]"#),
        HomeResponse { key: "voidTrader".into(), body: None },
      ],
      "tr".into(),
    );
    assert_eq!(state.nightwave.unwrap().season, Some(4));
    assert_eq!(state.events[0].progress, Some(75.0));
    assert_eq!(state.events[0].node, "Mars");
    assert_eq!((state.alerts[0].title.as_str(), state.alerts[0].rewards.as_str()), ("Gifts of the Lotus", "3 Credits"));
    assert_eq!(state.news[0].title, "yeni");
    assert_eq!(state.news[0].link, None);
    assert_eq!(state.banners.len(), 1);
    assert_eq!(state.banners[0].link, "https://x.y/tr/");
    assert_eq!(state.failed, vec!["voidTrader"]);
  }
}
