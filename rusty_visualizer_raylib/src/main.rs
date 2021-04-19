use std::f64::consts::TAU;

use raylib::prelude::*;

use rusty_visualizer_core::audio::{Audio, AudioMode};
use rusty_visualizer_core::fft::FFTSize;
use rusty_visualizer_core::iterator::StepByFloat;
use rusty_visualizer_core::settings::Settings;

#[allow(unused_macros)]
macro_rules! cstr {
    ($($arg:tt)*) => {std::ffi::CStr::from_bytes_with_nul(format!("{}\0", format!($($arg)*)).as_bytes()).ok()};
}

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

fn main() {
  let (mut rl, thread) = raylib::init()
      .size(1920, 1080)
      .title("Rusty Visualizer")
      .msaa_4x()
      .resizable()
      .build();

  let mut audio = Audio::from(Settings::load_default());

  audio.change_mode(AudioMode::Wave);

  while !rl.window_should_close() {
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::new(32, 32, 32, 32));

    draw(&mut d, &audio);
  }
}