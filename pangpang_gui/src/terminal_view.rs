use std::sync::Arc;

use eframe::{egui::{self, Stroke, TextFormat, Color32, TextStyle, text::LayoutJob, InputState}, epi};
use pangpang::{alacritty_terminal::{ansi::{Color, NamedColor, C0}, term::{self, SizeInfo}, Term, grid::Dimensions}};

trait ModifiersNumeric {
    fn numeric(&self) -> u8;
}

pub struct CreateParameter {
    rx: pangpang::terminal::PpTerminalMessageReceiver,
    rs: Arc<dyn epi::RepaintSignal>,
    id: String,
}

impl CreateParameter {
    pub fn new(rx: pangpang::terminal::PpTerminalMessageReceiver, rs: Arc<dyn epi::RepaintSignal>, id: String) -> Self {
        Self {rx, rs, id}
    }
}
#[pangpang::async_trait]
impl pangpang::terminal::NewTerminalParameter for CreateParameter {
    fn request_repaint(&self) {
        self.rs.request_repaint();
    }
    async fn receive_msg(&mut self) -> Option<pangpang::terminal::PpTerminalMessage> {
        self.rx.recv().await
    }

    fn profile_id(&self) -> &String {
        &self.id
    }
}

impl ModifiersNumeric for egui::Modifiers {
    fn numeric(&self) -> u8 {
        let mut n = 0u8;
        if self.shift {
            n |= 1;
        }
        if self.alt {
            n |= 2;
        }
        if self.command {
            n |= 4;
        }
        //if self.metakey {
        //    n |= 8;
        //}
        n + b'1'
    }
}

pub struct TerminalView {
    terminal: Arc<pangpang::pangpang_run_sync::Mutex<Term<pangpang::terminal::TerminalEventListener>>>,
    term_mode: term::TermMode,
    window_size: egui::Vec2,
    galley: Arc<egui::Galley>,
    sender: pangpang::terminal::PpTerminalMessageSender,
}

impl TerminalView {
    pub fn new(ui: &mut egui::Ui, sender: pangpang::terminal::PpTerminalMessageSender) -> Self {
        let config = pangpang::alacritty_terminal::config::MockConfig::default();
        let size = SizeInfo::new(80.0, 20.0, 1.0, 1.0, 0.0, 0.0, false);
        let term = Term::new(&config, size, pangpang::terminal::TerminalEventListener);
        let term_mode = *term.mode();
        Self {
            terminal: Arc::new(pangpang::pangpang_run_sync::Mutex::new(term)),
            term_mode,
            window_size: egui::vec2(0.0, 0.0),
            galley: ui.fonts().layout_job(LayoutJob::default()),
            sender,
        }
    }

    pub fn get_terminal_handler(&self) -> Arc<pangpang::pangpang_run_sync::Mutex<Term<pangpang::terminal::TerminalEventListener>>> {
        self.terminal.clone()
    }

    fn write_pty(&self, msg: pangpang::terminal::PpTerminalMessage) {
        if let Err(_) = self.sender.blocking_send(msg) {
            println!("connection lost!");
        }
    }

