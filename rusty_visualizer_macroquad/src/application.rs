pub trait Application {
  fn init() -> Self;

  fn setup(&mut self) {}

  fn show_ui(&self) -> bool {
    true
  }

  #[rustfmt::skip]
  fn ui_cfg(&self, ctx: &egui::CtxRef) {
    let mut style = egui::Style::default();

    let accent = egui::Color32::from_rgb(180, 50, 160);
    style.visuals.hyperlink_color = accent;
    style.visuals.selection.bg_fill = accent;
    style.spacing.slider_width = 200f32;

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
      "Roboto-Regular".to_string(),
      std::borrow::Cow::Borrowed(include_bytes!("../../assets/Roboto-Regular.ttf")),
    );

    fonts.fonts_for_family
      .get_mut(&egui::FontFamily::Proportional).unwrap()
      .insert(0, "Roboto-Regular".to_owned());

    let size = 20f32;

    fonts.family_and_size.insert(egui::TextStyle::Small, (egui::FontFamily::Proportional, size));
    fonts.family_and_size.insert(egui::TextStyle::Body, (egui::FontFamily::Proportional, size));
    fonts.family_and_size.insert(egui::TextStyle::Button, (egui::FontFamily::Proportional, size));
    fonts.family_and_size.insert(egui::TextStyle::Heading, (egui::FontFamily::Proportional, size * 1.2));
    fonts.family_and_size.insert(egui::TextStyle::Monospace, (egui::FontFamily::Proportional, size));

    ctx.set_style(style);
    ctx.set_fonts(fonts);
  }

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
