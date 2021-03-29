
use std::path::PathBuf;

use nannou::App;
use nannou::prelude::*;
use nannou::winit::event::{DeviceEvent, VirtualKeyCode};
use nannou::app::Builder;

#[allow(unused_variables)]
pub trait Application {
  fn init(app: &App) -> Self;

  fn on_event(app: &App, state: &mut Self, event: Event) {
    match event {
      Event::WindowEvent { id: _id, simple: Some(event), .. } => Self::on_window_event(app, state, event),
      Event::WindowEvent { id: _, simple: None, .. } => {}
      Event::DeviceEvent(_, event) => Self::on_device_event(app, state, event),
      Event::Update(update) => Self::on_update(app, state, update),
      Event::Suspended => Self::on_suspend(app, state),
      Event::Resumed => Self::on_resumed(app, state),
    }
  }

  fn on_window_event(app: &App, state: &mut Self, event: WindowEvent) {
    match event {
      Moved(position) => Self::on_window_moved(app, state, position),
      KeyPressed(key) => Self::on_key_pressed(app, state, key),
      KeyReleased(key) => Self::on_key_released(app, state, key),
      MouseMoved(position) => Self::on_mouse_moved(app, state, position),
      MousePressed(button) => Self::on_mouse_pressed(app, state, button),
      MouseReleased(button) => Self::on_mouse_released(app, state, button),
      MouseEntered => Self::on_mouse_entered(app, state),
      MouseExited => Self::on_mouse_exited(app, state),
      MouseWheel(data, phase) => Self::on_mouse_wheal(app, state, data, phase),
      Resized(new_size) => Self::on_resize(app, state, new_size),
      HoveredFile(event) => Self::on_hovered_file(app, state, event),
      DroppedFile(path) => Self::on_dropped_file(app, state, path),
      HoveredFileCancelled => Self::on_hovered_file_cancelled(app, state),
      Touch(event) => Self::on_touch(app, state, event),
      TouchPressure(pressure) => Self::on_touch_pressure(app, state, pressure),
      Focused => Self::on_focused(app, state),
      Unfocused => Self::on_unfocused(app, state),
      Closed => Self::on_closed(app, state)
    }
  }

  fn on_device_event(app: &App, state: &mut Self, event: DeviceEvent) {}

  fn on_update(app: &App, state: &mut Self, update: Update) {}
  fn on_suspend(app: &App, state: &mut Self) {}
  fn on_resumed(app: &App, state: &mut Self) {}

  // Window Events
  fn on_window_moved(app: &App, state: &mut Self, key: Point2) {}
  fn on_key_pressed(app: &App, state: &mut Self, key: VirtualKeyCode) {}
  fn on_key_released(app: &App, state: &mut Self, key: VirtualKeyCode) {}
  fn on_mouse_moved(app: &App, state: &mut Self, position: Point2) {}
  fn on_mouse_pressed(app: &App, state: &mut Self, button: MouseButton) {}
  fn on_mouse_released(app: &App, state: &mut Self, button: MouseButton) {}
  fn on_mouse_entered(app: &App, state: &mut Self) {}
  fn on_mouse_exited(app: &App, state: &mut Self) {}
  fn on_mouse_wheal(app: &App, state: &mut Self, data: MouseScrollDelta, phase: TouchPhase) {}
  fn on_resize(app: &App, state: &mut Self, new_size: Vector2) {}
  fn on_hovered_file(app: &App, state: &mut Self, path: PathBuf) {}
  fn on_dropped_file(app: &App, state: &mut Self, path: PathBuf) {}
  fn on_hovered_file_cancelled(app: &App, state: &mut Self) {}
  fn on_touch(app: &App, state: &mut Self, event: TouchEvent) {}
  fn on_touch_pressure(app: &App, state: &mut Self, pressure: TouchpadPressure) {}
  fn on_focused(app: &App, state: &mut Self) {}
  fn on_unfocused(app: &App, state: &mut Self) {}
  fn on_closed(app: &App, state: &mut Self) {}

  fn view(app: &App, state: &Self, frame: Frame) {}
}

pub fn build_application_from<App : 'static + Application>() -> Builder<App, Event> {
  nannou::app(App::init)
    .event(App::on_event)
    .simple_window(App::view)
}

#[allow(dead_code)]
pub fn run_application<App : 'static + Application>() {
  nannou::app(App::init)
    .event(App::on_event)
    .simple_window(App::view)
    .run();
}