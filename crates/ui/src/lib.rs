use gpui::{Context, FontWeight, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, prelude::*, px, rgb};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
enum Page {
  Home,
  Activities,
  Codex,
  Market,
  Settings,
  Activity(String, String),
  Category(String),
  Item(String, String),
}

pub struct Root {
  page: Page,
  tab: usize,
  platform: &'static str,
  dark: bool,
  buy: bool,
  data: Option<Value>,
  error: Option<String>,
  loading: bool,
  request: u64,
}

impl Default for Root {
  fn default() -> Self {
    Self { page: Page::Home, tab: 0, platform: "pc", dark: true, buy: false, data: None, error: None, loading: false, request: 0 }
  }
}

impl Root {
  pub fn new(cx: &mut Context<Self>) -> Self {
    let mut root = Self::default();
    root.open(Page::Home, cx);
    root
  }

  fn open(&mut self, page: Page, cx: &mut Context<Self>) {
    self.page = page;
    self.request += 1;
    self.data = None;
    self.error = None;
    self.loading = false;
    let url = match &self.page {
      Page::Home => Some(format!("https://api.warframestat.us/{}/", self.platform)),
      Page::Activity(_, endpoint) => Some(format!("https://api.warframestat.us/{}/{endpoint}/", self.platform)),
      Page::Category(category) => Some(format!("https://wfhub-api.cagatayldzz.com/api/{category}.json")),
      Page::Item(category, slug) => Some(format!("https://wfhub-api.cagatayldzz.com/api/{category}/{slug}.json")),
      Page::Market => Some("https://api.warframe.market/v2/orders/recent".into()),
      _ => None,
    };
    if let Some(url) = url {
      self.loading = true;
      let request = self.request;
      let task = cx.background_executor().spawn(async move {
        let mut data = fetch(&url)?;
        if url == "https://api.warframe.market/v2/orders/recent" {
          let items = fetch("https://api.warframe.market/v2/items")?;
          if let Some(object) = data.as_object_mut() {
            object.insert("items".into(), items);
          }
        }
        Ok::<_, String>(data)
      });
      cx.spawn(async move |this, cx| {
        let result = task.await;
        let _ = this.update(cx, |this, cx| {
          if this.request == request {
            this.loading = false;
            match result {
              Ok(data) => this.data = Some(data),
              Err(error) => this.error = Some(error),
            }
            cx.notify();
          }
        });
      })
      .detach();
    }
    cx.notify();
  }

  fn background(&self) -> u32 {
    if self.dark { 0x000000 } else { 0xffffff }
  }
  fn foreground(&self) -> u32 {
    if self.dark { 0xf5f5f5 } else { 0x171717 }
  }
  fn card(&self) -> u32 {
    if self.dark { 0x1b1b1e } else { 0xf1f1f4 }
  }
  fn accent(&self) -> u32 {
    if self.dark { 0xe0c873 } else { 0x73540a }
  }

  fn button(&self, id: impl Into<gpui::ElementId>, title: impl Into<SharedString>) -> gpui::Stateful<gpui::Div> {
    div().id(id).w_full().p_4().rounded(px(12.)).bg(rgb(self.card())).text_color(rgb(self.foreground())).cursor_pointer().child(title.into())
  }

