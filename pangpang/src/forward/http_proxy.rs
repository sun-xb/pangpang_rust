
use crate::{session::{PpSessionGuard, PpStream}, errors};





pub struct HttpProxy {
    notify: std::sync::Arc<tokio::sync::Notify>,
    server: std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), errors::Error>>>>,
}


impl HttpProxy {
    pub fn new(addr: std::net::SocketAddr, transport: std::sync::Arc<PpSessionGuard>) -> Self {
        let notify = std::sync::Arc::new(tokio::sync::Notify::default());
        let shutdown = notify.clone();
        
        let svr = async move {
            let http_client = hyper::Client::builder().build::<_, hyper::Body>(HttpConnector{ transport: transport.clone() });
            let make_svc = hyper::service::make_service_fn(|_| {
                let client = http_client.clone();
                let ts = transport.clone();
                let svc = hyper::service::service_fn(move |r: hyper::Request<hyper::Body>| {
                    Self::forward(r, client.clone(), ts.clone())
                });
                std::future::ready(Ok::<_, errors::Error>(svc))
            });
            let server = hyper::Server::bind(&addr).serve(make_svc);
            server.with_graceful_shutdown(shutdown.notified()).await.map_err(|e| errors::Error::HttpProxyServerError(e.to_string()))
        };
        Self {
            notify: notify,
            server: Box::pin(svr)
        }
    }

    pub async fn run(&mut self) -> Result<(), errors::Error> {
        self.server.as_mut().await
    }

    pub fn shutdown(&self) {
        self.notify.notify_one();
    }

    async fn forward(req: hyper::Request<hyper::Body>, client: hyper::Client<HttpConnector>, ts: std::sync::Arc<PpSessionGuard>) -> Result<hyper::Response<hyper::Body>, errors::Error> {
        if ts.is_closed().await {
            log::error!("http proxy server connection lost!");
            return Err(errors::Error::HttpProxyConnectionLost)
        }
        if hyper::Method::CONNECT != req.method() {
            return client.request(req).await.map_err(|e| errors::Error::HttpProxyRequestError(e.to_string()))
        }
        tokio::spawn(async move {
            let host = req.uri().host();
            if host.is_none() {
                log::error!("http forward got unknown host: {}", req.uri().to_string());
                return;
            }
            let host = host.unwrap().to_string();
            let port = req.uri().port().map_or(443, |p| p.as_u16());

            let upgraded = hyper::upgrade::on(req).await;
            if let Err(e) = upgraded {
                log::error!("http forward upgrade failed: {}", e.to_string());
                return;
            }
            let conn = ts.local_tunnel(&host, port).await;
            if let Err(e) = conn {
                log::error!("http forward connection lost: {}", e.to_string());
                return;
            }
            tokio::io::copy_bidirectional(&mut upgraded.unwrap(), &mut conn.unwrap()).await.unwrap_or_default();
        });
        Ok(hyper::Response::new(hyper::Body::empty()))
    }
}


#[derive(Clone)]
struct HttpConnector {
    transport: std::sync::Arc<PpSessionGuard>
}

impl hyper::service::Service<hyper::Uri> for HttpConnector {
    type Response = HttpStream;

    type Error = errors::Error;

    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Uri) -> Self::Future {
        let s = self.transport.clone();
        let f = async move {
            let host = req.host().ok_or(errors::Error::UrlParseError(req.to_string()))?;
            let port = *req.port_u16().get_or_insert_with(|| {
                match req.scheme_str() {
                    Some("https") => 443,
                    _ => 80
                }
            });
            let stream = s.local_tunnel(&host.to_string(), port).await?;
            Ok(HttpStream{ stream })
        };
        Box::pin(f)
    }
}



struct HttpStream {
    stream: Box<dyn PpStream>
}
impl hyper::client::connect::Connection for HttpStream {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
    }
}

impl tokio::io::AsyncWrite for HttpStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(self.get_mut().stream.as_mut()).poll_write(cx, buf)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(self.get_mut().stream.as_mut()).poll_flush(cx)
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(self.get_mut().stream.as_mut()).poll_shutdown(cx)
    }
}

impl tokio::io::AsyncRead for HttpStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(self.get_mut().stream.as_mut()).poll_read(cx, buf)
    }
}

