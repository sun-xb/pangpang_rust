



mod handler;
mod ssh_tunnel_stream;


use std::sync::{Arc, RwLock};

use alacritty_terminal::grid::Dimensions;
use tokio::sync::Mutex;



pub struct Profile {
    pub addr: String,
    pub username: String,
    pub password: String,
}

impl Profile {
    pub async  fn get_id(&self) -> String {
        ["ssh", self.username.as_str(), self.addr.as_str()].join("_")
    }
}

pub(crate) struct Session {
    s: Arc<Mutex<thrussh::client::Handle<handler::PpSshHandler>>>,
}

impl Session {
    pub async fn new(cfg: Profile) -> Self {
        let config = Arc::new(thrussh::client::Config::default());
        let mut s = thrussh::client::connect(config, cfg.addr, handler::PpSshHandler).await.unwrap();
        s.authenticate_password(cfg.username, cfg.password).await.unwrap();
        let s = Arc::new(Mutex::new(s));
        Self {
            s
        }
    }
}



#[async_trait::async_trait]
impl crate::PpTunnelSession for Session {
    async fn connect(&self, host: String, port: u32) -> Result<Box<dyn crate::PpStream>, crate::errors::Error> {
        let mut handle = self.s.lock().await;
        let ch = handle.channel_open_direct_tcpip(host, port, "127.0.0.1", 22).await.unwrap();
        let stream = ssh_tunnel_stream::SshTunnelStream::from(ch);
        Ok(Box::new(stream))
    }
}

#[async_trait::async_trait]
impl crate::PpTerminalSession for Session {
    async fn open_terminal(&self, size: crate::SizeInfo, msg_receiver: crate::PpTerminalMessageReceiver, r: Arc<RwLock<dyn crate::PpTermianlRender>>) -> Result<crate::terminal::Terminal, crate::errors::Error> {
        let mut ch = self.s.lock().await.channel_open_session().await.unwrap();
        ch.request_pty(
            false,
            "xterm-256color",
            size.columns() as u32,
            size.screen_lines() as u32,
            0,
            0,
            &[]
        ).await.unwrap();
        ch.request_shell(false).await.unwrap();
        Ok(crate::terminal::Terminal::new(ch, msg_receiver, r, size))
    }
}




