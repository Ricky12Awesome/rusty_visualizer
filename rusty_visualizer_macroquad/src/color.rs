use macroquad::color::Color;

pub trait AsColor {
  fn as_color(&self) -> Color;
}

pub trait GrayColor {
  fn gray_scale(n: u8) -> Color {
    Self::gray_scale_alpha(n, 255)
  }

  fn gray_scale_alpha(n: u8, a: u8) -> Color {
    Color::from_rgba(n, n, n, a)
  }
}

impl AsColor for [f32; 3] {
  fn as_color(&self) -> Color {
    Color::new(self[0], self[1], self[2], 1.0)
  }
}

impl AsColor for [f32; 4] {
  fn as_color(&self) -> Color {
    Color::new(self[0], self[1], self[2], self[3])
  }
}

impl AsColor for [u8; 3] {
  fn as_color(&self) -> Color {
    Color::from_rgba(self[0], self[1], self[2], 255)
  }
}

impl AsColor for [u8; 4] {
  fn as_color(&self) -> Color {
    Color::from_rgba(self[0], self[1], self[2], self[3])
  }
}

impl AsColor for egui::Color32 {
  fn as_color(&self) -> Color {
    Color::from_rgba(self.r(), self.g(), self.b(), self.a())
  }
}

impl GrayColor for Color {}