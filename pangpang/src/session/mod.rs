use crate::profile::Protocol;



mod local;
mod ssh;


pub trait AsyncStream: tokio::io::AsyncRead + tokio::io::AsyncWrite {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite> AsyncStream for T {}


#[async_trait::async_trait]
pub trait Session: Send + Sync {
    async fn open_shell(&self) -> anyhow::Result<()>;
    async fn direct_tcpip(&self, addr: &str, port: u16) -> anyhow::Result<Box<dyn AsyncStream>>;
    async fn forward_listen(&self) -> anyhow::Result<()>;
}



pub struct Builder<S: crate::storage::Storage> {
    storage: S
}

impl<S: crate::storage::Storage> Builder<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage
        }
    }

    #[async_recursion::async_recursion]
    pub async fn open_session(&self, id: &String) -> anyhow::Result<Box<dyn Session>> {
        let profile = self.storage.get(id).await?;
        let transport = match profile.transport {
            None => Box::new(local::LocalSession),
            Some(ref id) => self.open_session(id).await?
        };
        let s = match profile.protocol {
            Protocol::SSH(ref ssh) => ssh::SshSession::new(ssh, transport)
        };

        Ok(Box::new(s) as Box<dyn Session>)
    }
}

