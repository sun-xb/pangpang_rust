use std::sync::Arc;

mod error;
mod session;
mod channel;
mod stream;
mod listener;

pub use error::Error;
pub use session::Session;
pub use channel::Channel;
pub use stream::Stream;
pub use listener::Listener;

pub use ssh2::TraceFlags;
pub use ssh2::PtyModes;

pub async fn poll_ssh2_fn<R, F>(stream: &Arc<tokio::net::TcpStream>, session: &ssh2::Session, mut f: F) -> Result<R, Error>
    where F: FnMut() -> Result<R, ssh2::Error>
{
    loop {
        match f() {
            Ok(r) => return Ok(r),
            Err(e) if std::io::Error::from(ssh2::Error::from_errno(e.code())).kind() == std::io::ErrorKind::WouldBlock => {
                match session.block_directions() {
                    ssh2::BlockDirections::Inbound => stream.readable().await?,
                    ssh2::BlockDirections::Outbound => stream.writable().await?,
                    ssh2::BlockDirections::Both => {
                        let (readable, writable) = tokio::join!(stream.readable(), stream.writable());
                        readable?;
                        writable?;
                    }
                    ssh2::BlockDirections::None => {
                        unreachable!("libssh2 reports EAGAIN but is not blocked");
                    }
                }
            }
            Err(e) => return Err(Error::from(e)),
        }
    }
}


pub(crate) fn poll_ssh2_io_fn<R, F>(cx: &mut std::task::Context<'_>, stream: &Arc<tokio::net::TcpStream>, session: &ssh2::Session, mut f: F) -> std::task::Poll<std::io::Result<R>>
    where F: FnMut() -> Result<R, std::io::Error>
{
    loop {
        match f() {
            Ok(r) => return std::task::Poll::Ready(Ok(r)),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let poll = match session.block_directions() {
                    ssh2::BlockDirections::Inbound => stream.poll_read_ready(cx),
                    ssh2::BlockDirections::Outbound => stream.poll_write_ready(cx),
                    ssh2::BlockDirections::Both => {
                        match stream.poll_read_ready(cx) {
                            std::task::Poll::Pending => std::task::Poll::Pending,
                            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
                            std::task::Poll::Ready(Ok(())) => stream.poll_write_ready(cx),
                        }
                    }
                    ssh2::BlockDirections::None => {
                        unreachable!("libssh2 reports EAGAIN but is not blocked");
                    }
                };
                return match poll {
                    std::task::Poll::Pending => std::task::Poll::Pending,
                    std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
                    std::task::Poll::Ready(Ok(())) => continue,
                };
            }
            Err(e) => return std::task::Poll::Ready(Err(e)),
        }
    }
}

