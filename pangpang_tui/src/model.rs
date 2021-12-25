use tuirealm::{Application, NoUserEvent, EventListenerCfg, terminal::TerminalBridge, tui, Update, application::ApplicationResult, PollStrategy};

use crate::{PangPangId, PpMessage, left_panel::ProfileListView, central_view::CentralView};




pub(crate) struct Model {
    app: Application<PangPangId, PpMessage, NoUserEvent>,
    quit: bool,
    constraints: Vec<tui::layout::Constraint>,
    constraint_ids: Vec<PangPangId>,
}

impl Default for Model {
    fn default() -> Self {
        let mut app = Application::init(EventListenerCfg::default().default_input_listener(std::time::Duration::from_millis(10)));
        app.mount(PangPangId::CentralViewId, Box::new(CentralView::new()), vec![]).unwrap();
        app.mount(PangPangId::LeftPanelId, Box::new(ProfileListView::new()), vec![]).unwrap();
        app.active(&PangPangId::CentralViewId).unwrap();
        Self {
            app,
            quit: false,
            constraints: vec![tui::layout::Constraint::Percentage(0), tui::layout::Constraint::Percentage(100)],
            constraint_ids: vec![PangPangId::LeftPanelId, PangPangId::CentralViewId],
        }
    }
}

impl Update<PpMessage> for Model {
    fn update(&mut self, msg: Option<PpMessage>) -> Option<PpMessage> {
        match msg {
            Some(msg) if msg == PpMessage::AppClose => {
                self.quit = true;
            }
            Some(msg) if msg == PpMessage::ShowLeftPanel => {
                if let Some(c) = self.constraints.get_mut(0) {
                    if let tui::layout::Constraint::Percentage(percentage) = c {
                        if *percentage == 0 {
                            *percentage = 20;
                            self.app.active(&PangPangId::LeftPanelId).unwrap();
                        } else {
                            *percentage = 0;
                            self.app.active(&PangPangId::CentralViewId).unwrap();
                        }
                    }
                }
            }
            _ => {}
        };
        None
    }
}
impl Model {
    pub(crate) fn draw(&mut self, terminal: &mut TerminalBridge) {
        terminal.raw_mut().draw(|f| {
            let chunks = tui::layout::Layout::default()
                .direction(tui::layout::Direction::Horizontal)
                .constraints(self.constraints.as_ref())
                .split(f.size());
            for (idx, id) in self.constraint_ids.iter().enumerate() {
                self.app.view(id, f, chunks[idx]);
            }
        }).unwrap();
    }

    pub(crate) fn quit(&self) -> bool {
        self.quit
    }


    pub(crate) fn tick(&mut self) -> ApplicationResult<Vec<PpMessage>> {
        self.app.tick(PollStrategy::Once)
    }
}
