use std::ops::Deref;

use nannou::prelude::*;
use nannou::ui::prelude::*;

use crate::application::{Application, ApplicationDelegate, build_application_from};
use crate::audio::{Audio, AudioMode};
use crate::fft::FFTSize;
use crate::settings::Settings;

mod application;
mod visualizer;
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

impl ApplicationDelegate for State {
  fn on_update(&mut self, _app: &App, _update: Update) {
    let ui = &mut self.ui.set_widgets();

    let slider = |val: f32, min: f32, max: f32| {
      widget::Slider::new(val, min, max)
        .w_h(400.0, 20.0)
        .label_font_size(15)
        .rgb(0.3, 0.3, 0.3)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
    };

    for value in slider(self.hue, 0.0, 1.0)
      .top_right_with_margin(5.0)
      .label("Hue")
      .set(self.ids.hue, ui)
    {
      self.hue = value
    }

    for value in slider(self.hue_speed, 0.05, 5.0)
      .down(5.0)
      .label("Hue Speed (Epilepsy warning)")
      .set(self.ids.hue_speed, ui)
    {
      self.hue_speed = value
    }

    for value in slider(self.saturation, 0.0, 1.0)
      .down(5.0)
      .label("Saturation")
      .set(self.ids.saturation, ui)
    {
      self.saturation = value
    }

    for value in slider(self.lightness, 0.0, 1.0)
      .down(5.0)
      .label("Lightness")
      .set(self.ids.lightness, ui)
    {
      self.lightness = value
    }

    if let Some(index) = widget::DropDownList::new(&AudioMode::all_named(), None)
      .down(5.0)
      .label("Select Mode")
      .w_h(400.0, 20.0)
      .label_font_size(15)
      .rgb(0.3, 0.3, 0.3)
      .label_rgb(1.0, 1.0, 1.0)
      .border(0.0)
      .set(self.ids.select_audio_mode, ui)
    {
      self.audio.change_mode(AudioMode::ALL[index]);
    }

    if let Some(receiver) = &self.audio.receiver {
      let audio = receiver.lock().unwrap();
      let len = audio.data.len();
      let offset_width = -(self.size.x / 2f32);
      let gap = self.size.x / len as f32;

      self.hue += (audio.sum * self.hue_speed) / 10000f32;

      if self.hue >= 1.0 {
        self.hue = 0.0;
      }

      let color = hsl(self.hue, self.saturation, self.lightness);

      let points = (0..len).map(|i| {
        let color = &color;
        let if32 = i as f32;
        let point = pt2(offset_width + (if32 * gap), audio[i] * 500f32);

        (point, hsl(color.hue.to_degrees() / 360f32 + if32 / len as f32, color.saturation, color.lightness))
      });

      self.points = points.collect();
      self.gap = gap;
    }
  }

  fn on_resize(&mut self, _app: &App, new_size: Vector2) {
    self.size = new_size;
  }

  fn on_view(&self, app: &App, frame: Frame) {
    let draw = app.draw();

    draw.background().hsl(0.0, 0.0, 0.0125);

    draw.polyline().weight(2.0).points_colored(self.points.clone()).finish();

    for (Point2 { x, y }, color) in self.points.clone() {
      draw.rect()
        .x_y(x, 0.0)
        .w_h(self.gap, y / 1.5)
        .color(color)
        .finish()
    }

    draw.to_frame(app, &frame).unwrap();
    self.ui.draw_to_frame(app, &frame).unwrap();
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
}

fn main() {
  build_application_from::<State>()
    .size(1920, 1080)
    .run();
}
