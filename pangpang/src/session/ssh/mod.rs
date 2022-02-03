






use std::sync::Arc;

use tokio::sync::Mutex;

use super::{PpStream, PpSession, PpPty, PpTunnelGuard};

use crate::errors;

mod handler;
mod ssh_tunnel_stream;
mod profile;

pub use profile::Profile;

pub struct Session {
    s: Arc<Mutex<thrussh::client::Handle<handler::PpSshHandler>>>,
}

impl Session {
    pub async fn new(profile: &Profile) -> Result<Self, errors::Error> {
        let config = Arc::new(thrussh::client::Config::default());
        let mut s = thrussh::client::connect(config, (profile.address.as_str(), profile.port), handler::PpSshHandler).await?;
        s.authenticate_password(&profile.username, &profile.password).await?;
        let s = Arc::new(Mutex::new(s));
        Ok(Self { s })
    }

    pub async fn new_with_stream(stream: PpTunnelGuard, profile: &Profile) -> Result<Self, errors::Error> {
        let config = Arc::new(thrussh::client::Config::default());
        let mut s = thrussh::client::connect_stream(config, stream, handler::PpSshHandler).await?;
        s.authenticate_password(&profile.username, &profile.password).await?;
        let s = Arc::new(Mutex::new(s));
        Ok(Self { s })
    }
}



#[async_trait::async_trait]
impl PpSession for Session {
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

    async fn local_tunnel(
        &self,
        host: &String,
        port: u16,
    ) -> Result<Box<dyn PpStream>, errors::Error> {
        let mut handle = self.s.lock().await;
        let ch = handle.channel_open_direct_tcpip(host, port as u32, "127.0.0.1", 22).await?;
        let tunnel = ssh_tunnel_stream::SshTunnelStream::from(ch);
        Ok(Box::new(tunnel))
    }

    async fn remote_tunnel(&self, _host: &String, _port: u16) {
        unimplemented!()
    }

    async fn is_closed(&self) -> bool {
        self.s.lock().await.is_closed()
    }
}






