#![allow(dead_code, unused_variables)]

use std::f32::consts::TAU;

use egui::CtxRef;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use rusty_visualizer_core::audio::{Audio, AudioDevice, AudioMode};
use rusty_visualizer_core::settings::{AudioSettings, SettingsManager};

use crate::application::{Application, run_application};
use crate::color::{AsColor, GrayColor};

mod application;
mod color;

fn window_conf() -> Conf {
  Conf {
    window_title: "Rusty Visualizer".to_owned(),
    window_width: 1920,
    window_height: 1080,
    high_dpi: true,
    window_resizable: true,
    ..Default::default()
  }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Settings {
  audio: AudioSettings,
}

impl SettingsManager for Settings {
  const DEFAULT_PATH: &'static str = "./settings_macroquad.json";
}

struct App {
  settings: Settings,
  audio: Audio,
  state: State,
}

struct State {
  size: f32,
  radius: f32,
  line_gap: f32,
  offset_x: f32,
  offset_y: f32,
  color: [f32; 3],
  mode_index: usize,
  show_ui: bool,
}

impl Default for State {
  fn default() -> Self {
    Self {
      size: 1f32,
      radius: 150f32,
      line_gap: 1f32,
      offset_x: 0f32,
      offset_y: 0f32,
      color: [0.8f32; 3],
      mode_index: 0,
      show_ui: true,
    }
  }
}

impl Application for App {
  fn init() -> Self {
    let settings = Settings::load();
    let audio = Audio::from(&settings.audio);
    let state = State::default();

    Self { settings, audio, state }
  }

  fn setup(&mut self) {
    self.audio.change_device(AudioDevice::LOOPBACK);
  }

  fn show_ui(&self) -> bool {
    self.state.show_ui
  }

  fn ui_cfg(&self, ctx: &CtxRef) {
    let mut style = egui::Style::default();

    let accent = egui::Color32::from_rgb(180, 50, 160);
    style.visuals.hyperlink_color = accent;
    style.visuals.selection.bg_fill = accent;
    style.spacing.slider_width = 200f32;

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
      "Roboto-Regular".to_string(),
      std::borrow::Cow::Borrowed(include_bytes!("../../assets/Roboto-Regular.ttf")),
    );

    fonts
      .fonts_for_family
      .get_mut(&egui::FontFamily::Proportional)
      .unwrap()
      .insert(0, "Roboto-Regular".to_owned());

    let size = 20f32;

    fonts
      .family_and_size
      .insert(egui::TextStyle::Small, (egui::FontFamily::Proportional, size));
    fonts
      .family_and_size
      .insert(egui::TextStyle::Body, (egui::FontFamily::Proportional, size));
    fonts
      .family_and_size
      .insert(egui::TextStyle::Button, (egui::FontFamily::Proportional, size));
    fonts
      .family_and_size
      .insert(egui::TextStyle::Heading, (egui::FontFamily::Proportional, size * 1.2));
    fonts
      .family_and_size
      .insert(egui::TextStyle::Monospace, (egui::FontFamily::Proportional, size));

    ctx.set_style(style);
    ctx.set_fonts(fonts);
  }

  fn ui(&mut self, ctx: &CtxRef) {
    egui::SidePanel::left("Settings")
      .resizable(false)
      .default_width(200f32)
      .show(ctx, |ui| {
        let limit = screen_width().min(screen_height()) / 2f32;

        ui.add(egui::Slider::new(&mut self.state.size, 0.001f32..=5f32).text("Size"));
        ui.add(egui::Slider::new(&mut self.state.line_gap, 0.1f32..=5f32).text("Line Gap"));
        ui.add(egui::Slider::new(&mut self.state.radius, -limit..=limit).text("Radius"));
        ui.add(egui::Slider::new(&mut self.state.offset_x, 0f32..=limit * 2f32).text("Offset X"));
        ui.add(egui::Slider::new(&mut self.state.offset_y, 0f32..=limit * 2f32).text("Offset Y"));

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
      let center_w = self.state.offset_x + screen_width() / 2f32;
      let center_h = self.state.offset_y + screen_height() / 2f32;

      for i in 0..len {
        let value = audio[i] * 300f32 * self.state.size;
        let mut color = self.state.color.as_color();
        let gap = screen_width() / len as f32;

        color.r = clamp(color.r * audio[i] * 5f32, 0.2, 1.0);
        color.g = clamp(color.g * audio[i] * 5f32, 0.2, 1.0);
        color.b = clamp(color.b * audio[i] * 5f32, 0.2, 1.0);

        let theta = (TAU / len as f32) * i as f32;
        let radius = self.state.radius * audio.sum / 200f32;

        let x_inner = center_w + (radius - value) * theta.sin();
        let y_inner = center_h - (radius - value) * theta.cos();
        let x_outer = center_w + (radius + value) * theta.sin();
        let y_outer = center_h - (radius + value) * theta.cos();

        // draw_rectangle(x_inner, y_inner, 2.0, 2.0, color);

        draw_line(x_inner, y_inner, x_outer, y_outer, self.state.line_gap, color)

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
