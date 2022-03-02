use std::{sync::Arc, io::{Read, Write}};

use crate::async_ssh2::poll_ssh2_io_fn;





pub struct Stream {
    stream: Arc<tokio::net::TcpStream>,
    session: ssh2::Session,
    inner: ssh2::Stream,
}

impl Stream {
    pub(super) fn new(stream: Arc<tokio::net::TcpStream>, session: ssh2::Session, inner: ssh2::Stream) -> Self {
        Self{
            stream,
            session,
            inner
        }
    }
}

impl tokio::io::AsyncRead for Stream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        match poll_ssh2_io_fn(cx, &this.stream, &this.session, || this.inner.read(buf.initialize_unfilled())) {
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
            std::task::Poll::Ready(Ok(size)) => {
                buf.advance(size);
                std::task::Poll::Ready(Ok(()))
            }
        }
    }
}

impl tokio::io::AsyncWrite for Stream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();
        poll_ssh2_io_fn(cx, &this.stream, &this.session, || this.inner.write(buf))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        poll_ssh2_io_fn(cx, &this.stream, &this.session, || this.inner.flush())
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}