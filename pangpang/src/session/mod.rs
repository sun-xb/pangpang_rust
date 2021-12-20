use std::collections::HashMap;
use std::sync::Arc;
use std::usize;

use async_trait::async_trait;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::sync::Mutex;
use tokio::sync::Notify;

use crate::errors;
use crate::profile;

mod session_guard;
mod tunnel_guard;
mod pty_guard;
mod session_allocate;
pub use session_guard::PpSessionGuard;
pub use tunnel_guard::PpTunnelGuard;
pub use pty_guard::PpPtyGuard;

pub trait PpStream: AsyncRead + AsyncWrite + Send + Sync + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Sync + Unpin> PpStream for T {}
#[async_trait]
pub trait PpPty: PpStream {
    async fn resize(&mut self, width: usize, height: usize) -> Result<(), errors::Error>;
}

type SessionCacheType = HashMap<String, (usize, Arc<dyn PpSession>)>;
#[async_trait]
pub trait PpSession: Send + Sync + Unpin {
    async fn open_tunnel(
        &self,
        host: &String,
        port: u16,
    ) -> Result<Box<dyn PpStream>, errors::Error>;
    async fn open_pty(&self) -> Result<Box<dyn PpPty>, errors::Error>;
    async fn open_port_forward(&self);
}

pub struct PpSessionManager {
    config: Box<dyn crate::storage::Storage + Send + Sync>,
    session_cache: Arc<Mutex<SessionCacheType>>,
    connecting_map: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
}

impl PpSessionManager {
    pub fn new(config: Box<dyn crate::storage::Storage + Send + Sync>) -> Self {
        Self {
            config,
            session_cache: Arc::new(Mutex::new(SessionCacheType::new())),
            connecting_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn open_session(&self, id: &String) -> Result<PpSessionGuard, errors::Error> {
        let cfg = self.config.get(id)?;
        if cfg.capacity().contains(profile::Capacity::SESSION_CACHE) {
            self.open_session_from_cache(id).await
        } else {
            Ok(PpSessionGuard::new(self.alloc_session(id).await?, None, self.session_cache.clone()))
        }
    }

    #[async_recursion::async_recursion]
    pub async fn open_tunnel(&self, id: &String, host: &String, port: u16) -> Result<PpTunnelGuard, errors::Error> {
        let s = self.open_session(id).await?;
        Ok(PpTunnelGuard::new(s.open_tunnel(host, port).await?, s))
    }

    pub async fn open_pty(&self, id: &String) -> Result<PpPtyGuard, errors::Error> {
        let s = self.open_session(id).await?;
        Ok(PpPtyGuard::new(s.open_pty().await?, s))
    }

    #[async_recursion::async_recursion]
    async fn open_session_from_cache(&self, id: &String) -> Result<PpSessionGuard, errors::Error> {
        loop {
            if let Some((counter, s)) = self.session_cache.lock().await.get_mut(id) {
                *counter += 1;
                return Ok(PpSessionGuard::new(s.clone(), Some(id.to_owned()), self.session_cache.clone()));
            }
            let mut connecting = self.connecting_map.lock().await;
            match connecting.get(id) {
                Some(notify) => {
                    let n = notify.clone();
                    drop(connecting);
                    n.notified().await;
                }
                None => {
                    let notify = Arc::new(Notify::new());
                    connecting.insert(id.to_owned(), notify.clone());
                    drop(connecting);
                    let s = self.alloc_session(id).await?;
                    self.session_cache
                        .lock()
                        .await
                        .insert(id.to_owned(), (1, s.clone()));
                    notify.notify_waiters();
                    return Ok(PpSessionGuard::new(s, Some(id.to_owned()), self.session_cache.clone()));
                }
            };
        }
    }

    #[async_recursion::async_recursion]
    async fn alloc_session(&self, id: &String) -> Result<Arc<dyn PpSession>, errors::Error> {
        let prof = self.config.get(id)?;
        let alloc = session_allocate::Allocator;
        match prof.protocol {
            profile::Protocol::Ssh(cfg) => {
                alloc.ssh_alloc(self, &prof.address, prof.port, &prof.username, prof.transport, cfg).await
            }
        }
    }
}















