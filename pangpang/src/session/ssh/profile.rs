use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::{session::{PpSessionBuilder, PpSession, PpSessionGuard, PpTunnelGuard}, errors};



#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub address: String,
    pub username: String,
    pub password: String,
    pub port: u16,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            address: String::from("localhost"),
            username: String::from("root"),
            password: String::from("123456"),
            port: 22,
        }
    }
}

#[async_trait::async_trait]
impl PpSessionBuilder for Profile {
    async fn build(&self, transport: Option<PpSessionGuard>) -> Result<Arc<dyn PpSession>, errors::Error> {
        let session = match transport {
            None => super::Session::new(self).await?,
            Some(s) => {
                let stream = s.local_tunnel(&self.address, self.port).await?;
                let stream = PpTunnelGuard::new(stream, s);
                super::Session::new_with_stream(stream, self).await?
            }
        };
        Ok(Arc::new(session))
    }
}
