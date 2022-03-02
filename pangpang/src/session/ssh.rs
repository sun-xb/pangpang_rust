


pub(super) struct SshSession {
    config: crate::profile::SshProfile,
    transport: Box<dyn super::Session>
}


#[async_trait::async_trait]
impl super::Session for SshSession {
    async fn open_shell(&self) -> anyhow::Result<()> {
        self.transport.direct_tcpip(self.config.address.as_str(), self.config.port).await?;
        todo!()
    }

    async fn direct_tcpip(&self, addr: &str, port: u16) -> anyhow::Result<Box<dyn super::AsyncStream>> {
        self.transport.direct_tcpip(addr, port).await
    }

    async fn forward_listen(&self) -> anyhow::Result<()> {
        todo!()
    }
}

impl SshSession {
    pub fn new(config: &crate::profile::SshProfile, tp: Box<dyn super::Session>) -> Self {
        Self {
            config: config.clone(),
            transport: tp
        }
    }
}