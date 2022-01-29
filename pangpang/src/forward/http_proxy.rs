use crate::session::{PpSessionGuard, PpStream};





pub struct HttpProxy {
    notify: std::sync::Arc<tokio::sync::Notify>,
    server: std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), hyper::Error>>>>,
}


impl HttpProxy {
    pub async fn new(addr: String, transport: std::sync::Arc<PpSessionGuard>) -> Self {
        let notify = std::sync::Arc::new(tokio::sync::Notify::default());
        let shutdown = notify.clone();
        let addr = addr.parse().unwrap();
        
        let svr = async move {
            let http_client = hyper::Client::builder().build::<_, hyper::Body>(HttpConnector{ transport: transport.clone() });
            let make_svc = hyper::service::make_service_fn(|_| {
                let client = http_client.clone();
                let ts = transport.clone();
                let svc = hyper::service::service_fn(move |r: hyper::Request<hyper::Body>| {
                    Self::forward(r, client.clone(), ts.clone())
                });
                std::future::ready(Ok::<_, hyper::Error>(svc))
            });
            let server = hyper::Server::bind(&addr).serve(make_svc);
            server.with_graceful_shutdown(shutdown.notified()).await
        };
        Self {
            notify: notify,
            server: Box::pin(svr)
        }
    }

    pub async fn run(&mut self) {
        self.server.as_mut().await.unwrap();
    }

    pub fn shutdown(&self) {
        self.notify.notify_one();
    }

    async fn forward(req: hyper::Request<hyper::Body>, client: hyper::Client<HttpConnector>, ts: std::sync::Arc<PpSessionGuard>) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
        if hyper::Method::CONNECT != req.method() {
            return client.request(req).await
        }
        tokio::spawn(async move {
            let host = req.uri().host().unwrap();
            let host = host.to_string();
            let port = req.uri().port().unwrap();
            let port = port.as_u16();
            match hyper::upgrade::on(req).await {
                Ok(mut upgraded) => {
                    let mut conn = ts.local_tunnel(&host, port).await.unwrap();
                    if let Err(_e) = tokio::io::copy_bidirectional(&mut upgraded, &mut conn).await {
                        
                    }
                }
                Err(e) => panic!("{}", e)
            }
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

    type Error = hyper::Error;

    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Uri) -> Self::Future {
        let s = self.transport.clone();
        let f = async move {
            let host = req.host().unwrap();
            let port = *req.port_u16().get_or_insert_with(|| {
                match req.scheme_str() {
                    Some("https") => 443,
                    _ => 80
                }
            });
            let stream = s.local_tunnel(&host.to_string(), port).await.unwrap();
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

