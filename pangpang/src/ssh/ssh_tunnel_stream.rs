use std::future::Future;

use thrussh::ChannelMsg;

pub struct SshTunnelStream {
    channel: thrussh::client::Channel,
    read_buf: Vec<u8>,
}

impl From<thrussh::client::Channel> for SshTunnelStream {
    fn from(ch: thrussh::client::Channel) -> Self {
        Self {
            channel: ch,
            read_buf: Vec::new(),
        }
    }
}

impl crate::AsyncWrite for SshTunnelStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match Box::pin(self.get_mut().channel.data(buf)).as_mut().poll(cx) {
            std::task::Poll::Ready(Ok(_)) => std::task::Poll::Ready(Ok(buf.len())),
            std::task::Poll::Ready(Err(e)) => {
                let err = crate::errors::Error::PpStreamError(e.to_string());
                std::task::Poll::Ready(Err(err.into()))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match Box::pin(self.get_mut().channel.eof()).as_mut().poll(cx) {
            std::task::Poll::Ready(Ok(_)) => std::task::Poll::Ready(Ok(())),
            std::task::Poll::Ready(Err(e)) => {
                let err = crate::errors::Error::PpStreamError(e.to_string());
                std::task::Poll::Ready(Err(err.into()))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl crate::AsyncRead for SshTunnelStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut this = self.get_mut();
        loop {
            if !this.read_buf.is_empty() {
                let length = this.read_buf.len().min(buf.remaining());
                buf.put_slice(this.read_buf.drain(..length).as_slice());
                return std::task::Poll::Ready(Ok(()));
            }
            return match Box::pin(this.channel.wait()).as_mut().poll(cx) {
                std::task::Poll::Ready(Some(msg)) => match msg {
                    ChannelMsg::Data { data } => {
                        this.read_buf = data.to_vec();
                        continue
                    }
                    ChannelMsg::Eof | ChannelMsg::Close => {
                        let e = std::io::Error::from(std::io::ErrorKind::UnexpectedEof);
                        std::task::Poll::Ready(Err(e))
                    }
                    _ => continue,
                },
                std::task::Poll::Ready(None) => std::task::Poll::Ready(Ok(())),
                std::task::Poll::Pending => std::task::Poll::Pending,
            };
        }
    }
}

impl crate::PpStream for SshTunnelStream {}
