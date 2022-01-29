use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::errors;

use super::{PpSessionBuilder, PpSession, PpSessionGuard, PpStream};


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
    async fn local_tunnel(
        &self,
        _host: &String,
        _port: u16,
    ) -> Result<Box<dyn PpStream>, errors::Error> {
        unimplemented!()
    }
}