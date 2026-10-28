use gpui::{Context, FontWeight, Render, Window, div, prelude::*, px, rgb};

#[derive(Default)]
pub struct Root {
  tab: usize,
}

impl Root {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Render for Root {
  fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let tabs = ["Home", "Activities", "Codex", "Market", "Settings"];
    let selected = self.tab;
    let content = match selected {
      0 => div()
        .flex()
        .flex_col()
        .gap_4()
        .child(div().text_size(px(28.)).font_weight(FontWeight::BOLD).child("Welcome, Tenno"))
        .child(div().text_size(px(24.)).font_weight(FontWeight::BOLD).text_color(rgb(0xd3ba70)).child("NIGHTWAVE"))
        .child("News and world state are available in the iOS app."),
      1 => div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_size(px(26.)).font_weight(FontWeight::BOLD).child("Activities"))
        .children(wfhub_core::ACTIVITIES.iter().map(|(name, _)| div().py_2().child(*name))),
      2 => div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_size(px(26.)).font_weight(FontWeight::BOLD).child("Codex"))
        .children(wfhub_core::CATEGORIES.iter().map(|name| div().py_2().child(*name))),
      3 => div().child("Market is available in the iOS app."),
      _ => div().child("Settings are available in the iOS app."),
    };
    div()
      .size_full()
      .flex()
      .flex_col()
      .bg(rgb(0x000000))
      .text_color(rgb(0xffffff))
      .child(div().id("content").flex_1().overflow_y_scroll().p_4().child(content))
      .child(div().flex().justify_between().border_t_1().border_color(rgb(0x262627)).children(tabs.into_iter().enumerate().map(|(index, title)| {
        div()
          .id(("tab", index))
          .cursor_pointer()
          .p_2()
          .text_size(px(12.))
          .text_color(rgb(if selected == index { 0xe0c873 } else { 0xaaaaaa }))
          .child(title)
          .on_click(cx.listener(move |this, _, _, cx| {
            this.tab = index;
            cx.notify();
          }))
      })))
  }
}
