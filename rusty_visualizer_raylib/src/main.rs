#[macro_use]
extern crate rusty_visualizer_core;

use std::f64::consts::TAU;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use rusty_visualizer_core::audio::{Audio, AudioMode};
use rusty_visualizer_core::iterator::StepByFloat;
use rusty_visualizer_core::settings::Settings;
use rusty_visualizer_core::util::AnyErrorResult;

fn draw(d: &mut RaylibDrawHandle, audio: &Audio) {
  let w_center = d.get_screen_width() as f32 / 2f32;
  let h_center = d.get_screen_height() as f32 / 2f32;
  let radius = 400f32;

  if let Some(audio) = audio.data() {
    let len = audio.len();
    let lenf32 = len as f32;
    let gap = d.get_screen_width() as f32 / lenf32;

    let mut last = Vector2::new(0f32, h_center);

    for i in 0..len {
      let value = audio[i] * 500f32 * 2f32;
      let if32 = i as f32;
      let color = Color::color_from_hsv((360f32 / lenf32) * if32, 1.0, 1.0);

      let tmp_last = Vector2::new(gap * if32, h_center - value / 2f32);

      d.draw_line_ex(
        last,
        tmp_last,
        2f32, color,
      );

      last = tmp_last;

      let value = value / 2.5f32;
      let y = if value < 0.0 { h_center + value / 2f32 } else { h_center - value / 2f32 };

      d.draw_rectangle_rec(Rectangle::new(gap * if32, y, gap, value.abs()), color);
    }

    for (i, theta) in (0.0..TAU).step_by::<f32>(TAU / lenf32 as f64).enumerate() {
      let if32 = i as f32;
      let value = audio[i] as f32 * 200f32;
      let color = Color::color_from_hsv((360f32 / lenf32) * if32, 1.0, 1.0);

      let x_inner = w_center + radius * theta.sin();
      let y_inner = h_center - radius * theta.cos();

      let radius = radius + value;
      let x_outer = w_center + radius * theta.sin();
      let y_outer = h_center - radius * theta.cos();

      // d.draw_rectangle_rec(Rectangle::new(x_inner, y_inner, 2.0, 2.0), color);
      // d.draw_rectangle_rec(Rectangle::new(x_outer, y_outer, 2.0, 2.0), color);

      d.draw_line_ex(
        Vector2::new(x_inner, y_inner),
        Vector2::new(x_outer, y_outer),
        1.0, color,
      );
    }
  }
}

trait ApplyOptions {
  fn apply(&mut self, options: &Options) -> &mut Self;
}

impl ApplyOptions for RaylibBuilder {
  fn apply(&mut self, options: &Options) -> &mut Self {
    if options.raylib.msaa { self.msaa_4x(); }
    if options.raylib.undecorated { self.undecorated(); }
    if options.raylib.transparent { self.transparent(); }
    if options.raylib.resizeable { self.resizable(); }
    if options.raylib.start_fullscreen { self.fullscreen(); }
    self
  }
}

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
struct RaylibOptions {
  msaa: bool,
  resizeable: bool,
  transparent: bool,
  undecorated: bool,
  start_fullscreen: bool,
}

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
struct Options {
  raylib: RaylibOptions
}

fn main() -> AnyErrorResult<()> {
  let settings = Settings::<Options>::load_default()?;

  settings.save_default()?;

  let (mut rl, thread) = raylib::init()
      .size(1920, 1080)
      .title("Rusty Visualizer")
      .apply(&settings.options.unwrap_or_default())
      .build();

  let mut audio = Audio::from(&settings);

  audio.change_mode(AudioMode::Wave);

  while !rl.window_should_close() {
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::new(32, 32, 32, 255));
    // d.clear_background(Color::new(0, 0, 0, 0));

    draw(&mut d, &audio);
  }

  Ok(())
}