pub trait Application {
  fn init() -> Self;

  fn setup(&mut self) {}

  fn show_ui(&self) -> bool {
    true
  }

  #[rustfmt::skip]
  fn ui_cfg(&self, _ctx: &egui::CtxRef) {}

  fn ui(&mut self, _ctx: &egui::CtxRef) {}

  fn before_draw(&mut self) {}

  fn draw(&self) {}

  fn after_draw(&mut self) {}
}

pub async fn run_application<App: Application>() {
  let mut app = App::init();

  app.setup();
  egui_macroquad::cfg(|ctx| app.ui_cfg(ctx));

  loop {
    app.before_draw();
    app.draw();
    app.after_draw();

    if app.show_ui() {
      egui_macroquad::ui(|ctx| app.ui(ctx));
      egui_macroquad::draw();
    }

    macroquad::window::next_frame().await;
  }
}
