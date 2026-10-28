use gpui::{App, Application, Bounds, Pixels, Size, WindowBounds, WindowOptions, prelude::*, px, size};
mod ui;

use ui::Root;

struct Assets;

impl gpui::AssetSource for Assets {
  fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
    let bytes: Option<&'static [u8]> = match path {
      "nav/home.svg" => Some(include_bytes!("../assets/home.svg")),
      "nav/activities.svg" => Some(include_bytes!("../assets/activities.svg")),
      "nav/market.svg" => Some(include_bytes!("../assets/market.svg")),
      "nav/codex.svg" => Some(include_bytes!("../assets/codex.svg")),
      "nav/settings.svg" => Some(include_bytes!("../assets/settings.svg")),
      _ => None,
    };
    Ok(bytes.map(std::borrow::Cow::Borrowed))
  }
  fn list(&self, _: &str) -> anyhow::Result<Vec<gpui::SharedString>> {
    Ok(Vec::new())
  }
}

fn initial_window_size(cx: &App) -> Size<Pixels> {
  let height = cx.primary_display().map(|display| f32::from(display.bounds().size.height)).unwrap_or(900.0);
  let height = (height * 0.8).min(844.0);
  size(px(height * 390.0 / 844.0), px(height))
}

fn main() {
  Application::new().with_assets(Assets).run(move |cx| {
    let bounds = Bounds::centered(None, initial_window_size(cx), cx);
    let result = cx.open_window(
      WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), window_min_size: Some(size(px(240.), px(320.))), ..Default::default() },
      |_, cx| cx.new(Root::new),
    );
    if let Err(error) = result {
      eprintln!("Could not open window: {error}");
      cx.quit();
      return;
    }
    cx.on_window_closed(|cx| cx.quit()).detach();
    cx.activate(true);
  });
}
