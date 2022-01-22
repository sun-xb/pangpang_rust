

use druid::{widget::prelude::*, im::Vector};

mod cell;

pub type TerminalGrid = Vector<Vector<cell::Cell>>;
pub struct TerminalView {
    font: druid::FontDescriptor,
}

impl Default for TerminalView {
    fn default() -> Self {
        Self {
            font: druid::FontDescriptor::default(),
        }
    }
}

impl Widget<TerminalGrid> for TerminalView {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut TerminalGrid, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &TerminalGrid, _env: &Env) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &TerminalGrid, _data: &TerminalGrid, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &TerminalGrid, _env: &Env) -> Size {
        /*let font_size = self.font_size();
        let size = bc.max();

        self.cells.resize((size.height / font_size.height) as usize, Vec::new());
        let width = size.width / font_size.width;
        for line in &mut self.cells {
            line.resize(width as usize, cell::Cell::default());
        }
        size*/
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, grid: &TerminalGrid, env: &Env) {
        let font_size = self.font_size();
        let mut rect = ctx.region().bounding_box();

        while rect.area() > 0.0 {
            let mut line_region = rect.clone();
            line_region.y1 = line_region.y0 + font_size.height;
            rect.y0 = line_region.y1;

            while line_region.area() > 0.0 {
                if let Some(line) = grid.get ((line_region.y0 / font_size.height) as usize) {
                    if let Some(cell) = line.get((line_region.x0 / font_size.width) as usize) {
                        cell.draw(line_region.origin(), ctx, env);
                    }
                }
                line_region.x0 += font_size.width;
            }
        }
    }
}

impl TerminalView {
    fn font_size(&self) -> druid::Size {
        druid::Size::new(self.font.size/2.0, self.font.size)
    }
}

