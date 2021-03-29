use nannou::prelude::*;
use nannou::ui::prelude::*;
use crate::application::{Application, build_application_from};

mod application;
mod settings;
mod audio;
mod util;

struct State {
  ui: Ui,
  ids: Ids,
  hue: f32,
}

widget_ids! {
  struct Ids {
    hue,
  }
}

impl Application for State {
  fn init(app: &App) -> Self {
    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids::new(ui.widget_id_generator());

    State {
      ui,
      ids,
      hue: 0.0,
    }
  }

  fn on_update(_app: &App, state: &mut Self, _update: Update) {
    let ui = &mut state.ui.set_widgets();

    for value in widget::Slider::new(0.0, 0.0, 1.0)
      .top_left_with_margin(5.0)
      .w_h(600.0, 20.0)
      .label_font_size(15)
      .hsl(state.hue, 0.5, 0.5)
      .label_rgb(1.0, 1.0, 1.0)
      .border(0.0)
      .label("Hue")
      .set(state.ids.hue, ui)
    {
      state.hue = value
    }
  }

  fn view(app: &App, state: &Self, frame: Frame) {
    let draw = app.draw();

    draw.background().hsl(0.0, 0.0, 0.0125);

    draw.rect()
      .x_y(0.0, 0.0)
      .w_h(300.0, 300.0)
      .hsl(state.hue, 0.75, 0.5);

    draw.to_frame(app, &frame).unwrap();
    state.ui.draw_to_frame(app, &frame).unwrap();
  }
}

fn main() {
  build_application_from::<State>()
    .size(1920, 1080)
    .run();
}
