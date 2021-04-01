use nannou::prelude::*;
use nannou::ui::prelude::*;

use crate::application::{Application, build_application_from};
use crate::settings::Settings;
use crate::audio::{Audio, AudioMode};
use std::ops::Deref;
use crate::fft::FFTSize;

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
  hue_speed: f32,
  gap: f32,
}

widget_ids! {
  struct Ids {
    hue,
    saturation,
    lightness,
    hue_speed,
    select_audio_mode,
  }
}

impl Application for State {
  fn init(app: &App) -> Self {
    let settings = Settings::load_default();
    let mut audio = Audio::from(settings);
    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids::new(ui.widget_id_generator());

    audio.change_mode(AudioMode::FFT(FFTSize::FFT512));

    State {
      audio,
      points: Vec::new(),
      size: app.main_window().deref().rect().wh(),
      ui,
      ids,
      hue: 0.0,
      saturation: 1.0,
      lightness: 0.5,
      hue_speed: 0.05,
      gap: 1.0,
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
      .top_right_with_margin(5.0)
      .label("Hue")
      .set(state.ids.hue, ui)
    {
      state.hue = value
    }

    for value in slider(state.hue_speed, 0.05, 5.0)
      .down(5.0)
      .label("Hue Speed (Epilepsy warning)")
      .set(state.ids.hue_speed, ui)
    {
      state.hue_speed = value
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

    if let Some(index) = widget::DropDownList::new(&AudioMode::all_named(), None)
      .down(5.0)
      .label("Select Mode")
      .w_h(400.0, 20.0)
      .label_font_size(15)
      .rgb(0.3, 0.3, 0.3)
      .label_rgb(1.0, 1.0, 1.0)
      .border(0.0)
      .set(state.ids.select_audio_mode, ui)
    {
      state.audio.change_mode(AudioMode::ALL[index]);
    }

    if let Some(receiver) = &state.audio.receiver {
      let audio = receiver.lock().unwrap();
      let len = audio.data.len();
      let offset_width = -(state.size.x / 2f32);
      let gap = state.size.x / len as f32;

      state.hue += (audio.sum * state.hue_speed) / 10000f32;

      if state.hue >= 1.0 {
        state.hue = 0.0;
      }

      let color = hsl(state.hue, state.saturation, state.lightness);

      let points = (0..len).map(|i| {
        let color = &color;
        let if32 = i as f32;
        let point = pt2(offset_width + (if32 * gap), audio[i] * 500f32);

        (point, hsl(color.hue.to_degrees() / 360f32 + if32 / len as f32, color.saturation, color.lightness))
      });

      state.points = points.collect();
      state.gap = gap;
    }
  }

  fn on_resize(_app: &App, state: &mut Self, new_size: Vector2<f32>) {
    state.size = new_size;
  }

  fn view(app: &App, state: &Self, frame: Frame) {
    let draw = app.draw();

    draw.background().hsl(0.0, 0.0, 0.0125);

    draw.polyline().weight(2.0).points_colored(state.points.clone()).finish();

    for (Point2 { x, y }, color) in state.points.clone() {
      draw.rect()
        .x_y(x, 0.0)
        .w_h(state.gap, y / 1.5)
        .color(color)
        .finish()
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
