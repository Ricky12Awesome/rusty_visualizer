use std::borrow::Cow;

use egui::{Align2, CtxRef, FontDefinitions, FontFamily, Galley, LayerId, Order, pos2, Rect, Rgba, SidePanel, TextStyle, Ui};
use egui::text::Fonts;
use macroquad::color::Color;

use crate::{NOTO_SANS, NOTO_SANS_JP};

pub fn egui_draw_text(ctx: &CtxRef, text: impl ToString, x: f32, y: f32, _size: u32, color: Color) {
  lazy_static::lazy_static!(
    static ref FONT: Fonts = Fonts::new(1.0, font_def(22f32, 48f32));
  );

  let font = &FONT as &Fonts;
  let painter = ctx.layer_painter(LayerId::background());
  let color = Rgba::from_rgba_premultiplied(color.r, color.g, color.b, color.a).into();
  let galley = font.layout_no_wrap(text.to_string(), TextStyle::Heading, color);
  let anchor = Align2::LEFT_TOP;
  let rect = anchor.anchor_rect(Rect::from_min_size(pos2(x, y), galley.size()));

  painter.galley(rect.min, galley);
}

pub fn font_def(size: f32, heading: f32) -> FontDefinitions {
  let mut fonts = FontDefinitions::default();

  fonts.font_data.insert("NotoSans-Regular".to_string(), Cow::Borrowed(NOTO_SANS));
  fonts.font_data.insert("NotoSansJP-Regular".to_string(), Cow::Borrowed(NOTO_SANS_JP));

  let fonts_list = fonts.fonts_for_family.get_mut(&FontFamily::Proportional).unwrap();

  fonts_list.clear();
  fonts_list.push("NotoSans-Regular".to_owned());
  fonts_list.push("NotoSansJP-Regular".to_owned());

  let family = &mut fonts.family_and_size;

  family.insert(egui::TextStyle::Small, (FontFamily::Proportional, size));
  family.insert(egui::TextStyle::Body, (FontFamily::Proportional, size));
  family.insert(egui::TextStyle::Button, (FontFamily::Proportional, size));
  family.insert(egui::TextStyle::Heading, (FontFamily::Proportional, heading));
  family.insert(egui::TextStyle::Monospace, (FontFamily::Proportional, size));

  fonts
}