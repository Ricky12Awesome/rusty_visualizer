use eframe::egui::{CentralPanel, CtxRef, Painter, Pos2, Rect, Rgba, SidePanel, Slider, Vec2};
use eframe::epi::{App, Frame};
use rusty_visualizer_core::audio::{Audio, AudioDevices, AudioMode};
use rusty_visualizer_core::fft::FFTSize;
use rusty_visualizer_core::settings::Settings;

fn main() {
  eframe::run_native(Box::new(State::default()), Default::default());
}

struct State {
  settings: Settings<()>,
  audio: Audio,
  value: f32,
}

impl Default for State {
  fn default() -> Self {
    let settings = Settings::load_default("settings_egui.json");
    let mut audio = Audio::from(&settings);
    let value = 1.0f32;

    audio.change_device(AudioDevices::LOOPBACK);
    audio.change_mode(AudioMode::FFT(FFTSize::FFT16384));

    Self { settings, audio, value }
  }
}

impl App for State {
  fn name(&self) -> &str {
    "Rusty Visualizer [egui]"
  }

  fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
    SidePanel::right("test").show(ctx, |ui| {
      ui.heading("Testing");
      ui.add(Slider::new(&mut self.value, 0.001f32..=10.0f32).text("Value"));
    });

    CentralPanel::default().show(ctx, |ui| {
      let painter = ui.painter_at(ui.available_rect_before_wrap());

      if let Some(audio) = self.audio.data() {
        let len = audio.len();

        for i in 0..len {
          let value = audio[i] * 250f32 * self.value;
          let if32 = i as f32;
          let clip = painter.clip_rect();
          let gap = clip.width() / len as f32;

          painter.rect_filled(
            Rect::from_min_size(
              Pos2::new(gap * if32, (clip.height() / 2.0f32) - (value / 2f32)),
              Vec2::new(gap, value.abs()),
            ),
            1.0,
            Rgba::from_gray(0.75),
          );
        }
      }
    });
  }
}
