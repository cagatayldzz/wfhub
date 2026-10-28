use gpui::{Context, FontWeight, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, img, prelude::*, px, rgb};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone, PartialEq, Eq)]
enum Page {
  Home,
  Activities,
  Cycles,
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
  tab_pages: [Page; 5],
  history: [Vec<Page>; 5],
  scroll: gpui::ScrollHandle,
  platform: &'static str,
  dark: bool,
  buy: bool,
  data: Option<Value>,
  error: Option<String>,
  loading: bool,
  request: u64,
  search: String,
  search_focus: Option<gpui::FocusHandle>,
  name_focus: Option<gpui::FocusHandle>,
  name: String,
  saved_name: String,
  draft_platform: &'static str,
  platform_menu: bool,
  saved_message: String,
}

impl Default for Root {
  fn default() -> Self {
    Self {
      page: Page::Home,
      tab: 0,
      tab_pages: [Page::Home, Page::Activities, Page::Market, Page::Codex, Page::Settings],
      history: Default::default(),
      scroll: gpui::ScrollHandle::new(),
      platform: "pc",
      dark: true,
      buy: false,
      data: None,
      error: None,
      loading: false,
      request: 0,
      search: String::new(),
      search_focus: None,
      name_focus: None,
      name: "Tenno".into(),
      saved_name: "Tenno".into(),
      draft_platform: "pc",
      platform_menu: false,
      saved_message: String::new(),
    }
  }
}

impl Root {
  pub fn new(cx: &mut Context<Self>) -> Self {
    let mut root = Self { search_focus: Some(cx.focus_handle()), name_focus: Some(cx.focus_handle()), ..Self::default() };
    if let Some(settings) = settings_path().and_then(|path| std::fs::read(path).ok()).and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()) {
      if let Some(name) = settings.get("name").and_then(Value::as_str) {
        root.name = name.into();
        root.saved_name = name.into();
      }
      if let Some(platform) = settings.get("platform").and_then(Value::as_str) {
        root.platform = platform_code(platform);
        root.draft_platform = root.platform;
      }
      if let Some(dark) = settings.get("dark").and_then(Value::as_bool) {
        root.dark = dark;
      }
    }
    root.open(Page::Home, cx);
    let executor = cx.background_executor().clone();
    cx.spawn(async move |this, cx| {
      loop {
        executor.timer(Duration::from_secs(1)).await;
        if this.update(cx, |_, cx| cx.notify()).is_err() {
          break;
        }
      }
    })
    .detach();
    root
  }

  fn open(&mut self, page: Page, cx: &mut Context<Self>) {
    if self.page != page {
      self.history[self.tab].push(self.page.clone());
    }
    self.load_page(page, cx);
  }

  fn go_back(&mut self, cx: &mut Context<Self>) {
    if let Some(page) = self.history[self.tab].pop() {
      self.load_page(page, cx);
    }
  }

  fn select_tab(&mut self, tab: usize, cx: &mut Context<Self>) {
    if tab != self.tab {
      self.tab_pages[self.tab] = self.page.clone();
      self.tab = tab;
      self.load_page(self.tab_pages[tab].clone(), cx);
    }
  }

  fn load_page(&mut self, page: Page, cx: &mut Context<Self>) {
    if self.page != page {
      self.scroll.set_offset(gpui::point(px(0.), px(0.)));
    }
    self.page = page;
    self.tab_pages[self.tab] = self.page.clone();
    self.request += 1;
    self.data = None;
    self.error = None;
    self.loading = false;
    if matches!(self.page, Page::Activities) {
      self.loading = true;
      let request = self.request;
      let task = cx.background_executor().spawn(async move {
        std::thread::scope(|scope| {
          for (_, endpoint) in activity_routes() {
            scope.spawn(move || cache_image(Some(&activity_image(endpoint))));
          }
        });
      });
      cx.spawn(async move |this, cx| {
        task.await;
        let _ = this.update(cx, |this, cx| {
          if this.request == request {
            this.loading = false;
            cx.notify();
          }
        });
      })
      .detach();
    }
    if matches!(self.page, Page::Codex) {
      self.loading = true;
      let request = self.request;
      let task = cx.background_executor().spawn(async move {
        std::thread::scope(|scope| {
          for category in wfhub_core::CATEGORIES {
            scope.spawn(move || cache_image(Some(&format!("https://cdn.cagatayldzz.com/constants/wfhub/codex/{category}.png"))));
          }
        });
      });
      cx.spawn(async move |this, cx| {
        task.await;
        let _ = this.update(cx, |this, cx| {
          if this.request == request {
            this.loading = false;
            cx.notify();
          }
        });
      })
      .detach();
    }
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
        if url.starts_with("https://api.warframestat.us/")
          && url.matches('/').count() == 4
          && let Ok(banners) = fetch("https://cdn.cagatayldzz.com/constants/wfhub/api/events.json")
          && let Some(object) = data.as_object_mut()
        {
          object.insert("banners".into(), banners);
        }
        if url == "https://api.warframe.market/v2/orders/recent" {
          let items = fetch("https://api.warframe.market/v2/items")?;
          let orders = data.get("data").and_then(Value::as_array);
          let catalogue = items.get("data").and_then(Value::as_array);
          if let (Some(orders), Some(catalogue)) = (orders, catalogue) {
            let ids: std::collections::HashSet<&str> = ["sell", "buy"]
              .into_iter()
              .flat_map(|kind| {
                orders
                  .iter()
                  .filter(move |order| order.get("type").and_then(Value::as_str) == Some(kind))
                  .take(35)
                  .filter_map(|order| order.get("itemId").and_then(Value::as_str))
              })
              .collect();
            std::thread::scope(|scope| {
              for item in catalogue.iter().filter(|item| item.get("id").and_then(Value::as_str).is_some_and(|id| ids.contains(id))) {
                if let Some(icon) = item.pointer("/i18n/en/icon").and_then(Value::as_str) {
                  let image = format!("https://warframe.market/static/assets/{icon}");
                  scope.spawn(move || cache_image(Some(&image)));
                }
              }
            });
          }
          if let Some(object) = data.as_object_mut() {
            object.insert("items".into(), items);
          }
        }
        if let Some(banners) = data.get("banners").and_then(Value::as_array) {
          for banner in banners {
            cache_image(banner.get("imageLink").and_then(Value::as_str));
          }
        }
        if let Some(news) = data.get("news").and_then(Value::as_array) {
          for item in news.iter().take(2) {
            cache_image(item.get("imageLink").and_then(Value::as_str));
          }
        }
        if url.starts_with("https://api.warframestat.us/") && url.matches('/').count() == 4 {
          for category in ["excalibur-prime", "secondary"] {
            cache_image(Some(&format!("https://cdn.cagatayldzz.com/constants/wfhub/codex/{category}.png")));
          }
          cache_image(Some("https://cdn.cagatayldzz.com/constants/wfhub/sections/baro-ki-teer.png"));
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
}

mod navigation;
mod screens;
mod support;
use support::*;
