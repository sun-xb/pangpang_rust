

mod terminal;

use std::{sync::{Arc, RwLock}, collections::HashMap};

use eframe::{epi, egui::{self, FontDefinitions, FontFamily}};

struct PangPang {
    tx: pangpang::PpMsgSender,
    terminal_state: HashMap<String, Arc<RwLock<terminal::TermState>>>,
}

impl PangPang {
    pub fn new() -> Self {
        Self {
            tx: pangpang::run(),
            terminal_state: HashMap::new(),
        }
    }
}

impl epi::App for PangPang {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("New").clicked() {
                        self.tx.blocking_send(pangpang::PpMessage::Hello).expect("unable to say hello");
                    } else if ui.button("Open").clicked() {
                        let size = pangpang::SizeInfo::new(120.0, 30.0, 1.0, 1.0, 0., 0., false);
                        let (tx, rx) = pangpang::channel(1024);
                        let t = Arc::new(RwLock::new(terminal::TermState::new(tx, frame.repaint_signal().clone())));
                        self.terminal_state.insert("terminal_1".to_owned(), t.clone());
                        self.tx.blocking_send(pangpang::PpMessage::OpenTerminal(size, rx, t)).unwrap();
                    } else if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                egui::menu::menu(ui, "Help", |ui| {
                    if ui.button("About").clicked() {

                    }
                });
            });
        });
        egui::SidePanel::left("left_session_panel")
        .resizable(true).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| ui.heading("sessions"));
            ui.collapsing("local sessions", |ui| {
                ui.label("session 1");
                ui.label("session 2");
                ui.label("root@192.168.1.100:22");
                ui.label("rdp://223.234.123.111:3333");
            });
            ui.collapsing("cloud sessions", |ui| {
                ui.label("root@111.22.33.44:8022");
                ui.label("mysql://111.0.0.123:8000/db");
                ui.label("sqlite:///home/pangpang/pangpang.db");
            });
            ui.collapsing("remote server info", |ui| {
                ui.label("cpu usage");
                ui.label("memory info");
            });
            ui.collapsing("remote file manager", |ui| ui.label("..."));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(t) = self.terminal_state.get("terminal_1") {
                ui.add(terminal::TerminalView::new(t.clone()));
            }
        });
    }

    fn name(&self) -> &str {
        "pangpang app"
    }

    fn setup(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) {
        //for non-latin
        let name = "simfang";
        let mut fd = FontDefinitions::default();
        fd.font_data.insert(name.to_owned(), std::borrow::Cow::Borrowed(include_bytes!("../../simfang.ttf")));
        fd.fonts_for_family.get_mut(&FontFamily::Monospace).unwrap().push(name.to_owned());
        fd.fonts_for_family.get_mut(&FontFamily::Proportional).unwrap().push(name.to_owned());
        ctx.set_fonts(fd);
    }
}

fn main() {
    env_logger::init();
    let app = PangPang::new();
    let options = eframe::NativeOptions{
        transparent: true,
        resizable: true,
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
