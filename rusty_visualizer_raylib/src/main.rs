extern crate rusty_visualizer_core;

use std::f64::consts::TAU;

use raylib::prelude::*;
use rayui::theme::{RaylibHandleApplyTheme, Theme};
use serde::{Deserialize, Serialize};

use application::RaylibOptions;
use rusty_visualizer_core::audio::{Audio, AudioMode};
use rusty_visualizer_core::fft::FFTSize;
use rusty_visualizer_core::settings::{AudioSettings, SettingsManager};
use rusty_visualizer_core::util::AnyErrorResult;

use crate::application::{Application, run_application};

mod application;

#[derive(Default, Clone, Serialize, Deserialize)]
struct Settings {
  audio: AudioSettings,
  options: RaylibOptions
}

impl SettingsManager for Settings {
  const DEFAULT_PATH: &'static str = "./settings_raylib.json";
}

struct State {
  settings: Settings,
  audio: Audio,
  theme: Theme,
  scale: f32,
}

impl Application for State {
  fn init() -> Self {
    let settings = Settings::load();
    let audio = Audio::from(&settings.audio);
    let theme = Theme::default();
    let scale = 1.0f32;

    Self {
      settings,
      audio,
      theme,
      scale,
    }
  }

  fn setup(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread) {
    self.audio.change_mode(AudioMode::FFT(FFTSize::FFT16384));
    _rl.apply(&self.theme);
  }

  fn draw(&self, d: &mut RaylibDrawHandle) {
    let audio = &self.audio;
    let w_center = d.get_screen_width() as f32 / 2f32;
    let h_center = d.get_screen_height() as f32 / 2f32;
    let radius = 400f32;

    d.clear_background(rcolor(32, 32, 32, 255));

    if let Some(audio) = audio.data() {
      let len = audio.len();
      let lenf32 = len as f32;
      let gap = d.get_screen_width() as f32 / lenf32;

      let mut last = Vector2::new(0f32, h_center);

      for i in 0..len {
        let value = audio[i] * 500f32 * self.scale;
        let if32 = i as f32;
        let color = Color::color_from_hsv((360f32 / lenf32) * if32, 1.0, 1.0);

        let tmp_last = Vector2::new(gap * if32, h_center - value / 2f32);

        d.draw_line_ex(last, tmp_last, 2f32, color);

        last = tmp_last;

        let value = value / 2.5f32;
        let y = if value < 0.0 {
          h_center + value / 2f32
        } else {
          h_center - value / 2f32
        };

        d.draw_rectangle_rec(Rectangle::new(gap * if32, y, gap, value.abs()), color);

        let theta = i as f64 * (TAU / len as f64);
        let x = theta.sin() as f32;
        let y = theta.cos() as f32;

        let x_inner = w_center + radius * x;
        let y_inner = h_center - radius * y;

        let radius = radius + value;
        let x_outer = w_center + radius * x;
        let y_outer = h_center - radius * y;

        d.draw_line_ex(Vector2::new(x_inner, y_inner), Vector2::new(x_outer, y_outer), 1.0, color);
      }
    }
  }

  fn gui<G: RaylibDrawGui>(&mut self, d: &mut G) {
    self.scale = d.gui_slider(
      rrect(5, 5, 200, 30),
      None, rayui::rayui_str!("Scale"), self.scale,
      0.01, 2.0
    );
  }

  fn raylib_options(&self) -> RaylibOptions {
    self.settings.options.clone()
  }
}

fn main() -> AnyErrorResult<()> {
  run_application::<State>()
}
