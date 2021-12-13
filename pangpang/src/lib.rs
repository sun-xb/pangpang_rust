

mod storage;
mod errors;
mod ssh;


use std::{collections::HashMap, sync::{Arc, Weak}};

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};


//re-export
pub use alacritty_terminal;
pub use alacritty_terminal::term::SizeInfo;

pub trait PpStream: AsyncRead + AsyncWrite + Unpin + Send {}

#[async_trait::async_trait]
pub trait PpTunnel {
    async fn connect(&self, host: String, port: u32) -> Result<Box<dyn PpStream>, errors::Error>;
}

#[async_trait::async_trait]
pub trait PpPty {
    async fn open_pty(&self, size: SizeInfo) -> Result<Box<dyn PpStream>, errors::Error>;
}


#[derive(Debug)]
pub enum PpMessage {
    Hello,
    OpenShell(SizeInfo)
}
pub type PpMsgSender = tokio::sync::mpsc::Sender<PpMessage>;

pub enum Profile {
    SSH(ssh::Profile),
}


pub struct PangPang {
    //storage: Box<dyn storage::Storage>,
    tunnels: HashMap<String, Weak<dyn PpTunnel>>,
    ptys: HashMap<String, Weak<dyn PpPty>>,
}

impl PangPang {
    pub fn new() -> Self {
        Self {
            //storage: Box::new(storage::MockStorage::new()),
            tunnels: HashMap::new(),
            ptys: HashMap::new(),
        }
    }

    pub async fn open_tunnel(&self, id: String) -> Arc<dyn PpTunnel> {
        let tunnel = self.tunnels.get(&id).unwrap();
        tunnel.upgrade().unwrap()
    }

    pub async fn open_pty(&mut self, profile: Profile) -> Arc<dyn PpPty> {
        match profile {
            Profile::SSH(cfg) => {
                //does this need a lock?
                let id = cfg.get_id().await;
                if let Some(pty) = self.ptys.get(&id) {
                    if let Some(s) = pty.upgrade() {
                        return s;
                    }
                }
                let s = ssh::Session::new(cfg).await;
                let s = Arc::new(s);
                let w = Arc::downgrade(&s);
                self.ptys.insert(id, w);
                s
            }
        }
    }

    pub async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<PpMessage>) {
        while let Some(msg) = rx.recv().await {
            log::debug!("received msg: {:?}", msg);
            match msg {
                PpMessage::Hello => log::info!("say hello"),
                PpMessage::OpenShell(size) => {
                    let cfg = ssh::Profile {
                        addr: "localhost:22".to_owned(),
                        username: "root".to_owned(),
                        password: "123456".to_owned(),
                    };
                    let pty = self.open_pty(Profile::SSH(cfg)).await;
                    let mut s = pty.open_pty(size).await.unwrap();
                    tokio::spawn(async move {
                        let mut buffer = [0u8; 1024];
                        while let Ok(n) = s.read(&mut buffer[..]).await {
                            log::info!("receive data: {}", String::from_utf8(buffer[..n].to_vec()).unwrap());
                        }
                        log::debug!("disconnected");
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
