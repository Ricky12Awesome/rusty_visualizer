use nannou::prelude::*;
// use nannou::ui::prelude::*;
use crate::application::{Application, build_application_from};

mod application;
mod settings;
mod audio;
mod util;

struct State;

impl Application for State {
  fn init(_app: &App) -> Self {
    State {

    }
  }
}

fn main() {
  build_application_from::<State>()
    .size(1920, 1080)
    .run();
}
