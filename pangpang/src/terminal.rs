use std::sync::Arc;

pub use alacritty_terminal::term::RenderableContent;
use alacritty_terminal::{ansi::Processor, term::SizeInfo, Term, grid::Dimensions};
use log::info;
use thrussh::ChannelMsg;

struct EventListener;
impl alacritty_terminal::event::EventListener for EventListener {}

pub struct Terminal {
    pty: thrussh::client::Channel,
    msg_receiver: crate::PpTerminalMessageReceiver,
    render: Arc<std::sync::RwLock<dyn crate::PpTermianlRender>>,
    term: Term<EventListener>,
    processor: Processor,
}

impl Terminal {
    pub fn new(
        pty: thrussh::client::Channel,
        msg_receiver: crate::PpTerminalMessageReceiver,
        r: Arc<std::sync::RwLock<dyn crate::PpTermianlRender>>,
        size: SizeInfo,
    ) -> Self {
        let config = alacritty_terminal::config::MockConfig::default();
        Self {
            pty,
            msg_receiver,
            render: r,
            term: Term::new(&config, size, EventListener),
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
                                    for byte in data.to_vec() {
                                        self.processor.advance(&mut self.term, byte);
                                    }
                                    self.render.write().unwrap().render(self.term.renderable_content(), self.term.columns())
                                }
                                _ => {
                                    info!("unhandled msg: {:?}", msg);
                                }
                            }
                        }
                        None => break,
                    }
                }
                msg = self.msg_receiver.recv() => {
                    ui_msg = msg;
                }
            }
            if let Some(msg) = ui_msg {
                match msg {
                    crate::PpTerminalMessage::Input(s) => {
                        self.pty.data(&[s][..]).await.unwrap();
                    }
                    crate::PpTerminalMessage::ReSize(size) => {
                        self.pty.window_change(
                            size.columns() as u32,
                            size.screen_lines() as u32,
                            0, 0
                        ).await.unwrap();
                    }
                    crate::PpTerminalMessage::Flush => {
                        self.render.write().unwrap().render(self.term.renderable_content(), self.term.columns());
                    }
                }
            }
        }
        info!("terminal exit");
    }
}
