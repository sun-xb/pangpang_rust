use std::{sync::Arc, fmt::Debug};


pub use tokio::sync::Mutex;

use crate::{storage::Storage, terminal::{msg::PpTerminalMessageReceiver, Render}};
pub type PpMsgSender = tokio::sync::mpsc::Sender<PpMessage>;
pub type PpMsgReceiver = tokio::sync::mpsc::Receiver<PpMessage>;
pub enum PpMessage {
    Hello,
    NewTerminal(String, PpTerminalMessageReceiver, Arc<Mutex<dyn Render>>),
}
impl Debug for PpMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pangpang sync msg")
    }
}

pub fn run(cfg: Arc<Mutex<dyn Storage>>) -> PpMsgSender {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<PpMessage>(1024);
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async move {
            let pp = crate::PangPang::new(cfg);
            loop {
                match rx.recv().await {
                    None => break,
                    Some(msg) => {
                        match msg {
                            PpMessage::Hello => log::info!("ui thread say us hello!"),
                            PpMessage::NewTerminal(id, input, render) => {
                                if let Ok(mut term) = pp.open_terminal(id.clone(), input, render).await {
                                    tokio::spawn(async move {
                                        if let Err(e) = term.run().await {
                                            log::error!("terminal exited with error: {:?}, id: {}", e, id);
                                        } else {
                                            log::info!("terminal exited, id: {}", id);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
            log::info!("pangpang exited!");
        });
    });
    tx
}
