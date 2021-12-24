use std::sync::Arc;

use eframe::{egui::{text::LayoutJob, TextStyle, Color32, TextFormat, Stroke}, epi};
use pangpang::alacritty_terminal::{term::{TermMode, self}, ansi::{NamedColor, Color}};




#[derive(Clone)]
pub struct TerminalRender{
    is_visible: bool,
    mode: TermMode,
    layout: LayoutJob,
    cursor_pos: (usize, usize),
    display_offset: usize,
    repaint: Arc<dyn epi::RepaintSignal>,
}


impl TerminalRender {
    pub fn new(rs: Arc<dyn epi::RepaintSignal>) -> Self {
        Self {
            is_visible: true,
            mode: TermMode::empty(),
            layout: LayoutJob::default(),
            cursor_pos: (0, 0),
            display_offset: 0,
            repaint: rs,
        }
    }

    pub fn term_mode(&self) -> TermMode {
        self.mode
    }

    pub fn layout(&self) -> LayoutJob {
        self.layout.clone()
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        self.cursor_pos
    }

    pub fn display_offset(&self) -> usize {
        self.display_offset
    }
}

impl pangpang::terminal::Render for TerminalRender {
    fn draw(&mut self, render: pangpang::terminal::TerminalRender) {
        if !self.is_visible {
            return
        }
        self.mode = render.mode;
        self.cursor_pos = (render.cursor.point.column.0, render.cursor.point.line.0.try_into().unwrap());
        self.display_offset = render.display_offset;
        self.layout = LayoutJob::default();
        let mut first_char = true;
        for cell in render.display_iter {
            let mut fmt = TextFormat::default();
            fmt.style = TextStyle::Monospace;
            if cell.point.column == 0 && !first_char {
                self.layout.append("\n", 0.0, fmt);
            }
            first_char = false;
            if !cell.flags.contains(term::cell::Flags::WIDE_CHAR_SPACER)
                && !cell.flags.contains(term::cell::Flags::LEADING_WIDE_CHAR_SPACER)
            {
                fmt.italics = cell.flags.contains(term::cell::Flags::ITALIC)
                    || cell.flags.contains(term::cell::Flags::BOLD_ITALIC);
                fmt.color = color_to_color32(cell.fg);
                fmt.background = color_to_color32(cell.bg);
                if cell.flags.contains(term::cell::Flags::DOUBLE_UNDERLINE) {
                    fmt.strikethrough = Stroke::new(2.0, fmt.color);
                }
                if cell.flags.contains(term::cell::Flags::UNDERLINE) {
                    fmt.underline = Stroke::new(2.0, fmt.color);
                }

                if render.cursor.point == cell.point {
                    fmt.background = color_to_color32(Color::Named(NamedColor::Cursor));
                }

                if let Some(sr) = render.selection {
                    if sr.contains(cell.point) {
                        fmt.background = Color32::DARK_GRAY;
                    }
                }

                self.layout.append(cell.c.to_string().as_str(), 0.0, fmt);
            }
        }
        self.repaint.request_repaint();
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
