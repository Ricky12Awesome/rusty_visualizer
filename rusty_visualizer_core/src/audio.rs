use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use cpal::{Device, Host, InputCallbackInfo, Stream, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use num_complex::Complex32;
use num_traits::Zero;
use serde::Deserialize;
use serde::Serialize;

use crate::fft::{FFTMode, FFTSize, process_fft};
use crate::settings::AudioSettings;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum AudioMode {
  FFT(FFTSize),
  Wave,
}

impl AudioMode {
  pub const ALL: &'static [Self] = &[
    AudioMode::Wave,
    AudioMode::FFT(FFTSize::FFT16),
    AudioMode::FFT(FFTSize::FFT32),
    AudioMode::FFT(FFTSize::FFT64),
    AudioMode::FFT(FFTSize::FFT128),
    AudioMode::FFT(FFTSize::FFT256),
    AudioMode::FFT(FFTSize::FFT512),
    AudioMode::FFT(FFTSize::FFT1024),
    AudioMode::FFT(FFTSize::FFT2048),
    AudioMode::FFT(FFTSize::FFT4096),
    AudioMode::FFT(FFTSize::FFT8192),
    AudioMode::FFT(FFTSize::FFT16384),
  ];

  pub const fn name(&self) -> &'static str {
    match self {
      AudioMode::Wave => "Wave",
      AudioMode::FFT(FFTSize::FFT16) => "FFT 16",
      AudioMode::FFT(FFTSize::FFT32) => "FFT 32",
      AudioMode::FFT(FFTSize::FFT64) => "FFT 64",
      AudioMode::FFT(FFTSize::FFT128) => "FFT 128",
      AudioMode::FFT(FFTSize::FFT256) => "FFT 256",
      AudioMode::FFT(FFTSize::FFT512) => "FFT 512",
      AudioMode::FFT(FFTSize::FFT1024) => "FFT 1024",
      AudioMode::FFT(FFTSize::FFT2048) => "FFT 2048",
      AudioMode::FFT(FFTSize::FFT4096) => "FFT 4096",
      AudioMode::FFT(FFTSize::FFT8192) => "FFT 8192",
      AudioMode::FFT(FFTSize::FFT16384) => "FFT 16384",
    }
  }
}

#[derive(Clone)]
pub struct AudioData {
  pub data: Vec<f32>,
  pub sum: f32,
  pub mode: AudioMode,
}

impl AudioData {
  fn new(data: &[f32], mode: AudioMode) -> Self {
    let mut sum = 0.0f32;

    let process = |it: f32| {
      sum += it;
      it
    };

    let data = match mode {
      AudioMode::Wave => data.iter().map(|it| *it).map(process).collect(),
      AudioMode::FFT(size) => {
        let size_v = size as usize;
        let len = data.len() + size_v + 1;
        let mut buffer = vec![Complex32::zero(); len];

        for i in 0..data.len() {
          buffer[i] = Complex32::from(data[i]);
        }

        process_fft(&mut buffer, &size, FFTMode::Backward);

        buffer
          .iter()
          .map(|it| (it.re * it.re + it.im * it.im).sqrt().sqrt() / 10f32)
          .take(size_v)
          .map(process)
          .collect()
      }
    };

    AudioData { data, sum, mode }
  }
}

