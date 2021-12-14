

mod storage;
mod errors;
mod ssh;
mod terminal;


use std::{collections::HashMap, sync::{Arc, Weak, RwLock}, fmt::Debug};

use tokio::io::{AsyncRead, AsyncWrite};


//re-export
pub use alacritty_terminal;
pub use alacritty_terminal::term::SizeInfo;
pub use alacritty_terminal::term::RenderableContent;

pub trait PpStream: AsyncRead + AsyncWrite + Unpin + Send {}

#[async_trait::async_trait]
pub trait PpTunnel {
    async fn connect(&self, host: String, port: u32) -> Result<Box<dyn PpStream>, errors::Error>;
}

#[async_trait::async_trait]
pub trait PpTerminalSession {
    async fn open_terminal(&self, size: SizeInfo, r: Arc<RwLock<dyn PpTermianlRender>>) -> Result<terminal::Terminal, errors::Error>;
}

pub trait PpTermianlRender: Send + Sync + Debug {
    fn render(&mut self, r: RenderableContent, col: usize);
}


#[derive(Debug)]
pub enum PpMessage {
    Hello,
    //OpenTerminal(SizeInfo, Box<dyn PpTermianlRender>),
    OpenTerminal(SizeInfo, Arc<RwLock<dyn PpTermianlRender>>)
}
pub type PpMsgSender = tokio::sync::mpsc::Sender<PpMessage>;

pub enum Profile {
    SSH(ssh::Profile),
}


pub struct PangPang {
    //storage: Box<dyn storage::Storage>,
    tunnels: HashMap<String, Weak<dyn PpTunnel>>,
    sessions: HashMap<String, Weak<dyn PpTerminalSession>>,
}

impl PangPang {
    pub fn new() -> Self {
        Self {
            //storage: Box::new(storage::MockStorage::new()),
            tunnels: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    pub async fn open_tunnel(&self, id: String) -> Arc<dyn PpTunnel> {
        let tunnel = self.tunnels.get(&id).unwrap();
        tunnel.upgrade().unwrap()
    }

    pub async fn open_session(&mut self, profile: Profile) -> Arc<dyn PpTerminalSession> {
        match profile {
            Profile::SSH(cfg) => {
                //does this need a lock?
                let id = cfg.get_id().await;
                if let Some(s) = self.sessions.get(&id) {
                    if let Some(s) = s.upgrade() {
                        return s;
                    }
                }
                let s = ssh::Session::new(cfg).await;
                let s = Arc::new(s);
                let w = Arc::downgrade(&s);
                self.sessions.insert(id, w);
                s
            }
        }
    }

    pub async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<PpMessage>) {
        while let Some(msg) = rx.recv().await {
            log::debug!("received msg: {:?}", msg);
            match msg {
                PpMessage::Hello => log::info!("say hello"),
                PpMessage::OpenTerminal(size, r) => {
                    let cfg = ssh::Profile {
                        addr: "localhost:22".to_owned(),
                        username: "root".to_owned(),
                        password: "123456".to_owned(),
                    };
                    let session = self.open_session(Profile::SSH(cfg)).await;
                    let mut term = session.open_terminal(size, r).await.unwrap();
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
