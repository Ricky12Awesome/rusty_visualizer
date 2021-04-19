use std::fs::File;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::audio::{AudioDevice, AudioMode};
use crate::util::AnyErrorResult;

pub trait AudioSettings {
  fn device(&self) -> &AudioDevice<String>;
  fn mode(&self) -> &AudioMode;
  fn auto_play(&self) -> bool;
}

#[derive(Serialize, Deserialize)]
pub struct Settings<O> {
  pub device: AudioDevice<String>,
  pub mode: AudioMode,
  pub auto_play: bool,
  pub options: Option<O>,
}

impl<O> AudioSettings for Settings<O> {
  fn device(&self) -> &AudioDevice<String> { &self.device }
  fn mode(&self) -> &AudioMode { &self.mode }
  fn auto_play(&self) -> bool { self.auto_play }
}

impl<O: Serialize + DeserializeOwned + Default> Settings<O> {
  pub fn change_options<F: Fn(&mut O)>(&mut self, change: F) {
    match &mut self.options {
      Some(options) => change(options),
      None => {
        let mut options = O::default();

        change(&mut options);

        self.options = Some(options);
      }
    }
  }
}

impl<O: Serialize + DeserializeOwned> Settings<O> {
  pub fn new(device: AudioDevice<String>, options: Option<O>) -> Self {
    Settings {
      device,
      mode: AudioMode::Wave,
      auto_play: true,
      options,
    }
  }

  pub fn load<P: AsRef<Path>>(path: P) -> AnyErrorResult<Self> {
    let file = File::open(path)?;

    Ok(serde_json::from_reader(file)?)
  }

  pub fn save<P: AsRef<Path>>(&self, path: P) -> AnyErrorResult<()> {
    let mut file = File::create(path)?;

    Ok(serde_json::to_writer_pretty(&mut file, self)?)
  }

  pub fn load_default() -> AnyErrorResult<Self> {
    Settings::load("./settings.json")
  }

  pub fn save_default(&self) -> AnyErrorResult<()> {
    self.save("./settings.json")
  }
}

impl<O: Serialize + DeserializeOwned + Default> Default for Settings<O> {
  fn default() -> Self {
    Settings::new(AudioDevice::Default, Some(O::default()))
  }
}