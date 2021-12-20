use std::{ops::Deref, sync::Arc};

use tokio::sync::Mutex;

use super::{PpSession, SessionCacheType};

pub struct PpSessionGuard {
    inner: Arc<dyn PpSession>,
    id: Option<String>,
    cache: Arc<Mutex<SessionCacheType>>,
}

impl PpSessionGuard {
    pub fn new(
        inner: Arc<dyn PpSession>,
        id: Option<String>,
        cache: Arc<Mutex<SessionCacheType>>,
    ) -> Self {
        Self { inner, id, cache }
    }
}

impl Drop for PpSessionGuard {
    fn drop(&mut self) {
        if let Some(id) = &self.id {
            let cache = self.cache.clone();
            let session_id = id.clone();
            tokio::spawn(async move {
                let mut cache = cache.lock().await;
                if let Some((counter, _)) = cache.get_mut(&session_id) {
                    *counter -= 1;
                    if 0 == *counter {
                        cache.remove(&session_id);
                    }
                } else {
                    unreachable!("session must be cached")
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
