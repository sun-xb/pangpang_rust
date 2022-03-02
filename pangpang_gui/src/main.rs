
use tokio::io::{AsyncReadExt, AsyncWriteExt};




#[tokio::main]
async fn main() {
    let addr = "localhost:22";
    let username = "sun";
    let password = "pangpang";
    let mut s = pangpang::async_ssh2::Session::new(addr).await.unwrap();
    //s.trace(pangpang::async_ssh2::TraceFlags::all());
    s.handshake().await.unwrap();
    let methods = s.auth_methods(username).await.unwrap();
    println!("methods: {}", methods);

    s.userauth_password(username, password).await.unwrap();

    /*
    let mut vector: Vec<pangpang::async_ssh2::Channel> = Vec::new();
    for i in 0..100 {
        println!("aaaaa {}", i);
        vector.push(s.channel_session().await.unwrap());
    }
    */
    
    let mut ch = s.channel_session().await.unwrap();
    ch.request_pty("xterm-256color", None, None).await.unwrap();
    ch.shell().await.unwrap();
    println!("read pty");

    ch.write_all(b"ls -l\nexit\n").await.unwrap();
    while !ch.eof() {
        let mut buf = [0u8; 1024];
        let size = ch.read(&mut buf).await.unwrap();
        print!("{}", String::from_utf8(buf[..size].to_vec()).unwrap());
    }

    let mut ch = s.channel_direct_tcpip("ip.gs", 81, None).await.unwrap();
    ch.write_all(b"GET / HTTP/1.0\r\nHost: ip.gs\r\nUser-Agent: curl/7.79.1\r\n\r\n").await.unwrap();
    while !ch.eof() {
        let mut buf = [0u8; 1024];
        let size = ch.read(&mut buf).await.unwrap();
        print!("{}", String::from_utf8(buf[..size].to_vec()).unwrap());
    }

}