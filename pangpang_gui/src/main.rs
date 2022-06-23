use tokio::io::{AsyncReadExt, AsyncWriteExt};



async fn make_socket_pair() -> (tokio::net::TcpStream, tokio::net::TcpStream) {
    let listerner = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_addr = listerner.local_addr().unwrap();
    loop {
        let joined = tokio::join!(listerner.accept(), tokio::net::TcpStream::connect(local_addr));
        let incomming = joined.0.unwrap();
        let client = joined.1.unwrap();
        if client.local_addr().unwrap() != incomming.1 {
            continue;
        }
        break (incomming.0, client);
    }
}


async fn get_ip_gs_1() {
    let mut c = tokio::net::TcpStream::connect("ip.gs:80").await.unwrap();
    let mut buffer = [0u8; 1000];
    let (mut s1, mut s2) = make_socket_pair().await;
    s2.write_all(b"GET / HTTP/1.1\r\nHost: ip.gs\r\nConnection: close\r\nUser-Agent: curl/7.79.1\r\n\r\n").await.unwrap();
    tokio::select! {
        _ = tokio::io::copy_bidirectional(&mut s1, &mut c) => {}
        n = s2.read(&mut buffer[..]) => {
            println!("{}", String::from_utf8(buffer[..n.unwrap()].to_vec()).unwrap());
        }
    }
}

async fn get_ip_gs_2() {
    let mut session = pp::async_ssh2::Session::new(tokio::net::TcpStream::connect("127.0.0.1:22").await.unwrap()).await.unwrap();
    session.handshake().await.unwrap();
    session.auth_methods("root").await.unwrap();
    session.userauth_password("root", "123456").await.unwrap();
    let c = session.channel_direct_tcpip("ip.gs", 80, None).await.unwrap();
    //let c = tokio::net::TcpStream::connect("ip.gs:80").await.unwrap();
    let mut buffer = [0u8; 1000];
    let (s1, mut s2) = make_socket_pair().await;
    s2.write_all(b"GET / HTTP/1.1\r\nHost: ip.gs\r\nConnection: close\r\nUser-Agent: curl/7.79.1\r\n\r\n").await.unwrap();

    tokio::spawn(async move {
        let mut ch = c;
        let mut s = s1;
        
        /*ch.write_all(b"GET / HTTP/1.1\r\nHost: ip.gs\r\nConnection: close\r\nUser-Agent: curl/7.79.1\r\n\r\n").await.unwrap();
        let mut b = [0u8; 1000];
        let n = ch.read(&mut b[..]).await.unwrap();
        println!("The bytes: {}", String::from_utf8(b[..n].to_vec()).unwrap());*/
        tokio::io::copy_bidirectional(&mut ch, &mut s).await.unwrap();
    });
    let n = s2.read(&mut buffer[..]).await.unwrap();
    println!("The bytes: {}", String::from_utf8(buffer[..n].to_vec()).unwrap());
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    get_ip_gs_1().await;
    get_ip_gs_2().await;
}