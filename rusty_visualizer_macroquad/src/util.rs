use std::hash::Hash;
use egui::{Align2, Context, CtxRef, Id, LayerId, Order, pos2, Rgba, SidePanel, TextStyle, Ui};
use macroquad::color::Color;

pub fn better_draw_text(ctx: &CtxRef, id: impl Hash, text: impl ToString, x: f32, y: f32, size: u32, color: Color) {
  let painter = ctx.layer_painter(LayerId::new(egui::Order::Background, Id::new("Text")));
  
  painter.text(
    pos2(x, y),
    Align2::LEFT_TOP,
    text,
    TextStyle::Heading,
    Rgba::from_rgba_premultiplied(color.r, color.g, color.b, color.a).into(),
  );
}