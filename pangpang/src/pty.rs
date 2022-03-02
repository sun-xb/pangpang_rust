

#[async_trait::async_trait]
pub trait Pty: tokio::io::AsyncRead + tokio::io::AsyncWrite {
    async fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()>;
}


#[async_trait::async_trait]
impl Pty for super::async_ssh2::Channel {
    async fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.request_pty_size(width, height, None, None).await.map_err(|e| e.into())
    }
}

