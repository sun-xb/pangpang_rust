use std::{ops::Deref, sync::Arc};

use tokio::sync::Mutex;

use super::{PpSession, SessionCacheType};

pub struct PpSessionGuard {
    inner: Arc<dyn PpSession>,
    id: Option<String>,
    cache: Option<Arc<Mutex<SessionCacheType>>>,
}

impl PpSessionGuard {
    pub fn new(
        inner: Arc<dyn PpSession>,
        id: Option<String>,
        cache: Option<Arc<Mutex<SessionCacheType>>>,
    ) -> Self {
        Self { inner, id, cache }
    }
}

impl Drop for PpSessionGuard {
    fn drop(&mut self) {
        log::info!("pangpang session droped: {:?}", self.id);
        if let Some(id) = &self.id {
            let cache = self.cache.as_ref().expect("shouldn't panic while id is not none").clone();
            let session_id = id.clone();
            tokio::spawn(async move {
                let mut cache = cache.lock().await;
                let (counter, _) = cache.get_mut(&session_id).expect("session must be cached");
                *counter -= 1;
                if 0 == *counter {
                    cache.remove(&session_id);
                    log::info!("pangpang session removed from cache: {}", session_id);
                }
            });
        }
    }
}

impl Deref for PpSessionGuard {
    type Target = Arc<dyn PpSession>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
