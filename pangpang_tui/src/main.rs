use tuirealm::{terminal::TerminalBridge, Update, NoUserEvent, Event, event::{KeyEvent, Key}};


mod model;
mod central_view;
mod left_panel;

#[derive(PartialEq, Eq, Debug)]
enum PpMessage {
    AppClose,
    ShowLeftPanel,
    OpenTerminal(String),
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
enum PangPangId {
    CentralViewId,
    LeftPanelId,
}


fn global_event_handle(ev: &tuirealm::Event<NoUserEvent>) -> Option<PpMessage> {
    match ev {
        Event::Keyboard(KeyEvent{code: Key::Function(1), ..}) => Some(PpMessage::ShowLeftPanel),
        Event::Keyboard(KeyEvent{code: Key::Function(12), ..}) => Some(PpMessage::AppClose),
        _ => None
    }
}


fn main() {
    let mut model = model::Model::default();
    let mut terminal = TerminalBridge::new().unwrap();
    terminal.enable_raw_mode().unwrap();
    terminal.enter_alternate_screen().unwrap();

    while !model.quit() {
        if let Ok(msg) = model.tick() {
            for m in msg.into_iter() {
                let mut m = Some(m);
                while m.is_some() {
                    m = model.update(m);
                }
            }
        }
        model.draw(&mut terminal);
    }

    terminal.leave_alternate_screen().unwrap();
    terminal.disable_raw_mode().unwrap();
    terminal.clear_screen().unwrap();
}
