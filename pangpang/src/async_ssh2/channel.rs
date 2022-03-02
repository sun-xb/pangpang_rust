use std::{sync::Arc, future::Future};


use super::{poll_ssh2_fn, Stream, Error};



pub struct Channel {
    stream: Arc<tokio::net::TcpStream>,
    session: ssh2::Session,
    inner: ssh2::Channel,
}

impl Channel {
    pub(super) fn new(stream: Arc<tokio::net::TcpStream>, session: ssh2::Session, ch: ssh2::Channel) -> Self {
        Self {
            stream,
            session,
            inner: ch,
        }
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.session, || self.inner.close()).await
    }

    pub async fn shell(&mut self) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.session, || self.inner.shell()).await
    }

    pub async fn request_pty(&mut self, term: &str, mode: Option<ssh2::PtyModes>, dim: Option<(u32, u32, u32, u32)>) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.session, || self.inner.request_pty(term, mode.clone(), dim)).await
    }

    pub async fn request_pty_size(&mut self, width: u32, height: u32, width_px: Option<u32>, height_px: Option<u32>) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.session, || self.inner.request_pty_size(width, height, width_px, height_px)).await
    }

    pub fn stderr(&self) -> Stream {
        let inner = self.inner.stderr();
        Stream::new(self.stream.clone(), self.session.clone(), inner)
    }

    pub fn stream(&self, stream_id: i32) -> Stream {
        let inner = self.inner.stream(stream_id);
        Stream::new(self.stream.clone(), self.session.clone(), inner)
    }

    pub fn eof(&self) -> bool {
        self.inner.eof()
    }

    pub async fn request_auth_agent_forwarding(&mut self) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.session, || self.inner.request_auth_agent_forwarding()).await
    }

    pub async fn subsystem(&mut self, system: &str) -> Result<(), Error> {
        poll_ssh2_fn(&self.stream, &self.session, || self.inner.subsystem(system)).await
    }
}

impl tokio::io::AsyncRead for Channel {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut stream = self.stream(0);
        std::pin::Pin::new(&mut stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for Channel {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let mut stream = self.stream(0);
        std::pin::Pin::new(&mut stream).poll_write(cx, buf)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        let mut stream = self.stream(0);
        std::pin::Pin::new(&mut stream).poll_flush(cx)
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        match std::pin::Pin::new(&mut Box::pin(this.close())).poll(cx) {
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(Ok(())) => std::task::Poll::Ready(Ok(())),
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e.into()))
        }
    }
}