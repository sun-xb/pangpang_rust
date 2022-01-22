
use std::sync::Arc;

use druid::{AppLauncher, LocalizedString, WindowDesc, Data, Lens};

mod ui;
mod widgets;


#[derive(Data, Lens, Clone)]
struct PpState {
    terminal_grid: widgets::TerminalGrid,
    slider_test: f64,
    pp: Arc<pangpang::PangPang>
}

impl Default for PpState {
    fn default() -> Self {
        let config = Arc::new(pangpang::pangpang_run_sync::Mutex::new(pangpang::storage::MockStorage::new()));
        Self {
            terminal_grid: widgets::TerminalGrid::default(),
            slider_test: 0.5,
            pp: Arc::new(pangpang::PangPang::new(config))
        }
    }
}


pub fn main() {
    let window = WindowDesc::new(ui::main_window())
        .resizable(true)
        .title(LocalizedString::new("Fancy Colors"));

    AppLauncher::with_window(window)
        .log_to_console()
        .launch(PpState::default())
        .expect("launch failed");
}

