use num_complex::Complex32;
use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FFTMode {
  Forward,
  Backward,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[rustfmt::skip]
pub enum FFTSize {
  #[serde(rename = "16",    alias = "FFT16"   )] FFT16 =       16,
  #[serde(rename = "32",    alias = "FFT32"   )] FFT32 =       32,
  #[serde(rename = "64",    alias = "FFT64"   )] FFT64 =       64,
  #[serde(rename = "128",   alias = "FFT128"  )] FFT128 =     128,
  #[serde(rename = "256",   alias = "FFT256"  )] FFT256 =     256,
  #[serde(rename = "512",   alias = "FFT512"  )] FFT512 =     512,
  #[serde(rename = "1024",  alias = "FFT1024" )] FFT1024 =   1024,
  #[serde(rename = "2048",  alias = "FFT2048" )] FFT2048 =   2048,
  #[serde(rename = "4096",  alias = "FFT4096" )] FFT4096 =   4096,
  #[serde(rename = "8192",  alias = "FFT8192" )] FFT8192 =   8192,
  #[serde(rename = "16384", alias = "FFT16384")] FFT16384 = 16384,
}

fn inverse(data: &mut [Complex32], c: usize) {
  let mut i2 = 0;
  let n1 = c >> 1;

  for i in 0..c - 1 {
    if i < i2 {
      data.swap(i, i2);
    }

    let mut n2 = n1 as usize;

    while n2 <= i2 {
      i2 -= n2;
      n2 >>= 1;
    }

    i2 += n2;
  }
}

fn forward(data: &mut [Complex32], c: usize) {
  for i in 0..c {
    data[i].re /= c as f32;
    data[i].im /= c as f32;
  }
}

pub fn process_fft(data: &mut [Complex32], size: &FFTSize, mode: FFTMode) {
  let size = *size as usize;
  let ex = (size as f32).log2().floor();
  inverse(data, size);
  let mut n2 = 1;
  let mut n3 = -1f32;
  let mut n4 = 0.0f32;

  for _ in 0..ex as usize {
    let mut n5 = 1f32;
    let mut n6 = 0.0f32;
    let n7 = n2;

    n2 <<= 1;
    for i2 in 0..n7 {
      for i3 in (i2..size).step_by(n2) {
        let i4 = i3 + n7;
        let n8 = n5 * data[i4].re - n6 * data[i4].im;
        let n9 = n5 * data[i4].im + n6 * data[i4].re;

        data[i4].re = data[i3].re - n8;
        data[i4].im = data[i3].im - n9;
        data[i3].re += n8;
        data[i3].im += n9;
      }

      let n10 = n3 * n5 - n4 * n6;
      n6 = n4 * n5 + n3 * n6;
      n5 = n10;
    }

    n4 = if mode != FFTMode::Forward {
      -((1f32 - n3) / 2f32).sqrt()
    } else {
      ((1f32 - n3) / 2f32).sqrt()
    };

    n3 = ((1f32 + n3) / 2f32).sqrt();
  }

  if mode == FFTMode::Forward {
    forward(data, size);
  }
}
