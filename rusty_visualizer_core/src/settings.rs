use std::fs::File;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::audio::{Audio, AudioDevice, AudioMode, ToSerializableAudioDevice};

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioSettings {
  pub device: AudioDevice<String>,
  pub mode: AudioMode,
  pub auto_play: bool,
  pub auto_set: bool,
}

impl AudioSettings {
  pub fn new(device: AudioDevice<String>) -> Self {
    AudioSettings {
      device,
      mode: AudioMode::Wave,
      auto_play: true,
      auto_set: true,
    }
  }
}

impl Default for AudioSettings {
  fn default() -> Self {
    Self::new(AudioDevice::Default)
  }
}

pub trait SettingsManager: Sized + Serialize + DeserializeOwned + Default {
  const DEFAULT_PATH: &'static str = "./settings.json";


  fn load_from_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
    let file = File::open(path)?;

    Ok(serde_json::from_reader(file)?)
  }

  fn save_to_path<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    Ok(serde_json::to_writer_pretty(&mut file, self)?)
  }

  fn load_from_path_or_default<P: AsRef<Path>>(path: P) -> Self {
    Self::load_from_path(path).unwrap_or_default()
  }

  fn load() -> Self {
    Self::load_from_default_path().unwrap_or_default()
  }

  fn load_from_default_path() -> std::io::Result<Self> {
    Self::load_from_path(Self::DEFAULT_PATH)
  }

  fn save_to_default_path(&self) -> std::io::Result<()> {
    self.save_to_path(Self::DEFAULT_PATH)
  }
}

pub trait AudioManager {
  fn audio_settings(&mut self) -> &mut AudioSettings;
  fn audio(&mut self) -> &mut Audio;

  fn change_mode(&mut self, new_mode: AudioMode) {
    self.audio_settings().mode = new_mode;
    self.audio().change_mode(new_mode);
  }

  fn change_device(&mut self, new_device: impl ToSerializableAudioDevice) {
    let new_device = new_device.to_serializable(self.audio());

    self.audio_settings().device = new_device.clone();
    self.audio().change_device(new_device);
  }
}