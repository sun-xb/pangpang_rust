use std::sync::{Arc, RwLock};

use pangpang::alacritty_terminal::{
    ansi::{Color, NamedColor},
    term,
};
use eframe::{egui::{self, text::LayoutJob, Color32, TextFormat, Stroke, TextStyle}, epi::RepaintSignal};



pub struct TermState {
    layout: LayoutJob,
    repaint: Arc<dyn RepaintSignal>,
    msg_sender: pangpang::PpTerminalMessageSender,
    size: pangpang::SizeInfo,
}

impl TermState {
    pub fn new(msg_sender: pangpang::PpTerminalMessageSender, repaint: Arc<dyn RepaintSignal>) -> Self {
        Self {
            layout: LayoutJob::default(),
            repaint,
            msg_sender,
            size: pangpang::SizeInfo::new(80.0, 20.0, 1.0, 1.0, 0.0, 0.0, false),
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
            const SYSTEM_COLOURS: [u32; 16] = [
                0x000000, 0xcd0000, 0x00cd00, 0xcdcd00, 0x0000ee, 0xcd00cd, 0x00cdcd, 0xe5e5e5, 0x7f7f7f,
                0xff0000, 0x00ff00, 0xffff00, 0x5c5cff, 0xff00ff, 0x00ffff, 0xffffff,
                ];
            const CUBE: [u32; 6] = [0, 95, 135, 175, 215, 255];
            let rgb = if i < 16 {
                SYSTEM_COLOURS[i as usize]
            } else if i < 232 {
                let offset = (i - 16) as usize;
                (CUBE[offset / 36] << 16) | (CUBE[offset / 6 % 6] << 8) | CUBE[offset % 6]
            } else {
                let offset = (i - 232) * 10 + 8;
                (offset as u32) * 0x010101
            };
            Color32::from_rgb(((rgb>>16)&0xff) as u8, ((rgb>>8)&0xff) as u8, (rgb&0xff) as u8)
        }
    }
}

#[pangpang::async_trait]
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
        self.repaint.request_repaint();
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

        let mut window_resized = false;
        {
            let state = self.s.read().unwrap();
            for e in &ui.input().events {
                match e {
                    egui::Event::Key{key, pressed, modifiers: _} => {
                        if egui::Key::Enter == *key && *pressed {
                            state.msg_sender.blocking_send(pangpang::PpTerminalMessage::Input(13u8)).unwrap();
                        }
                    }
                    egui::Event::Text(s) => {
                        for byte in s.bytes() {
                            state.msg_sender.blocking_send(pangpang::PpTerminalMessage::Input(byte)).unwrap();
                        }
                    }
                    _ => {}
                }
            }
            if rect.width() != state.size.width() || rect.height() != state.size.height() {
                window_resized = true;
            }
        }
        if window_resized {
            let mut state = self.s.write().unwrap();
            state.size = pangpang::SizeInfo::new(
                rect.width(),
                rect.height(),
                ui.fonts().glyph_width(TextStyle::Monospace, 'x'),
                ui.fonts().row_height(TextStyle::Monospace),
                0.0, 0.0, false
            );
            state.msg_sender.blocking_send(pangpang::PpTerminalMessage::ReSize(state.size)).unwrap();
        }

        response
    }
}
