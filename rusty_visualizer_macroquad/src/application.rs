use egui::{Id, LayerId, Order};

pub trait Application {
  fn init() -> Self;

  fn setup(&mut self);

  fn show_ui(&self) -> bool {
    true
  }

  fn ui_cfg(&self, _ctx: &egui::CtxRef) {}

  fn ui(&mut self, _ctx: &egui::CtxRef) {}

  fn before_draw(&mut self) {}

  fn draw(&self, ctx: &egui::CtxRef) {}

  fn after_draw(&mut self) {}
}

pub async fn run_application<App: Application>() {
  let mut app = App::init();

  app.setup();
  egui_macroquad::cfg(|ctx| app.ui_cfg(ctx));

  loop {
    egui_macroquad::ui(|ctx| {
      app.before_draw();
      app.draw(ctx);
      app.after_draw();

      if app.show_ui() {
        app.ui(ctx);
      }
    });

    egui_macroquad::draw();

    // if app.show_ui() {
    //   egui_macroquad::ui(|ctx| app.ui(ctx));
    //   egui_macroquad::draw();
    // }

    macroquad::window::next_frame().await;
  }
}