impl Deref for AudioData {
  type Target = Vec<f32>;

  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

impl Default for AudioData {
  fn default() -> Self {
    AudioData::new(&[], AudioMode::Wave)
  }
}

unsafe impl Send for Audio {}

pub struct Audio {
  host: Host,
  mode: Arc<RwLock<AudioMode>>,
  stream: Option<Stream>,
  receiver: Option<Arc<RwLock<AudioData>>>,
  auto_play: bool,
}

impl From<&AudioSettings> for Audio {
  fn from(settings: &AudioSettings) -> Self {
    let host = cpal::default_host();
    let mode = Arc::new(RwLock::new(settings.mode));

    let mut audio = Audio {
      host,
      mode,
      stream: None,
      receiver: None,
      auto_play: settings.auto_play,
    };

    if settings.auto_set {
      audio.change_device(settings.device.clone());
    }

    return audio;
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AudioDevice<D: NamedAudioDevice = ()> {
  None,
  Default,
  Loopback,
  Input(D),
  Output(D),
}

impl AudioDevice {
  pub const NONE: Self = Self::None;
  pub const DEFAULT: Self = Self::Default;
  pub const LOOPBACK: Self = Self::Loopback;
}

impl<D: NamedAudioDevice> Default for AudioDevice<D> {
  fn default() -> Self {
    Self::Default
  }
}

pub trait NamedAudioDevice: Send {
  fn to_device(self, host: &Host) -> Option<Device>;
}

pub trait NamedAudioDeviceWithConfig: Send {
  fn to_device(self, host: &Host) -> Option<(SupportedStreamConfig, Device)>;
}

pub trait ToSerializableAudioDevice {
  fn to_serializable(self, audio: &Audio) -> AudioDevice<String>;
}

impl<D: NamedAudioDevice> ToSerializableAudioDevice for AudioDevice<D> {
  fn to_serializable(self, audio: &Audio) -> AudioDevice<String> {
    match self {
      AudioDevice::None => AudioDevice::None,
      AudioDevice::Default => AudioDevice::Default,
      AudioDevice::Loopback => AudioDevice::Loopback,
      AudioDevice::Input(device) => match device.to_device(audio.host()) {
        None => AudioDevice::None,
        Some(device) => AudioDevice::Input(device.name().unwrap_or_default()),
      },
      AudioDevice::Output(device) => match device.to_device(audio.host()) {
        None => AudioDevice::None,
        Some(device) => AudioDevice::Output(device.name().unwrap_or_default()),
      },
    }
  }
}

impl<D: NamedAudioDevice> NamedAudioDeviceWithConfig for AudioDevice<D> {
  fn to_device(self, host: &Host) -> Option<(SupportedStreamConfig, Device)> {
    match self {
      AudioDevice::Default => "default".to_device(host).map(|it| (it.default_input_config().unwrap(), it)),
      AudioDevice::Loopback => "loopback".to_device(host).map(|it| (it.default_output_config().unwrap(), it)),
      AudioDevice::Input(device) => device.to_device(host).map(|it| (it.default_input_config().unwrap(), it)),
      AudioDevice::Output(device) => device.to_device(host).map(|it| (it.default_output_config().unwrap(), it)),
      _ => None,
    }
  }
}

impl NamedAudioDevice for () {
  fn to_device(self, _host: &Host) -> Option<Device> {
    None
  }
}

impl NamedAudioDevice for &str {
  fn to_device(self, host: &Host) -> Option<Device> {
    match self {
      "default" => host.default_input_device(),
      "loopback" => host.default_output_device(),
      "none" | "" => None,
      _ => host
        .devices()
        .unwrap()
        .find(|it| it.name().unwrap_or_else(|_| String::from("")) == self),
    }
  }
}

impl NamedAudioDevice for String {
  fn to_device(self, host: &Host) -> Option<Device> {
    NamedAudioDevice::to_device(self.as_ref(), host)
  }
}

impl NamedAudioDevice for Device {
  fn to_device(self, _host: &Host) -> Option<Device> {
    Some(self)
  }
}

impl Audio {
  pub fn is_mode_fft(&self) -> bool {
    matches!(self.mode(), AudioMode::FFT(_))
  }

  pub fn host(&self) -> &Host {
    &self.host
  }

  pub fn stream(&self) -> &Option<Stream> {
    &self.stream
  }

  pub fn data(&self) -> Option<RwLockReadGuard<AudioData>> {
    self.receiver.as_ref()?.read().ok()
  }

  pub fn mode(&self) -> AudioMode {
    *self.mode.read().unwrap()
  }

  pub fn change_mode(&mut self, new_mode: AudioMode) {
    *self.mode.write().unwrap() = new_mode;
  }

  pub fn change_device(&mut self, new_device: impl NamedAudioDeviceWithConfig) {
    crossbeam_utils::thread::scope(|s| {
      s.spawn(|_| match new_device.to_device(&self.host) {
        None => {
          self.stream = None;
          self.receiver = None;
        }
        Some((config, device)) => {
          let sender = Arc::new(RwLock::new(AudioData::default()));
          let receiver = sender.clone();
          let mode = self.mode.clone();

          let stream = device
            .build_input_stream(
              &config.config(),
              move |data: &[f32], _: &InputCallbackInfo| {
                *sender.write().unwrap() = AudioData::new(data, *mode.read().unwrap());
              },
              move |err| println!("{:?}", err),
            )
            .unwrap();

          if self.auto_play {
            stream.play().unwrap();
          }

          self.stream = Some(stream);
          self.receiver = Some(receiver);

          println!("Changed Device: {}", device.name().unwrap_or_default());
        }
      });
    })
      .unwrap();
  }
}
