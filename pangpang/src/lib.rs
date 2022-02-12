


use std::sync::Arc;

//re-export
pub use alacritty_terminal;
pub use async_trait::async_trait;

pub mod errors;
pub mod session;
pub mod storage;
pub mod profile;
pub mod terminal;
pub mod forward;





pub struct PangPang {
    mgr: session::PpSessionManager,
}

impl PangPang {
    pub fn new(cfg: Arc<tokio::sync::Mutex<dyn storage::Storage>>) -> Self {
        Self {
            mgr: session::PpSessionManager::new(cfg),
        }
    }

    pub async fn open_terminal(
        &self,
        id: &String,
        input: terminal::msg::PpTerminalMessageReceiver,
        ui_render: Arc<tokio::sync::Mutex<dyn terminal::Render>>
    ) -> Result<terminal::Terminal, errors::Error> {
        let pty = self.mgr.open_pty(id).await?;
        Ok(terminal::Terminal::new(Box::new(pty), input, ui_render))
    }

    pub async fn local_http_proxy(&self, id: &String, addr: &String, port: u16) -> Result<forward::HttpProxy, errors::Error> {
        use std::net::ToSocketAddrs;
        let session = self.mgr.open_session(id).await?;
        let session = Arc::new(session);
        let server = forward::HttpProxy::new((addr.as_str(), port).to_socket_addrs().unwrap().next().unwrap(), session);
        Ok(server)
    }
}