    fn input_state(&self, input: &InputState) {
        let mut input_sequence: Vec<u8> = Vec::new();
        let mut modifiers_state = egui::Modifiers::default();
        let mut cursor_mode = b'[';
        if self.term_mode.contains(term::TermMode::APP_CURSOR) {
            cursor_mode = b'O';
        }
        for e in &input.events {
            match e {
                egui::Event::Key{key, pressed, modifiers} if *pressed => {
                    modifiers_state = *modifiers;
                    match *key {
                        egui::Key::ArrowUp => {
                            input_sequence.push(C0::ESC);
                            if modifiers.any() {
                                input_sequence.append(&mut b"[1;".to_vec());
                                input_sequence.push(modifiers.numeric());
                            }
                            input_sequence.push(cursor_mode);
                            input_sequence.push(b'A');
                        }
                        egui::Key::ArrowDown => {
                            input_sequence.push(C0::ESC);
                            if modifiers.any() {
                                input_sequence.append(&mut b"[1;".to_vec());
                                input_sequence.push(modifiers.numeric());
                            }
                            input_sequence.push(cursor_mode);
                            input_sequence.push(b'B');
                        }
                        egui::Key::ArrowRight => {
                            input_sequence.push(C0::ESC);
                            if modifiers.any() {
                                input_sequence.append(&mut b"[1;".to_vec());
                                input_sequence.push(modifiers.numeric());
                            }
                            input_sequence.push(cursor_mode);
                            input_sequence.push(b'C');
                        }
                        egui::Key::ArrowLeft => {
                            input_sequence.push(C0::ESC);
                            if modifiers.any() {
                                input_sequence.append(&mut b"[1;".to_vec());
                                input_sequence.push(modifiers.numeric());
                            }
                            input_sequence.push(cursor_mode);
                            input_sequence.push(b'D');
                        }
                        egui::Key::Backspace => {
                            if modifiers.shift {
                                input_sequence.push(C0::BS);
                            } else if modifiers.alt {
                                input_sequence.push(C0::ESC);
                                input_sequence.push(C0::DEL);
                            } else {
                                input_sequence.push(C0::DEL);
                            }
                        }
                        egui::Key::Tab => {
                            if modifiers.shift {
                                input_sequence.push(b'[');
                                input_sequence.push(b'Z');
                            } else {
                                input_sequence.push(C0::HT);
                            }
                        }
                        egui::Key::Enter => {
                            if modifiers.alt {
                                input_sequence.push(C0::ESC);
                            }
                            input_sequence.push(C0::CR);
                        }
                        egui::Key::Escape => {
                            if modifiers.alt {
                                input_sequence.push(C0::ESC);
                            }
                            input_sequence.push(C0::ESC);
                        }
                        egui::Key::Delete => {
                            input_sequence.push(C0::ESC);
                            input_sequence.push(b'[');
                            input_sequence.push(b'3');
                            if modifiers.any() {
                                input_sequence.push(b';');
                                input_sequence.push(modifiers.numeric());
                            }
                            input_sequence.push(b'~');
                        }
                        egui::Key::C => {
                            if modifiers.command {
                                input_sequence.push(b'C' - b'@');
                            }
                        }
                        egui::Key::Z => {
                            if modifiers.command {
                                input_sequence.push(b'Z' - b'@');
                            }
                        }
                        _ => {}
                    };
                }
                egui::Event::Text(text) => {
                    if modifiers_state.is_none() || modifiers_state.shift_only() {
                        for byte in text.bytes() {
                            input_sequence.push(byte);
                        }
                    }
                }
                _ => {}
            };
        }
        if !input_sequence.is_empty() {
            self.write_pty(pangpang::terminal::PpTerminalMessage::Input(input_sequence))
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let mut term = self.terminal.blocking_lock();
        self.term_mode = *term.mode();
        let content = term.renderable_content();
        let mut layout_job = LayoutJob::default();
        for cell in content.display_iter {
            let mut fmt = TextFormat::default();
            fmt.style = TextStyle::Monospace;
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

                if content.cursor.point == cell.point {
                    fmt.background = color_to_color32(Color::Named(NamedColor::Cursor));
                }

                layout_job.append(cell.c.to_string().as_str(), 0.0, fmt);
            }
            if cell.flags.contains(term::cell::Flags::WRAPLINE) ||
                cell.point.column >= (term.columns() - 1) {
                layout_job.append("\n", 0.0, fmt);
            }
        }
        if ui.available_size() != self.window_size {
            self.window_size = ui.available_size();
            let size = SizeInfo::new(self.window_size.x,
                self.window_size.y,
                ui.fonts().glyph_width(TextStyle::Monospace, 'x'),
                ui.fonts().row_height(TextStyle::Monospace), 0.0, 0.0, false
            );
            self.write_pty(pangpang::terminal::PpTerminalMessage::ReSize(
                (self.window_size.x / ui.fonts().glyph_width(TextStyle::Monospace, 'x')) as usize,
                (self.window_size.y / ui.fonts().row_height(TextStyle::Monospace)) as usize
            ));
            term.resize(size);
        }
        self.galley = ui.fonts().layout_job(layout_job);
    }
}

impl egui::Widget for &mut TerminalView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let inner = ui.allocate_ui(ui.available_size(), |ui| {
            self.draw(ui);
            ui.painter().galley(ui.min_rect().min, self.galley.clone());
            self.input_state(ui.input());
        });
        ui.memory().request_focus(inner.response.id);
        inner.response
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
