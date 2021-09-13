use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use rusty_visualizer_core::util::AnyErrorResult;

#[derive(Clone, Serialize, Deserialize)]
pub struct RaylibOptions {
  title: String,
  width: u32,
  height: u32,
  msaa: bool,
  resizeable: bool,
  transparent: bool,
  undecorated: bool,
  start_fullscreen: bool,
}

impl Default for RaylibOptions {
  fn default() -> Self {
    Self {
      title: String::from("Rusty Visualizer"),
      width: 1920,
      height: 1080,
      msaa: true,
      resizeable: true,
      transparent: false,
      undecorated: false,
      start_fullscreen: false,
    }
  }
}

pub trait ApplyOptions<O> {
  fn apply(&mut self, options: &O) -> &mut Self;
}

impl ApplyOptions<RaylibOptions> for RaylibBuilder {
  #[rustfmt::skip]
  fn apply(&mut self, options: &RaylibOptions) -> &mut Self {
    self.title(&options.title);
    self.size(options.width as i32, options.height as i32);

    if options.msaa { self.msaa_4x(); }
    if options.undecorated { self.undecorated(); }
    if options.transparent { self.transparent(); }
    if options.resizeable { self.resizable(); }
    if options.start_fullscreen { self.fullscreen(); }
    self
  }
}

#[allow(unused_variables)]
pub trait Application {
  fn init() -> Self;
  fn setup(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {}

  fn draw(&self, d: &mut RaylibDrawHandle);

  fn gui<G: RaylibDrawGui>(&mut self, d: &mut G) {}

  fn raylib_options(&self) -> RaylibOptions;
}

pub fn run_application<A: Application>() -> AnyErrorResult<()> {
  let mut app = A::init();
  let options = app.raylib_options();

  let (mut rl, thread) = raylib::init().apply(&options).build();

  app.setup(&mut rl, &thread);

  while !rl.window_should_close() {
    let mut d = rl.begin_drawing(&thread);

    app.draw(&mut d);
    app.gui(&mut d);
  }

  Ok(())
}
