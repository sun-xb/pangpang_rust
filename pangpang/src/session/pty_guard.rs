use tokio::io::{AsyncRead, AsyncWrite};

use crate::errors;

use super::{PpSessionGuard, PpPty};








pub struct PpPtyGuard {
    inner: Box<dyn PpPty>,
    _session: PpSessionGuard,
}

impl PpPtyGuard {
    pub fn new(inner: Box<dyn PpPty>, s: PpSessionGuard) -> Self {
        PpPtyGuard{ inner: inner, _session: s }
    }
}

#[async_trait::async_trait]
impl PpPty for PpPtyGuard {
    async fn resize(&mut self, width: usize, height: usize) -> Result<(), errors::Error> {
        self.inner.resize(width, height).await
    }
}
impl AsyncRead for PpPtyGuard {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(self.get_mut().inner.as_mut()).poll_read(cx, buf)
    }
}
impl AsyncWrite for PpPtyGuard {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(self.get_mut().inner.as_mut()).poll_write(cx, buf)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(self.get_mut().inner.as_mut()).poll_flush(cx)
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(self.get_mut().inner.as_mut()).poll_shutdown(cx)
    }
}

