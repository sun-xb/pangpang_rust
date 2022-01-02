use std::sync::Arc;

use eframe::{egui, epi};
use pangpang::{
    alacritty_terminal::{ansi::C0, term},
    pangpang_run_sync::Mutex,
};

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
    mouse_primary_key_down: bool,
}

impl TerminalView {
    pub fn new(
        sender: pangpang::terminal::msg::PpTerminalMessageSender,
        frame: epi::Frame,
    ) -> Self {
        Self {
            render_state: Arc::new(Mutex::new(terminal_render::TerminalRender::new(frame))),
            sender,
            window_size: egui::vec2(0.0, 0.0),
            mouse_primary_key_down: false,
        }
    }

    fn write_pty(&self, msg: pangpang::terminal::msg::PpTerminalMessage) {
        if let Err(_) = self.sender.blocking_send(msg) {
            println!("connection lost!");
        }
    }

    fn input_state(
        &mut self,
        ui: &egui::Ui,
        state: terminal_render::TerminalRender,
        galley: Arc<egui::Galley>,
    ) {
        let mut input_sequence: Vec<u8> = Vec::new();
        let mut modifiers_state = egui::Modifiers::default();
        let mut cursor_mode = b'[';
        if state.term_mode().contains(term::TermMode::APP_CURSOR) {
            cursor_mode = b'O';
        }
        for e in &ui.input().events {
            match e {
                egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                } if *pressed => {
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
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed: true,
                    modifiers: _,
                } => {
                    let cursor = galley
                        .cursor_from_pos(pos.to_vec2() - ui.min_rect().min.to_vec2())
                        .pcursor;
                    self.write_pty(pangpang::terminal::msg::PpTerminalMessage::Copy(
                        cursor.paragraph.try_into().unwrap(),
                        cursor.offset,
                    ));
                }
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: _,
                } => {
                    self.mouse_primary_key_down = *pressed;
                    if self.mouse_primary_key_down {
                        let cursor = galley
                            .cursor_from_pos(pos.to_vec2() - ui.min_rect().min.to_vec2())
                            .pcursor;
                        let paragraph: i32 = cursor.paragraph.try_into().unwrap();
                        let scroll: i32 = state.display_offset().try_into().unwrap();
                        let line = paragraph - scroll;
                        self.write_pty(pangpang::terminal::msg::PpTerminalMessage::SelectionStart(
                            line,
                            cursor.offset,
                        ));
                    }
                }
                egui::Event::PointerMoved(pos) => {
                    if self.mouse_primary_key_down {
                        let cursor = galley
                            .cursor_from_pos(pos.to_vec2() - ui.min_rect().min.to_vec2())
                            .pcursor;
                        let paragraph: i32 = cursor.paragraph.try_into().unwrap();
                        let scroll: i32 = state.display_offset().try_into().unwrap();
                        let line = paragraph - scroll;
                        self.write_pty(
                            pangpang::terminal::msg::PpTerminalMessage::SelectionUpdate(
                                line,
                                cursor.offset,
                            ),
                        );
                    }
                }
                _ => {}
            };
        }
        if !input_sequence.is_empty() {
            self.write_pty(pangpang::terminal::msg::PpTerminalMessage::Input(
                input_sequence,
            ));
        }
        let scroll_delta = ui.input().scroll_delta.y;
        if scroll_delta != 0.0 {
            if ui.rect_contains_pointer(ui.max_rect()) {
                if let Some(pos) = ui.input().pointer.hover_pos() {
                    if self.mouse_primary_key_down {
                        let cursor = galley
                            .cursor_from_pos(pos.to_vec2() - ui.min_rect().min.to_vec2())
                            .pcursor;
                        let paragraph: i32 = cursor.paragraph.try_into().unwrap();
                        let scroll: i32 = state.display_offset().try_into().unwrap();
                        let line = paragraph - scroll;
                        self.write_pty(
                            pangpang::terminal::msg::PpTerminalMessage::SelectionUpdate(
                                line,
                                cursor.offset,
                            ),
                        );
                    }
                }

                self.write_pty(pangpang::terminal::msg::PpTerminalMessage::Scroll(
                    (scroll_delta / ui.fonts().row_height(egui::TextStyle::Monospace)) as i32,
                ));
            }
        }
    }

    fn add_contents(&mut self, ui: &mut egui::Ui, response: &egui::Response) {
        let state = self.render_state.blocking_lock().clone();

        let where_to_put_background = ui.painter().add(egui::Shape::Noop);

        let terminal_pos = response.rect.left_top();
        let galley = ui.fonts().layout_job(state.layout());
        let cursor_pos = galley
            .pos_from_pcursor(egui::epaint::text::cursor::PCursor {
                paragraph: state.cursor_pos().1,
                offset: state.cursor_pos().0,
                prefer_next_row: false,
            })
            .translate(terminal_pos.to_vec2())
            .left_top();
        ui.output().text_cursor_pos = Some(cursor_pos);
        ui.painter().galley(terminal_pos, galley.clone());
        let visuals = ui.style().interact(&response);
        let shape = if response.has_focus() {
            self.input_state(ui, state, galley);
            egui::epaint::RectShape {
                rect: response.rect,
                corner_radius: visuals.corner_radius,
                fill: egui::Color32::TRANSPARENT,
                stroke: ui.visuals().selection.stroke,
            }
        } else {
            egui::epaint::RectShape {
                rect: response.rect,
                corner_radius: visuals.corner_radius,
                fill: egui::Color32::TRANSPARENT,
                stroke: visuals.bg_stroke,
            }
        };
        ui.painter().set(where_to_put_background, shape);
        if self.window_size != response.rect.size() {
            self.window_size = response.rect.size();
            let cell_width = ui.fonts().glyph_width(egui::TextStyle::Monospace, 'x');
            let cell_height = ui.fonts().row_height(egui::TextStyle::Monospace);
            self.write_pty(pangpang::terminal::msg::PpTerminalMessage::ReSize(
                (self.window_size.x / cell_width) as usize,
                (self.window_size.y / cell_height) as usize,
            ));
        }
    }
}

impl egui::Widget for &mut TerminalView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (id, rect) = ui.allocate_space(ui.available_size());
        let mut child_ui = ui.child_ui(rect, *ui.layout());
        let response = child_ui.interact(rect, id, egui::Sense::click_and_drag());
        self.add_contents(&mut child_ui, &response);
        if response.clicked() && !response.lost_focus() {
            response.request_focus();
        }
        response
    }
}
