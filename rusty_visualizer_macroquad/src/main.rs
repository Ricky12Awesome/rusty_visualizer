#![allow(unused)]

use std::borrow::Cow;
use std::f32::consts::TAU;

use egui::{Align, CtxRef, Order};
use image::RgbaImage;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use spotify_info::{TrackHandle, TrackInfo, TrackListener, TrackState};

use rusty_visualizer_core::audio::{Audio, AudioDevice, AudioMode, ToSerializableAudioDevice};
use rusty_visualizer_core::cpal::traits::{DeviceTrait, HostTrait};
use rusty_visualizer_core::settings::{AudioManager, AudioSettings, SettingsManager};

use crate::application::{Application, run_application};
use crate::cache::{ImageCache, ImageCacheType};
use crate::color::{AsColor, GrayColor};
use crate::util::{egui_draw_text, font_def};

mod application;
mod cache;
mod color;
mod util;

const AUDIO_DEVICE_SWITCH_NOT_SUPPORT: &str = "Not supported on linux because ALSA is terrible, you can use something like pavucontrol instead";

pub const NOTO_SANS: &[u8] = include_bytes!("../../assets/NotoSans-Regular.ttf");
pub const NOTO_SANS_JP: &[u8] = include_bytes!("../../assets/NotoSansJP-Regular.otf");

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

//region Settings
#[derive(Serialize, Deserialize, Clone, Default)]
struct Settings {
  audio: AudioSettings,
  state: State,
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
      AudioDevice::None => self.settings.state.audio.device_type = AudioDeviceType::None,
      AudioDevice::Default => self.settings.state.audio.device_type = AudioDeviceType::Default,
      AudioDevice::Loopback => self.settings.state.audio.device_type = AudioDeviceType::Loopback,
      AudioDevice::Input(name) => {
        self.settings.state.audio.device_type = AudioDeviceType::Input;
        self.settings.state.audio.input_device = Some(name.clone());
      }
      AudioDevice::Output(name) => {
        self.settings.state.audio.device_type = AudioDeviceType::Output;
        self.settings.state.audio.output_device = Some(name.clone());
      }
    }

    self.audio_settings().device = new_device.clone();
    self.audio().change_device(new_device);
  }
}
//endregion

//region State
#[derive(PartialEq, Debug, Clone)]
enum AudioDeviceType {
  None,
  Default,
  Loopback,
  Input,
  Output,
}

#[derive(Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
struct State {
  visualizer: VisualizerState,
  #[serde(skip)] audio: AudioState,
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
//endregion

struct App {
  settings: Settings,
  audio: Audio,
  runtime: tokio::runtime::Runtime,
  loop_thread: tokio::task::JoinHandle<()>,
  handle: TrackHandle,
  track: TrackInfo,
  state: TrackState,
  cache: ImageCache,
  cover_texture: Texture2D,
  bg_texture: Texture2D,
  font: Font,
}

impl App {
  fn change_track_if_changed(&mut self) {
    if let Some(track) = self.handle.read() {
      self.state = track.state;

      if !self.track.eq_ignore_state(&track) {
        self.track = track;
        self.track.state = TrackState::Stopped;
        self.on_track_change();
      }
    }
  }

  fn get_image(
    &mut self,
    url: &str,
    uid: String,
    typ: ImageCacheType,
    size: (Option<u32>, Option<u32>),
  ) -> Option<&RgbaImage> {
    self.cache.get(uid, url, typ, size)
  }

  fn get_texture(
    &mut self,
    url: &str,
    uid: String,
    typ: ImageCacheType,
    size: (Option<u32>, Option<u32>),
  ) -> Texture2D {
    match self.get_image(url, uid, typ, size) {
      None => Texture2D::empty(),
      Some(image) => Texture2D::from_rgba8(image.width() as u16, image.height() as u16, image.as_raw())
    }
  }

  fn center(texture: &Texture2D) -> (f32, f32) {
    (screen_width() / 2f32 - texture.width() / 2f32, (screen_height() / 2f32 - texture.height() / 2f32))
  }

  fn bottom_left(texture: &Texture2D) -> (f32, f32) {
    (00f32, screen_height() - texture.height())
  }

