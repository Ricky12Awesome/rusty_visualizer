use piston_window::math::Vec2d;
use piston_window::*;
use rusty_visualizer_core::audio::{Audio, AudioDevice, AudioMode};
use rusty_visualizer_core::fft::FFTSize::{FFT4096, FFT16384};
use rusty_visualizer_core::settings::Settings;

fn main() {
  let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [640, 480])
    .exit_on_esc(true)
    .build()
    .unwrap();

  let settings = Settings::<()>::load_default("./settings_piston.json");
  let mut audio = Audio::from(&settings);

  audio.change_device(AudioDevice::<()>::Loopback);
  audio.change_mode(AudioMode::FFT(FFT16384));

  while let Some(event) = window.next() {
    window.draw_2d(&event, |context, graphics, _device| {
      let [width, height] = context.get_view_size();

      clear([32.0 / 256.0; 4], graphics);

      if let Some(audio) = audio.data() {
        let len = audio.len();
        let lenf64 = len as f64;
        let gap = width / lenf64;

        for i in 0..len {
          let value = audio[i] as f64 * 500f64;
          let if64 = i as f64;

          rectangle(
            [0.8, 0.0, 0.0, 1.0], // red
            [gap * if64, (height / 2.0) - (value / 2.0), gap, value.abs()],
            context.transform,
            graphics,
          );
        }
      }
    });
  }
}
