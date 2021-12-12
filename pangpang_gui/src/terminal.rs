use std::sync::Arc;

use pangpang::alacritty_terminal::{
    ansi::{Color, NamedColor, Processor},
    config::MockConfig,
    event::EventListener,
    grid::Dimensions,
    sync::FairMutex,
    term::{self, SizeInfo},
    Term,
};
use eframe::egui::{self, text::LayoutJob, Color32, TextFormat, Stroke, TextStyle};

struct Listener;
impl EventListener for Listener {}

pub struct TerminalView {
    term: Arc<FairMutex<Term<Listener>>>,
}

impl TerminalView {
    pub fn new() -> Self {
        let config = Arc::new(MockConfig::default());
        let size_info = SizeInfo::new(120.0, 30.0, 1.0, 1.0, 0., 0., false);
        let term = Arc::new(FairMutex::new(Term::new(&config, size_info, Listener)));
        let mut processor = Processor::default();
        for c in b"Hello from \x1B[1;3;31mpangpang terminal\x1B[0m $ " {
            processor.advance(&mut *term.lock(), *c);
        }
        for c in "中文中文\r\n1234567890\r\n一二三四五\r\n\r\nvery loooooooooooooooooooooooooooooooooooooooooong".bytes() {
            processor.advance(&mut *term.lock(), c);
        }
        Self { term }
    }

    fn color_to_color32(&self, c: Color) -> Color32 {
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
}

impl egui::Widget for TerminalView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_at_least(ui.available_size(), egui::Sense::click_and_drag());
        
        let cell_width = ui.fonts().glyph_width(TextStyle::Monospace, 'z');
        let cell_height = ui.fonts().row_height(TextStyle::Monospace);
        let size_info = SizeInfo::new(rect.width(), rect.height(), cell_width, cell_height, 0., 0., false);

        let mut job = LayoutJob::default();
        let mut term = self.term.lock();
        if term.screen_lines() != size_info.screen_lines() || term.columns() != size_info.columns() {
            term.resize(size_info);
        }
        let content = term.renderable_content();
        for c in content.display_iter {
            let mut fmt = TextFormat::default();
            fmt.style = TextStyle::Monospace;
            if !c.flags.contains(term::cell::Flags::WIDE_CHAR_SPACER) &&
                !c.flags.contains(term::cell::Flags::LEADING_WIDE_CHAR_SPACER){
                fmt.italics = c.flags.contains(term::cell::Flags::ITALIC)
                    || c.flags.contains(term::cell::Flags::BOLD_ITALIC);
                fmt.color = self.color_to_color32(c.fg);
                fmt.background = self.color_to_color32(c.bg);
                if c.flags.contains(term::cell::Flags::DOUBLE_UNDERLINE) {
                    fmt.strikethrough = Stroke::new(2.0, fmt.color);
                }
                if c.flags.contains(term::cell::Flags::UNDERLINE) {
                    fmt.underline = Stroke::new(2.0, fmt.color);
                }

                if content.cursor.point == c.point {
                    fmt.background = self.color_to_color32(Color::Named(NamedColor::Cursor));
                }

                job.append(c.c.to_string().as_str(), 0.0, fmt);
            }
            if c.flags.contains(term::cell::Flags::WRAPLINE) ||
                c.point.column >= (term.columns() - 1) {
                job.append("\n", 0.0, fmt);
            }
        }
        let galley = ui.fonts().layout_job(job);
        //let pos = galley.pos_from_pcursor(egui::epaint::text::cursor::PCursor { paragraph: 3, offset: 2, prefer_next_row: false });
        //ui.ctx().output().text_cursor_pos = Some(pos.min);
        ui.painter().galley(rect.min, galley);
        response
    }
}
