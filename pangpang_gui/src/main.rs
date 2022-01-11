
use iced::Application;

mod terminal_view;

fn main() {
    PpMainWindow::run(iced::Settings {
        antialiasing: false,
        ..iced::Settings::default()
    }).unwrap();
}

struct PpMainWindow;

#[derive(Debug, Clone)]
enum PpMessage {

}

impl Application for PpMainWindow {
    type Executor = iced::executor::Default;

    type Message = PpMessage;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (PpMainWindow, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("PangPang App")
    }

    fn update(
        &mut self,
        _message: Self::Message
    ) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        terminal_view::TerminalView::new().into()
    }
}