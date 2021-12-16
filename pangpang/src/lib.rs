

mod storage;
mod errors;
mod ssh;
mod terminal;


use std::{collections::HashMap, sync::{Arc, Weak, Mutex}};

use alacritty_terminal::Term;
use tokio::io::{AsyncRead, AsyncWrite};


//re-export
pub use alacritty_terminal;
pub use alacritty_terminal::ansi::C0;
pub use alacritty_terminal::term::SizeInfo;
pub use alacritty_terminal::term::RenderableContent;
pub use thrussh::Sig;
pub use async_trait::async_trait;
pub use terminal::TerminalEventListener;

pub trait PpStream: AsyncRead + AsyncWrite + Unpin + Send {}

#[async_trait]
pub trait PpTunnelSession {
    async fn connect(&self, host: String, port: u32) -> Result<Box<dyn PpStream>, errors::Error>;
}

#[async_trait]
pub trait PpTerminalSession {
    async fn new_terminal(&self, handler: Arc<Mutex<Term<TerminalEventListener>>>, param: Box<dyn NewTerminalParameter>) -> Result<terminal::Terminal, errors::Error>;
}



#[derive(Debug)]
pub enum PpTerminalMessage {
    Input(Vec<u8>),
    ReSize(SizeInfo),
    Signal(thrussh::Sig),
}
pub type PpTerminalMessageSender = tokio::sync::mpsc::Sender<PpTerminalMessage>;
pub type PpTerminalMessageReceiver = tokio::sync::mpsc::Receiver<PpTerminalMessage>;
pub use tokio::sync::mpsc::channel;

#[async_trait]
pub trait NewTerminalParameter: Send + Sync {
    fn request_repaint(&self);
    async fn receive_msg(&mut self) -> Option<PpTerminalMessage>;
}

pub enum PpMessage {
    Hello,
    NewTerminal(Arc<Mutex<Term<TerminalEventListener>>>, Box<dyn NewTerminalParameter>),
}
impl std::fmt::Debug for PpMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hello => write!(f, "Hello"),
            Self::NewTerminal(_, _) => write!(f, "NewTerminal"),
        }
    }
}

pub type PpMsgSender = tokio::sync::mpsc::Sender<PpMessage>;

pub enum Profile {
    SSH(ssh::Profile),
}


pub struct PangPang {
    //storage: Box<dyn storage::Storage>,
    tunnel_sessions: HashMap<String, Weak<dyn PpTunnelSession>>,
    terminal_sessions: HashMap<String, Weak<dyn PpTerminalSession>>,
}

impl PangPang {
    pub fn new() -> Self {
        Self {
            //storage: Box::new(storage::MockStorage::new()),
            tunnel_sessions: HashMap::new(),
            terminal_sessions: HashMap::new(),
        }
    }

    pub async fn open_tunnel(&self, id: String) -> Arc<dyn PpTunnelSession> {
        let s = self.tunnel_sessions.get(&id).unwrap();
        s.upgrade().unwrap()
    }

    pub async fn open_session(&mut self, profile: Profile) -> Arc<dyn PpTerminalSession> {
        match profile {
            Profile::SSH(cfg) => {
                //does this need a lock?
                let id = cfg.get_id().await;
                if let Some(s) = self.terminal_sessions.get(&id) {
                    if let Some(s) = s.upgrade() {
                        return s;
                    }
                }
                let s = ssh::Session::new(cfg).await;
                let s = Arc::new(s);
                let w = Arc::downgrade(&s);
                self.terminal_sessions.insert(id, w);
                s
            }
        }
    }

    pub async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<PpMessage>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                PpMessage::Hello => log::info!("say hello"),
                PpMessage::NewTerminal(h, param) => {
                    let cfg = ssh::Profile {
                        addr: "localhost:22".to_owned(),
                        username: "root".to_owned(),
                        password: "123456".to_owned(),
                    };
                    let session = self.open_session(Profile::SSH(cfg)).await;
                    let mut term = session.new_terminal(h, param).await.unwrap();
                    tokio::spawn(async move {
                        term.run().await;
                    });
                }
            }
        }
        println!("pangpang backend exit");
    }
}




pub fn run() -> PpMsgSender {
    let (tx, rx) = tokio::sync::mpsc::channel::<PpMessage>(1024);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut pp = PangPang::new();
            pp.run(rx).await;
        });
    });
    tx
}




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
