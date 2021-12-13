use alacritty_terminal::{Term, ansi::Processor, term::SizeInfo};
pub use alacritty_terminal::term::RenderableContent;
use log::info;
use tokio::io::AsyncReadExt;




struct EventListener;
impl alacritty_terminal::event::EventListener for EventListener {}

pub struct Terminal {
    stream:     Box<dyn crate::PpStream>,
    render:     Box<dyn crate::PpTermianlRender>,
    term:       Term<EventListener>,
    processor:  Processor,
}

impl Terminal {
    pub fn new(s: Box<dyn crate::PpStream>, r: Box<dyn crate::PpTermianlRender>, size: SizeInfo) -> Self {
        let config = alacritty_terminal::config::MockConfig::default();
        Self {
            stream: s,
            render: r,
            term: Term::new(&config, size, EventListener),
            processor: Processor::default(),
        }
    }

    pub async fn run(&mut self) {
        let mut buffer = [0u8; 1024];
        while let Ok(n) = self.stream.read(&mut buffer[..]).await {
            for byte in &buffer[..n] {
                self.processor.advance(&mut self.term, *byte);
            }
            self.render.render(self.term.renderable_content());
        }
        info!("terminal exit");
    }
}