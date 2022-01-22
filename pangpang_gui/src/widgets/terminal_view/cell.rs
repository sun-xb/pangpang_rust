use std::ops::Add;

use druid::{TextLayout, RenderContext, Data};


#[derive(Clone, Copy, PartialEq, Eq, Data)]
pub struct Cell {
    foreground: u32,
    background: u32,
    character: char,
    underscore: bool,
    style: druid::FontStyle,
    weight: druid::FontWeight,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            foreground: druid::Color::GREEN.as_rgba_u32(),
            background: druid::Color::TRANSPARENT.as_rgba_u32(),
            character: ' ',
            underscore: false,
            style: druid::FontStyle::Regular,
            weight: druid::FontWeight::NORMAL,
        }
    }
}

impl Cell {
    pub fn draw(&self, origin: druid::Point, ctx: &mut druid::PaintCtx, env: &druid::Env) {
        let mut layout: TextLayout<String> = TextLayout::new();
        layout.set_text(self.character.to_string());
        layout.set_text_color(druid::Color::Rgba32(self.foreground));
        layout.set_font(
            druid::FontDescriptor::new(druid::FontFamily::MONOSPACE)
                .with_size(10.0)
                .with_style(self.style)
                .with_weight(self.weight)
        );
        layout.rebuild_if_needed(ctx.text(), env);

        let brush = ctx.solid_brush(druid::Color::Rgba32(self.background));
        let layout_size = layout.size();
        ctx.fill(druid::Rect::from_origin_size(origin, layout_size), &brush);

        layout.draw(ctx, origin);
        if self.underscore {
            let shape = layout.underline_for_range(0..1);
            let brush = ctx.solid_brush(druid::Color::Rgba32(self.foreground));
            ctx.stroke(
                shape.add(origin.to_vec2()),
                &brush,
                layout_size.height / 10.0,
            );
        }
    }
}
