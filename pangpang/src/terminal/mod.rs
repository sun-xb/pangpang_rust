


use std::sync::Arc;

use alacritty_terminal::{ansi::Processor, event::EventListener, Term};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex};
use log::{info, warn};

use crate::{session::{PpPtyGuard, PpPty}, errors};

mod msg;
pub use msg::*;
pub use tokio::sync::mpsc::channel;

pub struct TerminalEventListener;
impl EventListener for TerminalEventListener {
    fn send_event(&self, event: alacritty_terminal::event::Event) {
        println!("event listener: {:?}", event);
    }
}

#[async_trait::async_trait]
pub trait NewTerminalParameter: Send + Sync {
    fn profile_id(&self) -> &String;
    fn request_repaint(&self);
    async fn receive_msg(&mut self) -> Option<PpTerminalMessage>;
}

pub struct Terminal {
    pty: PpPtyGuard,
    handler: Arc<Mutex<Term<TerminalEventListener>>>,
    param: Box<dyn NewTerminalParameter>,
    processor: Processor,
}

impl Terminal {
    pub fn new(
        pty: PpPtyGuard,
        handler: Arc<Mutex<Term<TerminalEventListener>>>,
        param: Box<dyn NewTerminalParameter>
    ) -> Self {
        Self {
            pty,
            handler,
            param,
            processor: Processor::default(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            let mut buffer = [0u8; 1024];
            tokio::select! {
                pty_read = self.pty.read(&mut buffer) => {
                    match pty_read {
                        Ok(n) => {
                            let mut handler = self.handler.lock().await;
                            for i in 0..n {
                                self.processor.advance(&mut *handler, buffer[i]);
                            }
                            self.param.request_repaint();
                        }
                        Err(e) => {
                            warn!("pty returned error: {:?}", e);
                            break;
                        }
                    }
                }
                msg = self.param.receive_msg() => {
                    match msg {
                        None => {
                            if let Err(e) = self.pty.shutdown().await {
                                warn!("close channel failed with {:?}", e);
                            }
                            break;
                        }
                        Some(msg) => {
                            if let Err(e) = match msg {
                                PpTerminalMessage::Input(s) => {
                                    match self.pty.write_all(s.as_slice()).await {
                                        Err(e) => Err(errors::Error::from(e)),
                                        _ => Ok(()),
                                    }
                                }
                                PpTerminalMessage::ReSize(width, height) => {
                                    self.pty.resize(width, height).await
                                }
                            } {
                                warn!("write pty failed with {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
        }
        info!("terminal exit");
    }
}
