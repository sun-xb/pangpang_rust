use std::sync::Arc;

use eframe::{egui, epi};
use pangpang::{alacritty_terminal::{ansi::C0, term}, pangpang_run_sync::Mutex};


mod terminal_render;

trait ModifiersNumeric {
    fn numeric(&self) -> u8;
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
    pub render_state: Arc<Mutex<terminal_render::TerminalRender>>,
    sender: pangpang::terminal::msg::PpTerminalMessageSender,
    window_size: egui::Vec2,
}


impl TerminalView {
    pub fn new(sender: pangpang::terminal::msg::PpTerminalMessageSender, rs: Arc<dyn epi::RepaintSignal>) -> Self {
        Self {
            render_state: Arc::new(Mutex::new(terminal_render::TerminalRender::new(rs))),
            sender,
            window_size: egui::vec2(0.0, 0.0),
        }
    }

    fn write_pty(&self, msg: pangpang::terminal::msg::PpTerminalMessage) {
        if let Err(_) = self.sender.blocking_send(msg) {
            println!("connection lost!");
        }
    }

    fn input_state(&self, input: &egui::InputState, mode: term::TermMode) {
        let mut input_sequence: Vec<u8> = Vec::new();
        let mut modifiers_state = egui::Modifiers::default();
        let mut cursor_mode = b'[';
        if mode.contains(term::TermMode::APP_CURSOR) {
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
            self.write_pty(pangpang::terminal::msg::PpTerminalMessage::Input(input_sequence))
        }
    }

}

impl egui::Widget for &mut TerminalView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let inner = ui.allocate_ui(ui.available_size(), |ui| {
            let state = self.render_state.blocking_lock();
            ui.painter().galley(ui.min_rect().min, ui.fonts().layout_job(state.layout()));
            let mode = state.term_mode();
            drop(state);
            self.input_state(ui.input(), mode);
            if self.window_size != ui.available_size() {
                self.window_size = ui.available_size();
                let cell_width = ui.fonts().glyph_width(egui::TextStyle::Monospace, 'x');
                let cell_height = ui.fonts().row_height(egui::TextStyle::Monospace);
                self.write_pty(pangpang::terminal::msg::PpTerminalMessage::ReSize(
                    (self.window_size.x / cell_width) as usize,
                    (self.window_size.y / cell_height) as usize
                ));
            }
        });
        ui.memory().request_focus(inner.response.id);
        inner.response
    }
}


