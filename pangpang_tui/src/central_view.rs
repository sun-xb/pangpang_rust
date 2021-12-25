

use tuirealm::{MockComponent, NoUserEvent, Component, Event, tui, event::{KeyEvent, Key, KeyModifiers}, command::{Cmd, Direction}};

use crate::{PpMessage, global_event_handle};










#[derive(MockComponent)]
pub(crate) struct CentralView {
    component: tui_realm_stdlib::Container
}

impl CentralView {
    pub(crate) fn new() -> Self {
        let component = tui_realm_stdlib::Container::default()
            .layout(
                tuirealm::props::Layout::default()
                    .constraints(&[tui::layout::Constraint::Length(3), tui::layout::Constraint::Percentage(100)])
                    .direction(tui::layout::Direction::Vertical)
                    .margin(1)
            )
            .borders(tuirealm::props::Borders{
                sides: tuirealm::props::BorderSides::empty(),
                modifiers: tuirealm::props::BorderType::Thick,
                color: tuirealm::props::Color::Green
            })
            .children(vec![
                Box::new(
                    tui_realm_stdlib::Radio::default()
                        .borders(tuirealm::props::Borders { 
                            sides: tuirealm::props::BorderSides::BOTTOM,
                            modifiers: tuirealm::props::BorderType::Thick,
                            color: tuirealm::props::Color::Green
                        })
                        .rewind(true)
                        .choices(&["tab 1", "tab-2", "aaaaaaaaaaaaaaaaa", "tab-2", "tab-2", "tab-2", "tab-2", "tab-2", "tab-2", "tab-2", "tab-2", "tab-2", "tab-2", "tab-2"])
                ),
                Box::new(
                    tui_realm_stdlib::Paragraph::default()
                        .text(&[tuirealm::props::TextSpan::new("aaaaaaaaaaaaaaaaaaaaaaaaaaaa")])
                )
            ]);
        Self {
            component
        }
    }
}

impl Component<PpMessage, NoUserEvent> for CentralView {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<PpMessage> {
        if let Some(msg) = global_event_handle(&ev) {
            return Some(msg)
        }
        match ev {
            Event::Keyboard(KeyEvent{code: Key::Left, modifiers: KeyModifiers::ALT}) => {
                if let Some(tab_bar) = self.component.children.get_mut(0) {
                    tab_bar.perform(Cmd::Scroll(Direction::Left));
                    tab_bar.perform(Cmd::Move(Direction::Left));
                }
            }
            Event::Keyboard(KeyEvent{code: Key::Right, modifiers: KeyModifiers::ALT}) => {
                if let Some(tab_bar) = self.component.children.get_mut(0) {
                    tab_bar.perform(Cmd::Scroll(Direction::Right));
                    tab_bar.perform(Cmd::Move(Direction::Right));
                }
            }
            _ => {}
        }
        None
    }
}