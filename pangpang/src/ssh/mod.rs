



mod handler;
mod ssh_tunnel_stream;


use std::sync::Arc;

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
impl crate::PpTunnel for Session {
    async fn connect(&self, host: String, port: u32) -> Result<Box<dyn crate::PpStream>, crate::errors::Error> {
        let mut handle = self.s.lock().await;
        let ch = handle.channel_open_direct_tcpip(host, port, "127.0.0.1", 22).await.unwrap();
        let stream = ssh_tunnel_stream::SshTunnelStream::from(ch);
        Ok(Box::new(stream))
    }
}

#[async_trait::async_trait]
impl crate::PpTerminalSession for Session {
    async fn open_terminal(&self, size: crate::SizeInfo, r: Box<dyn crate::PpTermianlRender>) -> Result<crate::terminal::Terminal, crate::errors::Error> {
        let mut ch = self.s.lock().await.channel_open_session().await.unwrap();
        ch.request_pty(
            false,
            "xterm-256color",
            size.cell_width() as u32,
            size.cell_height() as u32,
            size.width() as u32,
            size.height() as u32,
            &[]
        ).await.unwrap();
        ch.request_shell(false).await.unwrap();
        let stream = ssh_tunnel_stream::SshTunnelStream::from(ch);
        Ok(crate::terminal::Terminal::new(Box::new(stream), r, size))
    }
}