  fn set_textures(&mut self, force: bool) {
    let uid = self.track.uid.clone();

    if let Some(url) = self.track.cover_url.clone() {
      self.cache.set_texture(
        &mut self.cover_texture,
        uid.clone(), &url, force,
        ImageCacheType::Cover, (Some(256), Some(256)),
      );
    }

    if let Some(url) = self.track.background_url.clone() {
      self.cache.set_texture(
        &mut self.bg_texture,
        uid, &url, force,
        ImageCacheType::Background,
        (Some(screen_width() as u32), Some(screen_height() as u32)),
      );
    }
  }

  fn on_track_change(&mut self) {
    self.set_textures(true);
  }
}

impl Application for App {
  fn init() -> Self {
    let settings = Settings::load();
    let audio = Audio::from(&settings.audio);
    let state = State::default();
    let handle = TrackHandle::default();
    let loop_handle = handle.clone();

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let loop_thread = runtime.spawn(async {
      let listener = TrackListener::bind_default().await.unwrap();
      listener.listen(loop_handle).await;
    });


    let noto_sans = load_ttf_font_from_bytes(NOTO_SANS).unwrap();
    // let noto_sans_jp = load_ttf_font_from_bytes(NOTO_SANS_JP).unwrap();

    Self {
      settings,
      audio,
      runtime,
      loop_thread,
      handle,
      cache: ImageCache::default(),
      track: TrackInfo::default(),
      state: TrackState::default(),
      cover_texture: Texture2D::empty(),
      bg_texture: Texture2D::empty(),
      font: noto_sans,
    }
  }

  fn setup(&mut self) {
    self.settings.state.audio.mode_index = AudioMode::ALL
      .iter()
      .position(|it| *it == self.settings.audio.mode)
      .unwrap_or_default();

    self.change_device(self.settings.audio.device.clone());
  }

  //region UI
  fn show_ui(&self) -> bool {
    self.settings.state.show_ui
  }

  fn ui_cfg(&self, ctx: &CtxRef) {
    let mut style = egui::Style::default();

    let accent = egui::Color32::from_rgb(180, 50, 160);
    style.visuals.hyperlink_color = accent;
    style.visuals.selection.bg_fill = accent;
    style.spacing.slider_width = 200f32;

    let mut fonts = font_def(22f32, 48f32);

    ctx.set_style(style);
    ctx.set_fonts(fonts);
  }