  fn body(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
    let heading = |title: String| div().text_size(px(26.)).font_weight(FontWeight::BOLD).child(title);
    let mut body = div().w_full().flex().flex_col().gap_3();
    let title = match &self.page {
      Page::Home => "Welcome, Tenno".into(),
      Page::Activities => "Activities".into(),
      Page::Codex => "Codex".into(),
      Page::Market => "Market".into(),
      Page::Settings => "Settings".into(),
      Page::Activity(title, _) => title.clone(),
      Page::Category(category) => title_case(category),
      Page::Item(_, slug) => title_case(slug),
    };
    if matches!(self.page, Page::Activity(..) | Page::Category(..) | Page::Item(..)) {
      body = body.child(div().id("back").cursor_pointer().text_color(rgb(self.accent())).child("‹ Back").on_click(cx.listener(|this, _, _, cx| {
        let page = match &this.page {
          Page::Item(category, _) => Page::Category(category.clone()),
          Page::Category(_) => Page::Codex,
          _ => Page::Activities,
        };
        this.open(page, cx);
      })));
    }
    body = body.child(heading(title));
    match &self.page {
      Page::Activities => {
        for (index, (name, endpoint)) in wfhub_core::ACTIVITIES.iter().enumerate() {
          let title = (*name).to_string();
          let endpoint = (*endpoint).to_string();
          body = body.child(
            self
              .button(("activity", index), format!("{name}    ›"))
              .on_click(cx.listener(move |this, _, _, cx| this.open(Page::Activity(title.clone(), endpoint.clone()), cx))),
          );
        }
      }
      Page::Codex => {
        for (index, category) in wfhub_core::CATEGORIES.iter().enumerate() {
          let category = (*category).to_string();
          body = body.child(
            self
              .button(("category", index), format!("{}    ›", title_case(&category)))
              .on_click(cx.listener(move |this, _, _, cx| this.open(Page::Category(category.clone()), cx))),
          );
        }
      }
      Page::Settings => {
        body = body.child(div().text_color(rgb(self.accent())).child("PLATFORM"));
        for (index, (code, label)) in [("pc", "PC"), ("xb1", "Xbox"), ("swi", "Nintendo Switch"), ("ps4", "PlayStation")].into_iter().enumerate() {
          let selected = self.platform == code;
          body = body.child(self.button(("platform", index), format!("{} {label}", if selected { "●" } else { "○" })).on_click(cx.listener(
            move |this, _, _, cx| {
              this.platform = code;
              cx.notify();
            },
          )));
        }
        body = body.child(div().mt_3().text_color(rgb(self.accent())).child("APPEARANCE"));
        body = body.child(self.button("theme", if self.dark { "Dark  ›" } else { "Light  ›" }).on_click(cx.listener(|this, _, _, cx| {
          this.dark = !this.dark;
          cx.notify();
        })));
      }
      Page::Market => {
        body = body.child(div().flex().gap_2().children([(false, "Sell"), (true, "Buy")].into_iter().enumerate().map(|(index, (buy, title))| {
          div()
            .id(("market-type", index))
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded(px(8.))
            .bg(rgb(if self.buy == buy { self.accent() } else { self.card() }))
            .text_color(rgb(if self.buy == buy { self.background() } else { self.foreground() }))
            .child(title)
            .on_click(cx.listener(move |this, _, _, cx| {
              this.buy = buy;
              cx.notify();
            }))
        })));
      }
      _ => {}
    }
    if self.loading {
      body = body.child(div().p_4().child("Loading…"));
    }
    if let Some(error) = &self.error {
      body = body.child(div().p_4().child(format!("Unable to load: {error}")));
      body = body.child(self.button("retry", "Retry").on_click(cx.listener(|this, _, _, cx| this.open(this.page.clone(), cx))));
    }
    if let Some(data) = &self.data {
      match &self.page {
        Page::Home => {
          for (title, key) in
            [("Nightwave", "nightwave"), ("Events", "events"), ("News", "news"), ("Baro Ki'Teer", "voidTrader"), ("Alerts", "alerts")]
          {
            if let Some(value) = data.get(key) {
              body = body.child(self.data_card(title, value));
            }
          }
        }
        Page::Category(category) => {
          if let Some(items) = data.as_array() {
            for (index, item) in items.iter().enumerate() {
              let name = label(item);
              let slug = item.get("slug").and_then(Value::as_str).unwrap_or("").to_string();
              if !slug.is_empty() {
                let category = category.clone();
                body = body.child(
                  self
                    .button(("item", index), format!("{name}    ›"))
                    .on_click(cx.listener(move |this, _, _, cx| this.open(Page::Item(category.clone(), slug.clone()), cx))),
                );
              }
            }
          }
        }
        Page::Market => {
          if let Some(orders) = data.get("data").and_then(Value::as_array) {
            for order in orders.iter().filter(|v| v.get("type").and_then(Value::as_str) == Some(if self.buy { "buy" } else { "sell" })).take(35) {
              let item_id = order.get("itemId").and_then(Value::as_str);
              let item = data
                .get("items")
                .and_then(|v| v.get("data"))
                .and_then(Value::as_array)
                .and_then(|items| items.iter().find(|item| item.get("id").and_then(Value::as_str) == item_id));
              let title = item.and_then(|v| v.pointer("/i18n/en/name")).and_then(Value::as_str).unwrap_or("Item");
              let seller = order.pointer("/user/ingameName").and_then(Value::as_str).unwrap_or("–");
              let price = order.get("platinum").and_then(Value::as_u64).unwrap_or(0);
              let quantity = order.get("quantity").and_then(Value::as_u64).unwrap_or(0);
              let card = div()
                .w_full()
                .p_4()
                .rounded(px(12.))
                .bg(rgb(self.card()))
                .flex()
                .flex_col()
                .gap_2()
                .child(div().font_weight(FontWeight::BOLD).child(title.to_string()))
                .child(div().text_size(px(13.)).text_color(rgb(0x999999)).child(format!("{seller} · ×{quantity}")))
                .child(div().text_color(rgb(self.accent())).font_weight(FontWeight::BOLD).child(format!("◈ {price} platinum")));
              body = body.child(card);
            }
          }
        }
        Page::Activity(title, _) | Page::Item(_, title) => {
          if let Some(items) = data.as_array() {
            if items.is_empty() {
              body = body.child(div().p_4().child("No data available"));
            }
            for item in items {
              body = body.child(self.data_card(&label(item), item));
            }
          } else {
            body = body.child(self.data_card(title, data));
          }
        }
        _ => {}
      }
    }
    body.into_any_element()
  }

