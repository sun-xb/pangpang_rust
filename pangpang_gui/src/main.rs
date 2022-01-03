use iced::Application;

mod terminal_view;

fn main() {
    PpMainWindow::run(iced::Settings {
        antialiasing: false,
        ..iced::Settings::default()
    }).unwrap();
}

struct PpMainWindow;

#[derive(Debug)]
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
        _message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        if let Some(s) = clipboard.read() {
            println!("clipboard data: {}", s);
        }
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        iced::Text::new("Hello, world!").into()
    }
}