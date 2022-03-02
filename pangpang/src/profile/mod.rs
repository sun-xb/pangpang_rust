use serde::{Serialize, Deserialize};



mod ssh_profile;

pub use ssh_profile::*;


pub trait ProfileId {
    fn id(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub transport: Option<String>,
    pub protocol: Protocol,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Protocol {
    SSH(SshProfile)
}

impl ProfileId for Profile {
    fn id(&self) -> String {
        match self.protocol {
            Protocol::SSH(ref ssh) => format!("ssh://{}", ssh.id())
        }
    }
}
