#![allow(dead_code, unused_variables)]

use std::f32::consts::TAU;

use egui::{Align, CtxRef};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use rusty_visualizer_core::audio::{Audio, AudioMode, AudioDevice, ToSerializableAudioDevice};
use rusty_visualizer_core::cpal::traits::{DeviceTrait, HostTrait};
use rusty_visualizer_core::settings::{AudioManager, AudioSettings, SettingsManager};

use crate::application::{Application, run_application};
use crate::color::AsColor;

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

impl AudioManager for App {
  fn audio_settings(&mut self) -> &mut AudioSettings {
    &mut self.settings.audio
  }

  fn audio(&mut self) -> &mut Audio {
    &mut self.audio
  }

  fn change_device(&mut self, new_device: impl ToSerializableAudioDevice) {
    let new_device = new_device.to_serializable(self.audio());

    match &new_device {
      AudioDevice::None => self.state.audio.device_type = AudioDeviceType::None,
      AudioDevice::Default => self.state.audio.device_type = AudioDeviceType::Default,
      AudioDevice::Loopback => self.state.audio.device_type = AudioDeviceType::Loopback,
      AudioDevice::Input(name) => {
        self.state.audio.device_type = AudioDeviceType::Input;
        self.state.audio.input_device = Some(name.clone());
      }
      AudioDevice::Output(name) => {
        self.state.audio.device_type = AudioDeviceType::Output;
        self.state.audio.output_device = Some(name.clone());
      }
    }

    self.audio_settings().device = new_device.clone();
    self.audio().change_device(new_device);

  }
}

struct App {
  settings: Settings,
  audio: Audio,
  state: State,
}

#[derive(PartialEq, Debug)]
enum AudioDeviceType {
  None,
  Default,
  Loopback,
  Input,
  Output,
}

struct AudioState {
  mode_index: usize,
  device_type: AudioDeviceType,
  input_device: Option<String>,
  output_device: Option<String>,
}

impl Default for AudioState {
  fn default() -> Self {
    Self {
      mode_index: 0,
      device_type: AudioDeviceType::Default,
      input_device: None,
      output_device: None,
    }
  }
}

struct VisualizerState {
  size: f32,
  radius: f32,
  line_gap: f32,
  offset_x: f32,
  offset_y: f32,
}

impl Default for VisualizerState {
  fn default() -> Self {
    Self {
      size: 0.5f32,
      radius: 50f32,
      line_gap: 1f32,
      offset_x: 0f32,
      offset_y: 0f32,
    }
  }
}

struct State {
  visualizer: VisualizerState,
  audio: AudioState,
  fg_color: [f32; 3],
  bg_color: [f32; 3],
  show_ui: bool,
}

