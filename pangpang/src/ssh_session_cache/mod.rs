use anyhow::Result;






enum SessionState {
    ChannelFull,
    ConnectionLost,
    Available
}

struct SessionDetail {
    count: usize,
    state: SessionState,
    session: crate::async_ssh2::Session,
}

pub struct SshSessionCache {
    cache: std::collections::HashMap<String, std::collections::HashMap<usize, SessionDetail>>,
}

impl SshSessionCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new()
        }
    }

    pub async fn open<R, E, F>(&mut self, id: &String, mut f: F) -> Result<R, E>
        where F: FnMut(&crate::async_ssh2::Session) -> Result<R, E> {
        if let Some(detail) = self.cache.get_mut(id) {
            for detail in detail.values_mut() {
                if let SessionState::Available = detail.state {
                    return f(&detail.session);
                }
            }
        }
        todo!()
    }
}