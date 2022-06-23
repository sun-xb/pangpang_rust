



#[cfg(windows)]
use std::os::windows::prelude::{FromRawSocket, AsRawSocket};
#[cfg(unix)]
use std::os::unix::prelude::{FromRawFd, AsRawFd};
use std::sync::Arc;

use super::{poll_ssh2_fn, Channel, Listener, Error};

#[derive(Clone)]
pub struct Session {
    stream: Arc<tokio::net::TcpStream>,
    inner: ssh2::Session,
}

impl Session {
    pub async fn new(stream: tokio::net::TcpStream) -> Result<Self, Error> {
        let stream = Arc::new(stream);
        let mut inner = ssh2::Session::new()?;
        #[cfg(windows)]
        let ssh2_tcp_stream = unsafe { std::net::TcpStream::from_raw_socket(stream.as_raw_socket()) };
        #[cfg(unix)]
        let ssh2_tcp_stream = unsafe { std::net::TcpStream::from_raw_fd(stream.as_raw_fd()) };
        inner.set_tcp_stream(ssh2_tcp_stream);
        inner.set_blocking(false);
        Ok(Self{
            stream,
            inner,
        })
    }

    pub async fn new_with_channel(ch: Channel) -> Result<Self, Error> {
        let listerner = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let local_addr = listerner.local_addr()?;
        let (s1, s2) = loop {
            let joined = tokio::join!(listerner.accept(), tokio::net::TcpStream::connect(local_addr));
            let incomming = joined.0?;
            let client = joined.1?;
            if client.local_addr()? != incomming.1 {
                continue;
            }
            break (incomming.0, client);
        };
        let _join_handle = tokio::spawn(async move {
            let mut channel = ch;
            let mut socket = s1;
            if let Err(e) = tokio::io::copy_bidirectional(&mut channel, &mut socket).await {
                log::warn!("jump host copy stream error: {}", e);
            }
            log::info!("jump host stream exit");
        });
        Self::new(s2).await
    }

    pub async fn handshake(&mut self) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.inner, || self.inner.clone().handshake()).await
    }

    pub async fn auth_methods(&self, username: &str) -> Result<&str, Error> {
        poll_ssh2_fn(&self.stream, &self.inner, || self.inner.auth_methods(username)).await
    }

    pub async fn userauth_password(&self, username: &str, password: &str) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.inner, || self.inner.userauth_password(username, password)).await
    }

    pub async fn userauth_agent(&self, username: &str) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.inner, || self.inner.userauth_agent(username)).await
    }

    pub async fn userauth_pubkey_file(&self, username: &str, pubkey: Option<&std::path::Path>, privatekey: &std::path::Path, passphrase: Option<&str>) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.inner, || self.inner.userauth_pubkey_file(username, pubkey, privatekey, passphrase)).await
    }

    pub async fn userauth_keyboard_interactive<P: ssh2::KeyboardInteractivePrompt>(&self, username: &str, prompter: &mut P) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.inner, || self.inner.userauth_keyboard_interactive(username, prompter)).await
    }

    pub fn authenticated(&self) -> bool {
        self.inner.authenticated()
    }

    pub fn host_key(&self) -> Option<(&[u8], ssh2::HostKeyType)> {
        self.inner.host_key()
    }

    pub async fn channel_session(&self) -> Result<Channel, Error> {
        let ch = poll_ssh2_fn(&self.stream, &self.inner, || self.inner.channel_session()).await?;
        Ok(Channel::new(self.stream.clone(), self.inner.clone(), ch))
    }

    pub async fn channel_direct_tcpip(&self, host: &str, port: u16, src: Option<(&str, u16)>) -> Result<Channel, Error> {
        let ch = poll_ssh2_fn(&self.stream, &self.inner, || self.inner.channel_direct_tcpip(host, port, src)).await?;
        Ok(Channel::new(self.stream.clone(), self.inner.clone(), ch))
    }

    pub async fn channel_forward_listen(&self, remote_port: u16, host: Option<&str>, queue_maxsize: Option<u32>) -> Result<Listener, Error> {
        let (listener, port) = poll_ssh2_fn(&self.stream, &self.inner, || self.inner.channel_forward_listen(remote_port, host, queue_maxsize)).await?;
        Ok(Listener::new(self.stream.clone(), self.inner.clone(), listener,  port))
    }

    pub fn trace(&self, bitmask: ssh2::TraceFlags) {
        self.inner.trace(bitmask)
    }
}