impl Default for State {
  fn default() -> Self {
    Self {
      visualizer: Default::default(),
      audio: Default::default(),
      fg_color: [0.8f32; 3],
      bg_color: [0.125f32; 3],
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
    self.state.audio.mode_index = AudioMode::ALL
      .iter()
      .position(|it| *it == self.settings.audio.mode)
      .unwrap_or_default();

    self.change_device(self.settings.audio.device.clone());
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

    let size = 22f32;
    let family = &mut fonts.family_and_size;

    family.insert(egui::TextStyle::Small, (egui::FontFamily::Proportional, size));
    family.insert(egui::TextStyle::Body, (egui::FontFamily::Proportional, size));
    family.insert(egui::TextStyle::Button, (egui::FontFamily::Proportional, size));
    family.insert(egui::TextStyle::Heading, (egui::FontFamily::Proportional, size * 1.2));
    family.insert(egui::TextStyle::Monospace, (egui::FontFamily::Proportional, size));

    ctx.set_style(style);
    ctx.set_fonts(fonts);
  }

  fn ui(&mut self, ctx: &CtxRef) {
    egui::SidePanel::left("Settings")
      .resizable(false)
      .default_width(200f32)
      .show(ctx, |ui| {
        let limit = screen_width().min(screen_height()) / 2f32;

        egui::CollapsingHeader::new("Visualizer").default_open(true).show(ui, |ui| {
          let state = &mut self.state.visualizer;

          ui.add(egui::Slider::new(&mut state.size, 0.001f32..=5f32).text("Size"));
          ui.add(egui::Slider::new(&mut state.line_gap, 0.1f32..=5f32).text("Line Gap"));
          ui.add(egui::Slider::new(&mut state.radius, -limit..=limit).text("Radius"));
          ui.add(egui::Slider::new(&mut state.offset_x, 0f32..=limit * 2f32).text("Offset X"));
          ui.add(egui::Slider::new(&mut state.offset_y, 0f32..=limit * 2f32).text("Offset Y"));
        });

        egui::CollapsingHeader::new("Audio").default_open(true).show(ui, |ui| {
          let name = AudioMode::ALL[self.state.audio.mode_index].name();

          if ui.add(
            egui::Slider::new(&mut self.state.audio.mode_index, 0..=AudioMode::ALL.len() - 1)
              .show_value(false)
              .text(format!("{}", name)),
          ).changed() {
            self.change_mode(AudioMode::ALL[self.state.audio.mode_index]);
          }

          let response = egui::ComboBox::from_label("Device Type")
            .selected_text(format!("{:?}", self.state.audio.device_type))
            .show_ui(ui, |ui| {
              [
                ui.selectable_value(&mut self.state.audio.device_type, AudioDeviceType::None, "None").clicked(),
                ui.selectable_value(&mut self.state.audio.device_type, AudioDeviceType::Default, "Default").clicked(),
                ui.selectable_value(&mut self.state.audio.device_type, AudioDeviceType::Loopback, "Loopback").clicked(),
                ui.selectable_value(&mut self.state.audio.device_type, AudioDeviceType::Input, "Input").clicked(),
                ui.selectable_value(&mut self.state.audio.device_type, AudioDeviceType::Output, "Output").clicked(),
              ]
            });

          let changed = response.inner.unwrap_or_default().contains(&true);

          match self.state.audio.device_type {
            AudioDeviceType::None if changed => self.change_device(AudioDevice::NONE),
            AudioDeviceType::Default if changed => self.change_device(AudioDevice::DEFAULT),
            AudioDeviceType::Loopback if changed => self.change_device(AudioDevice::LOOPBACK),
            AudioDeviceType::Input => {
              egui::ComboBox::from_label("Input Device")
                .selected_text(format!("{:.20}", self.state.audio.input_device.clone().unwrap_or_default()))
                .show_ui(ui, |ui| {
                  let devices = self.audio.host().input_devices().unwrap();
                  for device in devices {
                    let name = match device.name() {
                      Ok(name) => name,
                      Err(_) => continue,
                    };

                    if ui.selectable_label(false, name.clone()).clicked() {
                      self.change_device(AudioDevice::Input(name));
                    };
                  }
                });
            }
            AudioDeviceType::Output => {
              egui::ComboBox::from_label("Output Device")
                .selected_text(format!("{:.20}", self.state.audio.output_device.clone().unwrap_or_default()))
                .show_ui(ui, |ui| {
                  let devices = self.audio.host().output_devices().unwrap();
                  for device in devices {
                    let name = match device.name() {
                      Ok(name) => name,
                      Err(_) => continue,
                    };

                    if ui.selectable_label(false, name.clone()).clicked() {
                      self.change_device(AudioDevice::Output(name));
                    };
                  }
                });
            }
            _ => {}
          }
        });

        egui::CollapsingHeader::new("Color").default_open(true).show(ui, |ui| {
          ui.with_layout(
            egui::Layout::from_main_dir_and_cross_align(egui::Direction::LeftToRight, egui::Align::Min),
            |ui| {
              ui.color_edit_button_rgb(&mut self.state.fg_color);
              ui.label("Foreground Color");
            },
          );

          ui.with_layout(
            egui::Layout::from_main_dir_and_cross_align(egui::Direction::LeftToRight, egui::Align::Min),
            |ui| {
              ui.color_edit_button_rgb(&mut self.state.bg_color);
              ui.label("Background Color");
            },
          );
        });

        ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
          ui.add_space(6f32);

          if ui.button(" Save ").clicked() {
            if let Err(err) = self.settings.save_to_default_path() {
              eprintln!("{:?}", err);
            }
          }
        });
      });
  }

  fn before_draw(&mut self) {
    if is_key_pressed(KeyCode::H) {
      self.state.show_ui = !self.state.show_ui;
    }
  }

  fn draw(&self) {
    let state = &self.state.visualizer;
    clear_background(self.state.bg_color.as_color());

    if let Some(audio) = self.audio.data() {
      let len = audio.len();
      let center_w = state.offset_x + screen_width() / 2f32;
      let center_h = state.offset_y + screen_height() / 2f32;

      for i in 0..len {
        let value = audio[i] * 300f32 * state.size;
        let mut color = self.state.fg_color.as_color();
        let gap = screen_width() / len as f32;

        color.r = clamp(color.r * audio[i] * 5f32, 0.2, 1.0);
        color.g = clamp(color.g * audio[i] * 5f32, 0.2, 1.0);
        color.b = clamp(color.b * audio[i] * 5f32, 0.2, 1.0);

        let theta = (TAU / len as f32) * i as f32;
        let radius = state.radius * audio.sum / 200f32;

        let x_inner = center_w + (radius - value) * theta.sin();
        let y_inner = center_h - (radius - value) * theta.cos();
        let x_outer = center_w + (radius + value) * theta.sin();
        let y_outer = center_h - (radius + value) * theta.cos();

        // draw_rectangle(x_inner, y_inner, 2.0, 2.0, color);

        draw_line(x_inner, y_inner, x_outer, y_outer, state.line_gap, color)

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
