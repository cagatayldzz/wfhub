use gpui::{App, Application, Bounds, Pixels, Size, WindowBounds, WindowOptions, prelude::*, px, size};
use wfhub_ui::Root;

fn initial_window_size(cx: &App, phone: bool) -> Size<Pixels> {
  let (width, height) =
    cx.primary_display().map(|display| display.bounds().size).map_or((1440.0, 900.0), |screen| (f32::from(screen.width), f32::from(screen.height)));
  if phone {
    let height = (height * 0.8).min(844.0);
    size(px(height * 390.0 / 844.0), px(height))
  } else {
    size(px((width * 0.8).min(900.0)), px((height * 0.8).min(700.0)))
  }
}

fn main() {
  let phone = std::env::args().any(|arg| arg == "--phone");
  Application::new().run(move |cx| {
    let bounds = Bounds::centered(None, initial_window_size(cx, phone), cx);
    let result = cx.open_window(
      WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), window_min_size: Some(size(px(240.), px(320.))), ..Default::default() },
      |_, cx| cx.new(|_| Root::new()),
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
