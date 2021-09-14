use crate::application::{run_application, Application};
use crate::color::{AsColor, GrayColor};
use egui::CtxRef;
use macroquad::prelude::*;
use rusty_visualizer_core::audio::{Audio, AudioDevice, AudioMode};
use rusty_visualizer_core::settings::Settings;
use std::f32::consts::TAU;

mod application;
mod color;

fn window_conf() -> Conf {
  Conf {
    window_title: "Rusty Visualizer".to_owned(),
    window_width: 1280,
    window_height: 720,
    high_dpi: true,
    window_resizable: true,
    ..Default::default()
  }
}

struct App {
  settings: Settings<()>,
  audio: Audio,
  state: State,
}

struct State {
  size: f32,
  radius: f32,
  line_gap: f32,
  color: [f32; 3],
  mode_index: usize,
  show_ui: bool,
}

impl Default for State {
  fn default() -> Self {
    Self {
      size: 1f32,
      radius: 150f32,
      line_gap: 1.0,
      color: [0.8f32; 3],
      mode_index: 0,
      show_ui: true,
    }
  }
}

impl Application for App {
  fn init() -> Self {
    let settings = Settings::<()>::load_default("settings_macroquad.json");
    let audio = Audio::from(&settings);
    let state = State::default();

    Self { settings, audio, state }
  }

  fn setup(&mut self) {
    self.audio.change_device(AudioDevice::LOOPBACK);
  }

  fn show_ui(&self) -> bool {
    self.state.show_ui
  }

  fn ui(&mut self, ctx: &CtxRef) {
    egui::SidePanel::left("Settings")
      .resizable(false)
      .default_width(200f32)
      .show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut self.state.size, 0.001f32..=5f32).text("Size"));
        ui.add(egui::Slider::new(&mut self.state.line_gap, 0.1f32..=5f32).text("Line Gap"));
        ui.add(
          egui::Slider::new(&mut self.state.radius, 1f32..=screen_width().min(screen_height()) / 2f32)
            .clamp_to_range(true)
            .text("Radius"),
        );

        let name = AudioMode::ALL[self.state.mode_index].name();

        if ui
          .add(
            egui::Slider::new(&mut self.state.mode_index, 0..=AudioMode::ALL.len() - 1)
              .show_value(false)
              .text(format!("{}", name)),
          )
          .changed()
        {
          self.audio.change_mode(AudioMode::ALL[self.state.mode_index]);
        }

        ui.color_edit_button_rgb(&mut self.state.color);
      });
  }

  fn before_draw(&mut self) {
    if is_key_pressed(KeyCode::H) {
      self.state.show_ui = !self.state.show_ui;
    }
  }

  fn draw(&self) {
    clear_background(Color::gray_scale(32));

    if let Some(audio) = self.audio.data() {
      let len = audio.len();
      let center_w = screen_width() / 2f32;
      let center_h = screen_height() / 2f32;

      for i in 0..len {
        let value = audio[i] * 300f32 * self.state.size;
        let mut color = self.state.color.as_color();
        let gap = screen_width() / len as f32;

        color.r = clamp(color.r * audio[i] * 5f32, 0.2, 1.0);
        color.g = clamp(color.g * audio[i] * 5f32, 0.2, 1.0);
        color.b = clamp(color.b * audio[i] * 5f32, 0.2, 1.0);

        let theta = (TAU / len as f32) * i as f32;
        let radius = self.state.radius;

        let x_inner = center_w + (radius - value) * theta.sin();
        let y_inner = center_h - (radius - value) * theta.cos();
        let x_outer = center_w + (radius + value) * theta.sin();
        let y_outer = center_h - (radius + value) * theta.cos();

        // draw_rectangle(x_inner, y_inner, 2.0, 2.0, color);

        draw_line(
          x_inner,
          y_inner, //
          x_outer,
          y_outer, //
          self.state.line_gap,
          color, //
        )

        // draw_rectangle(gap * i as f32, 0f32, gap, value.abs(), color);
        //
        // draw_rectangle(
        //   gap * i as f32,
        //   screen_height() / 2f32 - value / 2f32,
        //   gap,
        //   value.abs() * 2f32,
        //   color,
        // );
        //
        // draw_rectangle(gap * i as f32, screen_height() - value, gap, value.abs(), color);
      }
    }
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  run_application::<App>().await;
}
