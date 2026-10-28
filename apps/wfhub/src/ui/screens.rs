use super::*;

impl Root {
  pub(super) fn body(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
    let heading = |title: String| div().text_size(px(28.)).font_weight(FontWeight::BOLD).child(title);
    let mut body = div().w_full().flex().flex_col().gap_3();
    let title = match &self.page {
      Page::Home => format!("Welcome, {}", self.saved_name),
      Page::Activities => "Activities".into(),
      Page::Cycles => "Cycles".into(),
      Page::Codex => "Codex".into(),
      Page::Market => "Market".into(),
      Page::Settings => "Settings".into(),
      Page::Activity(title, _) => title.clone(),
      Page::Category(category) => title_case(category),
      Page::Item(_, slug) => title_case(slug),
    };
    if matches!(self.page, Page::Activity(..) | Page::Category(..) | Page::Item(..) | Page::Cycles) {
      body = body.child(
        div()
          .relative()
          .w_full()
          .h(px(48.))
          .flex()
          .items_center()
          .justify_center()
          .child(div().font_weight(FontWeight::BOLD).text_size(px(17.)).child(title.clone()))
          .child(
            div()
              .id("back")
              .absolute()
              .left(px(0.))
              .top(px(2.))
              .size(px(44.))
              .rounded_full()
              .bg(rgb(self.card()))
              .cursor_pointer()
              .flex()
              .items_center()
              .justify_center()
              .text_size(px(28.))
              .child("‹")
              .on_click(cx.listener(|this, _, _, cx| {
                this.go_back(cx);
              })),
          ),
      );
    } else {
      body = body.child(heading(title));
    }
    match &self.page {
      Page::Activities => {
        body = body.child(div().text_size(px(14.)).text_color(rgb(0x999999)).child("Worldstate Details"));
        let focus = self.search_focus.as_ref().unwrap().clone();
        body = body.child(
          div()
            .id("activity-search")
            .track_focus(&focus)
            .cursor_text()
            .mt_2()
            .mb_1()
            .p_3()
            .rounded(px(10.))
            .bg(rgb(self.card()))
            .text_color(rgb(0x999999))
            .child(format!("⌕   {}", if self.search.is_empty() { "Search activities" } else { &self.search }))
            .on_click(move |_, window, _| window.focus(&focus))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
              if event.keystroke.key == "backspace" {
                this.search.pop();
              } else if let Some(text) = &event.keystroke.key_char {
                this.search.push_str(text);
              }
              cx.notify();
            })),
        );
        for (index, (name, endpoint)) in
          activity_routes().into_iter().filter(|(name, _)| name.to_lowercase().contains(&self.search.to_lowercase())).enumerate()
        {
          let title = (*name).to_string();
          let endpoint = (*endpoint).to_string();
          body = body.child(
            div()
              .id(("activity", index))
              .w_full()
              .p_3()
              .rounded(px(12.))
              .bg(rgb(self.card()))
              .border_1()
              .border_color(rgb(if self.dark { 0x303033 } else { 0xe2e2e5 }))
              .cursor_pointer()
              .flex()
              .items_center()
              .gap_3()
              .child(if self.loading {
                div().size(px(48.)).into_any_element()
              } else {
                img(image_path(&activity_image(&endpoint))).size(px(48.)).into_any_element()
              })
              .child(div().flex_1().font_weight(FontWeight::BOLD).child(name.to_string()))
              .child(div().text_color(rgb(0x999999)).child("›"))
              .on_click(cx.listener(move |this, _, _, cx| {
                this.open(if endpoint == "cycles" { Page::Cycles } else { Page::Activity(title.clone(), endpoint.clone()) }, cx)
              })),
          );
        }
      }
      Page::Cycles => {
        for (index, (name, endpoint)) in [
          ("Plains of Eidolon", "cetusCycle"),
          ("Orb Vallis", "vallisCycle"),
          ("Cambion Drift", "cambionCycle"),
          ("Zariman Ten Zero", "zarimanCycle"),
          ("Earth Forests", "earthCycle"),
          ("Duviri", "duviriCycle"),
        ]
        .into_iter()
        .enumerate()
        {
          body = body.child(
            self
              .button(("cycle", index), format!("{name}   ›"))
              .on_click(cx.listener(move |this, _, _, cx| this.open(Page::Activity(name.into(), endpoint.into()), cx))),
          );
        }
      }
      Page::Codex => {
        for (row, categories) in wfhub_core::CATEGORIES.chunks(2).enumerate() {
          let mut grid_row = div().w_full().flex().gap_2();
          for (column, category) in categories.iter().enumerate() {
            let category = (*category).to_string();
            let image_url = format!("https://cdn.cagatayldzz.com/constants/wfhub/codex/{category}.png");
            let artwork = if self.loading {
              div().w_full().h(px(100.)).into_any_element()
            } else {
              img(image_path(&image_url)).w_full().h(px(100.)).into_any_element()
            };
            grid_row = grid_row.child(
              div()
                .id(("category", row * 2 + column))
                .flex_1()
                .min_w_0()
                .cursor_pointer()
                .p(px(10.))
                .rounded(px(10.))
                .bg(gpui::linear_gradient(180., gpui::linear_color_stop(rgb(0x262e35), 0.), gpui::linear_color_stop(rgb(0x000000), 1.)))
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(artwork)
                .child(
                  div()
                    .h(px(40.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(15.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
                    .child(title_case(&category)),
                )
                .on_click(cx.listener(move |this, _, _, cx| this.open(Page::Category(category.clone()), cx))),
            );
          }
          if categories.len() == 1 {
            grid_row = grid_row.child(div().flex_1());
          }
          body = body.child(grid_row);
        }
      }
      Page::Settings => {
        let focus = self.name_focus.as_ref().unwrap().clone();
        let name_row = div()
          .id("settings-name")
          .track_focus(&focus)
          .cursor_text()
          .child(self.settings_row("⊙", "Tenno Name", &self.name, false))
          .on_click(move |_, window, _| window.focus(&focus))
          .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
            if event.keystroke.key == "backspace" {
              this.name.pop();
            } else if let Some(text) = &event.keystroke.key_char {
              this.name.push_str(text);
            }
            this.saved_message.clear();
            cx.notify();
          }));
        let platform = platform_label(self.draft_platform);
        let platform_row = div().id("settings-platform").cursor_pointer().child(self.settings_row("♧", "Platform", platform, true)).on_click(
          cx.listener(|this, _, _, cx| {
            this.platform_menu = !this.platform_menu;
            cx.notify();
          }),
        );
        body = body.child(div().mt_2().text_size(px(12.)).font_weight(FontWeight::BOLD).text_color(rgb(0x999999)).child("PROFILE"));
        body = body.child(
          div()
            .rounded(px(14.))
            .bg(rgb(self.card()))
            .flex()
            .flex_col()
            .child(name_row)
            .child(div().ml(px(60.)).border_t_1().border_color(rgb(if self.dark { 0x303033 } else { 0xdddddf })))
            .child(platform_row),
        );
        if self.platform_menu {
          for (index, (code, label)) in [("pc", "PC"), ("xb1", "Xbox"), ("swi", "Nintendo Switch"), ("ps4", "PlayStation")].into_iter().enumerate() {
            body = body.child(
              self.button(("platform-option", index), format!("{}  {label}", if self.draft_platform == code { "✓" } else { " " })).on_click(
                cx.listener(move |this, _, _, cx| {
                  this.draft_platform = code;
                  this.platform_menu = false;
                  this.saved_message.clear();
                  cx.notify();
                }),
              ),
            );
          }
        }
        body = body.child(div().mt_3().text_size(px(12.)).font_weight(FontWeight::BOLD).text_color(rgb(0x999999)).child("PREFERENCES"));
        body = body.child(
          div()
            .rounded(px(14.))
            .bg(rgb(self.card()))
            .flex()
            .flex_col()
            .child(self.settings_row("◎", "Language", "English", false))
            .child(div().ml(px(60.)).border_t_1().border_color(rgb(if self.dark { 0x303033 } else { 0xdddddf })))
            .child(
              div()
                .id("theme")
                .cursor_pointer()
                .child(self.settings_row("◐", "Appearance", if self.dark { "Dark" } else { "Light" }, true))
                .on_click(cx.listener(|this, _, _, cx| {
                  this.dark = !this.dark;
                  this.saved_message.clear();
                  cx.notify();
                })),
            ),
        );
        body = body.child(
          div()
            .id("save-settings")
            .mt_3()
            .w_full()
            .py_4()
            .rounded(px(12.))
            .bg(rgb(0xe0c873))
            .text_color(rgb(0x000000))
            .flex()
            .justify_center()
            .font_weight(FontWeight::BOLD)
            .cursor_pointer()
            .child("Save")
            .on_click(cx.listener(|this, _, _, cx| {
              let name = this.name.trim().to_string();
              let value = serde_json::json!({"name": name, "platform": this.draft_platform, "dark": this.dark});
              let result = settings_path().ok_or_else(|| "Settings directory unavailable".to_string()).and_then(|path| {
                if let Some(parent) = path.parent() {
                  std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::write(path, value.to_string()).map_err(|e| e.to_string())
              });
              match result {
                Ok(()) => {
                  this.saved_name = name;
                  this.platform = this.draft_platform;
                  this.saved_message = "Settings saved".into();
                }
                Err(error) => this.saved_message = format!("Unable to save: {error}"),
              }
              cx.notify();
            })),
        );
        if !self.saved_message.is_empty() {
          body = body.child(div().text_size(px(13.)).text_color(rgb(0x999999)).child(self.saved_message.clone()));
        }
      }
      Page::Market => {
        body = body.child(div().w_full().flex().rounded_full().bg(rgb(self.card())).p(px(2.)).children(
          [(false, "Sell"), (true, "Buy")].into_iter().enumerate().map(|(index, (buy, title))| {
            div()
              .id(("market-type", index))
              .cursor_pointer()
              .flex_1()
              .flex()
              .justify_center()
              .py_1()
              .rounded_full()
              .bg(rgb(if self.buy == buy { if self.dark { 0x56565c } else { 0xffffff } } else { self.card() }))
              .text_size(px(14.))
              .text_color(rgb(self.foreground()))
              .child(title)
              .on_click(cx.listener(move |this, _, _, cx| {
                this.buy = buy;
                cx.notify();
              }))
          }),
        ));
      }
      _ => {}
    }
    if self.loading && !matches!(self.page, Page::Activities | Page::Codex) {
      body = body.child(div().p_4().child("Loading…"));
    }
    if let Some(error) = &self.error {
      body = body.child(div().p_4().child(format!("Unable to load: {error}")));
      body = body.child(self.button("retry", "Retry").on_click(cx.listener(|this, _, _, cx| this.open(this.page.clone(), cx))));
    }
    if let Some(data) = &self.data {
      match &self.page {
        Page::Home => {
          if let Some(banners) = data.get("banners").and_then(Value::as_array) {
            for banner in banners {
              if let Some(url) = banner.get("imageLink").and_then(Value::as_str) {
                body = body.child(
                  div()
                    .relative()
                    .w_full()
                    .h(px(110.))
                    .rounded(px(12.))
                    .overflow_hidden()
                    .child(img(image_path(url)).absolute().inset_0().w_full().h_full().rounded(px(12.)).object_fit(gpui::ObjectFit::Fill))
                    .child(div().absolute().inset_0().rounded(px(12.)).bg(gpui::rgba(0x00000080)))
                    .child(
                      div()
                        .absolute()
                        .left(px(20.))
                        .top(px(38.))
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(self.accent()))
                        .child(banner.get("title").and_then(Value::as_str).unwrap_or("").to_uppercase()),
                    ),
                );
              }
            }
          }
          if let Some(nightwave) = data.get("nightwave") {
            let season = nightwave.get("season").map(Value::to_string).unwrap_or_else(|| "–".into());
            body = body.child(
              div()
                .id("home-nightwave")
                .cursor_pointer()
                .px_1()
                .py_4()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_size(px(26.)).font_weight(FontWeight::BOLD).text_color(rgb(self.accent())).child("NIGHTWAVE"))
                .child(format!("Nora's Mix Season {season}"))
                .child(div().font_weight(FontWeight::BOLD).child(countdown(nightwave.get("expiry"))))
                .border_b_1()
                .border_color(rgb(self.card()))
                .on_click(cx.listener(|this, _, _, cx| this.open(Page::Activity("Nightwave".into(), "nightwave".into()), cx))),
            );
          }
          if let Some(events) = data.get("events").and_then(Value::as_array).filter(|events| !events.is_empty()) {
            body = body.child(self.section_header("Events", "Latest Events", Some("events"), cx));
            for event in events {
              let title = event.get("description").and_then(Value::as_str).unwrap_or("Event");
              let node = event.get("node").or_else(|| event.get("victimNode")).and_then(Value::as_str).unwrap_or("–");
              let progress = event
                .get("currentScore")
                .and_then(Value::as_f64)
                .zip(event.get("maximumScore").and_then(Value::as_f64))
                .filter(|(_, max)| *max > 0.)
                .map(|(current, max)| (100. - current / max * 100.).clamp(0., 100.))
                .or_else(|| event.get("health").and_then(Value::as_f64).map(|health| health.clamp(0., 100.)));
              body = body.child(
                div()
                  .p_4()
                  .rounded(px(12.))
                  .bg(rgb(self.card()))
                  .border_1()
                  .border_color(rgb(if self.dark { 0x303033 } else { 0xe2e2e5 }))
                  .flex()
                  .flex_col()
                  .gap_1()
                  .child(
                    div().flex().items_center().gap_2().child(div().flex_1().min_w_0().font_weight(FontWeight::BOLD).child(title.to_string())).child(
                      div()
                        .flex_shrink_0()
                        .text_size(px(12.))
                        .font_weight(FontWeight::BOLD)
                        .child(progress.map(|p| format!("{p:.2}%")).unwrap_or_default()),
                    ),
                  )
                  .child(
                    div()
                      .flex()
                      .items_center()
                      .gap_2()
                      .child(div().flex_1().min_w_0().text_color(rgb(0x999999)).child(node.to_string()))
                      .child(div().flex_shrink_0().text_size(px(14.)).font_weight(FontWeight::BOLD).child(countdown(event.get("expiry")))),
                  ),
              );
            }
          }
          if let Some(news) = data.get("news").and_then(Value::as_array).filter(|news| !news.is_empty()) {
            body = body.child(self.section_header("News", "Latest news", Some("news"), cx));
            for item in news.iter().take(2) {
              if let Some(url) = item.get("imageLink").and_then(Value::as_str) {
                body = body.child(img(image_path(url)).w_full().h(px(150.)).rounded(px(5.)).object_fit(gpui::ObjectFit::Cover));
              }
              body = body.child(div().font_weight(FontWeight::BOLD).child(label(item).to_uppercase()));
            }
          }
          if let Some(trader) = data.get("voidTrader").filter(|value| value.is_object()) {
            let location = trader.get("location").and_then(Value::as_str).unwrap_or("");
            let character = trader.get("character").and_then(Value::as_str).unwrap_or("Baro Ki'Teer").to_uppercase();
            body = body.child(
              div()
                .id("home-trader")
                .relative()
                .w_full()
                .h(px(110.))
                .rounded(px(12.))
                .overflow_hidden()
                .cursor_pointer()
                .child(
                  img(image_path("https://cdn.cagatayldzz.com/constants/wfhub/sections/baro-ki-teer.png"))
                    .absolute()
                    .inset_0()
                    .w_full()
                    .h_full()
                    .rounded(px(12.))
                    .object_fit(gpui::ObjectFit::Fill),
                )
                .child(div().absolute().inset_0().rounded(px(12.)).bg(gpui::rgba(0x00000080)))
                .child(
                  div()
                    .absolute()
                    .left(px(20.))
                    .top(px(18.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_size(px(22.)).font_weight(FontWeight::BOLD).text_color(rgb(self.accent())).child(character))
                    .child(location.to_string())
                    .child(div().font_weight(FontWeight::BOLD).child(countdown(trader.get("expiry")))),
                )
                .on_click(cx.listener(|this, _, _, cx| this.open(Page::Activity("Baro Ki'Teer".into(), "voidTrader".into()), cx))),
            );
          }
          if let Some(alerts) = data.get("alerts").and_then(Value::as_array).filter(|alerts| !alerts.is_empty()) {
            body = body.child(self.section_header("Activities", "Latest Activities", Some("alerts"), cx));
            for alert in alerts {
              let mission = alert.get("mission");
              let title = mission.and_then(|m| m.get("node")).and_then(Value::as_str).unwrap_or("Alert");
              let items = mission
                .and_then(|m| m.pointer("/reward/items"))
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>())
                .unwrap_or_default();
              let reward = if items.is_empty() {
                mission
                  .and_then(|m| m.pointer("/reward/countedItems"))
                  .and_then(Value::as_array)
                  .map(|items| {
                    items
                      .iter()
                      .map(|item| {
                        format!(
                          "{} {}",
                          item.get("count").map(Value::to_string).unwrap_or_default(),
                          item.get("type").and_then(Value::as_str).unwrap_or("")
                        )
                      })
                      .collect::<Vec<_>>()
                      .join(", ")
                  })
                  .unwrap_or_default()
              } else {
                items.join(", ")
              };
              body = body.child(
                div()
                  .p_4()
                  .rounded(px(12.))
                  .bg(rgb(self.card()))
                  .border_1()
                  .border_color(rgb(if self.dark { 0x303033 } else { 0xe2e2e5 }))
                  .flex()
                  .flex_col()
                  .gap_1()
                  .child(div().font_weight(FontWeight::BOLD).child(title.to_string()))
                  .child(
                    div()
                      .flex()
                      .justify_between()
                      .gap_2()
                      .child(div().flex_1().min_w_0().text_color(rgb(0x999999)).child(reward))
                      .child(div().flex_shrink_0().text_size(px(14.)).font_weight(FontWeight::BOLD).child(countdown(alert.get("expiry")))),
                  ),
              );
            }
          }
          body = body.child(self.section_header("Codex", "Codex content", None, cx));
          body = body.child(div().flex().gap_2().children(
            [("Warframes", "warframes", "excalibur-prime"), ("Secondary", "secondary", "secondary")].into_iter().enumerate().map(
              |(index, (title, category, artwork))| {
                div()
                  .id(("home-codex", index))
                  .flex_1()
                  .cursor_pointer()
                  .p_2()
                  .rounded(px(10.))
                  .bg(gpui::linear_gradient(180., gpui::linear_color_stop(rgb(0x262e35), 0.), gpui::linear_color_stop(rgb(0x000000), 1.)))
                  .flex()
                  .flex_col()
                  .items_center()
                  .gap_1()
                  .child(img(image_path(&format!("https://cdn.cagatayldzz.com/constants/wfhub/codex/{artwork}.png"))).w_full().h(px(100.)))
                  .child(div().h(px(40.)).text_size(px(15.)).font_weight(FontWeight::BOLD).text_color(rgb(0xffffff)).child(title))
                  .on_click(cx.listener(move |this, _, _, cx| this.open(Page::Category(category.into()), cx)))
              },
            ),
          ));
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
            for (index, order) in
              orders.iter().filter(|v| v.get("type").and_then(Value::as_str) == Some(if self.buy { "buy" } else { "sell" })).take(35).enumerate()
            {
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
              let icon = item.and_then(|v| v.pointer("/i18n/en/icon")).and_then(Value::as_str);
              let slug = item.and_then(|v| v.get("slug")).and_then(Value::as_str).unwrap_or("").to_string();
              let artwork = div()
                .size(px(76.))
                .flex_shrink_0()
                .rounded(px(10.))
                .bg(gpui::linear_gradient(180., gpui::linear_color_stop(rgb(0x262e35), 0.), gpui::linear_color_stop(rgb(0x000000), 1.)))
                .p_2()
                .when_some(icon, |element, icon| {
                  element.child(img(image_path(&format!("https://warframe.market/static/assets/{icon}"))).size_full())
                });
              let card = div()
                .id(("market-order", index))
                .w_full()
                .p_3()
                .rounded(px(12.))
                .bg(rgb(self.card()))
                .border_1()
                .border_color(rgb(if self.dark { 0x303033 } else { 0xe2e2e5 }))
                .flex()
                .items_center()
                .gap_3()
                .child(artwork)
                .child(
                  div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_size(px(16.)).font_weight(FontWeight::BOLD).child(title.to_string()))
                    .child(div().text_size(px(12.)).text_color(rgb(0x999999)).child(format!("⊙  {seller}")))
                    .child(div().text_size(px(12.)).text_color(rgb(0x999999)).child(format!("×{quantity}"))),
                )
                .child(
                  div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap_1()
                    .child(div().text_size(px(17.)).text_color(rgb(self.accent())).font_weight(FontWeight::BOLD).child(format!("◆ {price}")))
                    .child(div().text_size(px(9.)).text_color(rgb(0x999999)).font_weight(FontWeight::BOLD).child("PLATINUM")),
                )
                .when(!slug.is_empty(), |card| {
                  card.cursor_pointer().on_click(move |_, _, cx| cx.open_url(&format!("https://warframe.market/items/{slug}")))
                });
              body = body.child(card);
            }
          }
        }
        Page::Activity(_, endpoint) if endpoint == "alerts" => {
          if let Some(items) = data.as_array() {
            if items.is_empty() {
              body = body.child(div().p_4().child("No data available"));
            }
            for item in items {
              let title = item.pointer("/mission/node").and_then(Value::as_str).unwrap_or("Alert");
              let mission = item.pointer("/mission/type").or_else(|| item.pointer("/mission/missionType")).and_then(Value::as_str).unwrap_or("");
              let timer = timer_text(item.get("expiry"));
              let mut card = div()
                .w_full()
                .p_4()
                .rounded(px(12.))
                .bg(rgb(self.card()))
                .border_1()
                .border_color(rgb(if self.dark { 0x303033 } else { 0xe2e2e5 }))
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_size(px(17.)).font_weight(FontWeight::BOLD).child(title.to_string()));
              if !timer.is_empty() {
                card = card.child(div().text_size(px(12.)).font_weight(FontWeight::BOLD).text_color(rgb(0x999999)).child(format!("◷  {timer}")));
              }
              if !mission.is_empty() {
                card = card.child(div().text_size(px(14.)).text_color(rgb(0x999999)).child(mission.to_string()));
              }
              if let Some(items) = item.pointer("/mission/reward/items").and_then(Value::as_array) {
                let rewards = items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ");
                if !rewards.is_empty() {
                  card = card.child(div().text_size(px(14.)).child(rewards));
                }
              }
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

  fn settings_row(&self, icon: &str, title: &str, value: &str, selectable: bool) -> impl IntoElement {
    div()
      .w_full()
      .p_4()
      .flex()
      .items_center()
      .gap_3()
      .child(
        div()
          .size(px(34.))
          .rounded(px(9.))
          .bg(gpui::rgba(0xe0c8731a))
          .flex()
          .items_center()
          .justify_center()
          .text_size(px(18.))
          .text_color(rgb(self.accent()))
          .child(icon.to_string()),
      )
      .child(
        div()
          .flex_1()
          .min_w_0()
          .flex()
          .flex_col()
          .gap_1()
          .child(div().text_size(px(14.)).text_color(rgb(0x999999)).child(title.to_string()))
          .child(div().text_size(px(16.)).font_weight(FontWeight::BOLD).child(value.to_string())),
      )
      .when(selectable, |row| row.child(div().text_color(rgb(0x999999)).child("⌃⌄")))
  }

  fn section_header(&self, title: &str, subtitle: &str, endpoint: Option<&str>, cx: &mut Context<Self>) -> impl IntoElement {
    let page = endpoint.map_or(Page::Codex, |endpoint| Page::Activity(title.to_string(), endpoint.to_string()));
    div()
      .pt_4()
      .flex()
      .items_end()
      .justify_between()
      .child(
        div()
          .flex()
          .flex_col()
          .gap_1()
          .child(div().text_size(px(22.)).font_weight(FontWeight::BOLD).child(title.to_string()))
          .child(div().text_size(px(14.)).text_color(rgb(0x999999)).child(subtitle.to_string())),
      )
      .child(
        div()
          .id(SharedString::from(format!("view-all-{title}")))
          .cursor_pointer()
          .text_color(rgb(self.accent()))
          .child("View all")
          .on_click(cx.listener(move |this, _, _, cx| this.open(page.clone(), cx))),
      )
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
