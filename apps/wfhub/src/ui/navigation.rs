use super::*;

impl Render for Root {
  fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let tabs = [
      ("nav/home.svg", "Home"),
      ("nav/activities.svg", "Activities"),
      ("nav/market.svg", "Market"),
      ("nav/codex.svg", "Codex"),
      ("nav/settings.svg", "Settings"),
    ];
    let selected = self.tab;
    div()
      .size_full()
      .relative()
      .flex()
      .flex_col()
      .bg(rgb(self.background()))
      .text_color(rgb(self.foreground()))
      .child(div().id("content").size_full().overflow_y_scroll().track_scroll(&self.scroll).px_4().pt_4().pb(px(90.)).child(self.body(cx)))
      .child(
        div()
          .absolute()
          .occlude()
          .bottom(px(12.))
          .left(px(16.))
          .right(px(16.))
          .flex()
          .justify_between()
          .rounded_full()
          .border_1()
          .border_color(rgb(if self.dark { 0x49494c } else { 0xd0d0d3 }))
          .bg(if self.dark { gpui::rgba(0x303033ed) } else { gpui::rgba(0xe9e9eced) })
          .h(px(58.))
          .p(px(3.))
          .children(tabs.into_iter().enumerate().map(|(index, (icon, title))| {
            div()
              .id(("tab", index))
              .cursor_pointer()
              .flex_1()
              .flex()
              .flex_col()
              .items_center()
              .justify_center()
              .rounded_full()
              .bg(if selected == index { gpui::rgba(0x09090bd9) } else { gpui::rgba(0x00000000) })
              .text_color(rgb(if selected == index { self.accent() } else { self.foreground() }))
              .child(gpui::svg().path(icon).size(px(24.)).text_color(rgb(if selected == index { self.accent() } else { self.foreground() })))
              .child(div().text_size(px(10.)).child(title))
              .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.select_tab(index, cx);
              }))
          })),
      )
  }
}
