use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::audio::AudioMode;

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
  pub device: String,
  pub mode: AudioMode,
}

#[allow(unused_variables, dead_code)]
impl Settings {
  pub fn new<S: Into<String>>(device: S) -> Self {
    Settings {
      device: device.into(),
      mode: AudioMode::Wave,
    }
  }

  pub fn loopback() -> Self {
    Settings::new("loopback")
  }

  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    let mut file = File::open(path)?;
    let mut data = String::new();

    file.read_to_string(&mut data)?;

    Ok(serde_json::from_str::<Settings>(data.as_str())?)
  }

  pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
    let mut file = File::create(path)?;
    let data = serde_json::to_string_pretty(self)?;

    file.write_all(data.as_bytes())
  }

  pub fn load_default() -> Self {
    Settings::load("./settings.json").unwrap_or(Settings::default())
  }

  pub fn save_default(&self) -> Result<(), Error> {
    self.save("./settings.json")
  }
}

impl Default for Settings {
  fn default() -> Self {
    Settings::new("default")
  }
}