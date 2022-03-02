use std::sync::Arc;

use super::{Channel, poll_ssh2_fn};




pub struct Listener {
    stream: Arc<tokio::net::TcpStream>,
    session: ssh2::Session,
    inner: ssh2::Listener,
    port: u16
}

impl Listener {
    pub(super) fn new(stream: Arc<tokio::net::TcpStream>, session: ssh2::Session, inner: ssh2::Listener, port: u16) -> Self {
        Self {
            stream,
            session,
            inner,
            port
        }
    }

    pub async fn accept(&mut self) -> anyhow::Result<Channel> {
        let ch = poll_ssh2_fn(&self.stream, &self.session, || self.inner.accept()).await?;
        Ok(Channel::new(self.stream.clone(), self.session.clone(), ch))
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

