use std::sync::Arc;

use pangpang::storage::Storage;
use tuirealm::{MockComponent, NoUserEvent, Component, Event, props::{Color, TableBuilder}, event::{KeyEvent, Key}, command::{Cmd, Direction}, State, StateValue};

use crate::{PpMessage, global_event_handle};







#[derive(MockComponent)]
pub(crate) struct ProfileListView {
    component: tui_realm_stdlib::List,
    _cfg: Arc<dyn pangpang::storage::Storage>,
}

impl ProfileListView {
    pub(crate) fn new() -> Self {
        let cfg = pangpang::storage::MockStorage::new();
        let cfg = Arc::new(cfg);
        let mut tb = TableBuilder::default();
        for (id, profile) in cfg.iter().enumerate() {
            if id > 0 {
                tb.add_row();
            }
            tb.add_col(tuirealm::props::TextSpan::from(profile.0).fg(Color::Cyan));
        }

        let component = tui_realm_stdlib::List::default()
            .title("profiles", tuirealm::props::Alignment::Left)
            .highlighted_color(Color::LightYellow)
            .highlighted_str("🚀")
            .rewind(true)
            .step(4)
            .scroll(true)
            .rows(tb.build());

        Self {
            component,
            _cfg: cfg,
        }
    }

}

impl Component<PpMessage, NoUserEvent> for ProfileListView {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<PpMessage> {
        if let Some(msg) = global_event_handle(&ev) {
            return Some(msg)
        }
        match ev {
            Event::Keyboard(KeyEvent{code: Key::Down, ..}) => {
                self.perform(Cmd::Move(Direction::Down));
            }
            Event::Keyboard(KeyEvent{code: Key::Up, ..}) => {
                self.perform(Cmd::Move(Direction::Up));
            }
            Event::Keyboard(KeyEvent{code: Key::Enter, ..}) |
            Event::Keyboard(KeyEvent{code: Key::Right, ..}) => {
                if let State::One(StateValue::Usize(i)) = self.state() {
                    if let Some(tuirealm::props::AttrValue::Table(rows)) = self.query(tuirealm::props::Attribute::Content) {
                        let content = &rows.get(i).unwrap().get(0).unwrap().content;
                        return Some(PpMessage::OpenTerminal(content.clone()));
                    }
                }
            }
            _ => {}
        }
        None
    }
}
