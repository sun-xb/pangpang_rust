use std::{sync::Arc, fmt::Debug};

use alacritty_terminal::Term;



pub use tokio::sync::Mutex;

use crate::storage::Storage;
pub type PpMsgSender = tokio::sync::mpsc::Sender<PpMessage>;
pub type PpMsgReceiver = tokio::sync::mpsc::Receiver<PpMessage>;
pub enum PpMessage {
    Hello,
    NewTerminal(Arc<Mutex<Term<crate::terminal::TerminalEventListener>>>, Box<dyn crate::terminal::NewTerminalParameter>),
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
                            PpMessage::NewTerminal(handler, parameter) => {
                                let mut term = pp.open_terminal(handler, parameter).await;
                                tokio::spawn(async move {
                                    term.run().await
                                });
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