  fn ui(&mut self, ctx: &CtxRef) {
    let id = egui::Id::new("UI");
    let layer = egui::LayerId::new(Order::Debug, id);
    let mut ui = egui::Ui::new(ctx.clone(), layer, id, ctx.available_rect(), ctx.input().screen_rect);

    egui::SidePanel::left("Settings")
      .resizable(false)
      .default_width(200f32)
      .show(ctx, |ui| {
        egui::CollapsingHeader::new("Visualizer").default_open(true).show(ui, |ui| {
          let limit = screen_width().min(screen_height()) / 2f32;
          let state = &mut self.settings.state.visualizer;

          ui.add(egui::Slider::new(&mut state.size, 0.001f32..=5f32).text("Size"));
          ui.add(egui::Slider::new(&mut state.line_gap, 0.1f32..=5f32).text("Line Gap"));
          ui.add(egui::Slider::new(&mut state.radius, -limit..=limit).text("Radius"));
          ui.add(egui::Slider::new(&mut state.offset_x, 0f32..=limit * 2f32).text("Offset X"));
          ui.add(egui::Slider::new(&mut state.offset_y, 0f32..=limit * 2f32).text("Offset Y"));
        });

        egui::CollapsingHeader::new("Audio").default_open(true).show(ui, |ui| {
          let name = AudioMode::ALL[self.settings.state.audio.mode_index].name();

          if ui.add(
            egui::Slider::new(&mut self.settings.state.audio.mode_index, 0..=AudioMode::ALL.len() - 1)
              .show_value(false)
              .text(name.to_string()),
          ).changed() {
            self.change_mode(AudioMode::ALL[self.settings.state.audio.mode_index]);
          }

          let response = egui::ComboBox::from_label("Device Type")
            .selected_text(format!("{:?}", self.settings.state.audio.device_type))
            .show_ui(ui, |ui| {
              [
                ui.selectable_value(&mut self.settings.state.audio.device_type, AudioDeviceType::None, "None").clicked(),
                ui.selectable_value(&mut self.settings.state.audio.device_type, AudioDeviceType::Default, "Default").clicked(),
                ui.selectable_value(&mut self.settings.state.audio.device_type, AudioDeviceType::Loopback, "Loopback").clicked(),
                ui.selectable_value(&mut self.settings.state.audio.device_type, AudioDeviceType::Input, "Input").clicked(),
                ui.selectable_value(&mut self.settings.state.audio.device_type, AudioDeviceType::Output, "Output").clicked(),
              ]
            });

          let changed = response.inner.unwrap_or_default().contains(&true);

          match self.settings.state.audio.device_type {
            AudioDeviceType::None if changed => self.change_device(AudioDevice::NONE),
            AudioDeviceType::Default if changed => self.change_device(AudioDevice::DEFAULT),
            AudioDeviceType::Loopback if changed => self.change_device(AudioDevice::LOOPBACK),
            AudioDeviceType::Input => {
              #[cfg(target_os = "linux")]
                ui.label(AUDIO_DEVICE_SWITCH_NOT_SUPPORT);
              #[cfg(not(target_os = "linux"))]
                egui::ComboBox::from_label("Input Device")
                .selected_text(format!("{:.20}", self.settings.state.audio.input_device.clone().unwrap_or_default()))
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
              #[cfg(target_os = "linux")]
                ui.label(AUDIO_DEVICE_SWITCH_NOT_SUPPORT);
              #[cfg(not(target_os = "linux"))]
                egui::ComboBox::from_label("Output Device")
                .selected_text(format!("{:.20}", self.settings.state.audio.output_device.clone().unwrap_or_default()))
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
              ui.color_edit_button_rgb(&mut self.settings.state.fg_color);
              ui.label("Foreground Color");
            },
          );

          ui.with_layout(
            egui::Layout::from_main_dir_and_cross_align(egui::Direction::LeftToRight, egui::Align::Min),
            |ui| {
              ui.color_edit_button_rgb(&mut self.settings.state.bg_color);
              ui.label("Background Color");
            },
          );
        });

        egui::CollapsingHeader::new("Currently Playing track").default_open(true).show(ui, |ui| {
          ui.label(format!("Title - {}", self.track.title));
          ui.label(format!("Artist - {:?}", self.track.artist));
          ui.label(format!("Album - {}", self.track.album));
          ui.label(format!("State - {:?}", self.state));
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
  //endregion

  fn before_draw(&mut self) {
    self.change_track_if_changed();
    self.set_textures(false);

    if is_key_pressed(KeyCode::H) {
      self.settings.state.show_ui = !self.settings.state.show_ui;
    }
  }

  fn draw(&self, ctx: &CtxRef) {
    let state = &self.settings.state.visualizer;
    clear_background(self.settings.state.bg_color.as_color());

    let color = if matches!(self.state, TrackState::Playing) { 128 } else { 32 };

    let (x, y) = App::center(&self.bg_texture);
    draw_texture(self.bg_texture, x, y, Color::gray_scale(color));

    let (x, y) = App::bottom_left(&self.cover_texture);
    draw_texture(self.cover_texture, 70f32 + x, y - 150f32, Color::gray_scale(color + 96));

    egui_draw_text(ctx, &self.track.title, 70f32 + x, y + 120f32, 48, Color::gray_scale(240));
    // draw_text_ex(&self.track.title, 70f32 + x, y + 175f32, TextParams {
    //   font_size: 48,
    //   font_scale: 1.0,
    //   color: Color::gray_scale(240),
    //   font: self.font,
    //   ..Default::default()
    // });

    if let Some(audio) = self.audio.data() {
      let len = audio.len();
      let center_w = state.offset_x + screen_width() / 2f32;
      let center_h = state.offset_y + screen_height() / 2f32;

      for i in 0..len {
        let value = audio[i] * 300f32 * state.size;
        let mut color = self.settings.state.fg_color.as_color();
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
