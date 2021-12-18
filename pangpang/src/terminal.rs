use std::sync::Arc;

use alacritty_terminal::{ansi::Processor, grid::Dimensions, event::EventListener, Term};
use log::info;
use thrussh::ChannelMsg;
pub struct TerminalEventListener;
impl EventListener for TerminalEventListener {
    fn send_event(&self, event: alacritty_terminal::event::Event) {
        println!("event listener: {:?}", event);
    }
}

pub struct Terminal {
    pty: thrussh::client::Channel,
    handler: Arc<std::sync::Mutex<Term<TerminalEventListener>>>,
    param: Box<dyn crate::NewTerminalParameter>,
    processor: Processor,
}

impl Terminal {
    pub fn new(
        pty: thrussh::client::Channel,
        handler: Arc<std::sync::Mutex<Term<TerminalEventListener>>>,
        param: Box<dyn crate::NewTerminalParameter>
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
            let mut ui_msg: Option<crate::PpTerminalMessage> = None;
            tokio::select! {
                pty_data = self.pty.wait() => {
                    match pty_data {
                        Some(msg) => {
                            match msg {
                                ChannelMsg::Data{data} => {
                                    let mut h = self.handler.lock().unwrap();
                                    for byte in data.to_vec() {
                                        self.processor.advance(&mut *h, byte);
                                    }
                                    self.param.request_repaint();
                                }
                                _ => {
                                    info!("unhandled msg: {:?}", msg);
                                }
                            }
                        }
                        None => break,
                    }
                }
                msg = self.param.receive_msg() => {
                    if msg.is_none() {
                        self.pty.eof().await.unwrap();
                        break;
                    }
                    ui_msg = msg;
                }
            }
            if let Some(msg) = ui_msg {
                match msg {
                    crate::PpTerminalMessage::Input(s) => {
                        self.pty.data(s.as_ref()).await.unwrap();
                    }
                    crate::PpTerminalMessage::ReSize(size) => {
                        self.pty.window_change(
                            size.columns() as u32,
                            size.screen_lines() as u32,
                            0, 0
                        ).await.unwrap();
                    }
                    crate::PpTerminalMessage::Signal(s) => {
                        self.pty.signal(s).await.unwrap();
                    }
                }
            }
        }
        info!("terminal exit");
    }
}
