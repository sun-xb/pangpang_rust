use serde::{Serialize, Deserialize};




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SshProfile {
    pub username: String,
    pub address: String,
    pub port: u16,
    pub auth_method: SshAuthMethod,
    pub host_key: String,
}

impl super::ProfileId for SshProfile {
    fn id(&self) -> String {
        format!("{}@{}:{}", self.username, self.address, self.port)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SshAuthMethod {
    Password(String),
    AuthorizedKey(String)
}