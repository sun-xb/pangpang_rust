use std::sync::Arc;

use pangpang::storage::Storage;




#[tokio::main]
async fn main() {
    let config = Arc::new(tokio::sync::Mutex::new(pangpang::storage::YamlStorage::default()));
    let id = config.lock().await.iter().next().unwrap().id();
    let pp = pangpang::PangPang::new(config);
    pp.local_http_proxy(&id, &String::from("127.0.0.1"), 1080).await.unwrap().run().await;
}

