use std::path::PathBuf;

use nannou::App;
use nannou::prelude::*;
use nannou::winit::event::{DeviceEvent, VirtualKeyCode};
use nannou::app::Builder;


#[allow(unused_variables)]
pub trait ApplicationDelegate {
  fn on_window_event(&mut self, app: &App, event: WindowEvent) {
    match event {
      Moved(position) => self.on_window_moved(app, position),
      KeyPressed(key) => self.on_key_pressed(app, key),
      KeyReleased(key) => self.on_key_released(app, key),
      MouseMoved(position) => self.on_mouse_moved(app, position),
      MousePressed(button) => self.on_mouse_pressed(app, button),
      MouseReleased(button) => self.on_mouse_released(app, button),
      MouseEntered => self.on_mouse_entered(app),
      MouseExited => self.on_mouse_exited(app),
      MouseWheel(data, phase) => self.on_mouse_wheal(app, data, phase),
      Resized(new_size) => self.on_resize(app, new_size),
      HoveredFile(event) => self.on_hovered_file(app, event),
      DroppedFile(path) => self.on_dropped_file(app, path),
      HoveredFileCancelled => self.on_hovered_file_cancelled(app),
      Touch(event) => self.on_touch(app, event),
      TouchPressure(pressure) => self.on_touch_pressure(app, pressure),
      Focused => self.on_focused(app),
      Unfocused => self.on_unfocused(app),
      Closed => self.on_closed(app),
    }
  }

  fn on_device_event(&mut self, app: &App, event: DeviceEvent) {}

  fn on_update(&mut self, app: &App, update: Update) {}
  fn on_suspend(&mut self, app: &App) {}
  fn on_resumed(&mut self, app: &App) {}

  fn on_window_moved(&mut self, app: &App, key: Point2) {}
  fn on_key_pressed(&mut self, app: &App, key: VirtualKeyCode) {}
  fn on_key_released(&mut self, app: &App, key: VirtualKeyCode) {}
  fn on_mouse_moved(&mut self, app: &App, position: Point2) {}
  fn on_mouse_pressed(&mut self, app: &App, button: MouseButton) {}
  fn on_mouse_released(&mut self, app: &App, button: MouseButton) {}
  fn on_mouse_entered(&mut self, app: &App) {}
  fn on_mouse_exited(&mut self, app: &App) {}
  fn on_mouse_wheal(&mut self, app: &App, data: MouseScrollDelta, phase: TouchPhase) {}
  fn on_resize(&mut self, app: &App, new_size: Vector2) {}
  fn on_hovered_file(&mut self, app: &App, path: PathBuf) {}
  fn on_dropped_file(&mut self, app: &App, path: PathBuf) {}
  fn on_hovered_file_cancelled(&mut self, app: &App) {}
  fn on_touch(&mut self, app: &App, event: TouchEvent) {}
  fn on_touch_pressure(&mut self, app: &App, pressure: TouchpadPressure) {}
  fn on_focused(&mut self, app: &App) {}
  fn on_unfocused(&mut self, app: &App) {}
  fn on_closed(&mut self, app: &App) {}

  fn view(&self, app: &App, frame: Frame) {}
}

pub trait Application: ApplicationDelegate {
  fn init(app: &App) -> Self;
  fn get_delegate(&self) -> Option<&'static mut dyn ApplicationDelegate> { None }

  fn on_event(app: &App, state: &mut Self, event: Event) {
    let delegate = state.get_delegate();

    match event {
      Event::WindowEvent { id: _, simple: None, .. } => {}
      Event::WindowEvent { id: _id, simple: Some(event), .. } => {
        state.on_window_event(app, event.clone());
        if let Some(delegate) = delegate {
          delegate.on_window_event(app, event);
        }
      }
      Event::DeviceEvent(_, event) => {
        state.on_device_event(app, event.clone());
        if let Some(delegate) = delegate {
          delegate.on_device_event(app, event);
        }
      }
      Event::Update(update) => {
        state.on_update(app, update.clone());
        if let Some(delegate) = delegate {
          delegate.on_update(app, update);
        }
      }
      Event::Suspended => {
        state.on_suspend(app);
        if let Some(delegate) = delegate {
          delegate.on_suspend(app);
        }
      }
      Event::Resumed => {
        state.on_resumed(app);
        if let Some(delegate) = delegate {
          delegate.on_resumed(app);
        }
      }
    }
  }

  fn on_view(app: &App, state: &Self, frame: Frame) {
    ApplicationDelegate::view(state, app, frame)
  }
}

pub fn build_application_from<App: 'static + Application>() -> Builder<App, Event> {
  nannou::app(App::init)
    .event(App::on_event)
    .simple_window(App::on_view)
}

#[allow(dead_code)]
pub fn run_application<App: 'static + Application>() {
  build_application_from::<App>().run()
}