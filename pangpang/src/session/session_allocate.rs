use std::sync::Arc;

use crate::errors;
use super::ssh;

use super::{PpSession, PpSessionManager};




pub struct Allocator;

impl Allocator {
    pub async fn ssh_alloc(&self, mgr: &PpSessionManager, addr: &String, port: u16, user: &String, transport: Option<String>, cfg: ssh::SshProfile) -> Result<Arc<dyn PpSession>, errors::Error> {
        let s = if let Some(id) = transport {
            let transport = mgr.open_tunnel(&id, addr, port).await?;
            ssh::Session::new_with_stream(transport, user, cfg).await?
        } else {
            ssh::Session::new(addr, port, user, cfg).await?
        };
        Ok(Arc::new(s))
    }
}