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
  pub const ALL: &'static [AudioMode] = &[
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

  pub fn all_named() -> Vec<String> {
    Self::ALL.iter()
        .map(|it| {
          match it {
            AudioMode::Wave => format!("Wave"),
            AudioMode::FFT(size) => format!("FFT {}", *size as usize),
          }
        })
        .collect()
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
      AudioMode::Wave => {
        data
            .iter()
            .map(|it| *it)
            .map(process)
            .collect()
      }
      AudioMode::FFT(size) => {
        let size_v = size as usize;
        let len = data.len() + size_v + 1;
        let mut buffer = vec![Complex32::zero(); len];

        for i in 0..data.len() {
          buffer[i] = Complex32::from(data[i]);
        }

        process_fft(&mut buffer, &size, FFTMode::Backward);

        buffer.iter()
            .map(|it| (it.re * it.re + it.im * it.im).sqrt().sqrt() / 10f32)
            .take(size_v)
            .map(process)
            .collect()
      }
    };

    AudioData {
      data,
      sum,
      mode,
    }
  }
}

impl Deref for AudioData {
  type Target = Vec<f32>;

  fn deref(&self) -> &Self::Target { &self.data }
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

impl<S: AudioSettings> From<&S> for Audio {
  fn from(settings: &S) -> Self {
    let host = cpal::default_host();
    let mode = Arc::new(RwLock::new(*settings.mode()));

    let mut audio = Audio {
      host,
      mode,
      stream: None,
      receiver: None,
      auto_play: settings.auto_play(),
    };

    audio.change_device(settings.device().clone());

    return audio;
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AudioDevice<D: NamedAudioDevice> {
  Default,
  Loopback,
  Input(D),
  Output(D),
}

pub trait NamedAudioDevice: Send {
  fn get_device(self, host: &Host) -> Option<Device>;
}

impl<D: NamedAudioDevice> AudioDevice<D> {
  fn get_device(self, host: &Host) -> Option<(SupportedStreamConfig, Device)> {
    match self {
      AudioDevice::Default => "default".get_device(host)
          .map(|it| (it.default_input_config().unwrap(), it)),
      AudioDevice::Loopback => "loopback".get_device(host)
          .map(|it| (it.default_output_config().unwrap(), it)),
      AudioDevice::Input(device) => device.get_device(host)
          .map(|it| (it.default_input_config().unwrap(), it)),
      AudioDevice::Output(device) => device.get_device(host)
          .map(|it| (it.default_output_config().unwrap(), it)),
    }
  }
}

impl NamedAudioDevice for &str {
  fn get_device(self, host: &Host) -> Option<Device> {
    match self {
      "default" => host.default_input_device(),
      "loopback" => host.default_output_device(),
      _ => host.devices().unwrap()
          .find(|it| it.name().unwrap_or(String::from("")) == self)
    }
  }
}


impl NamedAudioDevice for String {
  fn get_device(self, host: &Host) -> Option<Device> {
    NamedAudioDevice::get_device(self.as_ref(), host)
  }
}

impl NamedAudioDevice for Device {
  fn get_device(self, _host: &Host) -> Option<Device> {
    Some(self)
  }
}

impl Audio {
  pub fn is_mode_fft(&self) -> bool {
    if let AudioMode::FFT(_) = self.mode() { true } else { false }
  }

  pub fn host(&self) -> &Host { &self.host }

  pub fn stream(&self) -> &Option<Stream> { &self.stream }

  pub fn data(&self) -> Option<RwLockReadGuard<AudioData>> {
    self.receiver.as_ref()?.read().ok()
  }

  pub fn mode(&self) -> AudioMode {
    *self.mode.read().unwrap()
  }

  pub fn change_mode(&mut self, new_mode: AudioMode) {
    *self.mode.write().unwrap() = new_mode;
  }

  pub fn change_device<D: NamedAudioDevice>(&mut self, new_device: AudioDevice<D>) {
    crossbeam_utils::thread::scope(|s| {
      s.spawn(|_| {
        match new_device.get_device(&self.host) {
          None => {
            self.stream = None;
            self.receiver = None;
          }
          Some((config, device)) => {
            let sender = Arc::new(RwLock::new(AudioData::default()));
            let receiver = sender.clone();
            let mode = self.mode.clone();

            let stream = device.build_input_stream(
              &config.config(),
              move |data: &[f32], _: &InputCallbackInfo| {
                *sender.write().unwrap() = AudioData::new(data, *mode.read().unwrap());
              },
              move |err| { println!("{:?}", err) },
            ).unwrap();

            if self.auto_play {
              stream.play().unwrap();
            }

            self.stream = Some(stream);
            self.receiver = Some(receiver);
          }
        }
      });
    }).unwrap();
  }
}