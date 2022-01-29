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

mod pty_guard;
mod session_guard;
pub mod socks;
pub mod ssh;
pub mod telnet;
mod tunnel_guard;
pub use pty_guard::PpPtyGuard;
pub use session_guard::PpSessionGuard;
pub use tunnel_guard::PpTunnelGuard;

pub trait PpStream: AsyncRead + AsyncWrite + Send + Sync + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Sync + Unpin> PpStream for T {}

#[async_trait]
pub trait PpPty: PpStream {
    async fn resize(&mut self, width: usize, height: usize) -> Result<(), errors::Error>;
}

type SessionCacheType = HashMap<String, (usize, Arc<dyn PpSession>)>;
#[async_trait]
pub trait PpSession: Send + Sync + Unpin {
    async fn open_pty(&self) -> Result<Box<dyn PpPty>, errors::Error> {
        unreachable!()
    }
    async fn local_tunnel(
        &self,
        _host: &String,
        _port: u16,
    ) -> Result<Box<dyn PpStream>, errors::Error> {
        unreachable!()
    }
    async fn remote_tunnel(&self, _host: &String, _port: u16) {
        unreachable!()
    }
}

#[async_trait]
pub trait PpSessionBuilder {
    async fn build(
        &self,
        transport: Option<PpSessionGuard>,
    ) -> Result<Arc<dyn PpSession>, errors::Error>;
}

#[derive(Clone)]
pub struct PpSessionManager {
    config: Arc<Mutex<dyn crate::storage::Storage>>,
    session_cache: Arc<Mutex<SessionCacheType>>,
    connecting_map: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
}

impl PpSessionManager {
    pub fn new(config: Arc<Mutex<dyn crate::storage::Storage>>) -> Self {
        Self {
            config,
            session_cache: Arc::new(Mutex::new(SessionCacheType::new())),
            connecting_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn open_session(&self, id: &String) -> Result<PpSessionGuard, errors::Error> {
        let cfg = self.config.lock().await.get(id)?.clone();
        if cfg.capacity().contains(profile::Capacity::SESSION_CACHE) {
            self.open_session_from_cache(id).await
        } else {
            Ok(PpSessionGuard::new(
                self.alloc_session(id).await?,
                None,
                None,
            ))
        }
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
                log::info!("open session from cache, id: {}, ref: {}", id, counter);
                return Ok(PpSessionGuard::new(
                    s.clone(),
                    Some(id.to_owned()),
                    Some(self.session_cache.clone()),
                ));
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
                    self.connecting_map.lock().await.remove(id).unwrap();
                    notify.notify_waiters();
                    return Ok(PpSessionGuard::new(
                        s,
                        Some(id.to_owned()),
                        Some(self.session_cache.clone()),
                    ));
                }
            };
        }
    }

    #[async_recursion::async_recursion]
    async fn alloc_session(&self, id: &String) -> Result<Arc<dyn PpSession>, errors::Error> {
        let prof = self.config.lock().await.get(id)?.clone();
        let mut transport_session: Option<PpSessionGuard> = None;
        if let Some(id) = prof.transport {
            transport_session = Some(self.open_session(&id).await?);
        }
        match prof.protocol {
            profile::Protocol::Ssh(builder) => builder.build(transport_session).await,
            profile::Protocol::Socks(builder) => builder.build(transport_session).await,
            profile::Protocol::Telnet(builder) => builder.build(transport_session).await,
        }
    }
}
