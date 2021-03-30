use nannou::prelude::*;
use nannou::ui::prelude::*;

use crate::application::{Application, build_application_from};
use crate::settings::Settings;
use crate::audio::Audio;

mod application;
mod settings;
mod audio;
mod util;
mod fft;

struct State {
  audio: Audio,
  ui: Ui,
  ids: Ids,
  hue: f32,
  saturation: f32,
  lightness: f32,
}

widget_ids! {
  struct Ids {
    hue,
    saturation,
    lightness,
  }
}

impl Application for State {
  fn init(app: &App) -> Self {
    let settings = Settings::load_default();
    let audio = Audio::from(settings);
    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids::new(ui.widget_id_generator());

    State {
      audio,
      ui,
      ids,
      hue: 0.0,
      saturation: 1.0,
      lightness: 0.5,
    }
  }

  fn on_update(_app: &App, state: &mut Self, _update: Update) {
    let ui = &mut state.ui.set_widgets();

    let slider = |val: f32, min: f32, max: f32| {
      widget::Slider::new(val, min, max)
        .w_h(400.0, 20.0)
        .label_font_size(15)
        .rgb(0.3, 0.3, 0.3)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
    };

    for value in slider(state.hue, 0.0, 1.0)
      .top_left_with_margin(5.0)
      .label("Hue")
      .set(state.ids.hue, ui)
    {
      state.hue = value
    }

    for value in slider(state.saturation, 0.0, 1.0)
      .down(5.0)
      .label("Saturation")
      .set(state.ids.saturation, ui)
    {
      state.saturation = value
    }

    for value in slider(state.lightness, 0.0, 1.0)
      .down(5.0)
      .label("Lightness")
      .set(state.ids.lightness, ui)
    {
      state.lightness = value
    }
  }

  fn view(app: &App, state: &Self, frame: Frame) {
    let draw = app.draw();

    draw.background().hsl(0.0, 0.0, 0.0125);

    if let Some(receiver) = &state.audio.receiver {
      let audio = receiver.lock().unwrap();
      let len = audio.data.len();
      let Vector2 { x: width, y: height } = frame.rect().wh();
      let offset = width / 2f32;
      let gap = width / len as f32;

      for i in 0..len {

        draw.rect()
          .x_y(offset + (i as f32 * gap) , -(height / 2f32))
          .w_h(gap, audio.data[i])
          .hsl(state.hue, state.saturation, state.lightness);
      }
    }

    draw.to_frame(app, &frame).unwrap();
    state.ui.draw_to_frame(app, &frame).unwrap();
  }
}

fn main() {
  build_application_from::<State>()
    .size(1920, 1080)
    .run();
}
