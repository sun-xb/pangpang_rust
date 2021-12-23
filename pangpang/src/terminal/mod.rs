



use alacritty_terminal::{event::EventListener, term::RenderableContent};

mod terminal;
pub use terminal::Terminal;
pub mod msg;
pub use tokio::sync::mpsc::channel;

pub struct TerminalEventListener;
impl EventListener for TerminalEventListener {
    fn send_event(&self, event: alacritty_terminal::event::Event) {
        println!("event listener: {:?}", event);
    }
}

pub type TerminalRender<'a> = RenderableContent<'a>;
pub trait Render: Send + Sync {
    fn draw(&mut self, render: TerminalRender);
}


