


pub mod pangpang_run_sync;
pub mod errors;
pub mod session;
pub mod storage;
pub mod profile;
pub mod terminal;
pub mod ssh;




use std::sync::Arc;

//re-export
pub use alacritty_terminal;
use alacritty_terminal::Term;
pub use async_trait::async_trait;
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

    pub async fn open_session(&self, id: &String) -> session::PpSessionGuard {
        let sess = self.mgr.open_session(id).await.unwrap();
        sess
    }

    pub async fn open_tunnel(&self, id: &String) -> session::PpTunnelGuard {
        self.mgr.open_tunnel(id, &"host".to_string(), 100).await.unwrap()
    }

    pub async fn open_pty(&self, id: &String) -> session::PpPtyGuard {
        self.mgr.open_pty(id).await.unwrap()
    }

    pub async fn open_terminal(
        &self,
        handler: Arc<Mutex<Term<terminal::TerminalEventListener>>>,
        param: Box<dyn terminal::NewTerminalParameter>
    ) -> terminal::Terminal {
        terminal::Terminal::new(
            self.open_pty(param.profile_id()).await,
            handler,
            param
        )
    }
}