  fn data_card(&self, title: &str, value: &Value) -> impl IntoElement {
    let mut card = div()
      .w_full()
      .p_4()
      .rounded(px(12.))
      .bg(rgb(self.card()))
      .flex()
      .flex_col()
      .gap_2()
      .child(div().font_weight(FontWeight::BOLD).text_size(px(16.)).child(title.to_string()));
    if let Some(array) = value.as_array() {
      for item in array.iter().take(3) {
        card = card.child(div().text_color(rgb(self.foreground())).child(label(item)));
      }
    } else if let Some(object) = value.as_object() {
      for key in ["item", "description", "message", "season", "location", "node", "state", "faction", "missionType", "expiry", "platinum", "quantity"]
      {
        if let Some(v) = object.get(key) {
          let text = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => continue,
          };
          if !text.is_empty() && text != title {
            card = card.child(div().text_size(px(14.)).text_color(rgb(0x999999)).child(format!("{}: {text}", title_case(key))));
          }
        }
      }
    }
    card
  }
}

fn title_case(value: &str) -> String {
  value
    .replace('-', " ")
    .split_whitespace()
    .map(|part| {
      let mut chars = part.chars();
      chars.next().map(|first| first.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
    })
    .collect::<Vec<_>>()
    .join(" ")
}

fn label(item: &Value) -> String {
  for key in ["item", "name", "title", "node", "message", "description"] {
    if let Some(value) = item.get(key).and_then(Value::as_str).filter(|value| !value.is_empty()) {
      return value.to_string();
    }
  }
  "Details".into()
}

fn fetch(url: &str) -> Result<Value, String> {
  let agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(30))).build().new_agent();
  let response = agent.get(url).header("Accept", "application/json").call().map_err(|error| error.to_string())?;
  serde_json::from_reader(response.into_body().into_reader()).map_err(|error| error.to_string())
}

impl Render for Root {
  fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let tabs = ["Home", "Activities", "Market", "Codex", "Settings"];
    let selected = self.tab;
    div()
      .size_full()
      .flex()
      .flex_col()
      .bg(rgb(self.background()))
      .text_color(rgb(self.foreground()))
      .child(div().id("content").flex_1().overflow_y_scroll().p_4().child(self.body(cx)))
      .child(div().flex().justify_between().border_t_1().border_color(rgb(self.card())).children(tabs.into_iter().enumerate().map(
        |(index, title)| {
          div()
            .id(("tab", index))
            .cursor_pointer()
            .p_2()
            .text_size(px(12.))
            .text_color(rgb(if selected == index { self.accent() } else { self.foreground() }))
            .child(title)
            .on_click(cx.listener(move |this, _, _, cx| {
              this.tab = index;
              let page = match index {
                0 => Page::Home,
                1 => Page::Activities,
                2 => Page::Market,
                3 => Page::Codex,
                _ => Page::Settings,
              };
              this.open(page, cx);
            }))
        },
      )))
  }
}
