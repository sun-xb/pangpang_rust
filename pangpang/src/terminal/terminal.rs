use std::sync::Arc;

use alacritty_terminal::{ansi::Processor, config::MockConfig, term::SizeInfo, Term};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex};

use crate::{session::PpPty, errors};

use super::{TerminalEventListener, Render, msg::{PpTerminalMessage, PpTerminalMessageReceiver}};




type AlacrittyTerminal = Term<TerminalEventListener>;
pub struct Terminal {
    pty: Box<dyn PpPty>,
    input: PpTerminalMessageReceiver,
    ui_render: Arc<Mutex<dyn Render>>,
    term: AlacrittyTerminal,
    processor: Processor,
}


impl Terminal {
    pub fn new(pty: Box<dyn PpPty>, input: PpTerminalMessageReceiver, ui_render: Arc<Mutex<dyn Render>>) -> Self {
        let cfg = Arc::new(MockConfig::default());
        let size = SizeInfo::new(80.0, 20.0, 1.0, 1.0, 0.0, 0.0, false);
        Self {
            pty, input, ui_render,
            term: AlacrittyTerminal::new(&cfg, size, TerminalEventListener),
            processor: Processor::new(),
        }
    }

    pub async fn run(&mut self) -> Result<(),errors::Error> {
        loop {
            let mut buffer = [0u8; 1500];
            tokio::select! {
                n = self.pty.read(&mut buffer) => {
                    match n {
                        Ok(len) => {
                            for byte in &buffer[..len] {
                                self.processor.advance(&mut self.term, *byte);
                            }
                            self.ui_render.lock().await.draw(self.term.renderable_content());
                        }
                        Err(e) => {
                            return Err(errors::Error::ReadPtyError(format!("read pty error: {:?}", e)));
                        }
                    }
                }
                m = self.input.recv() => {
                    match m {
                        Some(msg) => {
                            self.handle_input(msg).await?
                        }
                        None => {
                            if let Err(e) = self.pty.shutdown().await {
                                return Err(errors::Error::WritePtyError(format!("close channel failed with {:?}", e)));
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_input(&mut self, msg: PpTerminalMessage) -> Result<(), errors::Error> {
        match msg {
            PpTerminalMessage::Input(s) => {
                match self.pty.write_all(s.as_slice()).await {
                    Err(e) => Err(errors::Error::WritePtyError(format!("input to pty error: {:?}", e))),
                    _ => Ok(()),
                }
            }
            PpTerminalMessage::ReSize(width, height) => {
                self.term.resize(SizeInfo::new(width as f32, height as f32, 1.0, 1.0, 0.0, 0.0, false));
                self.pty.resize(width, height).await
            }
        }
    }
}