use raylib::prelude::*;

use rusty_visualizer_core::audio::{Audio, AudioMode};
use rusty_visualizer_core::settings::Settings;

// macro_rules! cstr {
//     ($x:literal) => {std::ffi::CStr::from_bytes_with_nul(format!("{}\0", $x).as_bytes()).ok()};
// }

fn main() {
  let (mut rl, thread) = raylib::init()
    .size(1920, 1080)
    .title("Rusty Visualizer")
    .resizable()
    .build();

  let mut audio = Audio::from(Settings::load_default());

  audio.change_mode(AudioMode::Wave);

  while !rl.window_should_close() {
    let mut d = rl.begin_drawing(&thread);
    let h_center = d.get_screen_height() as f32 / 2f32;
    d.clear_background(Color::new(32, 32, 32, 32));

    if let Some(audio) = audio.get_data() {
      let len = audio.len();
      let lenf32 = len as f32;
      let gap = d.get_screen_width() as f32 / lenf32;

      let mut last = Vector2::new(0f32, 0f32);

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
        let offset = value / 2f32;

        if value < 0.0 {
          d.draw_rectangle_rec(Rectangle::new(gap * if32, h_center + offset, gap, value.abs()), color);
        } else {
          d.draw_rectangle_rec(Rectangle::new(gap * if32, h_center - offset, gap, value.abs()), color);
        }
      }
    }
  }
}