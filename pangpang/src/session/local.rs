

pub(super) struct LocalSession;

#[async_trait::async_trait]
impl super::Session for LocalSession {
    async fn open_shell(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn direct_tcpip(&self, addr: &str, port: u16) -> anyhow::Result<Box<dyn super::AsyncStream>> {
        let s = tokio::net::TcpStream::connect((addr, port)).await?;
        Ok(Box::new(s))
    }

    async fn forward_listen(&self) -> anyhow::Result<()> {
        todo!()
    }
}


