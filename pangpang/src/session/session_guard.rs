use std::{sync::Arc, ops::Deref};

use tokio::sync::Mutex;

use super::{SessionCacheType, PpSession};





pub struct PpSessionGuard {
    inner: Arc<dyn PpSession>,
    id: Option<String>,
    cache: Arc<Mutex<SessionCacheType>>,
}

impl PpSessionGuard {
    pub fn new(inner: Arc<dyn PpSession>, id: Option<String>, cache: Arc<Mutex<SessionCacheType>>) -> Self {
        Self {
            inner, id, cache
        }
    }
}

impl Drop for PpSessionGuard {
    fn drop(&mut self) {
        if let Some(id) = &self.id {
            let mut cache = self.cache.blocking_lock();
            if let Some((counter, _)) = cache.get_mut(id) {
                *counter -= 1;
                if 0 == *counter {
                    cache.remove(id);
                }
            } else {
                unreachable!("session must be cached")
            }
        }
    }
}

impl Deref for PpSessionGuard {
    type Target = Arc<dyn PpSession>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}