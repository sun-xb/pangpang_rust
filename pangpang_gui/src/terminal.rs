use std::sync::{Arc, RwLock};

use pangpang::alacritty_terminal::{
    ansi::{Color, NamedColor},
    term,
};
use eframe::egui::{self, text::LayoutJob, Color32, TextFormat, Stroke, TextStyle};


#[derive(Debug)]
pub struct TermState {
    layout: LayoutJob,
}
impl TermState {
    pub fn new() -> Self {
        Self {
            layout: LayoutJob::default(),
        }
    }
}

fn color_to_color32(c: Color) -> Color32 {
    match c {
        Color::Spec(rgb) => Color32::from_rgb(rgb.r, rgb.g, rgb.b),
        Color::Named(c) => {
            match c {
                NamedColor::Black => Color32::BLACK,
                NamedColor::Red => Color32::RED,
                NamedColor::Green => Color32::GREEN,
                NamedColor::Yellow => Color32::YELLOW,
                NamedColor::Blue => Color32::BLUE,
                NamedColor::Magenta => Color32::from_rgb(228, 0, 127),
                NamedColor::Cyan => Color32::from_rgb(0, 255, 255),
                NamedColor::White => Color32::WHITE,
                NamedColor::BrightBlack => Color32::BLACK, // ???
                NamedColor::BrightRed => Color32::LIGHT_RED,
                NamedColor::BrightGreen => Color32::LIGHT_GREEN,
                NamedColor::BrightYellow => Color32::LIGHT_YELLOW,
                NamedColor::BrightBlue => Color32::LIGHT_BLUE,
                NamedColor::BrightMagenta => Color32::from_rgb(228, 0, 127), //???
                NamedColor::BrightCyan => Color32::from_rgb(224, 255, 255),
                NamedColor::BrightWhite => Color32::WHITE, // ???
                NamedColor::Foreground => Color32::GOLD,
                NamedColor::Background => Color32::TRANSPARENT,
                NamedColor::Cursor => Color32::GRAY,
                NamedColor::DimBlack => Color32::BLACK, // ???
                NamedColor::DimRed => Color32::DARK_RED,
                NamedColor::DimGreen => Color32::DARK_GREEN,
                NamedColor::DimYellow => Color32::from_rgb(173, 255, 47), // ???
                NamedColor::DimBlue => Color32::DARK_BLUE,
                NamedColor::DimMagenta => Color32::from_rgb(139, 0, 139),
                NamedColor::DimCyan => Color32::from_rgb(0, 139, 139),
                NamedColor::DimWhite => Color32::WHITE, // ??/
                NamedColor::BrightForeground => Color32::LIGHT_YELLOW, // ???
                NamedColor::DimForeground => Color32::GRAY,
            }
        }
        Color::Indexed(i) => {
            println!("indexed color: {}", i);
            Color32::BLACK
        }
    }
}

impl pangpang::PpTermianlRender for TermState {
    fn render(&mut self, r: pangpang::RenderableContent, col: usize) {
        self.layout = LayoutJob::default();
        for c in r.display_iter {
            let mut fmt = TextFormat::default();
            fmt.style = TextStyle::Monospace;
            if !c.flags.contains(term::cell::Flags::WIDE_CHAR_SPACER) &&
                !c.flags.contains(term::cell::Flags::LEADING_WIDE_CHAR_SPACER){
                fmt.italics = c.flags.contains(term::cell::Flags::ITALIC)
                    || c.flags.contains(term::cell::Flags::BOLD_ITALIC);
                fmt.color = color_to_color32(c.fg);
                fmt.background = color_to_color32(c.bg);
                if c.flags.contains(term::cell::Flags::DOUBLE_UNDERLINE) {
                    fmt.strikethrough = Stroke::new(2.0, fmt.color);
                }
                if c.flags.contains(term::cell::Flags::UNDERLINE) {
                    fmt.underline = Stroke::new(2.0, fmt.color);
                }

                if r.cursor.point == c.point {
                    fmt.background = color_to_color32(Color::Named(NamedColor::Cursor));
                }

                self.layout.append(c.c.to_string().as_str(), 0.0, fmt);
            }
            if c.flags.contains(term::cell::Flags::WRAPLINE) ||
                c.point.column >= (col - 1) {
                self.layout.append("\n", 0.0, fmt);
            }
        }
    }
}
pub struct TerminalView {
    s: Arc<RwLock<TermState>>,
}

impl TerminalView {
    pub fn new(s: Arc<RwLock<TermState>>) -> Self {
        Self { s }
    }
}

impl egui::Widget for TerminalView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_at_least(ui.available_size(), egui::Sense::click_and_drag());
        
        
        let job = self.s.read().unwrap().layout.clone();
        let galley = ui.fonts().layout_job(job);
        //let pos = galley.pos_from_pcursor(egui::epaint::text::cursor::PCursor { paragraph: 3, offset: 2, prefer_next_row: false });
        //ui.ctx().output().text_cursor_pos = Some(pos.min);
        ui.painter().galley(rect.min, galley);
        response
    }
}
