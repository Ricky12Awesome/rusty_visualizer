use nannou::prelude::*;
use nannou::ui::prelude::*;

use crate::application::{Application, build_application_from};
use crate::settings::Settings;
use crate::audio::Audio;
use std::ops::Deref;

mod application;
mod settings;
mod audio;
mod util;
mod fft;

struct State {
  audio: Audio,
  points: Vec<(Point2, Hsl)>,
  size: Vector2<f32>,
  ui: Ui,
  ids: Ids,
  hue: f32,
  saturation: f32,
  lightness: f32,
  divide: usize,
}

widget_ids! {
  struct Ids {
    hue,
    saturation,
    lightness,
    divide,
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
      points: Vec::new(),
      size: app.main_window().deref().rect().wh(),
      ui,
      ids,
      hue: 0.0,
      saturation: 1.0,
      lightness: 0.5,
      divide: 1,
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
      .bottom_left_with_margin(5.0)
      .label("Hue")
      .set(state.ids.hue, ui)
    {
      state.hue = value
    }

    for value in slider(state.saturation, 0.0, 1.0)
      .up(5.0)
      .label("Saturation")
      .set(state.ids.saturation, ui)
    {
      state.saturation = value
    }

    for value in slider(state.lightness, 0.0, 1.0)
      .up(5.0)
      .label("Lightness")
      .set(state.ids.lightness, ui)
    {
      state.lightness = value
    }

    for value in slider(state.divide as f32, 1.0, 16.0)
      .up(5.0)
      .label("Divide")
      .set(state.ids.divide, ui)
    {
      state.divide = value as usize
    }

    if let Some(receiver) = &state.audio.receiver {
      let audio = receiver.lock().unwrap();
      let len = audio.data.len() / state.divide;
      let offset_width = -(state.size.x / 2f32);
      let gap = state.size.x / len as f32;

      let color = hsl(state.hue, state.saturation, state.lightness);
      let points = (0..len).map(|i| {
        let point = pt2(offset_width + (i as f32 * gap), audio[i] * 500f32);

        (point, color)
      });

      state.points = points.collect();

      // for i in 0..len {
      //   draw.rect()
      //     .x_y(offset + (i as f32 * gap),  0.0)
      //     .w_h(gap, audio[i] * 1000f32)
      //     .hsl(state.hue, state.saturation, state.lightness);
      // }
    }
  }

  fn on_resize(app: &App, state: &mut Self, new_size: Vector2<f32>) {
    state.size = new_size;
  }

  fn view(app: &App, state: &Self, frame: Frame) {
    let draw = app.draw();

    draw.background().hsl(0.0, 0.0, 0.0125);

    draw.polyline().weight(2.0).points_colored(state.points.clone());

    draw.to_frame(app, &frame).unwrap();
    state.ui.draw_to_frame(app, &frame).unwrap();
  }
}

fn main() {
  build_application_from::<State>()
    .size(1920, 1080)
    .run();
}
