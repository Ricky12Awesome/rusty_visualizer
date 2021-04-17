// TODO: This class should probably be moved into another crate since it doesn't really belong here

use std::marker::PhantomData;
use std::ops::Range;

use num_traits::{AsPrimitive, FromPrimitive};

#[derive(Debug)]
pub struct FloatStepIterator<Out = f64> {
  pub start: f64,
  pub end: f64,
  pub step: f64,
  counter: f64,
  _out: PhantomData<Out>,
}

pub trait StepByFloat {
  fn step_by<Out>(self, step: f64) -> FloatStepIterator<Out>;
}

impl<N: AsPrimitive<f64>> StepByFloat for Range<N> {
  fn step_by<Out>(self, step: f64) -> FloatStepIterator<Out> {
    FloatStepIterator::from(self, step)
  }
}

impl<Out> FloatStepIterator<Out> {
  pub fn new(start: f64, end: f64, step: f64) -> Self {
    FloatStepIterator { start, end, step, counter: start, _out: PhantomData::default() }
  }

  pub fn from<N: AsPrimitive<f64>>(range: Range<N>, step: f64) -> Self {
    FloatStepIterator::new(range.start.as_(), range.end.as_(), step)
  }
}

impl<Out: FromPrimitive> std::iter::Iterator for FloatStepIterator<Out> {
  type Item = Out;

  fn next(&mut self) -> Option<Self::Item> {
    const TIMES: f64 = 100000.0;
    let mut result = None;

    if ((self.counter * TIMES) + (self.step * TIMES)).ceil() <= (self.end * TIMES).ceil() {
      result = Out::from_f64(self.counter);
      self.counter += self.step;
    }

    result
  }
}

#[cfg(test)]
mod tests {
  use std::f64::consts::TAU;

  use crate::iterator::StepByFloat;

  const ITERATIONS: usize = 32768;

  #[test]
  fn check_for_length_f64() {
    for i in 0..ITERATIONS {
      let vec = (0.0..TAU).step_by(TAU / i as f64).collect::<Vec<f64>>();
      assert_eq!(vec.len(), i, "Found incorrect match: {} != {}", vec.len(), i);
    }
  }

  #[test]
  fn check_for_length_f32() {
    for i in 0..ITERATIONS {
      let vec = (0.0..TAU).step_by(TAU / i as f64).collect::<Vec<f32>>();
      assert_eq!(vec.len(), i, "Found incorrect match: {} != {}", vec.len(), i);
    }
  }
}