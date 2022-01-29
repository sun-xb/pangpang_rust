


use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::errors;

use super::{PpSessionBuilder, PpSession, PpSessionGuard, PpPty};


#[derive(Clone, Serialize, Deserialize)]
pub struct  Profile {
    pub address: String,
    pub port: u16,
}

#[async_trait::async_trait]
impl PpSessionBuilder for Profile {
    async fn build(
        &self,
        _transport: Option<PpSessionGuard>,
    ) -> Result<Arc<dyn PpSession>, errors::Error> {
        Ok(Arc::new(Session))
    }
}

pub struct Session;
#[async_trait::async_trait]
impl PpSession for Session {
    async fn open_pty(&self) -> Result<Box<dyn PpPty>, errors::Error> {
        unimplemented!()
    }
}

