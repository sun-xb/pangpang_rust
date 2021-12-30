


mod terminal_view;
mod tab_view;


use std::sync::Arc;

use eframe::{epi, egui};

struct PangPang {
    ts: f64,
    cfg: Arc<pangpang::pangpang_run_sync::Mutex<dyn pangpang::storage::Storage>>,
    pp_sender: pangpang::pangpang_run_sync::PpMsgSender,
    tab_view: tab_view::TabView,
}

impl PangPang {
    pub fn new() -> Self {
        let cfg = pangpang::storage::MockStorage::new();
        let cfg = Arc::new(pangpang::pangpang_run_sync::Mutex::new(cfg));
        Self {
            ts: 0.0,
            cfg: cfg.clone(),
            pp_sender: pangpang::pangpang_run_sync::run(cfg),
            tab_view: tab_view::TabView::new(),
        }
    }

    fn fps_control(&mut self, ctx: &egui::CtxRef) {
        let fps = 60.0;
        let ts = (1.0/fps) - (ctx.input().time - self.ts);
        self.ts = ctx.input().time;
        if ts > 0.0  {
            self.ts += ts;
            std::thread::sleep(std::time::Duration::from_millis((ts * 1000.0) as u64));
        }
    }

    fn open_terminal(&mut self, id: String, title: String, frame: epi::Frame) {
        let (tx, rx) = pangpang::terminal::channel(1024);
        let view = terminal_view::TerminalView::new(tx, frame);
        self.pp_sender.blocking_send(pangpang::pangpang_run_sync::PpMessage::NewTerminal(id, rx, view.render_state.clone())).unwrap();
        self.tab_view.insert(title, view);
    }
}

impl epi::App for PangPang {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        self.fps_control(ctx);
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);
            egui::menu::bar(ui, |ui| {
                egui::menu::menu_button(ui, "File", |ui| {
                    if ui.button("New").clicked() {
                        self.pp_sender.blocking_send(pangpang::pangpang_run_sync::PpMessage::Hello).expect("unable to say hello");
                    } else if ui.button("Open").clicked() {
                        
                    } else if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                egui::menu::menu_button(ui, "Help", |ui| {
                    if ui.button("About").clicked() {
                        
                    }
                });
            });
        });
        egui::SidePanel::left("left_session_panel")
        .resizable(true).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| ui.heading("sessions"));
            ui.collapsing("sessions", |ui| {
                let cfg = self.cfg.clone();
                for (_, profile) in cfg.blocking_lock().iter() {
                    let btn = egui::Button::new(profile.id())
                        .frame(false)
                        .wrap(false);
                    if ui.add(btn).clicked() {
                        self.open_terminal(profile.id(), profile.address.clone(), frame.clone());
                    }
                }
            });
            ui.collapsing("remote server info", |ui| {
                ui.label("cpu usage");
                ui.label("memory info");
            });
            ui.collapsing("remote file manager", |ui| ui.label("..."));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.tab_view);
        });
    }

    fn name(&self) -> &str {
        "pangpang app"
    }

    fn setup(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
        //for non-latin
        let name = "simfang";
        let mut fd = egui::FontDefinitions::default();
        fd.font_data.insert(name.to_owned(), egui::FontData::from_static(include_bytes!("../../fonts/simfang.ttf")));
        fd.fonts_for_family.get_mut(&egui::FontFamily::Monospace).unwrap().push(name.to_owned());
        fd.fonts_for_family.get_mut(&egui::FontFamily::Proportional).unwrap().push(name.to_owned());
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
        initial_window_size: Some(egui::vec2(1600.0, 1000.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
