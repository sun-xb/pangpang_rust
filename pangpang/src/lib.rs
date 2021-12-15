

mod storage;
mod errors;
mod ssh;
mod terminal;


use std::{collections::HashMap, sync::{Arc, Weak, RwLock}};

use tokio::io::{AsyncRead, AsyncWrite};


//re-export
pub use alacritty_terminal;
pub use alacritty_terminal::term::SizeInfo;
pub use alacritty_terminal::term::RenderableContent;
pub use async_trait::async_trait;

pub trait PpStream: AsyncRead + AsyncWrite + Unpin + Send {}

#[async_trait]
pub trait PpTunnelSession {
    async fn connect(&self, host: String, port: u32) -> Result<Box<dyn PpStream>, errors::Error>;
}

#[async_trait]
pub trait PpTerminalSession {
    async fn open_terminal(&self, size: SizeInfo,msg_receiver: PpTerminalMessageReceiver, r: Arc<RwLock<dyn PpTermianlRender>>) -> Result<terminal::Terminal, errors::Error>;
}

pub trait PpTermianlRender: Send + Sync {
    fn render(&mut self, r: RenderableContent, col: usize);
}

#[derive(Debug)]
pub enum PpTerminalMessage {
    Input(u8),
    ReSize(SizeInfo),
    Flush,
}
pub type PpTerminalMessageSender = tokio::sync::mpsc::Sender<PpTerminalMessage>;
pub type PpTerminalMessageReceiver = tokio::sync::mpsc::Receiver<PpTerminalMessage>;
pub use tokio::sync::mpsc::channel;



pub enum PpMessage {
    Hello,
    OpenTerminal(SizeInfo, PpTerminalMessageReceiver, Arc<RwLock<dyn PpTermianlRender>>)
}
impl std::fmt::Debug for PpMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hello => write!(f, "Hello"),
            Self::OpenTerminal(_, _, _) => write!(f, "OpenTerminal"),
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
                PpMessage::OpenTerminal(size, msg_receiver, r) => {
                    let cfg = ssh::Profile {
                        addr: "localhost:22".to_owned(),
                        username: "root".to_owned(),
                        password: "123456".to_owned(),
                    };
                    let session = self.open_session(Profile::SSH(cfg)).await;
                    let mut term = session.open_terminal(size, msg_receiver, r).await.unwrap();
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
