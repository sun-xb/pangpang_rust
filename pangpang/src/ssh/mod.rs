






use std::sync::Arc;

use tokio::sync::Mutex;


mod handler;
mod ssh_tunnel_stream;

use crate::{session::{PpStream, PpSession, PpPty, PpTunnelGuard}, errors};


pub struct SshProfile {
    pub password: String,
}

pub struct Session {
    s: Arc<Mutex<thrussh::client::Handle<handler::PpSshHandler>>>,
}

impl Session {
    pub async fn new(addr: &String, port: u16, username: &String, cfg: SshProfile) -> Result<Self, errors::Error> {
        let config = Arc::new(thrussh::client::Config::default());
        let mut s = thrussh::client::connect(config, (addr.as_str(), port), handler::PpSshHandler).await?;
        s.authenticate_password(username, cfg.password).await?;
        let s = Arc::new(Mutex::new(s));
        Ok(Self { s })
    }

    pub async fn new_with_stream(stream: PpTunnelGuard, username: &String, cfg: SshProfile) -> Result<Self, errors::Error> {
        let config = Arc::new(thrussh::client::Config::default());
        let mut s = thrussh::client::connect_stream(config, stream, handler::PpSshHandler).await?;
        s.authenticate_password(username, cfg.password).await?;
        let s = Arc::new(Mutex::new(s));
        Ok(Self { s })
    }
}



#[async_trait::async_trait]
impl PpSession for Session {
    async fn open_tunnel(
        &self,
        host: &String,
        port: u16,
    ) -> Result<Box<dyn PpStream>, errors::Error> {
        let mut handle = self.s.lock().await;
        let ch = handle.channel_open_direct_tcpip(host, port as u32, "127.0.0.1", 22).await?;
        let tunnel = ssh_tunnel_stream::SshTunnelStream::from(ch);
        Ok(Box::new(tunnel))
    }
    async fn open_pty(&self) -> Result<Box<dyn PpPty>, errors::Error> {
        let mut ch = self.s.lock().await.channel_open_session().await?;
        ch.request_pty(
            false,
            "xterm-256color",
            80,
            20,
            0,
            0,
            &[]
        ).await?;
        ch.request_shell(false).await?;
        let term = ssh_tunnel_stream::SshTunnelStream::from(ch);
        Ok(Box::new(term))
    }
    async fn open_port_forward(&self) {
        todo!()
    }
}






