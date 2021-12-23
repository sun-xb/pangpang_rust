


pub mod pangpang_run_sync;
pub mod errors;
pub mod session;
pub mod storage;
pub mod profile;
pub mod terminal;





use std::sync::Arc;

//re-export
pub use alacritty_terminal;
pub use async_trait::async_trait;
use terminal::msg::PpTerminalMessageReceiver;
use tokio::sync::Mutex;




pub struct PangPang {
    mgr: session::PpSessionManager,
}

impl PangPang {
    fn new(cfg: Arc<Mutex<dyn storage::Storage>>) -> Self {
        Self {
            mgr: session::PpSessionManager::new(cfg),
        }
    }

    pub async fn open_session(&self, id: &String) -> Result<session::PpSessionGuard, errors::Error> {
        self.mgr.open_session(id).await
    }

    pub async fn open_tunnel(&self, id: &String) -> Result<session::PpTunnelGuard, errors::Error> {
        self.mgr.open_tunnel(id, &"host".to_string(), 100).await
    }

    pub async fn open_pty(&self, id: &String) -> Result<session::PpPtyGuard, errors::Error> {
        self.mgr.open_pty(id).await
    }

    pub async fn open_terminal(
        &self,
        id: String,
        input: PpTerminalMessageReceiver,
        ui_render: Arc<Mutex<dyn terminal::Render>>
    ) -> Result<terminal::Terminal, errors::Error> {
        Ok(terminal::Terminal::new(
            Box::new(self.open_pty(&id).await?),
            input, ui_render
        ))
    }
}

